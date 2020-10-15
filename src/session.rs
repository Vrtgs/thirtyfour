use crate::common::command::FormatRequestData;
use crate::common::config::WebDriverConfig;
use crate::error::WebDriverResult;
use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::webdrivercommands::WebDriverCommands;
use crate::SessionId;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
pub struct WebDriverSession {
    session_id: SessionId,
    conn: Arc<dyn WebDriverHttpClientAsync>,
    config: WebDriverConfig,
}

impl WebDriverSession {
    pub fn new(session_id: SessionId, conn: Arc<dyn WebDriverHttpClientAsync>) -> Self {
        Self {
            session_id,
            conn,
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
        self.conn.execute(request.format_request(&self.session_id)).await
    }
}

#[async_trait]
impl WebDriverCommands for WebDriverSession {
    fn session(&self) -> &WebDriverSession {
        &self
    }
}
