use std::fmt::Debug;

use async_trait::async_trait;

use crate::error::WebDriverResult;
use crate::RequestData;
use std::time::Duration;

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

    fn set_request_timeout(&mut self, timeout: Duration);

    async fn execute(&self, request_data: RequestData) -> WebDriverResult<serde_json::Value>;
}
