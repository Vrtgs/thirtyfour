use base64::encode;
use reqwest::header::{
    HeaderMap, InvalidHeaderValue, ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_TYPE, USER_AGENT,
};
use reqwest::Client;

use urlparse::urlparse;

use crate::command::Command;
use crate::constant::SELENIUM_VERSION;

pub enum RemoteConnectionError {
    InvalidUrl(String),
    ClientError(String),
}

impl From<InvalidHeaderValue> for RemoteConnectionError {
    fn from(value: InvalidHeaderValue) -> Self {
        RemoteConnectionError::InvalidUrl(value.to_string())
    }
}

impl From<reqwest::Error> for RemoteConnectionError {
    fn from(value: reqwest::Error) -> Self {
        RemoteConnectionError::ClientError(value.to_string())
    }
}

/// A connection with the Remote WebDriver server.
#[derive(Debug)]
pub struct RemoteConnection {
    url: String,
    client: Client,
}

impl RemoteConnection {
    pub fn new(remote_server_addr: &str) -> Result<Self, RemoteConnectionError> {
        let headers = RemoteConnection::build_headers(remote_server_addr)?;
        Ok(RemoteConnection {
            url: remote_server_addr.to_owned(),
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    fn build_headers(remote_server_addr: &str) -> Result<HeaderMap, RemoteConnectionError> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse()?);
        headers.insert(CONTENT_TYPE, "application/json;charset=UTF-8".parse()?);
        let rustc_version = rustc_version_runtime::version().to_string();
        headers.insert(
            USER_AGENT,
            format!("selenium/{} (rust {})", SELENIUM_VERSION, rustc_version).parse()?,
        );

        let parsed_url = urlparse(remote_server_addr);
        if let (Some(username), Some(password)) = (parsed_url.username, parsed_url.password) {
            let base64_string = encode(&format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, format!("Basic {}", base64_string).parse()?);
        }
        headers.insert(CONNECTION, "keep-alive".parse()?);

        Ok(headers)
    }
}
