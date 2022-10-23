use crate::action_chain::ActionChain;
use crate::common::config::WebDriverConfig;
use crate::error::WebDriverResult;
use crate::session::scriptret::ScriptRet;
use crate::Cookie;
use crate::Form;
use crate::{By, Rect, SessionId, SwitchTo, WebElement};
use crate::{TimeoutConfiguration, WebDriverStatus, WindowHandle};
use serde_json::Value;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// The SessionHandle contains a shared reference to the [`fantoccini::Client`]
/// to allow sending commands to the underlying WebDriver.
#[derive(Clone)]
pub struct SessionHandle {
    pub(crate) client: fantoccini::Client,
    /// The session id for this webdriver session.
    session_id: SessionId,
    /// The config used by this instance.
    config: WebDriverConfig,
}

impl Debug for SessionHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionHandle")
            .field("session_id", &self.session_id)
            .field("config", &self.config)
            .finish()
    }
}

impl SessionHandle {
    /// Create new SessionHandle from a fantoccini Client.
    pub fn new(client: fantoccini::Client, session_id: SessionId) -> WebDriverResult<Self> {
        Ok(Self {
            client,
            session_id,
            config: WebDriverConfig::default(),
        })
    }

    /// Create new `SessionHandle` with the specified `WebDriverConfig`.
    #[allow(dead_code)]
    pub(crate) fn new_with_config(
        client: fantoccini::Client,
        session_id: SessionId,
        config: WebDriverConfig,
    ) -> WebDriverResult<Self> {
        Ok(Self {
            client,
            session_id,
            config,
        })
    }

    /// Clone this session handle but attach the specified `WebDriverConfig`.
    ///
    /// See `WebDriver::clone_with_config()`.
    pub(crate) fn clone_with_config(
        self: &Arc<SessionHandle>,
        config: WebDriverConfig,
    ) -> Arc<Self> {
        Arc::new(Self {
            client: self.client.clone(),
            session_id: self.session_id.clone(),
            config,
        })
    }

    /// Convert a fantoccini `Element` into a thirtyfour `WebElement`.
    pub(crate) fn wrap_element(
        self: &Arc<SessionHandle>,
        element: fantoccini::elements::Element,
    ) -> WebElement {
        WebElement::new(element, self.clone())
    }

