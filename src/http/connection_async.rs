use std::fmt::Debug;

use async_trait::async_trait;

use crate::{common::command::Command, error::WebDriverResult, SessionId};

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
    async fn execute(
        &self,
        session_id: &SessionId,
        command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value>;
}
