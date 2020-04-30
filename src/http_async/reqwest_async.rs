use std::fmt::Debug;

use async_trait::async_trait;

use crate::http_async::connection_async::{RemoteConnectionAsync, RemoteConnectionAsyncCreate};
use crate::{
    common::{
        command::{Command, RequestMethod},
        connection_common::reqwest_support::build_reqwest_headers,
    },
    error::{WebDriverError, WebDriverResult},
    SessionId,
};

/// Asynchronous http to the remote WebDriver server.
#[derive(Debug)]
pub struct ReqwestDriverAsync {
    url: String,
    client: reqwest::Client,
}

impl RemoteConnectionAsyncCreate for ReqwestDriverAsync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self> {
        let headers = build_reqwest_headers(remote_server_addr)?;
        Ok(ReqwestDriverAsync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: reqwest::Client::builder().default_headers(headers).build()?,
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
