use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// The request method for a request.
#[derive(Debug, Clone, strum::EnumString, strum::Display, Serialize, Deserialize)]
#[strum(serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum RequestMethod {
    /// CONNECT request.
    Connect,
    /// GET request.
    Get,
    /// DELETE request.
    Delete,
    /// HEAD request.
    Head,
    /// OPTIONS request.
    Options,
    /// PATCH request.
    Patch,
    /// POST request.
    Post,
    /// PUT request.
    Put,
    /// TRACE request.
    Trace,
}

impl From<RequestMethod> for http::Method {
    fn from(value: RequestMethod) -> Self {
        match value {
            RequestMethod::Connect => http::Method::CONNECT,
            RequestMethod::Get => http::Method::GET,
            RequestMethod::Delete => http::Method::DELETE,
            RequestMethod::Head => http::Method::HEAD,
            RequestMethod::Options => http::Method::OPTIONS,
            RequestMethod::Patch => http::Method::PATCH,
            RequestMethod::Post => http::Method::POST,
            RequestMethod::Put => http::Method::PUT,
            RequestMethod::Trace => http::Method::TRACE,
        }
    }
}

/// RequestData is a wrapper around the data required to make an HTTP request.
#[derive(Debug, Clone)]
pub struct RequestData {
    /// The request method.
    pub method: RequestMethod,
    /// The request URI.
    pub uri: String,
    /// The request body.
    pub body: Option<serde_json::Value>,
}

impl RequestData {
    /// Create a new RequestData struct.
    pub fn new<S: Into<String>>(method: RequestMethod, uri: S) -> Self {
        RequestData {
            method,
            uri: uri.into(),
            body: None,
        }
    }

    /// Add a request body.
    pub fn add_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

impl Display for RequestData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(body) = &self.body {
            write!(
                f,
                "{} {} {}",
                self.method,
                self.uri,
                serde_json::to_string(body).unwrap_or_default()
            )
        } else {
            write!(f, "{} {}", self.method, self.uri)
        }
    }
}
