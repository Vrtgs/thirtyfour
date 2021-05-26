use std::fmt::Debug;

use async_trait::async_trait;

use crate::http::connection_async::{HttpClientCreateParams, WebDriverHttpClientAsync};
use crate::{
    common::connection_common::reqwest_support::build_reqwest_headers,
    error::{WebDriverError, WebDriverResult},
    RequestData, RequestMethod,
};
use std::time::Duration;

/// Asynchronous http to the remote WebDriver server.
#[derive(Debug)]
pub struct ReqwestDriverAsync {
    url: String,
    client: reqwest::Client,
    timeout: Duration,
}

#[async_trait]
impl WebDriverHttpClientAsync for ReqwestDriverAsync {
    fn create(params: HttpClientCreateParams) -> WebDriverResult<Self> {
        let url = params.server_url.trim_end_matches('/').to_owned();
        let headers = build_reqwest_headers(&url)?;
        let mut driver = ReqwestDriverAsync {
            url,
            client: reqwest::Client::builder().default_headers(headers).build()?,
            timeout: Duration::from_secs(120),
        };
        if let Some(timeout) = params.timeout {
            driver.set_request_timeout(timeout);
        }
        Ok(driver)
    }

    fn set_request_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Execute the specified command and return the data as serde_json::Value.
    async fn execute(&self, request_data: RequestData) -> WebDriverResult<serde_json::Value> {
        let url = self.url.clone() + &request_data.url;
        let mut request = match request_data.method {
            RequestMethod::Get => self.client.get(&url),
            RequestMethod::Post => self.client.post(&url),
            RequestMethod::Delete => self.client.delete(&url),
        };
        request = request.timeout(self.timeout);

        if let Some(x) = request_data.body {
            request = request.json(&x);
        }

        let resp = request.send().await?;

        match resp.status().as_u16() {
            200..=399 => Ok(resp.json().await?),
            400..=599 => {
                let status = resp.status().as_u16();
                Err(WebDriverError::parse(status, resp.text().await?))
            }
            _ => unreachable!(),
        }
    }
}
