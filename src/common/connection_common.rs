use base64::encode;
use reqwest;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_TYPE, USER_AGENT};

use urlparse::urlparse;

use crate::error::{RemoteConnectionError, WebDriverResult};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Construct the request headers used for every WebDriver request.
pub fn build_headers(remote_server_addr: &str) -> Result<HeaderMap, RemoteConnectionError> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/json".parse()?);
    headers.insert(CONTENT_TYPE, "application/json;charset=UTF-8".parse()?);
    headers.insert(
        USER_AGENT,
        format!("thirtyfour/{} (rust)", VERSION).parse()?,
    );

    let parsed_url = urlparse(remote_server_addr);
    if let (Some(username), Some(password)) = (parsed_url.username, parsed_url.password) {
        let base64_string = encode(&format!("{}:{}", username, password));
        headers.insert(AUTHORIZATION, format!("Basic {}", base64_string).parse()?);
    }
    headers.insert(CONNECTION, "keep-alive".parse()?);

    Ok(headers)
}

pub fn unwrap_string(value: &serde_json::Value) -> WebDriverResult<String> {
    let s: String = serde_json::from_value(value.clone())?;
    Ok(s)
}

pub fn unwrap_strings(value: &serde_json::Value) -> WebDriverResult<Vec<String>> {
    let v: Vec<String> = serde_json::from_value(value.clone())?;
    Ok(v)
}

pub fn unwrap_bool(value: &serde_json::Value) -> WebDriverResult<bool> {
    let b: bool = serde_json::from_value(value.clone())?;
    Ok(b)
}
