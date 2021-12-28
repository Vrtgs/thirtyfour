use log::warn;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot;

use crate::common::command::FormatRequestData;
use crate::common::config::WebDriverConfig;
use crate::error::WebDriverResult;
use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::query::ElementPoller;
use crate::{DesiredCapabilities, SessionId};

pub type ToRequestData = Box<dyn FormatRequestData + Send + Sync>;

#[derive(Debug)]
pub enum SessionCommand {
    WebDriverCommand(ToRequestData, oneshot::Sender<WebDriverResult<serde_json::Value>>),
    Internal(InternalCommand),
}

#[derive(Debug)]
pub enum InternalCommand {
    GetSessionId(oneshot::Sender<SessionId>),
    GetConfig(oneshot::Sender<WebDriverConfig>),
    SetConfig(WebDriverConfig),
    GetCapabilities(oneshot::Sender<DesiredCapabilities>),
    SetRequestTimeout(Duration),
    SetPoller(ElementPoller),
}

pub struct WebDriverSession {
    session_id: SessionId,
    http_client: Box<dyn WebDriverHttpClientAsync>,
    config: Arc<WebDriverConfig>,
    capabilities: serde_json::Value,
}

impl fmt::Debug for WebDriverSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebDriverSession")
            .field("session_id", &self.session_id)
            .field("config", &self.config)
            .finish()
    }
}

impl WebDriverSession {
    pub fn new(
        session_id: SessionId,
        http_client: Box<dyn WebDriverHttpClientAsync>,
        capabilities: serde_json::Value,
    ) -> Self {
        Self {
            session_id: session_id.clone(),
            http_client,
            config: Arc::new(WebDriverConfig::new(session_id)),
            capabilities,
        }
    }

    pub fn config(&self) -> Arc<WebDriverConfig> {
        self.config.clone()
    }

    /// Run the webdriver session, by executing commands received over the channel
    /// and sending any responses back via the bundled oneshot channel.
    ///
    /// This will terminate cleanly when the sender is closed.
    pub async fn run(mut self, mut rx: UnboundedReceiver<SessionCommand>) -> WebDriverResult<()> {
        while let Some(msg) = rx.recv().await {
            match msg {
                SessionCommand::WebDriverCommand(cmd, ret_tx) => {
                    let value =
                        self.http_client.execute(cmd.format_request(&self.session_id)).await;
                    if let Err(e) = ret_tx.send(value) {
                        warn!(
                            "Command response failed to send (receiver hung up): cmd={:?} response={:?}", cmd,
                            e
                        );
                    }
                }
                SessionCommand::Internal(cmd) => match cmd {
                    InternalCommand::GetSessionId(ret_tx) => {
                        if let Err(e) = ret_tx.send(self.session_id.clone()) {
                            warn!("Command response failed to send (receiver hung up): cmd=GetSessionId response={:?}", e);
                        }
                    }
                    InternalCommand::GetConfig(ret_tx) => {
                        if let Err(e) = ret_tx.send((*self.config).clone()) {
                            warn!("Command response failed to send (receiver hung up): cmd=GetConfig response={:?}", e);
                        }
                    }
                    InternalCommand::SetConfig(cfg) => {
                        self.config = Arc::new(cfg);
                    }
                    InternalCommand::GetCapabilities(ret_tx) => {
                        if let Err(e) =
                            ret_tx.send(DesiredCapabilities::new(self.capabilities.clone()))
                        {
                            warn!("Command response failed to send (receiver hung up): cmd=GetCapabilities {:?}", e);
                        }
                    }
                    InternalCommand::SetRequestTimeout(timeout) => {
                        self.http_client.set_request_timeout(timeout);
                    }
                    InternalCommand::SetPoller(poller) => {
                        self.config.set_query_poller(poller);
                    }
                },
            }
        }

        Ok(())
    }
}
