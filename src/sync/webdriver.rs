use std::sync::Arc;
use std::time::Duration;

use log::error;
use serde::Deserialize;

use crate::common::command::{
    By, Command, DesiredCapabilities, ElementId, SessionId, TimeoutConfiguration, WindowHandle,
};
use crate::common::constant::MAGIC_ELEMENTID;
use crate::error::{WebDriverError, WebDriverResult};
use crate::sync::RemoteConnectionSync;
use crate::sync::WebElement;

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

    fn unwrap_string(&self, value: &serde_json::Value) -> WebDriverResult<String> {
        value
            .as_str()
            .ok_or(WebDriverError::JsonError(format!(
                "Value is not a string: {:?}",
                value
            )))
            .map(|x| x.to_owned())
    }

    fn unwrap_strings(&self, value: &serde_json::Value) -> WebDriverResult<Vec<String>> {
        let values = value.as_array().ok_or(WebDriverError::JsonError(format!(
            "String array not found in value: {:?}",
            value
        )))?;
        values.iter().map(|x| self.unwrap_string(x)).collect()
    }

    fn unwrap_element(&self, value: &serde_json::Value) -> WebDriverResult<WebElement> {
        let id_str = value[MAGIC_ELEMENTID]
            .as_str()
            .ok_or(WebDriverError::JsonError(format!(
                "ElementId not found in value: {:?}",
                value
            )))?;
        Ok(WebElement::new(self.conn.clone(), ElementId::from(id_str)))
    }

    fn unwrap_elements(&self, value: &serde_json::Value) -> WebDriverResult<Vec<WebElement>> {
        let values = value.as_array().ok_or(WebDriverError::JsonError(format!(
            "ElementId array not found in value: {:?}",
            value
        )))?;
        values.iter().map(|x| self.unwrap_element(x)).collect()
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
        self.unwrap_string(&v["value"])
    }

    pub fn page_source(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetPageSource(&self.session_id))?;
        self.unwrap_string(&v["value"])
    }

    pub fn title(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetTitle(&self.session_id))?;
        Ok(v["value"].as_str().unwrap_or_default().to_owned())
    }

    pub fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::FindElement(&self.session_id, by))?;
        self.unwrap_element(&v["value"])
    }

    pub fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self
            .conn
            .execute(Command::FindElements(&self.session_id, by))?;
        self.unwrap_elements(&v["value"])
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
        self.unwrap_string(&v["value"])
            .map(|x| WindowHandle::from(x))
    }

    pub fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))?;
        let strings = self.unwrap_strings(&v["value"])?;
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

#[cfg(test)]
mod tests {
    use crate::error::WebDriverError;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_webdriver_sync() -> Result<(), WebDriverError> {
        let caps = serde_json::json!({
            "browserName": "chrome",
            "version": "",
            "platform": "any"
        });

        let driver = WebDriver::new("http://localhost:4444/wd/hub", caps)?;
        driver.get("https://mozilla.org")?;
        driver.find_element(By::Tag("div"))?;
        println!("Title = {}", driver.title()?);
        thread::sleep(Duration::new(3, 0));

        Ok(())
    }
}
