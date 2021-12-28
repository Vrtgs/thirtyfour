use std::fmt::Debug;

use async_trait::async_trait;

use crate::http::connection_async::{HttpClientCreateParams, WebDriverHttpClientAsync};
use crate::{error::WebDriverResult, RequestData};
use std::time::Duration;

/// Null driver that satisfies the build but does nothing.
#[derive(Debug)]
pub struct NullDriverAsync {
    url: String,
}

#[async_trait]
impl WebDriverHttpClientAsync for NullDriverAsync {
    fn create(params: HttpClientCreateParams) -> WebDriverResult<Self> {
        let mut driver = NullDriverAsync {
            url: params.server_url.to_string(),
        };
        if let Some(timeout) = params.timeout {
            driver.set_request_timeout(timeout);
        }
        Ok(driver)
    }

    fn set_request_timeout(&mut self, _timeout: Duration) {}

    async fn execute(&self, _request_data: RequestData) -> WebDriverResult<serde_json::Value> {
        // Silence dead_code warning.
        let _ = self.url;
        Ok(serde_json::Value::Null)
    }
}
