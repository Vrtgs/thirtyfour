use crate::common::config::WebDriverConfig;
use crate::error::WebDriverResult;
use crate::session::handle::SessionHandle;
use crate::{Capabilities, SessionId};
use std::ops::Deref;
use std::sync::Arc;

/// The `WebDriver` struct encapsulates an async Selenium WebDriver browser
/// session.
///
/// # Example:
/// ```no_run
/// use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
///
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// let caps = DesiredCapabilities::firefox();
/// // NOTE: this assumes you have a WebDriver compatible server running
/// //       at http://localhost:4444
/// //       e.g. `geckodriver -p 4444`
/// // NOTE: If using selenium 3.x, use "http://localhost:4444/wd/hub/session" for the url.
/// let driver = WebDriver::new("http://localhost:4444", caps).await?;
/// driver.goto("https://www.rust-lang.org/").await?;
/// // Always remember to close the session.
/// driver.quit().await?;
/// #         Ok(())
/// #     })
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct WebDriver {
    /// The underlying session handle.
    pub handle: Arc<SessionHandle>,
}

impl WebDriver {
    /// Create a new WebDriver as follows:
    ///
    /// # Example
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::firefox();
    /// // NOTE: this assumes you have a WebDriver compatible server running
    /// //       at http://localhost:4444
    /// //       e.g. `geckodriver -p 4444`
    /// // NOTE: If using selenium 3.x, use "http://localhost:4444/wd/hub/session" for the url.
    /// let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// ## Using Selenium Server
    /// - For selenium 3.x, you need to also add "/wd/hub/session" to the end of the url
    ///   (e.g. "http://localhost:4444/wd/hub/session")
    /// - For selenium 4.x and later, no path should be needed on the url.
    ///
    /// ## Troubleshooting
    ///
    /// - If the webdriver appears to hang or give no response, please check that the
    ///   capabilities object is of the correct type for that webdriver.
    pub async fn new<C>(server_url: &str, capabilities: C) -> WebDriverResult<Self>
    where
        C: Into<Capabilities>,
    {
        Self::new_with_config(server_url, capabilities, WebDriverConfig::default()).await
    }

    /// Create a new `WebDriver` with the specified `WebDriverConfig`.
    ///
    /// Use `WebDriverConfig::builder().build()` to construct the config.
    pub async fn new_with_config<C>(
        server_url: &str,
        capabilities: C,
        config: WebDriverConfig,
    ) -> WebDriverResult<Self>
    where
        C: Into<Capabilities>,
    {
        #[cfg(not(any(feature = "rustls-tls", feature = "native-tls")))]
        panic!("please set either the rustls-tls or native-tls feature");

        #[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
        {
            use crate::upstream::ClientBuilder;
            use crate::TimeoutConfiguration;
            let caps: Capabilities = capabilities.into();

            #[cfg(feature = "native-tls")]
            let mut builder = ClientBuilder::native();
            #[cfg(feature = "rustls-tls")]
            let mut builder = ClientBuilder::rustls();

            let client = builder.capabilities(caps.clone()).connect(server_url).await?;

            // Set default timeouts.
            let timeouts = TimeoutConfiguration::default();
            client.update_timeouts(timeouts).await?;
            let session_id = client.session_id().await?.expect("session id is not valid");

            Ok(Self {
                handle: Arc::new(SessionHandle::new_with_config(
                    client,
                    SessionId::from(session_id),
                    config,
                )?),
            })
        }
    }

    /// Clone this `WebDriver` keeping the session handle, but supplying a new `WebDriverConfig`.
    ///
    /// This still uses the same underlying client, and still controls the same browser
    /// session, but uses a different `WebDriverConfig` for this instance.
    ///
    /// This is useful in cases where you want to specify a custom poller configuration (or
    /// some other configuration option) for only one instance of `WebDriver`.
    pub fn clone_with_config(&self, config: WebDriverConfig) -> Self {
        Self {
            handle: self.handle.clone_with_config(config),
        }
    }

    /// End the webdriver session and close the browser.
    ///
    /// **NOTE:** The browser will not close automatically when `WebDriver` goes out of scope.
    ///           Thus if you intend for the browser to close once you are done with it, then
    ///           you must call this method at that point, and await it.
    pub async fn quit(self) -> WebDriverResult<()> {
        let client = self.handle.client.clone();
        client.close().await?;
        Ok(())
    }
}

/// The Deref implementation allows the WebDriver to "fall back" to SessionHandle and
/// exposes all of the methods there without requiring us to use an async_trait.
/// See documentation at the top of this module for more details on the design.
impl Deref for WebDriver {
    type Target = Arc<SessionHandle>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
