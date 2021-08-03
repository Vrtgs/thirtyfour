use crate::common::command::FormatRequestData;
use crate::common::config::WebDriverConfig;
use crate::error::WebDriverResult;
use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::runtime::imports::Mutex;
use crate::webdrivercommands::WebDriverCommands;
use crate::SessionId;
use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

pub struct WebDriverSession {
    session_id: SessionId,
    conn: Arc<Mutex<dyn WebDriverHttpClientAsync>>,
    config: WebDriverConfig,
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
    pub fn new(session_id: SessionId, conn: Arc<Mutex<dyn WebDriverHttpClientAsync>>) -> Self {
        Self {
            session_id,
            conn,
            config: WebDriverConfig::default(),
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

    /// Reset the WebDriver configuration back to the defaults.
    pub fn config_clear(&mut self) {
        self.config = WebDriverConfig::default()
    }

    pub async fn execute(
        &self,
        request: Box<dyn FormatRequestData + Send + Sync>,
    ) -> WebDriverResult<serde_json::Value> {
        let conn = self.conn.lock().await;
        conn.execute(request.format_request(&self.session_id)).await
    }

    pub async fn set_request_timeout(&mut self, timeout: Duration) -> WebDriverResult<()> {
        let mut conn = self.conn.lock().await;
        conn.set_request_timeout(timeout);
        Ok(())
    }
}

#[async_trait]
impl WebDriverCommands for WebDriverSession {
    fn session(&self) -> &WebDriverSession {
        self
    }
}
