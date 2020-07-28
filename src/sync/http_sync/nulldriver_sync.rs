use std::fmt::Debug;

use crate::sync::http_sync::connection_sync::WebDriverHttpClientSync;
use crate::{
    common::command::{Command, RequestMethod},
    error::{WebDriverError, WebDriverResult},
    SessionId,
};

/// Null driver that satisfies the build but does nothing.
#[derive(Debug)]
pub struct NullDriverSync {
    url: String,
}

impl WebDriverHttpClientSync for NullDriverSync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self> {
        Ok(NullDriverSync {
            url: remote_server_addr.to_string(),
        })
    }

    fn execute(
        &self,
        _session_id: &SessionId,
        _command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }
}
