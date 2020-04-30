use std::fmt::Debug;

use crate::sync::http_sync::connection_sync::{RemoteConnectionSync, RemoteConnectionSyncCreate};
use crate::{
    common::{
        command::{Command, RequestMethod},
        connection_common::build_headers,
    },
    error::{WebDriverError, WebDriverResult},
    SessionId,
};

/// Synchronous connection to the remote WebDriver server.
#[derive(Debug)]
pub struct ReqwestDriverSync {
    url: String,
    client: reqwest::blocking::Client,
}

impl RemoteConnectionSyncCreate for ReqwestDriverSync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self> {
        let headers = build_headers(remote_server_addr)?;
        Ok(ReqwestDriverSync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: reqwest::blocking::Client::builder().default_headers(headers).build()?,
        })
    }
}

impl RemoteConnectionSync for ReqwestDriverSync {
    /// Execute the specified command and return the data as serde_json::Value.
    fn execute(
        &self,
        session_id: &SessionId,
        command: Command,
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

        let resp = request.send()?;

        match resp.status().as_u16() {
            200..=399 => Ok(resp.json()?),
            400..=599 => {
                let status = resp.status().as_u16();
                let body: serde_json::Value = resp.json()?;
                Err(WebDriverError::parse(status, body))
            }
            _ => unreachable!(),
        }
    }
}
