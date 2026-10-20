//! WebSocket transport for WebDriver BiDi.
//!
//! One WebSocket per WebDriver session (BiDi has no per-message session
//! multiplexing — that's handled by `New Session` over HTTP). Outbound
//! frames are written by a writer task; inbound frames are demuxed by a
//! reader task into:
//!
//! - oneshot channels for command responses, keyed by `id`.
//! - a `tokio::sync::broadcast` channel for events.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

use crate::bidi::command::RawEvent;
use crate::bidi::error::{BidiError, BidiErrorEnvelope};
use crate::error::{WebDriverError, WebDriverResult};

/// Channel buffer for the broadcast event stream. If a subscriber falls
/// further behind than this, it'll see `RecvError::Lagged` and miss frames.
const EVENT_BUFFER: usize = 1024;

#[derive(Debug, Clone)]
pub(crate) struct BidiTransport {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    out: mpsc::UnboundedSender<String>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value, BidiError>>>>,
    events: broadcast::Sender<RawEvent>,
    next_id: AtomicU64,
    /// Per-event-name reference count. Bumped on `BiDi::subscribe::<E>()`,
    /// decremented when the returned stream drops. Held across the wire
    /// `session.subscribe` / `session.unsubscribe` call so concurrent
    /// callers serialize on the lock instead of double-subscribing.
    subscriptions: Mutex<HashMap<&'static str, usize>>,
}

impl BidiTransport {
    /// Connect to the BiDi WebSocket URL.
    pub(crate) async fn connect(url: &str) -> WebDriverResult<Self> {
        let ws = crate::common::ws::connect(url)
            .await
            .map_err(|e| WebDriverError::HttpError(format!("BiDi WebSocket connect: {e}")))?;

        let (mut sink, mut stream) = ws.split();

        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
        let (events_tx, _events_rx0) = broadcast::channel::<RawEvent>(EVENT_BUFFER);

        let inner = Arc::new(Inner {
            out: out_tx,
            pending: Mutex::new(HashMap::new()),
            events: events_tx,
            next_id: AtomicU64::new(1),
            subscriptions: Mutex::new(HashMap::new()),
        });

        // Writer task.
        tokio::spawn(async move {
            while let Some(text) = out_rx.recv().await {
                if sink.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            let _ = sink.close().await;
        });

        // Reader task — consumes inbound frames and dispatches them.
        let inner_for_reader = Arc::clone(&inner);
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                let frame = match message {
                    Ok(Message::Text(t)) => t.as_str().to_owned(),
                    Ok(Message::Binary(b)) => match std::str::from_utf8(&b) {
                        Ok(s) => s.to_owned(),
                        Err(_) => continue,
                    },
                    Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => continue,
                    Ok(Message::Close(_)) | Err(_) => break,
                };
                inner_for_reader.dispatch(&frame).await;
            }
            inner_for_reader.fail_all_pending("BiDi WebSocket closed").await;
        });

        Ok(Self {
            inner,
        })
    }

    /// Send a BiDi command by `method` with raw `params` and await the
    /// response `result`.
    pub(crate) async fn send_raw(&self, method: &str, params: Value) -> Result<Value, BidiError> {
        // BiDi tolerates `"params": null` for some commands but the spec
        // requires an object. Coerce null → {} for safety.
        let params = if params.is_null() {
            serde_json::json!({})
        } else {
            params
        };
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let (resp_tx, resp_rx) = oneshot::channel();
        self.inner.pending.lock().await.insert(id, resp_tx);

        let frame = OutgoingCommand {
            id,
            method,
            params: &params,
        };
        let serialized = serde_json::to_string(&frame).map_err(|e| BidiError {
            command: method.to_string(),
            error: "unknown error".to_string(),
            message: format!("serialise command: {e}"),
            stacktrace: None,
        })?;
        if self.inner.out.send(serialized).is_err() {
            return Err(BidiError {
                command: method.to_string(),
                error: "unknown error".to_string(),
                message: "BiDi WebSocket connection closed".to_string(),
                stacktrace: None,
            });
        }

        match resp_rx.await {
            Ok(result) => result,
            Err(_) => Err(BidiError {
                command: method.to_string(),
                error: "unknown error".to_string(),
                message: "BiDi response channel dropped".to_string(),
                stacktrace: None,
            }),
        }
    }

    /// Subscribe to every event on this transport.
    pub(crate) fn subscribe_events(&self) -> broadcast::Receiver<RawEvent> {
        self.inner.events.subscribe()
    }

    /// Idempotently send `session.subscribe` for a typed event method.
    ///
    /// Bumps the per-method refcount; sends the wire-level subscribe only
    /// when the count transitions 0 → 1. Holds the subscriptions lock
    /// across the wire call so concurrent callers wait for the in-flight
    /// subscribe to ack before returning.
    pub(crate) async fn ensure_subscribed(&self, method: &'static str) -> Result<(), BidiError> {
        let mut subs = self.inner.subscriptions.lock().await;
        let current = *subs.get(method).unwrap_or(&0);
        if current == 0 {
            self.send_raw("session.subscribe", serde_json::json!({ "events": [method] })).await?;
        }
        subs.insert(method, current + 1);
        Ok(())
    }

    /// Decrement the refcount for a typed event method; if it hits zero,
    /// send the wire-level `session.unsubscribe`. Best-effort — errors
    /// are swallowed since this is called from `Drop`.
    pub(crate) async fn release_subscription(&self, method: &'static str) {
        let mut subs = self.inner.subscriptions.lock().await;
        let Some(entry) = subs.get_mut(method) else {
            return;
        };
        if *entry == 0 {
            return;
        }
        *entry -= 1;
        if *entry == 0 {
            subs.remove(method);
            // Hold the lock across the unsubscribe so a concurrent
            // `ensure_subscribed` call blocks until we're done.
            let _ = self
                .send_raw("session.unsubscribe", serde_json::json!({ "events": [method] }))
                .await;
        }
    }
}

