use log::error;
use serde::Deserialize;

use crate::error::{RemoteConnectionError, WebDriverError, WebDriverErrorValue};
use crate::remote::command::{Command, DesiredCapabilities, SessionId};
use crate::remote::connection_sync::RemoteConnectionSync;

#[derive(Debug)]
pub struct WebDriverSync {
    session_id: SessionId,
    capabilities: serde_json::Value,
    conn: RemoteConnectionSync,
}

impl WebDriverSync {
    pub fn new(
        remote_server_addr: &str,
        capabilities: DesiredCapabilities,
    ) -> Result<Self, WebDriverError> {
        let conn = RemoteConnectionSync::new(remote_server_addr)?;
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
        Ok(WebDriverSync {
            session_id,
            capabilities: actual_capabilities,
            conn,
        })
    }

    pub fn get<S: Into<String>>(&self, url: S) -> Result<(), WebDriverError> {
        self.conn
            .execute(Command::NavigateTo(&self.session_id, url.into()))?;
        Ok(())
    }

    pub fn quit(&self) -> Result<(), WebDriverError> {
        self.conn
            .execute(Command::DeleteSession(&self.session_id))?;
        Ok(())
    }

    pub fn title(&self) -> Result<String, WebDriverError> {
        let v = self.conn.execute(Command::GetTitle(&self.session_id))?;
        Ok(v["value"].as_str().unwrap_or_default().to_owned())
    }
}

impl Drop for WebDriverSync {
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
    use std::thread;
    use std::time::Duration;

    use crate::error::WebDriverError;
    use crate::remote::command::SessionId;

    use super::*;

    #[test]
    fn test_webdriver_sync() -> Result<(), WebDriverError> {
        let caps = serde_json::json!({
            "browserName": "chrome",
            "version": "",
            "platform": "any"
        });

        let driver = WebDriverSync::new("http://localhost:4444/wd/hub", caps)?;
        driver.get("https://mozilla.org");
        println!("Title = {}", driver.title()?);
        thread::sleep(Duration::new(3, 0));

        Ok(())
    }
}
