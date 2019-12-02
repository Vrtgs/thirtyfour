use std::convert::TryFrom;

use base64::encode;
use reqwest;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_TYPE, HeaderMap, InvalidHeaderValue, USER_AGENT,
};
use serde::de::DeserializeOwned;

use urlparse::urlparse;

use crate::command::{Command, RequestMethod};
use crate::constant::SELENIUM_VERSION;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");


/// CommandError is the main error type returned from all selenium requests.
pub enum CommandError {
    ConnectionError(String),
    RequestError(String),
    WebDriverError(String),
    JsonError(String),
}

impl From<serde_json::error::Error> for CommandError {
    fn from(value: serde_json::error::Error) -> Self {
        CommandError::JsonError(value.to_string())
    }
}

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

/// Construct the request headers used for every WebDriver request.
fn build_headers(remote_server_addr: &str) -> Result<HeaderMap, RemoteConnectionError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse()?);
    headers.insert(CONTENT_TYPE, "application/json;charset=UTF-8".parse()?);
    let rustc_version = rustc_version_runtime::version().to_string();
    headers.insert(
        USER_AGENT,
        format!("thirtyfour/{} (rust {})", VERSION, rustc_version).parse()?,
    );

    let parsed_url = urlparse(remote_server_addr);
    if let (Some(username), Some(password)) = (parsed_url.username, parsed_url.password) {
        let base64_string = encode(&format!("{}:{}", username, password));
        headers.insert(AUTHORIZATION, format!("Basic {}", base64_string).parse()?);
    }
    headers.insert(CONNECTION, "keep-alive".parse()?);

    Ok(headers)
}

/// Asynchronous connection with the Remote WebDriver server.
#[derive(Debug)]
pub struct RemoteConnectionAsync {
    url: String,
    client: reqwest::Client,
}

impl RemoteConnectionAsync {
    /// Create a new RemoteConnectionSync instance.
    pub fn new(remote_server_addr: &str) -> Result<Self, RemoteConnectionError> {
        let headers = build_headers(remote_server_addr)?;
        Ok(RemoteConnectionAsync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }

    /// Execute the specified command and return the deserialized data.
    /// The return type must implement DeserializeOwned.
    pub async fn execute<T>(&self, command: Command) -> Result<T, CommandError>
    where
        T: DeserializeOwned,
    {
        let request_data = command.format_request();
        let url = self.url.clone() + &request_data.url;
        let resp_result = match request_data.method {
            RequestMethod::Get => self.client.get(&url).send().await,
            RequestMethod::Post => self
                .client
                .post(&request_data.url)
                .json(&request_data.body)
                .send().await,
            RequestMethod::Delete => self
                .client
                .delete(&request_data.url)
                .json(&request_data.body)
                .send().await,
        };

        let resp = resp_result.map_err(|e| CommandError::WebDriverError(e.to_string()))?;

        match resp.status().as_u16() {
            200..=399 => {
                Ok(resp
                    .json().await
                    .map_err(|e| CommandError::JsonError(e.to_string()))?)
            }
            400..=599 => {
                // TODO: capture error data into CommandError::WebDriverError.
                Err(CommandError::WebDriverError(
                    "something bad happened".to_owned(),
                ))
            }
            _ => Err(CommandError::WebDriverError("unknown result".to_owned())),
        }
    }
}

/// Synchronous connection with the Remote WebDriver server.
#[derive(Debug)]
pub struct RemoteConnectionSync {
    url: String,
    client: reqwest::blocking::Client,
}

impl RemoteConnectionSync {
    /// Create a new RemoteConnectionSync instance.
    pub fn new(remote_server_addr: &str) -> Result<Self, RemoteConnectionError> {
        let headers = build_headers(remote_server_addr)?;
        Ok(RemoteConnectionSync {
            url: remote_server_addr.trim_end_matches('/').to_owned(),
            client: reqwest::blocking::Client::builder().default_headers(headers).build()?,
        })
    }

    /// Execute the specified command and return the deserialized data.
    /// The return type must implement DeserializeOwned.
    pub fn execute<T>(&self, command: Command) -> Result<T, CommandError>
    where
        T: DeserializeOwned,
    {
        let request_data = command.format_request();
        let url = self.url.clone() + &request_data.url;
        let resp_result = match request_data.method {
            RequestMethod::Get => self.client.get(&url).send(),
            RequestMethod::Post => self
                .client
                .post(&request_data.url)
                .json(&request_data.body)
                .send(),
            RequestMethod::Delete => self
                .client
                .delete(&request_data.url)
                .json(&request_data.body)
                .send(),
        };

        let resp = resp_result.map_err(|e| CommandError::WebDriverError(e.to_string()))?;

        match resp.status().as_u16() {
            200..=399 => {
                Ok(resp
                    .json()
                    .map_err(|e| CommandError::JsonError(e.to_string()))?)
            }
            400..=599 => {
                // TODO: capture error data into CommandError::WebDriverError.
                Err(CommandError::WebDriverError(
                    "something bad happened".to_owned(),
                ))
            }
            _ => Err(CommandError::WebDriverError("unknown result".to_owned())),
        }
    }
}
