use base64::encode;
use reqwest;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_TYPE, USER_AGENT};

use urlparse::urlparse;

use crate::error::RemoteConnectionError;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Construct the request headers used for every WebDriver request.
pub fn build_headers(remote_server_addr: &str) -> Result<HeaderMap, RemoteConnectionError> {
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
