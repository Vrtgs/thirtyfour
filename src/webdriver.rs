use async_trait::async_trait;
use log::error;
use serde::Serialize;
use serde_json::Value;
use std::marker::PhantomData;

use crate::common::config::WebDriverConfig;
use crate::http::connection_async::WebDriverHttpClientAsync;
#[cfg(not(any(feature = "tokio-runtime", feature = "async-std-runtime")))]
use crate::http::nulldriver_async::NullDriverAsync;
#[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
use crate::http::reqwest_async::ReqwestDriverAsync;
#[cfg(feature = "async-std-runtime")]
use crate::http::surf_async::SurfDriverAsync;
use crate::session::spawn_session_task;
use crate::webdrivercommands::{start_session, WebDriverCommands};
use crate::{
    common::command::Command, error::WebDriverResult, session::WebDriverSession,
    DesiredCapabilities, SessionId,
};
use futures::executor::block_on;
use std::time::Duration;

#[cfg(not(any(feature = "tokio-runtime", feature = "async-std-runtime")))]
/// The WebDriver struct represents a browser session.
///
/// For full documentation of all WebDriver methods,
/// see the [WebDriverCommands](trait.WebDriverCommands.html) trait.
pub type WebDriver = GenericWebDriver<NullDriverAsync>;
#[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
/// The WebDriver struct represents a browser session.
///
/// For full documentation of all WebDriver methods,
/// see the [WebDriverCommands](trait.WebDriverCommands.html) trait.
pub type WebDriver = GenericWebDriver<ReqwestDriverAsync>;
#[cfg(feature = "async-std-runtime")]
/// The WebDriver struct represents a browser session.
///
/// For full documentation of all WebDriver methods,
/// see the [WebDriverCommands](trait.WebDriverCommands.html) trait.
pub type WebDriver = GenericWebDriver<SurfDriverAsync>;

/// **NOTE:** For WebDriver method documentation,
/// see the [WebDriverCommands](trait.WebDriverCommands.html) trait.
///
/// The `thirtyfour` crate uses a generic struct that implements the
/// `WebDriverCommands` trait. The generic struct is then implemented for
/// a specific HTTP client. This enables `thirtyfour` to support different
/// HTTP clients in order to target different async runtimes. If you do not
/// require a specific async runtime or if you are using tokio then the
/// default will work fine.
///
/// The `GenericWebDriver` struct encapsulates an async Selenium WebDriver browser
/// session. For the sync driver, see
/// [sync::GenericWebDriver](sync/struct.GenericWebDriver.html).
///
/// # Example:
/// ```rust
/// use thirtyfour::prelude::*;
/// use thirtyfour::support::block_on;
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
#[derive(Clone, Debug)]
pub struct GenericWebDriver<T: WebDriverHttpClientAsync> {
    pub session: WebDriverSession,
    capabilities: Value,
    quit_on_drop: bool,
    phantom: PhantomData<T>,
}

impl<T: 'static> GenericWebDriver<T>
where
    T: WebDriverHttpClientAsync,
{
    /// The GenericWebDriver struct is not intended to be created directly.
    ///
    /// Instead you would use the WebDriver struct, which wires up the
    /// GenericWebDriver with a HTTP client for making requests to the
    /// WebDriver server.
    ///
    /// Create a new WebDriver as follows:
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
        let conn = T::create(remote_server_addr)?;
        let (session_id, session_capabilities) = start_session(&conn, capabilities).await?;
        let tx = spawn_session_task(Box::new(conn));

        let driver = GenericWebDriver {
            session: WebDriverSession::new(session_id, tx),
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

    pub fn session_id(&self) -> &SessionId {
        self.session.session_id()
    }

    pub fn config(&self) -> &WebDriverConfig {
        self.session.config()
    }

    pub fn config_mut(&mut self) -> &mut WebDriverConfig {
        self.session.config_mut()
    }

    /// Set the request timeout for the HTTP client.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use std::time::Duration;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::chrome();
    /// let mut driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// driver.set_request_timeout(Duration::from_secs(180)).await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_request_timeout(&mut self, timeout: Duration) -> WebDriverResult<()> {
        self.session.set_request_timeout(timeout).await
    }
}

#[async_trait]
impl<T> WebDriverCommands for GenericWebDriver<T>
where
    T: WebDriverHttpClientAsync,
{
    fn session(&self) -> &WebDriverSession {
        &self.session
    }
}

impl<T> Drop for GenericWebDriver<T>
where
    T: WebDriverHttpClientAsync,
{
    /// Close the current session when the WebDriver struct goes out of scope.
    fn drop(&mut self) {
        if self.quit_on_drop && !self.session.session_id().is_empty() {
            if let Err(e) = block_on(self.cmd(Command::DeleteSession)) {
                error!("Failed to close session: {:?}", e);
            }
        }
    }
}
