/// The request method for a request.
#[derive(Debug, Clone)]
pub enum RequestMethod {
    /// GET request.
    Get,
    /// POST request.
    Post,
    /// DELETE request.
    Delete,
}

impl From<RequestMethod> for http::Method {
    fn from(value: RequestMethod) -> Self {
        match value {
            RequestMethod::Get => http::Method::GET,
            RequestMethod::Post => http::Method::POST,
            RequestMethod::Delete => http::Method::DELETE,
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
