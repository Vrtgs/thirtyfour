use serde_json::Value;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::path::Path;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use fantoccini::cookies::Cookie;
use fantoccini::elements::Element;
use fantoccini::error::CmdError;
use fantoccini::wd::{Capabilities, TimeoutConfiguration, WebDriverStatus, WindowHandle};

use crate::action_chain::ActionChain;
use crate::common::config::WebDriverConfig;
use crate::error::{WebDriverError, WebDriverResult};
use crate::session::scriptret::ScriptRet;
use crate::{By, Rect, SessionId, SwitchTo, WebElement};

/// The SessionHandle contains a shared reference to the [`WebDriverConfig`] as well
/// as the [`fantoccini::Client`] to allow sending commands to the underlying WebDriver.
#[derive(Clone)]
pub struct SessionHandle {
    pub(crate) client: fantoccini::Client,
    pub config: WebDriverConfig,
}

impl Debug for SessionHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.config.get_session_id())
    }
}

impl SessionHandle {
    /// Create new SessionHandle from a fantoccini Client.
    #[allow(dead_code)]
    pub(crate) async fn new(
        client: fantoccini::Client,
        capabilities: Capabilities,
    ) -> WebDriverResult<Self> {
        let session_id = client.session_id().await?.expect("session id to be valid");
        Ok(Self {
            client,
            config: WebDriverConfig::new(SessionId::from(session_id), capabilities),
        })
    }

    fn wrap_element(&self, element: Element) -> WebElement {
        WebElement::new(element, self.clone())
    }

    /// Return a clone of the capabilities as originally requested.
    pub fn capabilities(&mut self) -> Capabilities {
        self.config.get_capabilities()
    }

    /// Get the session ID.
    pub async fn session_id(&self) -> WebDriverResult<SessionId> {
        let id = self.client.session_id().await?;
        Ok(SessionId::from(id.unwrap_or_default()))
    }

    /// Get a clone of the `WebDriverConfig`. You can update the config by modifying
    /// it directly.
    pub fn config(&self) -> WebDriverConfig {
        self.config.clone()
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

    // /// Set the request timeout for the HTTP client.
    // ///
    // /// # Example
    // /// ```no_run
    // /// # use thirtyfour::prelude::*;
    // /// # use std::time::Duration;
    // /// # use thirtyfour::support::block_on;
    // /// #
    // /// # fn main() -> WebDriverResult<()> {
    // /// #     block_on(async {
    // /// let caps = DesiredCapabilities::chrome();
    // /// let mut driver = WebDriver::new("http://localhost:4444", caps).await?;
    // /// driver.set_request_timeout(Duration::from_secs(180)).await?;
    // /// #         driver.quit().await?;
    // /// #         Ok(())
    // /// #     })
    // /// # }
    // /// ```
    // pub async fn set_request_timeout(&mut self, _timeout: Duration) -> WebDriverResult<()> {
    //     unimplemented!()
    // }

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
    /// #         driver.get("http://webappdemo").await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#, Vec::new()).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(handles[1].clone()).await?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo").await?;
    /// // Close the tab. This will return to the original tab.
    /// driver.close().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
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
    /// driver.get("http://webappdemo").await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get<S>(&self, url: S) -> WebDriverResult<()>
    where
        S: AsRef<str>,
    {
        Ok(self.client.goto(url.as_ref()).await?)
    }

    /// Get the current URL as a String.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use url::Url;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// driver.get("http://webappdemo").await?;
    /// let url = driver.current_url().await?;
    /// #         assert_eq!(url, Url::parse("http://webappdemo/")?);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn current_url(&self) -> WebDriverResult<url::Url> {
        Ok(self.client.current_url().await?)
    }