impl Inner {
    async fn dispatch(self: &Arc<Self>, frame: &str) {
        let value: Value = match serde_json::from_str(frame) {
            Ok(v) => v,
            Err(_) => return,
        };

        match value.get("type").and_then(Value::as_str) {
            Some("success") => {
                let Some(id) = value.get("id").and_then(Value::as_u64) else {
                    return;
                };
                if let Some(tx) = self.pending.lock().await.remove(&id) {
                    let result = value.get("result").cloned().unwrap_or(Value::Null);
                    let _ = tx.send(Ok(result));
                }
            }
            Some("error") => {
                let env: BidiErrorEnvelope = match serde_json::from_value(value.clone()) {
                    Ok(e) => e,
                    Err(_) => BidiErrorEnvelope {
                        error: "unknown error".to_string(),
                        message: "malformed BiDi error envelope".to_string(),
                        stacktrace: None,
                    },
                };
                if let Some(id) = value.get("id").and_then(Value::as_u64)
                    && let Some(tx) = self.pending.lock().await.remove(&id)
                {
                    let _ = tx.send(Err(env.into_error("<response>")));
                }
                // Errors with `id: null` are protocol-level (e.g. malformed
                // command). The spec only specifies that they may appear; we
                // don't have a pending request to wake.
            }
            Some("event") => {
                if let Some(method) = value.get("method").and_then(Value::as_str) {
                    let params = value.get("params").cloned().unwrap_or(Value::Null);
                    let _ = self.events.send(RawEvent {
                        method: method.to_string(),
                        params,
                    });
                }
            }
            _ => {}
        }
    }

    async fn fail_all_pending(&self, message: &str) {
        let mut pending = self.pending.lock().await;
        for (_, tx) in pending.drain() {
            let _ = tx.send(Err(BidiError {
                command: "<connection>".to_string(),
                error: "unknown error".to_string(),
                message: message.to_string(),
                stacktrace: None,
            }));
        }
    }
}

/// Wire shape of an outbound BiDi command frame.
#[derive(Serialize)]
struct OutgoingCommand<'a> {
    id: u64,
    method: &'a str,
    params: &'a Value,
}
