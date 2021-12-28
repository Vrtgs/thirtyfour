use std::ops::{Deref, DerefMut};

use crate::http::connection_async::{HttpClientCreateParams, WebDriverHttpClientAsync};

use crate::error::WebDriverError;
use crate::runtime::imports::HttpClientAsync;
use crate::session::handle::SessionHandle;
use crate::session::start::start_session;
use crate::{common::command::Command, error::WebDriverResult};
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WebDriverBuilder {
    create_params: HttpClientCreateParams,
    capabilities: Result<serde_json::Value, String>,
}

impl WebDriverBuilder {
    /// Create a new WebDriverBuilder instance. You can use this to
    /// customize the WebDriver parameters before building it.
    pub fn new<C>(server_url: &str, capabilities: C) -> Self
    where
        C: Serialize,
    {
        Self {
            create_params: HttpClientCreateParams {
                server_url: server_url.to_string(),
                timeout: Some(Duration::from_secs(120)),
            },
            capabilities: serde_json::to_value(capabilities).map_err(|e| e.to_string()),
        }
    }

    /// Set the request timeout for the HTTP client.
    pub fn timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.create_params.timeout = timeout;
        self
    }

    /// Build a WebDriver instance that uses the default HTTP client.
    pub async fn build<'a>(&self) -> WebDriverResult<WebDriver> {
        self.build_custom_client::<HttpClientAsync>().await
    }

    /// Build a WebDriver instance that uses a HTTP client that implements WebDriverHttpClientAsync.
    pub async fn build_custom_client<T: 'static + WebDriverHttpClientAsync>(
        &self,
    ) -> WebDriverResult<WebDriver> {
        let capabilities = self.capabilities.clone().map_err(WebDriverError::SessionCreateError)?;
        let http_client = T::create(self.create_params.clone())?;
        let handle = start_session(Box::new(http_client), capabilities).await?;
        Ok(WebDriver {
            handle,
        })
    }
}

/// The `WebDriver` struct encapsulates an async Selenium WebDriver browser
/// session.
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
///         driver.quit().await?;
///         Ok(())
///     })
/// }
/// ```
#[derive(Debug)]
pub struct WebDriver {
    pub handle: SessionHandle,
}

impl WebDriver {
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
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// **NOTE:** If the webdriver appears to hang or give no response, please check that the
    ///     capabilities object is of the correct type for that webdriver.
    pub async fn new<C>(server_url: &str, capabilities: C) -> WebDriverResult<Self>
    where
        C: Serialize,
    {
        WebDriverBuilder::new(server_url, capabilities).build().await
    }

    /// Creates a new WebDriver just like the `new` function. Allows a
    /// configurable timeout for all HTTP requests including the session creation.
    ///
    /// Create a new WebDriver as follows:
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// # use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new_with_timeout("http://localhost:4444/wd/hub", &caps, Some(Duration::from_secs(120))).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn new_with_timeout<C>(
        server_url: &str,
        capabilities: C,
        timeout: Option<Duration>,
    ) -> WebDriverResult<Self>
    where
        C: Serialize,
    {
        WebDriverBuilder::new(server_url, capabilities).timeout(timeout).build().await
    }

    pub async fn new_with_client<C>(
        client: Box<dyn WebDriverHttpClientAsync>,
        capabilities: C,
    ) -> WebDriverResult<Self>
    where
        C: Serialize,
    {
        let caps = serde_json::to_value(capabilities)
            .map_err(|e| WebDriverError::SessionCreateError(e.to_string()))?;
        let handle = start_session(client, caps).await?;
        Ok(Self {
            handle,
        })
    }

    /// End the webdriver session and close the browser.
    ///
    /// **NOTE:** The browser will not close automatically when `WebDriver` goes out of scope.
    ///           Thus if you intend for the browser to close once you are done with it, then
    ///           you must call this method at that point, and await it.
    pub async fn quit(self) -> WebDriverResult<()> {
        self.handle.cmd(Command::DeleteSession).await?;
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
