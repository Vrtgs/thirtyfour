use crate::common::command::FormatRequestData;
use crate::common::config::WebDriverConfig;
use crate::error::{WebDriverError, WebDriverResult};
use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::webdrivercommands::WebDriverCommands;
use crate::{RequestData, SessionId};
use async_trait::async_trait;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::channel::oneshot;
use futures::SinkExt;
use futures::StreamExt;
use std::time::Duration;

#[derive(Debug)]
pub enum SessionMessage {
    Request(RequestData, oneshot::Sender<WebDriverResult<serde_json::Value>>),
    SetRequestTimeout(Duration),
}

pub fn spawn_session_task(
    conn: Box<dyn WebDriverHttpClientAsync>,
) -> UnboundedSender<SessionMessage> {
    let (tx, rx) = unbounded();

    #[cfg(feature = "async-std-runtime")]
    {
        async_std::task::spawn(session_runner(rx, conn));
    }

    #[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
    {
        tokio::spawn(session_runner(rx, conn));
    }

    tx
}

async fn session_runner(
    mut rx: UnboundedReceiver<SessionMessage>,
    mut conn: Box<dyn WebDriverHttpClientAsync>,
) {
    // This will return None when the sender hangs up.
    while let Some(msg) = rx.next().await {
        match msg {
            SessionMessage::Request(data, tx) => {
                let ret = conn.execute(data).await;
                tx.send(ret).expect("Failed to send response");
            }
            SessionMessage::SetRequestTimeout(timeout) => {
                conn.set_request_timeout(timeout);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct WebDriverSession {
    session_id: SessionId,
    tx: UnboundedSender<SessionMessage>,
    config: WebDriverConfig,
}

impl WebDriverSession {
    pub fn new(session_id: SessionId, tx: UnboundedSender<SessionMessage>) -> Self {
        Self {
            session_id,
            tx,
            config: WebDriverConfig::new(),
        }
    }

    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    pub fn config(&self) -> &WebDriverConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut WebDriverConfig {
        &mut self.config
    }

    pub async fn execute(
        &self,
        request: Box<dyn FormatRequestData + Send + Sync>,
    ) -> WebDriverResult<serde_json::Value> {
        let (ret_tx, ret_rx) = oneshot::channel();
        self.tx
            .clone()
            .send(SessionMessage::Request(request.format_request(&self.session_id), ret_tx))
            .await
            .map_err(|e| {
                WebDriverError::UnknownResponse(format!("Failed to send request to server: {}", e))
            })?;

        match ret_rx.await {
            Ok(x) => x,
            Err(oneshot::Canceled) => Err(WebDriverError::UnknownResponse(
                "Failed to get response from server".to_string(),
            )),
        }
    }

    pub async fn set_request_timeout(&mut self, timeout: Duration) -> WebDriverResult<()> {
        self.tx.clone().send(SessionMessage::SetRequestTimeout(timeout)).await.map_err(|e| {
            WebDriverError::UnknownResponse(format!("Failed to send request to server: {}", e))
        })?;
        Ok(())
    }
}

#[async_trait]
impl WebDriverCommands for WebDriverSession {
    fn session(&self) -> &WebDriverSession {
        &self
    }
}
