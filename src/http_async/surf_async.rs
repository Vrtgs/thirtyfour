use std::fmt::Debug;

use async_trait::async_trait;

use crate::http_async::connection_async::WebDriverHttpClientAsync;
use crate::{
    common::command::{Command, RequestMethod},
    error::{WebDriverError, WebDriverResult},
    SessionId,
};

/// Asynchronous http to the remote WebDriver server.
#[derive(Debug)]
pub struct SurfDriverAsync {
    url: String,
}

#[async_trait]
impl WebDriverHttpClientAsync for SurfDriverAsync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self> {
        Ok(SurfDriverAsync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
        })
    }

    /// Execute the specified command and return the data as serde_json::Value.
    async fn execute(
        &self,
        session_id: &SessionId,
        command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value> {
        let request_data = command.format_request(session_id);
        let url = self.url.clone() + &request_data.url;
        let mut request = match request_data.method {
            RequestMethod::Get => surf::get(&url),
            RequestMethod::Post => surf::post(&url),
            RequestMethod::Delete => surf::delete(&url),
        };
        if let Some(x) = request_data.body {
            request = request.body(x);
        }

        let mut resp = request.await?;

        match resp.status() {
            x if x.is_success() || x.is_redirection() => Ok(resp.body_json().await?),
            x if x.is_client_error() || x.is_server_error() => {
                let status = resp.status();
                let body: serde_json::Value =
                    resp.body_json().await.unwrap_or(serde_json::Value::Null);
                Err(WebDriverError::parse(status as u16, body))
            }
            _ => unreachable!(),
        }
    }
}
