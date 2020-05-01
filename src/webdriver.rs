use std::marker::PhantomData;
use std::sync::Arc;

use futures::executor::block_on;
use log::error;
use serde::Serialize;
use serde_json::Value;

use async_trait::async_trait;

use crate::http_async::connection_async::{RemoteConnectionAsync, RemoteConnectionAsyncCreate};
#[cfg(not(any(feature = "tokio-runtime", feature = "async-std-runtime")))]
use crate::http_async::nulldriver_async::NullDriverAsync;
#[cfg(feature = "tokio-runtime")]
use crate::http_async::reqwest_async::ReqwestDriverAsync;
#[cfg(feature = "async-std-runtime")]
use crate::http_async::surf_async::SurfDriverAsync;
use crate::webdrivercommands::{start_session, WebDriverCommands, WebDriverSession};
use crate::{common::command::Command, error::WebDriverResult, DesiredCapabilities, SessionId};

#[cfg(not(any(feature = "tokio-runtime", feature = "async-std-runtime")))]
pub type WebDriver = GenericWebDriver<NullDriverAsync>;
#[cfg(feature = "tokio-runtime")]
pub type WebDriver = GenericWebDriver<ReqwestDriverAsync>;
#[cfg(feature = "async-std-runtime")]
pub type WebDriver = GenericWebDriver<SurfDriverAsync>;

/// The WebDriver struct encapsulates an async Selenium WebDriver browser
/// session. For the sync driver, see
/// [sync::WebDriver](sync/struct.WebDriver.html).
///
/// See the [WebDriverCommands](trait.WebDriverCommands.html) trait for WebDriver methods.
///
/// # Example:
/// ```rust
/// use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
///
/// fn main() -> WebDriverResult<()> {
///     block_on(async {
///         let caps = DesiredCapabilities::chrome();
///         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
///         driver.get("http://webappdemo").await?;
///         Ok(())
///     })
/// }
/// ```
#[derive(Debug)]
pub struct GenericWebDriver<T: RemoteConnectionAsync + RemoteConnectionAsyncCreate> {
    pub session_id: SessionId,
    conn: Arc<dyn RemoteConnectionAsync>,
    capabilities: Value,
    quit_on_drop: bool,
    phantom: PhantomData<T>,
}

impl<T: 'static> GenericWebDriver<T>
where
    T: RemoteConnectionAsync + RemoteConnectionAsyncCreate,
{
    /// Create a new async WebDriver struct.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn new<C>(remote_server_addr: &str, capabilities: C) -> WebDriverResult<Self>
    where
        C: Serialize,
    {
        let conn = Arc::new(T::create(remote_server_addr)?);
        let (session_id, session_capabilities) = start_session(conn.clone(), capabilities).await?;
        let driver = GenericWebDriver {
            session_id,
            conn,
            capabilities: session_capabilities,
            quit_on_drop: true,
            phantom: PhantomData,
        };

        Ok(driver)
    }

    /// Return a clone of the capabilities as returned by Selenium.
    pub fn capabilities(&self) -> DesiredCapabilities {
        DesiredCapabilities::new(self.capabilities.clone())
    }

    /// End the webdriver session.
    pub async fn quit(mut self) -> WebDriverResult<()> {
        self.cmd(Command::DeleteSession).await?;
        self.quit_on_drop = false;
        Ok(())
    }
}

#[async_trait]
impl<T> WebDriverCommands for GenericWebDriver<T>
where
    T: RemoteConnectionAsync + RemoteConnectionAsyncCreate,
{
    async fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.conn.execute(&self.session_id, command).await
    }

    fn session(&self) -> WebDriverSession {
        WebDriverSession::new(&self.session_id, self.conn.clone())
    }
}

impl<T> Drop for GenericWebDriver<T>
where
    T: RemoteConnectionAsync + RemoteConnectionAsyncCreate,
{
    /// Close the current session when the WebDriver struct goes out of scope.
    fn drop(&mut self) {
        if self.quit_on_drop && !(*self.session_id).is_empty() {
            if let Err(e) = block_on(self.cmd(Command::DeleteSession)) {
                error!("Failed to close session: {:?}", e);
            }
        }
    }
}
