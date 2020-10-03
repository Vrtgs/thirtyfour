use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use futures::executor::block_on;
use log::error;
use serde::Serialize;
use serde_json::Value;

use crate::common::config::WebDriverConfig;
use crate::http::connection_async::WebDriverHttpClientAsync;
#[cfg(not(any(feature = "tokio-runtime", feature = "async-std-runtime")))]
use crate::http::nulldriver_async::NullDriverAsync;
#[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
use crate::http::reqwest_async::ReqwestDriverAsync;
#[cfg(feature = "async-std-runtime")]
use crate::http::surf_async::SurfDriverAsync;
use crate::webdrivercommands::{start_session, WebDriverCommands};
use crate::{
    common::command::Command, error::WebDriverResult, session::WebDriverSession,
    DesiredCapabilities, SessionId,
};

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
#[derive(Debug)]
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
        let conn = Arc::new(T::create(remote_server_addr)?);
        let (session_id, session_capabilities) = start_session(conn.clone(), capabilities).await?;
        let driver = GenericWebDriver {
            session: WebDriverSession::new(session_id, conn),
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
