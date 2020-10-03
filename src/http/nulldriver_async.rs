use std::fmt::Debug;

use async_trait::async_trait;

use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::{common::command::Command, error::WebDriverResult, SessionId};

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

    async fn execute(
        &self,
        _session_id: &SessionId,
        _command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }
}
