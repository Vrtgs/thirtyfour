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

#[cfg(feature = "async-std-runtime")]
pub mod surf_support {
    use base64::encode;
    use std::collections::HashMap;

    use urlparse::urlparse;

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    const ACCEPT: &'static str = "accept";
    const AUTHORIZATION: &str = "authorization";
    const CONNECTION: &str = "connection";
    const CONTENT_TYPE: &str = "content-type";
    const USER_AGENT: &str = "user-agent";

    /// Construct the request headers used for every WebDriver request (surf only).
    pub fn build_isahc_headers(server_url: &str) -> HashMap<&'static str, String> {
        let mut headers = HashMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json;charset=UTF-8".parse().unwrap());
        headers.insert(USER_AGENT, format!("thirtyfour/{} (rust)", VERSION).parse().unwrap());

        let parsed_url = urlparse(server_url);
        if let (Some(username), Some(password)) = (parsed_url.username, parsed_url.password) {
            let base64_string = encode(&format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, format!("Basic {}", base64_string).parse().unwrap());
        }
        headers.insert(CONNECTION, "keep-alive".parse().unwrap());

        headers
    }
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
    pub fn build_reqwest_headers(server_url: &str) -> WebDriverResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json;charset=UTF-8".parse().unwrap());
        headers.insert(USER_AGENT, format!("thirtyfour/{} (rust)", VERSION).parse().unwrap());

        let parsed_url = urlparse(server_url);
        if let (Some(username), Some(password)) = (parsed_url.username, parsed_url.password) {
            let base64_string = encode(&format!("{}:{}", username, password));
            headers.insert(AUTHORIZATION, format!("Basic {}", base64_string).parse().unwrap());
        }
        headers.insert(CONNECTION, "keep-alive".parse().unwrap());

        Ok(headers)
    }
}
