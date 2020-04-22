use std::fmt::Debug;

use async_trait::async_trait;

use crate::{
    common::{
        command::{Command, RequestMethod},
        connection_common::build_headers,
    },
    error::{WebDriverError, WebDriverResult},
    SessionId,
};

// NOTE: The remote connection is intended to be pluggable in order to facilitate
//       switching to a different HTTP client if desired. For example this should
//       allow switching to a client such as `surf` in order to work with an
//       async-std runtime rather than tokio.

#[async_trait]
pub trait RemoteConnectionAsync: Debug + Send + Sync {
    async fn execute(
        &self,
        session_id: &SessionId,
        command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value>;
}

/// Asynchronous connection to the remote WebDriver server.
#[derive(Debug)]
pub struct ReqwestDriverAsync {
    url: String,
    client: reqwest::Client,
}

impl ReqwestDriverAsync {
    /// Create a new ReqwestDriverAsync instance.
    pub fn new(remote_server_addr: &str) -> Result<Self, WebDriverError> {
        let headers = build_headers(remote_server_addr)?;
        Ok(ReqwestDriverAsync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }
}

#[async_trait]
impl RemoteConnectionAsync for ReqwestDriverAsync {
    /// Execute the specified command and return the data as serde_json::Value.
    async fn execute(
        &self,
        session_id: &SessionId,
        command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value> {
        let request_data = command.format_request(session_id);
        let url = self.url.clone() + &request_data.url;
        let mut request = match request_data.method {
            RequestMethod::Get => self.client.get(&url),
            RequestMethod::Post => self.client.post(&url),
            RequestMethod::Delete => self.client.delete(&url),
        };
        if let Some(x) = request_data.body {
            request = request.json(&x);
        }

        let resp = request.send().await?;

        match resp.status().as_u16() {
            200..=399 => Ok(resp.json().await?),
            400..=599 => {
                let status = resp.status().as_u16();
                let body: serde_json::Value = resp.json().await?;
                Err(WebDriverError::parse(status, body))
            }
            _ => unreachable!(),
        }
    }
}
