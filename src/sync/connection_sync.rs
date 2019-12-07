use crate::common::command::{Command, RequestMethod};
use crate::common::connection_common::build_headers;
use crate::error::{RemoteConnectionError, WebDriverError, WebDriverResult};

/// Synchronous remote connection with the Remote WebDriver server.
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

    /// Execute the specified command and return the data as serde_json::Value.
    pub fn execute(&self, command: Command) -> WebDriverResult<serde_json::Value> {
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
