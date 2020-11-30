use std::fmt::Debug;

use async_trait::async_trait;

use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::{
    error::{WebDriverError, WebDriverResult},
    RequestData, RequestMethod,
};

/// Asynchronous http to the remote WebDriver server.
#[derive(Debug)]
pub struct SurfDriverAsync {
    url: String,
    client: surf::Client,
}

#[async_trait]
impl WebDriverHttpClientAsync for SurfDriverAsync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self> {
        Ok(SurfDriverAsync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: surf::Client::new(),
        })
    }

    /// Execute the specified command and return the data as serde_json::Value.
    async fn execute(&self, request_data: RequestData) -> WebDriverResult<serde_json::Value> {
        let url = self.url.clone() + &request_data.url;
        let mut request = match request_data.method {
            RequestMethod::Get => self.client.get(&url),
            RequestMethod::Post => self.client.post(&url),
            RequestMethod::Delete => self.client.delete(&url),
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
