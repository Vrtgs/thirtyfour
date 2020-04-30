use serde::de::DeserializeOwned;

use crate::error::WebDriverResult;

pub fn convert_json<T>(value: &serde_json::Value) -> WebDriverResult<T>
where
    T: DeserializeOwned,
{
    let s: T = serde_json::from_value(value.clone())?;
    Ok(s)
}

pub fn convert_json_vec<T>(value: &serde_json::Value) -> WebDriverResult<Vec<T>>
where
    T: DeserializeOwned,
{
    let v: Vec<T> = serde_json::from_value(value.clone())?;
    Ok(v)
}

#[cfg(feature = "tokio-runtime")]
pub mod reqwest_support {
    use base64::encode;
    use reqwest::{
        self,
        header::{HeaderMap, ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_TYPE, USER_AGENT},
    };

    use urlparse::urlparse;

    use crate::error::WebDriverResult;

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Construct the request headers used for every WebDriver request (reqwest only).
    pub fn build_reqwest_headers(remote_server_addr: &str) -> WebDriverResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json;charset=UTF-8".parse().unwrap());
        headers.insert(USER_AGENT, format!("thirtyfour/{} (rust)", VERSION).parse().unwrap());

        let parsed_url = urlparse(remote_server_addr);
        if let (Some(username), Some(password)) = (parsed_url.username, parsed_url.password) {
            let base64_string = encode(&format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, format!("Basic {}", base64_string).parse().unwrap());
        }
        headers.insert(CONNECTION, "keep-alive".parse().unwrap());

        Ok(headers)
    }
}
