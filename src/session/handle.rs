use std::fmt;
use std::fmt::{Debug, Formatter};
use std::future::Future;
#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::action_chain::ActionChain;
use crate::common::command::Command;
use crate::common::config::WebDriverConfig;
use crate::common::connection_common::{convert_json, convert_json_vec};
use crate::error::{WebDriverError, WebDriverResult};
#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use crate::runtime::imports::{AsyncWriteExt, File};
use crate::session::scriptret::ScriptRet;
use crate::session::task::{InternalCommand, SessionCommand};
use crate::webelement::{convert_element_async, convert_elements_async};
use crate::{
    By, Cookie, DesiredCapabilities, ExtensionCommand, OptionRect, Rect, ScriptArgs, SessionId,
    SwitchTo, TimeoutConfiguration, WebElement, WindowHandle,
};

/// The SessionHandle contains a shared reference to the `WebDriverConfig` as well
/// as a sender to send commands to the async task that runs the session.
///
/// `SessionHandle` is intended to be stored in `WebDriver` and a reference to it
/// passed around to all elements or anything that needs to communicate with the session.
///
/// The reason elements contain a reference rather than a clone is specifically so that
/// Rust will not let you "use" the session after you call `WebDriver::quit()`
/// (because that will consume the `WebDriver` instance and drop the `SessionHandle`).
pub struct SessionHandle {
    pub tx: UnboundedSender<SessionCommand>,
    pub config: Arc<WebDriverConfig>,
}

impl Debug for SessionHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.config.get_session_id())
    }
}

impl SessionHandle {
    /// Convenience wrapper for running WebDriver commands.
    ///
    /// For `thirtyfour` internal use only.
    pub async fn cmd(&self, command: Command) -> WebDriverResult<serde_json::Value> {
        let (ret_tx, ret_rx) = oneshot::channel();
        self.tx.send(SessionCommand::WebDriverCommand(Box::new(command), ret_tx))?;
        ret_rx.await?
    }

    /// Return a clone of the capabilities as returned by Selenium.
    pub async fn capabilities(&mut self) -> WebDriverResult<DesiredCapabilities> {
        let (ret_tx, ret_rx) = oneshot::channel();
        self.tx.send(SessionCommand::Internal(InternalCommand::GetCapabilities(ret_tx)))?;
        Ok(ret_rx.await?)
    }

    /// Get the session ID.
    pub async fn session_id(&self) -> WebDriverResult<SessionId> {
        let (ret_tx, ret_rx) = oneshot::channel();
        self.tx.send(SessionCommand::Internal(InternalCommand::GetSessionId(ret_tx)))?;
        Ok(ret_rx.await?)
    }

    /// Get a clone of the `WebDriverConfig`. You can update the config by modifying
    /// it and passing it to `set_config()`.
    pub async fn config(&self) -> WebDriverResult<WebDriverConfig> {
        let (ret_tx, ret_rx) = oneshot::channel();
        self.tx.send(SessionCommand::Internal(InternalCommand::GetConfig(ret_tx)))?;
        Ok(ret_rx.await?)
    }

    /// Set the `WebDriverConfig` for this session. Currently this allows you to set
    /// the default `ElementPoller` used by `WebDriver::query()` and `WebElement::wait_until()`.
    ///
    /// If you implement your own Trait in order to extend `WebDriver`, this lets you store
    /// configuration inside the inner `SessionHandle` that can be accessed by any instance
    /// that contains a reference to it.
    pub async fn set_config(&self, config: WebDriverConfig) -> WebDriverResult<()> {
        self.tx.send(SessionCommand::Internal(InternalCommand::SetConfig(config)))?;
        Ok(())
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
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_request_timeout(&mut self, timeout: Duration) -> WebDriverResult<()> {
        self.tx.send(SessionCommand::Internal(InternalCommand::SetRequestTimeout(timeout)))?;
        Ok(())
    }

    /// Close the current window or tab.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(&handles[1]).await?;
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
        self.cmd(Command::CloseWindow).await.map(|_| ())
    }

