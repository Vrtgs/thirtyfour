use std::fmt::Debug;

use async_trait::async_trait;

use crate::{common::command::Command, error::WebDriverResult, SessionId};

/// Trait for executing HTTP requests to selenium/webdriver.
/// As long as you have some struct that implements RemoteConnectionAsync
/// and RemoteConnectionAsyncCreate, you can turn it into a WebDriver like
/// this:
///
/// ```ignore
/// // Assuming MyHttpClient implements RemoteConnectionAsync and RemoteConnectionAsyncCreate.
/// pub type MyWebDriver = GenericWebDriver<MyHttpClient>;
/// ```
#[async_trait]
pub trait RemoteConnectionAsync: Debug + Send + Sync {
    async fn execute(
        &self,
        session_id: &SessionId,
        command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value>;
}

/// Trait for creating a RemoteConnectionAsync trait object generically.
pub trait RemoteConnectionAsyncCreate: Sized {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self>;
}
