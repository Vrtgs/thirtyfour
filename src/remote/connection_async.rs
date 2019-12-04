use crate::remote::command::{Command, RequestMethod};
use crate::remote::connection_common::{build_headers};
use crate::error::{WebDriverError, RemoteConnectionError};

/// Asynchronous remote with the Remote WebDriver server.
#[derive(Debug)]
pub struct RemoteConnectionAsync {
    url: String,
    client: reqwest::Client,
}

impl RemoteConnectionAsync {
    /// Create a new RemoteConnectionSync instance.
    pub fn new(remote_server_addr: &str) -> Result<Self, RemoteConnectionError> {
        let headers = build_headers(remote_server_addr)?;
        Ok(RemoteConnectionAsync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    /// Execute the specified command and return the deserialized data.
    /// The return type must implement DeserializeOwned.
    pub async fn execute<'a>(&self, command: Command<'a>) -> Result<serde_json::Value, WebDriverError>
    {
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
            .send().await
            .map_err(|e| WebDriverError::RequestFailed(e.to_string()))?;

        match resp.status().as_u16() {
            200..=399 => Ok(resp
                .json().await
                .map_err(|e| WebDriverError::JsonError(e.to_string()))?),
            400..=599 => {
                let status = resp.status().as_u16();
                let body: serde_json::Value = resp
                    .json().await
                    .map_err(|e| WebDriverError::JsonError(e.to_string()))?;

                Err(WebDriverError::parse(status, body))
            }
            _ => Err(WebDriverError::RequestFailed(format!(
                "Unknown response: {:?}",
                resp.json().await
                    .map_err(|e| WebDriverError::JsonError(e.to_string()))?
            ))),
        }
    }
}
