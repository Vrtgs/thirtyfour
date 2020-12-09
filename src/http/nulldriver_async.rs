use std::fmt::Debug;

use async_trait::async_trait;

use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::{common::command::Command, error::WebDriverResult, RequestData, SessionId};

/// Null driver that satisfies the build but does nothing.
#[derive(Debug)]
pub struct NullDriverAsync {
    url: String,
}

#[async_trait]
impl WebDriverHttpClientAsync for NullDriverAsync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self> {
        Ok(NullDriverAsync {
            url: remote_server_addr.to_string(),
        })
    }

    fn set_request_timeout(&mut self, timeout: Duration) {}

    async fn execute(&self, _request_data: RequestData) -> WebDriverResult<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }
}