    /// The session id for this webdriver session.
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }

    /// The configuration used by this instance.
    ///
    /// NOTE: It's sometimes useful to have separate instances pointing at the same
    ///       underlying browser session but using different configurations.
    ///       See [`WebDriver::clone_with_config()`] for more details.
    ///
    /// [`WebDriver::clone_with_config()`]: crate::WebDriver::clone_with_config()
    pub fn config(&self) -> &WebDriverConfig {
        &self.config
    }

    /// Get the WebDriver status.
    ///
    /// # Example
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::chrome();
    /// let mut driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let status = driver.status().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn status(&self) -> WebDriverResult<WebDriverStatus> {
        Ok(self.client.status().await?)
    }

    /// Close the current window or tab. This will close the session if no other windows exist.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// // Open a new tab.
    /// driver.new_tab().await?;
    ///
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.windows().await?;
    /// driver.switch_to_window(handles[1].clone()).await?;
    ///
    /// // We are now controlling the new tab.
    /// driver.goto("https://www.rust-lang.org").await?;
    ///
    /// // Close the tab. This will return to the original tab.
    /// driver.close_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn close_window(&self) -> WebDriverResult<()> {
        self.client.close_window().await?;
        Ok(())
    }

    /// Close the current window or tab. This will close the session if no other windows exist.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to close_window()")]
    pub async fn close(&self) -> WebDriverResult<()> {
        self.client.close_window().await?;
        Ok(())
    }

    /// Navigate to the specified URL.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.goto("https://www.rust-lang.org").await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn goto<S>(&self, url: S) -> WebDriverResult<()>
    where
        S: AsRef<str>,
    {
        Ok(self.client.goto(url.as_ref()).await?)
    }

    /// Navigate to the specified URL.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to goto()")]
    pub async fn get<S>(&self, url: S) -> WebDriverResult<()>
    where
        S: AsRef<str>,
    {
        self.goto(url).await
    }

    /// Get the current URL.
    pub async fn current_url(&self) -> WebDriverResult<url::Url> {
        Ok(self.client.current_url().await?)
    }

    /// Get the page source as a String.
    pub async fn source(&self) -> WebDriverResult<String> {
        Ok(self.client.source().await?)
    }

    /// Get the page source as a String.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to source()")]
    pub async fn page_source(&self) -> WebDriverResult<String> {
        self.source().await
    }

    /// Get the page title as a String.
    pub async fn title(&self) -> WebDriverResult<String> {
        Ok(self.client.title().await?)
    }

    /// Search for an element on the current page using the specified selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    ///           [`WebDriver::query`] method instead.
    ///
    /// [`WebDriver::query`]: crate::extensions::query::ElementQueryable::query
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let elem_button = driver.find(By::Id("my-element-id")).await?;
    /// let elem_text = driver.find(By::Name("my-text-input")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find(self: &Arc<SessionHandle>, by: impl Into<By>) -> WebDriverResult<WebElement> {
        let by = by.into();
        let elem = self.client.find(by.locator()).await?;
        Ok(self.wrap_element(elem))
    }

    /// Search for an element on the current page using the specified selector.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to find()")]
    pub async fn find_element(self: &Arc<SessionHandle>, by: By) -> WebDriverResult<WebElement> {
        self.find(by).await
    }

    /// Search for all elements on the current page that match the specified selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    ///           [`WebDriver::query`] method instead.
    ///
    /// [`WebDriver::query`]: crate::extensions::query::ElementQueryable::query
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let elems = driver.find_all(By::ClassName("section")).await?;
    /// for elem in elems {
    ///     assert!(elem.attr("class").await?.expect("Missing class on element").contains("section"));
    /// }
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find_all(
        self: &Arc<SessionHandle>,
        by: impl Into<By>,
    ) -> WebDriverResult<Vec<WebElement>> {
        let by = by.into();
        let elems = self.client.find_all(by.locator()).await?;
        Ok(elems.into_iter().map(|x| self.wrap_element(x)).collect())
    }

    /// Search for all elements on the current page that match the specified selector.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to find_all()")]
    pub async fn find_elements(
        self: &Arc<SessionHandle>,
        by: By,
    ) -> WebDriverResult<Vec<WebElement>> {
        self.find_all(by).await
    }

    /// Locate a form on the page.
    ///
    /// Through the returned `Form`, HTML forms can be filled out and submitted.
    pub async fn form(&self, by: impl Into<By>) -> WebDriverResult<Form> {
        let by = by.into();
        let form = self.client.form(by.locator()).await?;
        Ok(form)
    }

    /// Execute the specified Javascript synchronously and return the result.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let ret = driver.execute(r#"
    ///     let elem = document.getElementById("button1");
    ///     elem.click();
    ///     return elem;
    ///     "#, Vec::new()
    /// ).await?;
    /// let elem_out: WebElement = ret.element()?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    /// To supply an element as an input argument to a script, use
    /// [`WebElement::to_json`] as follows:
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let elem = driver.find(By::Id("button1")).await?;
    /// let ret = driver.execute(r#"
    ///     arguments[0].innerHTML = arguments[1];
    ///     return arguments[0];
    ///     "#, vec![elem.to_json()?, serde_json::to_value("TESTING")?]
    /// ).await?;
    /// let elem_out = ret.element()?;
    /// assert_eq!(elem_out.element_id(), elem.element_id());
    /// assert_eq!(elem_out.text().await?, "TESTING");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute(
        self: &Arc<SessionHandle>,
        script: &str,
        args: Vec<Value>,
    ) -> WebDriverResult<ScriptRet> {
        let v = self.client.execute(script, args).await?;
        Ok(ScriptRet::new(self.clone(), v))
    }

    /// Execute the specified Javascript synchronously and return the result.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to execute()")]
    pub async fn execute_script(
        self: &Arc<SessionHandle>,
        script: &str,
        args: Vec<Value>,
    ) -> WebDriverResult<ScriptRet> {
        self.execute(script, args).await
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let ret = driver.execute_async(r#"
    ///     // Selenium automatically provides an extra argument which is a
    ///     // function that receives the return value(s).
    ///     let done = arguments[0];
    ///     window.setTimeout(() => {
    ///         let elem = document.getElementById("button1");
    ///         elem.click();
    ///         done(elem);
    ///     }, 1000);
    ///     "#, Vec::new()
    /// ).await?;
    /// let elem_out: WebElement = ret.element()?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    /// To supply an element as an input argument to a script, use
    /// [`WebElement::to_json`] as follows:
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let elem = driver.find(By::Id("button1")).await?;
    /// let args = vec![elem.to_json()?, serde_json::to_value("TESTING")?];
    /// let ret = driver.execute_async(r#"
    ///     // Selenium automatically provides an extra argument which is a
    ///     // function that receives the return value(s).
    ///     let done = arguments[2];
    ///     window.setTimeout(() => {
    ///         arguments[0].innerHTML = arguments[1];
    ///         done(arguments[0]);
    ///     }, 1000);
    ///     "#, args
    /// ).await?;
    /// let elem_out = ret.element()?;
    /// assert_eq!(elem_out.element_id(), elem.element_id());
    /// assert_eq!(elem_out.text().await?, "TESTING");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute_async(
        self: &Arc<SessionHandle>,
        script: &str,
        args: Vec<Value>,
    ) -> WebDriverResult<ScriptRet> {
        let v = self.client.execute_async(script, args).await?;
        Ok(ScriptRet::new(self.clone(), v))
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to execute_async()")]
    pub async fn execute_script_async(
        self: &Arc<SessionHandle>,
        script: &str,
        args: Vec<Value>,
    ) -> WebDriverResult<ScriptRet> {
        self.execute_async(script, args).await
    }

    /// Get the current window handle.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// // Get the current window handle.
    /// let handle = driver.window().await?;
    ///
    /// // Open a new tab.
    /// driver.new_tab().await?;
    ///
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.windows().await?;
    /// driver.switch_to_window(handles[1].clone()).await?;
    ///
    /// // We are now controlling the new tab.
    /// driver.goto("https://www.rust-lang.org/").await?;
    /// assert_ne!(driver.window().await?, handle);
    ///
    /// // Switch back to original tab.
    /// driver.switch_to_window(handle.clone()).await?;
    /// assert_eq!(driver.window().await?, handle);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn window(&self) -> WebDriverResult<WindowHandle> {
        Ok(self.client.window().await?)
    }

    /// Get the current window handle.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to window()")]
    pub async fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        self.window().await
    }

    /// Get all window handles for the current session.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// assert_eq!(driver.windows().await?.len(), 1);
    /// // Open a new tab.
    /// driver.new_tab().await?;
    ///
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.windows().await?;
    /// assert_eq!(handles.len(), 2);
    /// driver.switch_to_window(handles[1].clone()).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn windows(&self) -> WebDriverResult<Vec<WindowHandle>> {
        Ok(self.client.windows().await?)
    }

    /// Get all window handles for the current session.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to windows()")]
    pub async fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        self.windows().await
    }

    /// Maximize the current window.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.maximize_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn maximize_window(&self) -> WebDriverResult<()> {
        self.client.maximize_window().await?;
        Ok(())
    }

    /// Minimize the current window.
    ///
    /// # Example:
    /// ```no_run
    /// # // Minimize is not currently working on Chrome, but does work
    /// # // on Firefox/geckodriver.
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.minimize_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn minimize_window(&self) -> WebDriverResult<()> {
        self.client.minimize_window().await?;
        Ok(())
    }

    /// Make the current window fullscreen.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.fullscreen_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.client.fullscreen_window().await?;
        Ok(())
    }

    /// Get the current window rectangle, in pixels.
    ///
    /// The returned Rect struct has members `x`, `y`, `width`, `height`,
    /// all i32.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::Rect;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.set_window_rect(0, 0, 600, 400).await?;
    /// let rect = driver.get_window_rect().await?;
    /// assert_eq!(rect, Rect::new(0, 0, 600, 400));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_window_rect(&self) -> WebDriverResult<Rect> {
        let (x, y, w, h) = self.client.get_window_rect().await?;
        Ok(Rect::new(x as i64, y as i64, w as i64, h as i64))
    }

    /// Set the current window rectangle, in pixels.
    ///
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.set_window_rect(0, 0, 500, 400).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_window_rect(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> WebDriverResult<()> {
        Ok(self.client.set_window_rect(x, y, width, height).await?)
    }

    /// Go back. This is equivalent to clicking the browser's back button.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.back().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn back(&self) -> WebDriverResult<()> {
        Ok(self.client.back().await?)
    }

    /// Go forward. This is equivalent to clicking the browser's forward button.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.forward().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn forward(&self) -> WebDriverResult<()> {
        self.client.forward().await?;
        Ok(())
    }

    /// Refresh the current page.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.refresh().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn refresh(&self) -> WebDriverResult<()> {
        Ok(self.client.refresh().await?)
    }

    /// Get all timeouts for the current session.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let timeouts = driver.get_timeouts().await?;
    /// println!("Page load timeout = {:?}", timeouts.page_load());
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_timeouts(&self) -> WebDriverResult<TimeoutConfiguration> {
        let timeouts = self.client.get_timeouts().await?;
        Ok(timeouts)
    }

    /// Set all timeouts for the current session.
    ///
    /// **NOTE:** Setting the implicit wait timeout to a non-zero value will interfere with the use
    /// of [`WebDriver::query`] and [`WebElement::wait_until`].
    /// It is therefore recommended to use these methods (which provide polling
    /// and explicit waits) instead rather than increasing the implicit wait timeout.
    ///
    /// [`WebDriver::query`]: crate::extensions::query::ElementQueryable::query
    /// [`WebElement::wait_until`]: crate::extensions::query::ElementWaitable::wait_until
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// // Setting timeouts to None means those timeout values will not be updated.
    /// let timeouts = TimeoutConfiguration::new(None, Some(Duration::new(11, 0)), None);
    /// driver.update_timeouts(timeouts).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn update_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.client.update_timeouts(timeouts).await?;
        Ok(())
    }

    /// Set all timeouts for the current session.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to update_timeouts()")]
    pub async fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.update_timeouts(timeouts).await
    }

    /// Set the implicit wait timeout.
    ///
    /// This is how long the WebDriver will wait when querying elements.
    /// By default this is set to 0 seconds.
    ///
    /// **NOTE:** Setting the implicit wait timeout to a non-zero value will interfere with the use
    /// of [`WebDriver::query`] and [`WebElement::wait_until`].
    /// It is therefore recommended to use these methods (which provide polling
    /// and explicit waits) instead rather than increasing the implicit wait timeout.
    ///
    /// [`WebDriver::query`]: crate::extensions::query::ElementQueryable::query
    /// [`WebElement::wait_until`]: crate::extensions::query::ElementWaitable::wait_until
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let delay = Duration::new(11, 0);
    /// driver.set_implicit_wait_timeout(delay).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_implicit_wait_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.update_timeouts(timeouts).await
    }

    /// Set the script timeout.
    ///
    /// This is how long the WebDriver will wait for a Javascript script to execute.
    /// By default this is set to 60 seconds.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let delay = Duration::new(11, 0);
    /// driver.set_script_timeout(delay).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.update_timeouts(timeouts).await
    }

    /// Set the page load timeout.
    ///
    /// This is how long the WebDriver will wait for the page to finish loading.
    /// By default this is set to 60 seconds.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let delay = Duration::new(11, 0);
    /// driver.set_page_load_timeout(delay).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.update_timeouts(timeouts).await
    }

    /// Create a new action chain for this session.
    ///
    /// Action chains can be used to simulate more complex user input actions
    /// involving key combinations, mouse movements, mouse click, right-click,
    /// and more.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let elem_text = driver.find(By::Name("input1")).await?;
    /// let elem_button = driver.find(By::Id("button-set")).await?;
    ///
    /// driver.action_chain()
    ///     .send_keys_to_element(&elem_text, "thirtyfour")
    ///     .move_to_element_center(&elem_button)
    ///     .click()
    ///     .perform()
    ///     .await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn action_chain(self: &Arc<SessionHandle>) -> ActionChain {
        ActionChain::new(self.clone())
    }

    /// Create a new Actions chain.
    ///
    /// Also see [`WebDriver::action_chain`] for a builder-based alternative.
    ///
    /// [`WebDriver::action_chain`]: SessionHandle::action_chain
    ///
    /// ```ignore
    /// let mouse_actions = MouseActions::new("mouse")
    ///     .then(PointerAction::Down {
    ///         button: MOUSE_BUTTON_LEFT,
    ///     })
    ///     .then(PointerAction::MoveBy {
    ///         duration: Some(Duration::from_secs(2)),
    ///         x: 100,
    ///         y: 0,
    ///     })
    ///     .then(PointerAction::Up {
    ///         button: MOUSE_BUTTON_LEFT,
    ///     });
    /// client.perform_actions(mouse_actions).await?;
    /// ```
    ///
    /// See the documentation for [`Actions`] for more information.
    /// Perform the specified input actions.
    ///
    /// [`Actions`]: fantoccini::actions::Actions
    pub async fn perform_actions(
        &self,
        actions: impl Into<crate::actions::Actions>,
    ) -> WebDriverResult<()> {
        self.client.perform_actions(actions).await?;
        Ok(())
    }

    /// Get all cookies.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::cookie::SameSite;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let cookies = driver.get_all_cookies().await?;
    /// for cookie in &cookies {
    ///     println!("Got cookie: {}", cookie.value());
    /// }
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_all_cookies(&self) -> WebDriverResult<Vec<Cookie<'static>>> {
        Ok(self.client.get_all_cookies().await?)
    }

    /// Get all cookies.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to get_all_cookies()")]
    pub async fn get_cookies(&self) -> WebDriverResult<Vec<Cookie<'static>>> {
        self.get_all_cookies().await
    }

    /// Get the specified cookie.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::cookie::SameSite;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let cookie = driver.get_named_cookie("key").await?;
    /// println!("Got cookie: {}", cookie.value());
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_named_cookie(&self, name: &str) -> WebDriverResult<Cookie<'static>> {
        Ok(self.client.get_named_cookie(name).await?)
    }

    /// Get the specified cookie.
    #[deprecated(since = "0.30.0", note = "This method has been renamed to get_named_cookie()")]
    pub async fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie<'static>> {
        self.get_named_cookie(name).await
    }

    /// Delete the specified cookie.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::cookie::SameSite;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.delete_cookie("key").await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn delete_cookie(&self, name: &str) -> WebDriverResult<()> {
        Ok(self.client.delete_cookie(name).await?)
    }

    /// Delete all cookies.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::cookie::SameSite;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.delete_all_cookies().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn delete_all_cookies(&self) -> WebDriverResult<()> {
        Ok(self.client.delete_all_cookies().await?)
    }

    /// Add the specified cookie.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::cookie::SameSite;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.goto("https://wikipedia.org").await?;
    /// let mut cookie = Cookie::new("key", "value");
    /// cookie.set_domain("wikipedia.org");
    /// cookie.set_path("/");
    /// cookie.set_same_site(Some(SameSite::Lax));
    /// driver.add_cookie(cookie.clone()).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn add_cookie(&self, cookie: Cookie<'static>) -> WebDriverResult<()> {
        self.client.add_cookie(cookie).await?;
        Ok(())
    }

    /// Take a screenshot of the current window and return it as PNG bytes.
    pub async fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        Ok(self.client.screenshot().await?)
    }

    /// Take a screenshot of the current window and write it to the specified filename.
    pub async fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png().await?;
        let mut file = File::create(path).await?;
        file.write_all(&png).await?;
        Ok(())
    }

    /// Return a SwitchTo struct for switching to another window or frame.
    #[deprecated(
        since = "0.30.0",
        note = "SwitchTo has been deprecated. Use WebDriver::switch_to_*() methods instead"
    )]
    pub fn switch_to(self: &Arc<SessionHandle>) -> SwitchTo {
        SwitchTo::new(self.clone())
    }

    /// Set the current window name.
    ///
    /// Useful for switching between windows/tabs using [`WebDriver::switch_to_named_window`]
    ///
    /// [`WebDriver::switch_to_named_window`]: SessionHandle::switch_to_named_window
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// // Get the current window handle.
    /// let handle = driver.window().await?;
    /// driver.set_window_name("main").await?;
    ///
    /// // Open a new tab.
    /// let new_handle = driver.new_tab().await?;
    ///
    /// // Get window handles and switch to the new tab.
    /// driver.switch_to_window(new_handle).await?;
    ///
    /// // We are now controlling the new tab.
    /// driver.goto("https://www.rust-lang.org").await?;
    /// assert_ne!(driver.window().await?, handle);
    ///
    /// // Switch back to original tab using window name.
    /// driver.switch_to_named_window("main").await?;
    /// assert_eq!(driver.window().await?, handle);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_window_name(
        self: &Arc<SessionHandle>,
        window_name: &str,
    ) -> WebDriverResult<()> {
        let script = format!(r#"window.name = "{}""#, window_name);
        self.execute(&script, Vec::new()).await?;
        Ok(())
    }

    /// Execute the specified function in a new browser tab, closing the tab when complete.
    ///
    /// The return value will be that of the supplied function, unless an error occurs while
    /// opening or closing the tab.
    ///
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let window_title = driver.in_new_tab(|| async {
    ///     driver.goto("https://www.google.com").await?;
    ///     driver.title().await
    /// }).await?;
    /// assert_eq!(window_title, "Google");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn in_new_tab<F, Fut, T>(&self, f: F) -> WebDriverResult<T>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = WebDriverResult<T>> + Send,
        T: Send,
    {
        let handle = self.window().await?;

        // Open new tab.
        let tab_handle = self.new_tab().await?;
        self.switch_to_window(tab_handle).await?;

        let result = f().await;

        // Close tab.
        self.close_window().await?;
        self.switch_to_window(handle).await?;

        result
    }
}
