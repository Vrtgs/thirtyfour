use crate::error::WebDriverResult;
use crate::session::handle::SessionHandle;
use crate::Capabilities;
use std::ops::{Deref, DerefMut};

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
#[derive(Clone)]
pub struct WebDriver {
    pub handle: SessionHandle,
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
    #[allow(unused_variables)]
    pub async fn new<C>(server_url: &str, capabilities: C) -> WebDriverResult<Self>
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

            Ok(Self {
                handle: SessionHandle::new(client).await?,
            })
        }
    }

    // /// Creates a new WebDriver just like the `new` function. Allows a
    // /// configurable timeout for all HTTP requests including the session creation.
    // ///
    // /// Create a new WebDriver as follows:
    // ///
    // /// # Example
    // /// ```no_run
    // /// # use thirtyfour::prelude::*;
    // /// # use thirtyfour::support::block_on;
    // /// # use std::time::Duration;
    // /// #
    // /// # fn main() -> WebDriverResult<()> {
    // /// #     block_on(async {
    // /// let caps = DesiredCapabilities::chrome();
    // /// let driver = WebDriver::new_with_timeout("http://localhost:4444", &caps, Some(Duration::from_secs(120))).await?;
    // /// #         driver.quit().await?;
    // /// #         Ok(())
    // /// #     })
    // /// # }
    // /// ```
    // pub async fn new_with_timeout<C>(
    //     _server_url: &str,
    //     _capabilities: C,
    //     _timeout: Option<Duration>,
    // ) -> WebDriverResult<Self>
    // where
    //     C: Into<Capabilities>,
    // {
    //     unimplemented!()
    // }

    /// End the webdriver session and close the browser.
    ///
    /// **NOTE:** The browser will not close automatically when `WebDriver` goes out of scope.
    ///           Thus if you intend for the browser to close once you are done with it, then
    ///           you must call this method at that point, and await it.
    pub async fn quit(self) -> WebDriverResult<()> {
        self.handle.client.close().await?;
        Ok(())
    }
}

/// The Deref implementation allows the WebDriver to "fall back" to SessionHandle and
/// exposes all of the methods there without requiring us to use an async_trait.
/// See documentation at the top of this module for more details on the design.
impl Deref for WebDriver {
    type Target = SessionHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for WebDriver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}