    /// Get the page source as a String.
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
    /// driver.get("http://webappdemo").await?;
    /// let source = driver.page_source().await?;
    /// #         assert!(source.starts_with(r#"<html lang="en">"#));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn page_source(&self) -> WebDriverResult<String> {
        Ok(self.client.source().await?)
    }

    /// Get the page title as a String.
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
    /// driver.get("http://webappdemo").await?;
    /// let title = driver.title().await?;
    /// #         assert_eq!(title, "Demo Web App");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn title(&self) -> WebDriverResult<String> {
        Ok(self.client.title().await?)
    }

    /// Search for an element on the current page using the specified selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    /// [WebDriver::query()](struct.WebDriver.html#method.query) method instead.
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem_text = driver.find_element(By::Name("input1")).await?;
    /// let elem_button = driver.find_element(By::Id("button-set")).await?;
    /// let elem_result = driver.find_element(By::Id("input-result")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let elem = self.client.find(by.locator()).await.map_err(|e| match e {
            // It's generally only useful to know the element query that failed.
            CmdError::NoSuchElement(_) => WebDriverError::NoSuchElement(by.to_string()),
            x => WebDriverError::CmdError(x),
        })?;
        Ok(self.wrap_element(elem))
    }

    /// Search for all elements on the current page that match the specified selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    /// [WebDriver::query()](struct.WebDriver.html#method.query) method instead.
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
    /// #         driver.get("http://webappdemo").await?;
    /// let elems = driver.find_elements(By::ClassName("section")).await?;
    /// for elem in elems {
    ///     assert!(elem.get_attribute("class").await?.expect("Missing class on element").contains("section"));
    /// }
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let elems = self.client.find_all(by.locator()).await.map_err(|e| match e {
            // It's generally only useful to know the element query that failed.
            CmdError::NoSuchElement(_) => WebDriverError::NoSuchElement(by.to_string()),
            x => WebDriverError::CmdError(x),
        })?;
        Ok(elems.into_iter().map(|x| self.wrap_element(x)).collect())
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         // Use find_element() to wait for the page to load.
    /// #         driver.find_element(By::Id("button1")).await?;
    /// let ret = driver.execute_script(r#"
    ///     let elem = document.getElementById("button1");
    ///     elem.click();
    ///     return elem;
    ///     "#, Vec::new()
    /// ).await?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.text().await?, "BUTTON 1");
    /// let elem = driver.find_element(By::Id("button-result")).await?;
    /// assert_eq!(elem.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
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
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let ret = driver.execute_script(r#"
    ///     arguments[0].innerHTML = arguments[1];
    ///     return arguments[0];
    ///     "#, vec![elem.to_json()?, serde_json::to_value("TESTING")?]
    /// ).await?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id(), elem.element_id());
    /// assert_eq!(elem_out.text().await?, "TESTING");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute_script(
        &self,
        script: &str,
        args: Vec<Value>,
    ) -> WebDriverResult<ScriptRet> {
        let v = self.client.execute(script, args).await?;
        Ok(ScriptRet::new(self.clone(), v))
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         // Use find_element() to wait for the page to load.
    /// #         driver.find_element(By::Id("button1")).await?;
    /// let ret = driver.execute_script_async(r#"
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
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.text().await?, "BUTTON 1");
    /// let elem = driver.find_element(By::Id("button-result")).await?;
    /// assert_eq!(elem.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
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
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let args = vec![elem.to_json()?, serde_json::to_value("TESTING")?];
    /// let ret = driver.execute_script_async(r#"
    ///     // Selenium automatically provides an extra argument which is a
    ///     // function that receives the return value(s).
    ///     let done = arguments[2];
    ///     window.setTimeout(() => {
    ///         arguments[0].innerHTML = arguments[1];
    ///         done(arguments[0]);
    ///     }, 1000);
    ///     "#, args
    /// ).await?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id(), elem.element_id());
    /// assert_eq!(elem_out.text().await?, "TESTING");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute_script_async(
        &self,
        script: &str,
        args: Vec<Value>,
    ) -> WebDriverResult<ScriptRet> {
        let v = self.client.execute_async(script, args).await?;
        Ok(ScriptRet::new(self.clone(), v))
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Get the current window handle.
    /// let handle = driver.current_window_handle().await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#, Vec::new()).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(handles[1].clone()).await?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo").await?;
    /// assert_ne!(driver.current_window_handle().await?, handle);
    /// // Switch back to original tab.
    /// driver.switch_to().window(handle.clone()).await?;
    /// assert_eq!(driver.current_window_handle().await?, handle);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        Ok(self.client.window().await?)
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// assert_eq!(driver.window_handles().await?.len(), 1);
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#, Vec::new()).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// assert_eq!(handles.len(), 2);
    /// driver.switch_to().window(handles[1].clone()).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        Ok(self.client.windows().await?)
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
    /// #         driver.get("http://webappdemo").await?;
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
    /// ```ignore
    /// # // Minimize is not currently working on Chrome, but does work
    /// # // on Firefox/geckodriver.
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
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
    /// #         driver.get("http://webappdemo").await?;
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
    /// #         driver.get("http://webappdemo").await?;
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// driver.back().await?;
    /// #         assert_eq!(driver.title().await?, "");
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// #         driver.back().await?;
    /// #         assert_eq!(driver.title().await?, "");
    /// driver.forward().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// driver.refresh().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
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
    /// #         let set_timeouts = TimeoutConfiguration::new(
    /// #             Some(Duration::new(1, 0)),
    /// #             Some(Duration::new(2, 0)),
    /// #             Some(Duration::new(3, 0))
    /// #         );
    /// #         driver.set_timeouts(set_timeouts.clone()).await?;
    /// let timeouts = driver.get_timeouts().await?;
    /// println!("Page load timeout = {:?}", timeouts.page_load());
    /// #         assert_eq!(timeouts.script(), Some(Duration::new(1, 0)));
    /// #         assert_eq!(timeouts.page_load(), Some(Duration::new(2, 0)));
    /// #         assert_eq!(timeouts.implicit(), Some(Duration::new(3, 0)));
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
    /// of [WebDriver::query()](query/index.html), [WebElement::query()](query/index.html) and
    /// [WebElement::wait_until()](query/index.html).
    /// It is therefore recommended to use these methods (which provide polling
    /// and explicit waits) instead rather than increasing the implicit wait timeout.
    ///
    /// **NOTE:** If you set timeouts to values greater than 120 seconds,
    ///           remember to also increase the request timeout.
    ///           See `WebDriver::set_request_timeout()` for more details.
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
    /// driver.set_timeouts(timeouts.clone()).await?;
    /// #         let got_timeouts = driver.get_timeouts().await?;
    /// #         assert_eq!(got_timeouts.page_load(), Some(Duration::new(11, 0)));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.client.update_timeouts(timeouts).await?;
        Ok(())
    }

    /// Set the implicit wait timeout. This is how long the WebDriver will
    /// wait when querying elements.
    ///
    /// By default this is set to 0 seconds.
    ///
    /// **NOTE:** Setting this to a higher number will interfere with the use of
    /// [WebDriver::query()](query/index.html), [WebElement::query()](query/index.html) and
    /// [WebElement::wait_until()](query/index.html).
    /// It is therefore recommended to use these methods (which provide polling
    /// and explicit waits) instead rather than increasing the implicit wait timeout.
    ///
    /// **NOTE:** If you set timeouts to values greater than 120 seconds,
    ///           remember to also increase the request timeout.
    ///           See `WebDriver::set_request_timeout()` for more details.
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
    /// #         let got_timeouts = driver.get_timeouts().await?;
    /// #         assert_eq!(got_timeouts.implicit(), Some(delay));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_implicit_wait_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.set_timeouts(timeouts).await
    }

    /// Set the script timeout. This is how long the WebDriver will wait for a
    /// Javascript script to execute.
    ///
    /// By default this is set to 60 seconds.
    ///
    /// **NOTE:** If you set timeouts to values greater than 120 seconds,
    ///           remember to also increase the request timeout.
    ///           See `WebDriver::set_request_timeout()` for more details.
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
    /// #         let got_timeouts = driver.get_timeouts().await?;
    /// #         assert_eq!(got_timeouts.script(), Some(delay));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.set_timeouts(timeouts).await
    }

    /// Set the page load timeout. This is how long the WebDriver will wait
    /// for the page to finish loading.
    ///
    /// By default this is set to 60 seconds.
    ///
    /// **NOTE:** If you set timeouts to values greater than 120 seconds,
    ///           remember to also increase the request timeout.
    ///           See `WebDriver::set_request_timeout()` for more details.
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
    /// #         let got_timeouts = driver.get_timeouts().await?;
    /// #         assert_eq!(got_timeouts.page_load(), Some(delay));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.set_timeouts(timeouts).await
    }

    /// Create a new action chain for this session. Action chains can be used
    /// to simulate more complex user input actions involving key combinations,
    /// mouse movements, mouse click, right-click, and more.
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem_text = driver.find_element(By::Name("input1")).await?;
    /// let elem_button = driver.find_element(By::Id("button-set")).await?;
    ///
    /// driver.action_chain()
    ///     .send_keys_to_element(&elem_text, "thirtyfour")
    ///     .move_to_element_center(&elem_button)
    ///     .click()
    ///     .perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("input-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "thirtyfour");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn action_chain(&self) -> ActionChain {
        ActionChain::new(self.clone())
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
    /// #         driver.get("https://wikipedia.org").await?;
    /// #         let mut set_cookie = Cookie::new("key", "value");
    /// #         set_cookie.set_domain("wikipedia.org");
    /// #         set_cookie.set_path("/");
    /// #         set_cookie.set_same_site(Some(SameSite::Lax));
    /// #         driver.add_cookie(set_cookie).await?;
    /// let cookies = driver.get_cookies().await?;
    /// for cookie in &cookies {
    ///     println!("Got cookie: {}", cookie.value());
    /// }
    /// #         assert_eq!(
    /// #             cookies.iter().filter(|x| x.value() == "value").count(), 1);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_cookies(&self) -> WebDriverResult<Vec<Cookie<'static>>> {
        Ok(self.client.get_all_cookies().await?)
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
    /// #         driver.get("https://wikipedia.org").await?;
    /// #         let mut set_cookie = Cookie::new("key", "value");
    /// #         set_cookie.set_domain("wikipedia.org");
    /// #         set_cookie.set_path("/");
    /// #         set_cookie.set_same_site(Some(SameSite::Lax));
    /// #         driver.add_cookie(set_cookie).await?;
    /// let cookie = driver.get_cookie("key").await?;
    /// println!("Got cookie: {}", cookie.value());
    /// #         assert_eq!(cookie.value(),"value");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie<'static>> {
        Ok(self.client.get_named_cookie(name).await?)
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
    /// #         driver.get("https://wikipedia.org").await?;
    /// #         let mut set_cookie = Cookie::new("key","value");
    /// #         set_cookie.set_domain("wikipedia.org");
    /// #         set_cookie.set_path("/");
    /// #         set_cookie.set_same_site(Some(SameSite::Lax));
    /// #         driver.add_cookie(set_cookie).await?;
    /// #         assert!(driver.get_cookie("key").await.is_ok());
    /// driver.delete_cookie("key").await?;
    /// #         assert!(driver.get_cookie("key").await.is_err());
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
    /// #         driver.get("https://wikipedia.org").await?;
    /// #         let mut set_cookie = Cookie::new("key", "value");
    /// #         set_cookie.set_domain("wikipedia.org");
    /// #         set_cookie.set_path("/");
    /// #         set_cookie.set_same_site(Some(SameSite::Lax));
    /// #         driver.add_cookie(set_cookie).await?;
    /// #         assert!(driver.get_cookie("key").await.is_ok());
    /// driver.delete_all_cookies().await?;
    /// #         assert!(driver.get_cookie("key").await.is_err());
    /// #         assert!(driver.get_cookies().await?.is_empty());
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
    /// #         driver.get("https://wikipedia.org").await?;
    /// let mut cookie = Cookie::new("key", "value");
    /// cookie.set_domain("wikipedia.org");
    /// cookie.set_path("/");
    /// cookie.set_same_site(Some(SameSite::Lax));
    /// driver.add_cookie(cookie.clone()).await?;
    /// #         let got_cookie = driver.get_cookie("key").await?;
    /// #         assert_eq!(got_cookie.value(), cookie.value());
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
    pub fn switch_to(&self) -> SwitchTo {
        SwitchTo::new(self.clone())
    }

    /// Set the current window name.
    /// Useful for switching between windows/tabs using `driver.switch_to().window_name(name)`.
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Get the current window handle.
    /// let handle = driver.current_window_handle().await?;
    /// driver.set_window_name("main").await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#, Vec::new()).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(handles[1].clone()).await?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo").await?;
    /// assert_ne!(driver.current_window_handle().await?, handle);
    /// // Switch back to original tab using window name.
    /// driver.switch_to().window_name("main").await?;
    /// assert_eq!(driver.current_window_handle().await?, handle);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_window_name(&self, window_name: &str) -> WebDriverResult<()> {
        let script = format!(r#"window.name = "{}""#, window_name);
        self.execute_script(&script, Vec::new()).await?;
        Ok(())
    }

    /// Execute the specified function in a new browser tab, closing the tab when complete.
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// #         // Get the current window handle.
    /// #         let handle = driver.current_window_handle().await?;
    /// let window_title = driver.in_new_tab(|| async {
    ///     driver.get("https://www.google.com").await?;
    ///     driver.title().await
    /// }).await?;
    /// #         assert_eq!(window_title, "Google");
    /// #         assert_eq!(driver.current_window_handle().await?, handle);
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
        let existing_handles = self.window_handles().await?;
        let handle = self.current_window_handle().await?;

        // Open new tab.
        self.execute_script(r#"window.open("about:blank", target="_blank");"#, Vec::new()).await?;
        let mut new_handles = self.window_handles().await?;
        new_handles.retain(|h| !existing_handles.contains(h));
        if new_handles.len() != 1 {
            return Err(WebDriverError::CustomError("couldn't find new tab".to_string()));
        }
        self.switch_to().window(new_handles[0].clone()).await?;
        let result = f().await;

        // Close tab.
        self.execute_script(r#"window.close();"#, Vec::new()).await?;
        self.switch_to().window(handle).await?;

        result
    }
}
