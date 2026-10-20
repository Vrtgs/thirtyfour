//! WebSocket transport for CDP — flat session mode.
//!
//! One WebSocket per browser, many logical sessions multiplexed over it via
//! `sessionId`. Built on `tokio-tungstenite`. Outbound frames are written
//! by a writer task; inbound frames are demuxed by a reader task into:
//!
//! - oneshot channels for command responses, keyed by `(sessionId, id)`.
//! - a `tokio::sync::broadcast` channel for events.
//!
//! Subscribers filter the broadcast on their side (typically by
//! `sessionId` and `method`).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;

use super::Transport;
use crate::cdp::command::RawEvent;
use crate::cdp::error::{CdpError, CdpErrorEnvelope};
use crate::cdp::ids::SessionId;
use crate::error::{WebDriverError, WebDriverErrorInner, WebDriverResult};

/// Channel buffer for the broadcast event stream. If a subscriber falls
/// further behind than this, it'll see `RecvError::Lagged` and miss frames.
const EVENT_BUFFER: usize = 1024;

#[derive(Debug, Clone)]
pub(crate) struct WsTransport {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    /// Channel for outbound serialized JSON frames.
    out: mpsc::UnboundedSender<String>,
    /// Pending command responses keyed by `(sessionId, id)`. `sessionId` is
    /// `None` for messages addressed to the root browser session.
    pending: Mutex<HashMap<PendingKey, oneshot::Sender<Result<Value, CdpError>>>>,
    /// Broadcast channel for events. One subscriber per
    /// [`crate::cdp::CdpSession`] (and one for `subscribe_all`).
    events: broadcast::Sender<RawEvent>,
    /// Monotonic id allocator for outgoing commands.
    next_id: AtomicU64,
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct PendingKey {
    session: Option<SessionId>,
    id: u64,
}

impl WsTransport {
    /// Connect to the given browser-level CDP WebSocket URL.
    pub(crate) async fn connect(url: &str) -> WebDriverResult<Self> {
        let ws = crate::common::ws::connect(url)
            .await
            .map_err(|e| WebDriverError::HttpError(format!("CDP WebSocket connect: {e}")))?;

        let (mut sink, mut stream) = ws.split();

        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
        let (events_tx, _events_rx0) = broadcast::channel::<RawEvent>(EVENT_BUFFER);

        let inner = Arc::new(Inner {
            out: out_tx,
            pending: Mutex::new(HashMap::new()),
            events: events_tx.clone(),
            next_id: AtomicU64::new(1),
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

        // Reader task — consumes inbound messages and dispatches them.
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
            // Connection closed — fail any outstanding pending requests.
            inner_for_reader.fail_all_pending("CDP WebSocket closed").await;
        });

        Ok(Self {
            inner,
        })
    }

    /// Send a typed CDP command. `session` is `None` for the root browser
    /// session; otherwise the `sessionId` of an attached target.
    pub(crate) async fn send_raw_sessioned(
        &self,
        method: &str,
        params: Value,
        session: Option<&SessionId>,
    ) -> Result<Value, CdpError> {
        // CDP rejects `"params": null` for unit-struct commands. Coerce to
        // the empty object the wire format expects (matches HTTP transport).
        let params = if params.is_null() {
            serde_json::json!({})
        } else {
            params
        };
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let key = PendingKey {
            session: session.cloned(),
            id,
        };
        let (resp_tx, resp_rx) = oneshot::channel();
        self.inner.pending.lock().await.insert(key, resp_tx);

        let frame = OutgoingCommand {
            id,
            method,
            params: &params,
            session_id: session,
        };
        let serialized = serde_json::to_string(&frame).map_err(|e| CdpError {
            command: method.to_string(),
            code: -32603,
            message: format!("serialise command: {e}"),
            data: None,
        })?;
        if self.inner.out.send(serialized).is_err() {
            return Err(CdpError {
                command: method.to_string(),
                code: -32000,
                message: "CDP WebSocket connection closed".to_string(),
                data: None,
            });
        }

        match resp_rx.await {
            Ok(result) => result,
            Err(_) => Err(CdpError {
                command: method.to_string(),
                code: -32000,
                message: "CDP response channel dropped".to_string(),
                data: None,
            }),
        }
    }