    /// Navigate to the specified URL.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// driver.get("http://webappdemo").await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get<S: Into<String> + Send>(&self, url: S) -> WebDriverResult<()> {
        self.cmd(Command::NavigateTo(url.into())).await.map(|_| ())
    }

    /// Get the current URL as a String.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// driver.get("http://webappdemo").await?;
    /// let url = driver.current_url().await?;
    /// #         assert_eq!(url, "http://webappdemo/");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn current_url(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetCurrentUrl).await?;
        convert_json(&v["value"])
    }

    /// Get the page source as a String.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// driver.get("http://webappdemo").await?;
    /// let source = driver.page_source().await?;
    /// #         assert!(source.starts_with(r#"<html lang="en">"#));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn page_source(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetPageSource).await?;
        convert_json(&v["value"])
    }

    /// Get the page title as a String.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// driver.get("http://webappdemo").await?;
    /// let title = driver.title().await?;
    /// #         assert_eq!(title, "Demo Web App");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn title(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetTitle).await?;
        Ok(v["value"].as_str().unwrap_or_default().to_owned())
    }

    /// Search for an element on the current page using the specified selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    /// [WebDriver::query()](type.WebDriver.html) method instead.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
    pub async fn find_element(&self, by: By<'_>) -> WebDriverResult<WebElement<'_>> {
        let v = self.cmd(Command::FindElement(by.into())).await?;
        convert_element_async(self, &v["value"])
    }

    /// Search for all elements on the current page that match the specified selector.
    ///
    /// **NOTE**: For more powerful element queries including polling and filters, see the
    /// [WebDriver::query()](type.WebDriver.html) method instead.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
    pub async fn find_elements(&self, by: By<'_>) -> WebDriverResult<Vec<WebElement<'_>>> {
        let v = self.cmd(Command::FindElements(by.into())).await?;
        convert_elements_async(self, &v["value"])
    }

    /// Execute the specified Javascript synchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         // Use find_element() to wait for the page to load.
    /// #         driver.find_element(By::Id("button1")).await?;
    /// let ret = driver.execute_script(r#"
    ///     let elem = document.getElementById("button1");
    ///     elem.click();
    ///     return elem;
    ///     "#
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
    pub async fn execute_script(&self, script: &str) -> WebDriverResult<ScriptRet<'_>> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), Vec::new())).await?;
        Ok(ScriptRet::new(self, v["value"].clone()))
    }

    /// Execute the specified Javascript synchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::ScriptArgs;
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let mut args = ScriptArgs::new();
    /// args.push(elem.clone())?;
    /// args.push("TESTING")?;
    /// let ret = driver.execute_script_with_args(r#"
    ///     arguments[0].innerHTML = arguments[1];
    ///     return arguments[0];
    ///     "#, &args
    /// ).await?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id, elem.element_id);
    /// assert_eq!(elem_out.text().await?, "TESTING");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute_script_with_args(
        &self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRet<'_>> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), args.get_args())).await?;
        Ok(ScriptRet::new(self, v["value"].clone()))
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         // Use find_element() to wait for the page to load.
    /// #         driver.find_element(By::Id("button1")).await?;
    /// let ret = driver.execute_async_script(r#"
    ///     // Selenium automatically provides an extra argument which is a
    ///     // function that receives the return value(s).
    ///     let done = arguments[0];
    ///     window.setTimeout(() => {
    ///         let elem = document.getElementById("button1");
    ///         elem.click();
    ///         done(elem);
    ///     }, 1000);
    ///     "#
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
    pub async fn execute_async_script(&self, script: &str) -> WebDriverResult<ScriptRet<'_>> {
        let v = self.cmd(Command::ExecuteAsyncScript(script.to_owned(), Vec::new())).await?;
        Ok(ScriptRet::new(self, v["value"].clone()))
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::ScriptArgs;
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let mut args = ScriptArgs::new();
    /// args.push(elem.clone())?;
    /// args.push("TESTING")?;
    /// let ret = driver.execute_async_script_with_args(r#"
    ///     // Selenium automatically provides an extra argument which is a
    ///     // function that receives the return value(s).
    ///     let done = arguments[2];
    ///     window.setTimeout(() => {
    ///         arguments[0].innerHTML = arguments[1];
    ///         done(arguments[0]);
    ///     }, 1000);
    ///     "#, &args
    /// ).await?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id, elem.element_id);
    /// assert_eq!(elem_out.text().await?, "TESTING");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute_async_script_with_args(
        &self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRet<'_>> {
        let v = self.cmd(Command::ExecuteAsyncScript(script.to_owned(), args.get_args())).await?;
        Ok(ScriptRet::new(self, v["value"].clone()))
    }

    /// Get the current window handle.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Get the current window handle.
    /// let handle = driver.current_window_handle().await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(&handles[1]).await?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo").await?;
    /// assert_ne!(driver.current_window_handle().await?, handle);
    /// // Switch back to original tab.
    /// driver.switch_to().window(&handle).await?;
    /// assert_eq!(driver.current_window_handle().await?, handle);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        let v = self.cmd(Command::GetWindowHandle).await?;
        convert_json::<String>(&v["value"]).map(WindowHandle::from)
    }

    /// Get all window handles for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// assert_eq!(driver.window_handles().await?.len(), 1);
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// assert_eq!(handles.len(), 2);
    /// driver.switch_to().window(&handles[1]).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self.cmd(Command::GetWindowHandles).await?;
        let strings: Vec<String> = convert_json_vec(&v["value"])?;
        Ok(strings.iter().map(WindowHandle::from).collect())
    }

    /// Maximize the current window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// driver.maximize_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn maximize_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::MaximizeWindow).await.map(|_| ())
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
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// driver.minimize_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn minimize_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::MinimizeWindow).await.map(|_| ())
    }

    /// Make the current window fullscreen.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// driver.fullscreen_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::FullscreenWindow).await.map(|_| ())
    }

    /// Get the current window rectangle, in pixels.
    ///
    /// The returned Rect struct has members `x`, `y`, `width`, `height`,
    /// all i32.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::OptionRect;
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let option_rect = OptionRect::new().with_pos(1, 1).with_size(800, 600);
    /// driver.set_window_rect(option_rect.clone()).await?;
    /// let rect = driver.get_window_rect().await?;
    /// assert_eq!(OptionRect::from(rect), option_rect);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_window_rect(&self) -> WebDriverResult<Rect> {
        let v = self.cmd(Command::GetWindowRect).await?;
        convert_json(&v["value"])
    }

    /// Set the current window rectangle, in pixels.
    ///
    /// This requires an OptionRect, which is similar to Rect except all
    /// members are wrapped in Option.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::OptionRect;
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// let r = OptionRect::new().with_size(1280, 720);
    /// driver.set_window_rect(r).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// You can also convert from a Rect if you want to get the window size
    /// and modify it before setting it again.
    /// ```rust
    /// use thirtyfour::OptionRect;
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// let rect = driver.get_window_rect().await?;
    /// let option_rect = OptionRect::from(rect);
    /// driver.set_window_rect(option_rect.with_width(1024)).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_window_rect(&self, rect: OptionRect) -> WebDriverResult<()> {
        self.cmd(Command::SetWindowRect(rect)).await.map(|_| ())
    }

    /// Go back. This is equivalent to clicking the browser's back button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
        self.cmd(Command::Back).await.map(|_| ())
    }

    /// Go forward. This is equivalent to clicking the browser's forward button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
        self.cmd(Command::Forward).await.map(|_| ())
    }

    /// Refresh the current page.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
        self.cmd(Command::Refresh).await.map(|_| ())
    }

    /// Get all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
        let v = self.cmd(Command::GetTimeouts).await?;
        convert_json(&v["value"])
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
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
        self.cmd(Command::SetTimeouts(timeouts)).await.map(|_| ())
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
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
        ActionChain::new(self)
    }

    /// Get all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #         driver.add_cookie(set_cookie).await?;
    /// let cookies = driver.get_cookies().await?;
    /// for cookie in &cookies {
    ///     println!("Got cookie: {}", cookie.value());
    /// }
    /// #         assert_eq!(
    /// #             cookies.iter().filter(|x| x.value() == &serde_json::json!("value")).count(), 1);
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
        let v = self.cmd(Command::GetAllCookies).await?;
        convert_json_vec::<Cookie>(&v["value"])
    }

    /// Get the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #         driver.add_cookie(set_cookie).await?;
    /// let cookie = driver.get_cookie("key").await?;
    /// println!("Got cookie: {}", cookie.value());
    /// #         assert_eq!(cookie.value(), &serde_json::json!("value"));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie> {
        let v = self.cmd(Command::GetNamedCookie(name.to_string())).await?;
        convert_json::<Cookie>(&v["value"])
    }

    /// Delete the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let set_cookie = Cookie::new("key", serde_json::json!("value"));
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
        self.cmd(Command::DeleteCookie(name.to_string())).await.map(|_| ())
    }

    /// Delete all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let set_cookie = Cookie::new("key", serde_json::json!("value"));
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
        self.cmd(Command::DeleteAllCookies).await.map(|_| ())
    }

    /// Add the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let cookie = Cookie::new("key", serde_json::json!("value"));
    /// driver.add_cookie(cookie).await?;
    /// #         let got_cookie = driver.get_cookie("key").await?;
    /// #         assert_eq!(got_cookie.value(), &serde_json::json!("value"));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn add_cookie(&self, cookie: Cookie) -> WebDriverResult<()> {
        self.cmd(Command::AddCookie(cookie)).await.map(|_| ())
    }

    /// Take a screenshot of the current window and return it as a
    /// base64-encoded String.
    pub async fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::TakeScreenshot).await?;
        convert_json(&v["value"])
    }

    /// Take a screenshot of the current window and return it as PNG bytes.
    pub async fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64().await?;
        let bytes: Vec<u8> = base64::decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of the current window and write it to the specified
    /// filename.
    #[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
    pub async fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png().await?;
        let mut file = File::create(path).await?;
        file.write_all(&png).await?;
        Ok(())
    }

    /// Return a SwitchTo struct for switching to another window or frame.
    pub fn switch_to(&self) -> SwitchTo {
        SwitchTo::new(self)
    }

    /// Set the current window name.
    /// Useful for switching between windows/tabs using `driver.switch_to().window_name(name)`.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Get the current window handle.
    /// let handle = driver.current_window_handle().await?;
    /// driver.set_window_name("main").await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(&handles[1]).await?;
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
        self.execute_script(&script).await?;
        Ok(())
    }

    /// Running an extension command.
    /// Extension commands are browser specific commands and using browser specific endpoints and
    /// parameters.
    ///
    /// # Example
    /// ```no_run
    /// use thirtyfour::prelude::*;
    /// use thirtyfour::{ExtensionCommand, RequestMethod};
    /// use thirtyfour::support::block_on;
    /// use serde::Serialize;
    ///
    /// #[derive(Debug, Serialize)]
    /// pub struct AddonInstallCommand {
    ///     pub path: String,
    ///     pub temporary: Option<bool>
    /// }
    ///
    /// impl ExtensionCommand for AddonInstallCommand {
    ///     fn parameters_json(&self)-> Option<serde_json::Value>{
    ///        Some( serde_json::to_value(self).unwrap())
    ///     }
    ///
    ///     fn method(&self)-> RequestMethod {
    ///         RequestMethod::Post
    ///     }
    ///
    ///     fn endpoint(&self)->String {
    ///         String::from("/moz/addon/install")
    ///     }
    /// }
    ///
    ///
    /// fn main()->WebDriverResult<()>{
    ///     block_on(async {
    ///         let caps = DesiredCapabilities::firefox();
    ///         let driver = WebDriver::new("http://localhost:4444", &caps).await?;
    ///
    ///         let install_command = AddonInstallCommand {
    ///             path: String::from("/path/to/addon.xpi"),
    ///             temporary: Some(true)
    ///         };
    ///
    ///         let response = driver.extension_command(install_command).await?;
    ///
    ///         assert_eq!(response.is_string(), true);
    ///         driver.quit().await?;
    ///
    ///         Ok(())
    ///     })
    /// }
    ///
    /// ```
    pub async fn extension_command<T: ExtensionCommand + Send + Sync + 'static>(
        &self,
        ext_cmd: T,
    ) -> WebDriverResult<serde_json::Value> {
        let response = self.cmd(Command::ExtensionCommand(Box::new(ext_cmd))).await?;
        Ok(response["value"].clone())
    }

    /// Execute the specified function in a new browser tab, closing the tab when complete.
    /// The return value will be that of the supplied function, unless an error occurs while
    /// opening or closing the tab.
    ///
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
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
        self.execute_script(r#"window.open("about:blank", target="_blank");"#).await?;
        let mut new_handles = self.window_handles().await?;
        new_handles.retain(|h| !existing_handles.contains(h));
        if new_handles.len() != 1 {
            return Err(WebDriverError::NotFound(
                "new tab".to_string(),
                "Unable to find window handle for new tab".to_string(),
            ));
        }
        self.switch_to().window(&new_handles[0]).await?;
        let result = f().await;

        // Close tab.
        self.execute_script(r#"window.close();"#).await?;
        self.switch_to().window(&handle).await?;

        result
    }
}
