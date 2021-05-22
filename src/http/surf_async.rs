use std::fmt::Debug;

use async_trait::async_trait;

use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::{
    common::connection_common::surf_support::build_isahc_headers,
    error::{WebDriverError, WebDriverResult},
    RequestData, RequestMethod,
};
use isahc::prelude::Configurable;
use std::time::Duration;
use surf::Client;

/// Asynchronous http to the remote WebDriver server.
#[derive(Debug)]
pub struct SurfDriverAsync {
    url: String,
    client: Client,
}

fn setup_client(remote_server_addr: &str, timeout: Duration) -> Client {
    let headers = build_isahc_headers(remote_server_addr);

    let backing_client = isahc::HttpClient::builder()
        .timeout(timeout)
        .default_headers(headers)
        .build()
        .expect("Error creating HTTP client");
    let http_client = http_client::isahc::IsahcClient::from_client(backing_client);
    Client::with_http_client(http_client)
}

#[async_trait]
impl WebDriverHttpClientAsync for SurfDriverAsync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self> {
        Ok(SurfDriverAsync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: setup_client(remote_server_addr, Duration::from_secs(120)),
        })
    }

    fn set_request_timeout(&mut self, timeout: Duration) {
        // Currently it looks like the only way to increase the timeout is by recreating the client.
        // https://github.com/http-rs/surf/issues/267
        self.client = setup_client(&self.url, timeout);
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
                Err(WebDriverError::parse(status as u16, resp.body_string().await?))
            }
            _ => unreachable!(),
        }
    }
}
