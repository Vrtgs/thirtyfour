use crate::error::{RemoteConnectionError, WebDriverError};
use crate::remote::command::{Command, RequestMethod};
use crate::remote::connection_common::build_headers;

/// Synchronous remote with the Remote WebDriver server.
#[derive(Debug)]
pub struct RemoteConnectionSync {
    url: String,
    client: reqwest::blocking::Client,
}

impl RemoteConnectionSync {
    /// Create a new RemoteConnectionSync instance.
    pub fn new(remote_server_addr: &str) -> Result<Self, RemoteConnectionError> {
        let headers = build_headers(remote_server_addr)?;
        Ok(RemoteConnectionSync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    /// Execute the specified command and return the deserialized data.
    /// The return type must implement DeserializeOwned.
    pub fn execute(&self, command: Command) -> Result<serde_json::Value, WebDriverError> {
        let request_data = command.format_request();
        let url = self.url.clone() + &request_data.url;
        let mut request = match request_data.method {
            RequestMethod::Get => self.client.get(&url),
            RequestMethod::Post => self.client.post(&url),
            RequestMethod::Delete => self.client.delete(&url),
        };
        if request_data.has_body() {
            request = request.json(&request_data.body);
        }

        let resp = request
            .send()
            .map_err(|e| WebDriverError::RequestFailed(e.to_string()))?;

        match resp.status().as_u16() {
            200..=399 => Ok(resp
                .json()
                .map_err(|e| WebDriverError::JsonError(e.to_string()))?),
            400..=599 => {
                let status = resp.status().as_u16();
                let body: serde_json::Value = resp
                    .json()
                    .map_err(|e| WebDriverError::JsonError(e.to_string()))?;
                Err(WebDriverError::parse(status, body))
            }
            _ => Err(WebDriverError::RequestFailed(format!(
                "Unknown response: {:?}",
                resp.json()
                    .map_err(|e| WebDriverError::JsonError(e.to_string()))?
            ))),
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
    fn test_sync() -> Result<(), WebDriverError> {
        let conn =
            RemoteConnectionSync::new("http://localhost:4444/wd/hub").expect("Failed to connect");
        let caps = serde_json::json!({
            "browserName": "chrome",
            "version": "",
            "platform": "any"
        });

        let v = conn.execute(Command::NewSession(caps))?;
        let session_id = SessionId::from(v["sessionId"].as_str().unwrap());
        conn.execute(Command::NavigateTo(
            &session_id,
            "https://google.com.au".to_owned(),
        ))?;
        conn.execute(Command::Status)?;
        thread::sleep(Duration::new(3, 0));
        conn.execute(Command::DeleteSession(&session_id))?;

        Ok(())
    }
}
