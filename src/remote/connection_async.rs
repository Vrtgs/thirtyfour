use crate::remote::command::{Command, RequestMethod};
use crate::remote::connection_common::{build_headers, CommandError, RemoteConnectionError};

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
    pub async fn execute<'a>(&self, command: Command<'a>) -> Result<serde_json::Value, CommandError>
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

        let resp = request.send().await.map_err(|e| CommandError::WebDriverError(e.to_string()))?;

        match resp.status().as_u16() {
            200..=399 => Ok(resp
                .json()
                .await
                .map_err(|e| CommandError::JsonError(e.to_string()))?),
            400..=599 => {
                // TODO: capture error data into CommandError::WebDriverError.
                Err(CommandError::WebDriverError(format!(
                    "something bad happened: {:?}",
                    resp
                )))
            }
            _ => Err(CommandError::WebDriverError("unknown result".to_owned())),
        }
    }
}
