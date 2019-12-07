use std::sync::Arc;
use std::time::Duration;

use log::error;
use serde::Deserialize;

use crate::common::command::{
    By, Command, DesiredCapabilities, SessionId, TimeoutConfiguration, WindowHandle,
};
use crate::common::connection_common::{unwrap_string, unwrap_strings};
use crate::error::WebDriverResult;
use crate::RemoteConnectionAsync;
use crate::WebElement;
use crate::webelement::{unwrap_element_async, unwrap_elements_async};

pub struct WebDriver {
    session_id: SessionId,
    capabilities: serde_json::Value,
    conn: Arc<RemoteConnectionAsync>,
}

impl WebDriver {
    pub async fn new(
        remote_server_addr: &str,
        capabilities: DesiredCapabilities,
    ) -> WebDriverResult<Self> {
        let conn = Arc::new(RemoteConnectionAsync::new(remote_server_addr)?);
        let v = conn.execute(Command::NewSession(capabilities)).await?;

        #[derive(Debug, Deserialize)]
        struct ConnectionData {
            #[serde(rename(deserialize = "sessionId"))]
            session_id: String,
            value: serde_json::Value,
        }

        let data: ConnectionData = serde_json::from_value(v)?;
        let session_id = SessionId::from(data.session_id);
        let actual_capabilities = data.value;
        Ok(WebDriver {
            session_id,
            capabilities: actual_capabilities,
            conn,
        })
    }

    pub fn capabilities(&self) -> &DesiredCapabilities {
        &self.capabilities
    }

    pub async fn close(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::CloseWindow(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn quit(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteSession(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn get<S: Into<String>>(&self, url: S) -> WebDriverResult<()> {
        self.conn
            .execute(Command::NavigateTo(&self.session_id, url.into())).await
            .map(|_| ())
    }

    pub async fn current_url(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetCurrentUrl(&self.session_id)).await?;
        unwrap_string(&v["value"])
    }

    pub async fn page_source(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetPageSource(&self.session_id)).await?;
        unwrap_string(&v["value"])
    }

    pub async fn title(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetTitle(&self.session_id)).await?;
        Ok(v["value"].as_str().unwrap_or_default().to_owned())
    }

    pub async fn find_element<'a>(&self, by: By<'a>) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::FindElement(&self.session_id, by)).await?;
        unwrap_element_async(self.conn.clone(), self.session_id.clone(),&v["value"])
    }

    pub async fn find_elements<'a>(&self, by: By<'a>) -> WebDriverResult<Vec<WebElement>> {
        let v = self
            .conn
            .execute(Command::FindElements(&self.session_id, by)).await?;
        unwrap_elements_async(&self.conn, &self.session_id, &v["value"])
    }

    pub async fn execute_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self.conn.execute(Command::ExecuteScript(
            &self.session_id,
            script.to_owned(),
            args,
        )).await?;
        Ok(v["value"].clone())
    }

    pub async fn execute_async_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self.conn.execute(Command::ExecuteAsyncScript(
            &self.session_id,
            script.to_owned(),
            args,
        )).await?;
        Ok(v["value"].clone())
    }

    pub async fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        let v = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id)).await?;
        unwrap_string(&v["value"])
            .map(|x| WindowHandle::from(x))
    }

    pub async fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id)).await?;
        let strings = unwrap_strings(&v["value"])?;
        Ok(strings.iter().map(|x| WindowHandle::from(x)).collect())
    }

    pub async fn mazimize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MaximizeWindow(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn minimize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MinimizeWindow(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::FullscreenWindow(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn back(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Back(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn forward(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Forward(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn refresh(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Refresh(&self.session_id)).await
            .map(|_| ())
    }

    pub async fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SetTimeouts(&self.session_id, timeouts)).await
            .map(|_| ())
    }

    pub async fn implicitly_wait(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.set_timeouts(timeouts).await
    }

    pub async fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.set_timeouts(timeouts).await
    }

    pub async fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.set_timeouts(timeouts).await
    }
}

impl Drop for WebDriver {
    fn drop(&mut self) {
        if !(*self.session_id).is_empty() {
            if let Err(_) = async_std::task::block_on(self.quit()) {
                error!("Failed to close session");
            }
        }
    }
}