    /// Subscribe to every event on this transport (across all sessions).
    pub(crate) fn subscribe_events(&self) -> broadcast::Receiver<RawEvent> {
        self.inner.events.subscribe()
    }
}

impl Inner {
    async fn dispatch(self: &Arc<Self>, frame: &str) {
        let value: Value = match serde_json::from_str(frame) {
            Ok(v) => v,
            Err(_) => return,
        };
        let session_id =
            value.get("sessionId").and_then(Value::as_str).map(|s| SessionId::from(s.to_string()));

        // Response (has `id`)? Resolve the matching pending request.
        if let Some(id) = value.get("id").and_then(Value::as_u64) {
            let key = PendingKey {
                session: session_id,
                id,
            };
            let pending = self.pending.lock().await.remove(&key);
            if let Some(tx) = pending {
                let result = if let Some(err) = value.get("error") {
                    let env: CdpErrorEnvelope =
                        serde_json::from_value(err.clone()).unwrap_or(CdpErrorEnvelope {
                            code: -32603,
                            message: "malformed CDP error envelope".to_string(),
                            data: None,
                        });
                    Err(env.into_error("<response>"))
                } else {
                    Ok(value.get("result").cloned().unwrap_or(Value::Null))
                };
                let _ = tx.send(result);
            }
            return;
        }

        // Event (no `id`).
        if let Some(method) = value.get("method").and_then(Value::as_str) {
            let params = value.get("params").cloned().unwrap_or(Value::Null);
            let _ = self.events.send(RawEvent {
                method: method.to_string(),
                params,
                session_id,
            });
        }
    }

    async fn fail_all_pending(&self, message: &str) {
        let mut pending = self.pending.lock().await;
        for (_, tx) in pending.drain() {
            let _ = tx.send(Err(CdpError {
                command: "<connection>".to_string(),
                code: -32000,
                message: message.to_string(),
                data: None,
            }));
        }
    }
}

/// Wire shape of an outbound command frame.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutgoingCommand<'a> {
    id: u64,
    method: &'a str,
    params: &'a Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<&'a SessionId>,
}

impl Transport for WsTransport {
    /// Root-session send (no `sessionId`). For child sessions go through
    /// [`crate::cdp::CdpSession`] which carries a sessionId.
    async fn send_raw(&self, method: &str, params: Value) -> WebDriverResult<Value> {
        self.send_raw_sessioned(method, params, None)
            .await
            .map_err(|e| WebDriverError::from_inner(WebDriverErrorInner::ParseError(e.to_string())))
    }
}

// Internal: silence unused import in non-test builds where Deserialize isn't used.
#[allow(dead_code)]
#[derive(Deserialize)]
struct _Unused;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Helper to serialise an `OutgoingCommand` into a JSON value so we can
    /// assert wire-shape contracts.
    fn frame(method: &str, params: &Value, session: Option<&SessionId>, id: u64) -> Value {
        let cmd = OutgoingCommand {
            id,
            method,
            params,
            session_id: session,
        };
        serde_json::to_value(&cmd).unwrap()
    }

    #[test]
    fn outgoing_frame_root_session_omits_session_id() {
        let v = frame("Browser.getVersion", &json!({}), None, 1);
        assert_eq!(v["id"], 1);
        assert_eq!(v["method"], "Browser.getVersion");
        assert_eq!(v["params"], json!({}));
        assert!(v.get("sessionId").is_none(), "root session must NOT include sessionId");
    }

    #[test]
    fn outgoing_frame_with_session_id_emits_camel_case() {
        let session = SessionId::from("S1");
        let v = frame("Page.navigate", &json!({"url": "x"}), Some(&session), 7);
        assert_eq!(v["sessionId"], "S1");
        assert_eq!(v["params"]["url"], "x");
        assert_eq!(v["id"], 7);
    }

    #[test]
    fn pending_key_hash_eq_distinguishes_session() {
        let mut map: HashMap<PendingKey, u32> = HashMap::new();
        map.insert(
            PendingKey {
                session: None,
                id: 1,
            },
            10,
        );
        map.insert(
            PendingKey {
                session: Some(SessionId::from("S1")),
                id: 1,
            },
            20,
        );
        // Same id, different sessions → different keys.
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get(&PendingKey {
                session: None,
                id: 1
            }),
            Some(&10)
        );
        assert_eq!(
            map.get(&PendingKey {
                session: Some(SessionId::from("S1")),
                id: 1
            }),
            Some(&20)
        );
    }
}
