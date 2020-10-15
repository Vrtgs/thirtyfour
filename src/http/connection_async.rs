use std::fmt::Debug;

use async_trait::async_trait;

use crate::error::WebDriverResult;

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

/// Trait for executing HTTP requests to selenium/webdriver.
/// As long as you have some struct that implements WebDriverHttpClientAsync
/// you can turn it into a WebDriver like this:
///
/// ```ignore
/// // Assuming MyHttpClient implements WebDriverHttpClientAsync.
/// pub type MyWebDriver = GenericWebDriver<MyHttpClient>;
/// ```
#[async_trait]
pub trait WebDriverHttpClientAsync: Debug + Send + Sync {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self>
    where
        Self: Sized;

    async fn execute(&self, request_data: RequestData) -> WebDriverResult<serde_json::Value>;
}
