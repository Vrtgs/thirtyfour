#[derive(Debug, Clone)]
pub enum RequestMethod {
    Get,
    Post,
    Delete,
}

#[derive(Debug, Clone)]
pub struct RequestData {
    pub method: RequestMethod,
    pub url: String,
    pub body: Option<serde_json::Value>,
}

impl RequestData {
    pub fn new<S: Into<String>>(method: RequestMethod, url: S) -> Self {
        RequestData {
            method,
            url: url.into(),
            body: None,
        }
    }

    pub fn add_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}
