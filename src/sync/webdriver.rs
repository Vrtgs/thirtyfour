use std::sync::Arc;
use std::time::Duration;

use log::error;
use serde::Deserialize;

use crate::common::command::{
    By, Command, DesiredCapabilities, SessionId, TimeoutConfiguration, WindowHandle,
};
use crate::common::connection_common::{unwrap_string, unwrap_strings};
use crate::error::WebDriverResult;
use crate::sync::action_chain::ActionChain;
use crate::sync::webelement::{unwrap_element_sync, unwrap_elements_sync, WebElement};
use crate::sync::RemoteConnectionSync;

#[derive(Debug, Clone)]
pub struct WebDriver {
    session_id: SessionId,
    capabilities: serde_json::Value,
    conn: Arc<RemoteConnectionSync>,
}

impl WebDriver {
    pub fn new(
        remote_server_addr: &str,
        capabilities: DesiredCapabilities,
    ) -> WebDriverResult<Self> {
        let conn = Arc::new(RemoteConnectionSync::new(remote_server_addr)?);
        let v = conn.execute(Command::NewSession(capabilities))?;

        #[derive(Debug, Deserialize)]
        struct ConnectionData {
            #[serde(default, rename(deserialize = "sessionId"))]
            session_id: String,
            #[serde(default)]
            capabilities: serde_json::Value,
        }

        #[derive(Debug, Deserialize)]
        struct ConnectionResp {
            #[serde(default)]
            session_id: String,
            value: ConnectionData,
        }

        let resp: ConnectionResp = serde_json::from_value(v)?;
        let data = resp.value;
        let session_id = SessionId::from(if resp.session_id.is_empty() {
            data.session_id
        } else {
            resp.session_id
        });
        let actual_capabilities = data.capabilities;
        Ok(WebDriver {
            session_id,
            capabilities: actual_capabilities,
            conn,
        })
    }

    pub fn capabilities(&self) -> &DesiredCapabilities {
        &self.capabilities
    }

    pub fn close(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::CloseWindow(&self.session_id))
            .map(|_| ())
    }

    pub fn quit(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteSession(&self.session_id))
            .map(|_| ())
    }

    pub fn get<S: Into<String>>(&self, url: S) -> WebDriverResult<()> {
        self.conn
            .execute(Command::NavigateTo(&self.session_id, url.into()))
            .map(|_| ())
    }

    pub fn current_url(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetCurrentUrl(&self.session_id))?;
        unwrap_string(&v["value"])
    }

    pub fn page_source(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetPageSource(&self.session_id))?;
        unwrap_string(&v["value"])
    }

    pub fn title(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetTitle(&self.session_id))?;
        unwrap_string(&v["value"])
    }

    pub fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::FindElement(&self.session_id, by))?;
        unwrap_element_sync(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    pub fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self
            .conn
            .execute(Command::FindElements(&self.session_id, by))?;
        unwrap_elements_sync(&self.conn, &self.session_id, &v["value"])
    }

    pub fn execute_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self.conn.execute(Command::ExecuteScript(
            &self.session_id,
            script.to_owned(),
            args,
        ))?;
        Ok(v["value"].clone())
    }

    pub fn execute_async_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self.conn.execute(Command::ExecuteAsyncScript(
            &self.session_id,
            script.to_owned(),
            args,
        ))?;
        Ok(v["value"].clone())
    }

    pub fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        let v = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id))?;
        unwrap_string(&v["value"]).map(|x| WindowHandle::from(x))
    }

    pub fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))?;
        let strings = unwrap_strings(&v["value"])?;
        Ok(strings.iter().map(|x| WindowHandle::from(x)).collect())
    }

    pub fn mazimize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MaximizeWindow(&self.session_id))
            .map(|_| ())
    }

    pub fn minimize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MinimizeWindow(&self.session_id))
            .map(|_| ())
    }

    pub fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::FullscreenWindow(&self.session_id))
            .map(|_| ())
    }

    pub fn back(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Back(&self.session_id))
            .map(|_| ())
    }

    pub fn forward(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Forward(&self.session_id))
            .map(|_| ())
    }

    pub fn refresh(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Refresh(&self.session_id))
            .map(|_| ())
    }

    pub fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SetTimeouts(&self.session_id, timeouts))
            .map(|_| ())
    }

    pub fn implicitly_wait(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.set_timeouts(timeouts)
    }

    pub fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.set_timeouts(timeouts)
    }

    pub fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.set_timeouts(timeouts)
    }

    pub fn action_chain(&self) -> ActionChain {
        ActionChain::new(self.conn.clone(), self.session_id.clone())
    }
}

impl Drop for WebDriver {
    fn drop(&mut self) {
        if !(*self.session_id).is_empty() {
            if let Err(_) = self.quit() {
                error!("Failed to close session");
            }
        }
    }
}
