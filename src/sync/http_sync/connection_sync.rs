use std::fmt::Debug;

use crate::{common::command::Command, error::WebDriverResult, SessionId};

/// Trait for executing HTTP requests to selenium/webdriver.
/// As long as you have some struct that implements RemoteConnectionSync
/// and RemoteConnectionSyncCreate, you can turn it into a WebDriver like
/// this:
///
/// ```ignore
/// // Assuming MyHttpClient implements RemoteConnectionSync and RemoteConnectionSyncCreate.
/// pub type MyWebDriver = GenericWebDriver<MyHttpClient>;
/// ```
pub trait RemoteConnectionSync: Debug + Send + Sync {
    fn execute(
        &self,
        session_id: &SessionId,
        command: Command<'_>,
    ) -> WebDriverResult<serde_json::Value>;
}

/// Trait for creating a RemoteConnectionAsync trait object generically.
pub trait RemoteConnectionSyncCreate: Sized {
    fn create(remote_server_addr: &str) -> WebDriverResult<Self>;
}
