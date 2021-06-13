#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use crate::runtime::imports::{AsyncWriteExt, File};
use async_trait::async_trait;
use base64::decode;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use std::path::Path;
use std::time::Duration;

use crate::action_chain::ActionChain;
use crate::common::command::ExtensionCommand;
use crate::common::command::{Command, FormatRequestData};
use crate::common::connection_common::{convert_json, convert_json_vec};
use crate::error::{WebDriverError, WebDriverResult};
use crate::http::connection_async::WebDriverHttpClientAsync;
use crate::session::WebDriverSession;
use crate::webelement::{convert_element_async, convert_elements_async};
use crate::{
    By, Cookie, OptionRect, Rect, ScriptArgs, SessionId, SwitchTo, TimeoutConfiguration,
    WebElement, WindowHandle,
};

/// Start a new WebDriver session, returning the session id and the
/// capabilities JSON that was received back from the server.
pub async fn start_session<C>(
    conn: &dyn WebDriverHttpClientAsync,
    capabilities: C,
) -> WebDriverResult<(SessionId, serde_json::Value)>
where
    C: Serialize,
{
    let caps = serde_json::to_value(capabilities)?;
    let v = match conn
        .execute(Command::NewSession(caps.clone()).format_request(&SessionId::null()))
        .await
    {
        Ok(x) => Ok(x),
        Err(e) => {
            // Selenium sometimes gives a bogus 500 error "Chrome failed to start".
            // Retry if we get a 500. If it happens twice in a row then the second error
            // will be returned.
            if let WebDriverError::UnknownError(x) = &e {
                if x.status == 500 {
                    conn.execute(Command::NewSession(caps).format_request(&SessionId::null())).await
                } else {
                    Err(e)
                }
            } else {
                Err(e)
            }
        }
    }?;

    #[derive(Debug, Deserialize)]
    struct ConnectionData {
        #[serde(default, rename(deserialize = "sessionId"))]
        session_id: String,
        #[serde(default)]
        capabilities: serde_json::Value,
    }

    #[derive(Debug, Deserialize)]
    struct ConnectionResp {
        #[serde(default, rename(deserialize = "sessionId"))]
        session_id: String,
        value: ConnectionData,
    }

    let resp: ConnectionResp = serde_json::from_value(v)?;
    let data = resp.value;
    let session_id = SessionId::from(if resp.session_id.is_empty() {
        data.session_id
    } else {
        resp.session_id
    });

    // Set default timeouts.
    conn.execute(Command::SetTimeouts(TimeoutConfiguration::default()).format_request(&session_id))
        .await?;

    Ok((session_id, data.capabilities))
}

/// All browser-level W3C WebDriver commands are implemented under this trait.
///
/// ----
///
/// `Thirtyfour` is structured as follows:
/// - The `WebDriverCommands` trait contains all of the methods you would
///   typically call in order to interact with the browser.
/// - The `GenericWebDriver` struct implements the `WebDriverCommands` trait
///   for a generic HTTP client.
/// - The `WebDriver` struct is the `GenericWebDriver` implemented for a
///   specific HTTP client.
///
/// You only need to use `WebDriver` in your code. Just create an instance
/// of the `WebDriver` struct and it will have access to all of the methods
/// from the `WebDriverCommands` trait.
///
/// For example:
/// ```rust
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// let caps = DesiredCapabilities::chrome();
/// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
/// driver.get("http://webappdemo").await?;
/// assert_eq!(driver.current_url().await?, "http://webappdemo/");
/// #         driver.quit().await?;
/// #         Ok(())
/// #     })
/// # }
/// ```
#[async_trait]
pub trait WebDriverCommands {
    /// Get the current session and http client.
    ///
    /// For `thirtyfour` internal use only.
    fn session(&self) -> &WebDriverSession;

    /// Convenience wrapper for running WebDriver commands.
    ///
    /// For `thirtyfour` internal use only.
    async fn cmd(&self, command: Command) -> WebDriverResult<serde_json::Value> {
        self.session().execute(Box::new(command)).await
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
    async fn close(&self) -> WebDriverResult<()> {
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
    async fn get<S: Into<String> + Send>(&self, url: S) -> WebDriverResult<()> {
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
    async fn current_url(&self) -> WebDriverResult<String> {
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
    async fn page_source(&self) -> WebDriverResult<String> {
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
    async fn title(&self) -> WebDriverResult<String> {
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
    async fn find_element<'a>(&'a self, by: By<'_>) -> WebDriverResult<WebElement<'a>> {
        let v = self.cmd(Command::FindElement(by.get_w3c_selector())).await?;
        convert_element_async(self.session(), &v["value"])
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
    async fn find_elements<'a>(&'a self, by: By<'_>) -> WebDriverResult<Vec<WebElement<'a>>> {
        let v = self.cmd(Command::FindElements(by.get_w3c_selector())).await?;
        convert_elements_async(self.session(), &v["value"])
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
    async fn execute_script<'a>(&'a self, script: &str) -> WebDriverResult<ScriptRet<'a>> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), Vec::new())).await?;
        Ok(ScriptRet::new(self.session(), v["value"].clone()))
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
    async fn execute_script_with_args<'a>(
        &'a self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRet<'a>> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), args.get_args())).await?;
        Ok(ScriptRet::new(self.session(), v["value"].clone()))
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
    async fn execute_async_script<'a>(&'a self, script: &str) -> WebDriverResult<ScriptRet<'a>> {
        let v = self.cmd(Command::ExecuteAsyncScript(script.to_owned(), Vec::new())).await?;
        Ok(ScriptRet::new(self.session(), v["value"].clone()))
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
    async fn execute_async_script_with_args<'a>(
        &'a self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRet<'a>> {
        let v = self.cmd(Command::ExecuteAsyncScript(script.to_owned(), args.get_args())).await?;
        Ok(ScriptRet::new(self.session(), v["value"].clone()))
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
    async fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
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
    async fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
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
    async fn maximize_window(&self) -> WebDriverResult<()> {
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
    async fn minimize_window(&self) -> WebDriverResult<()> {
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
    async fn fullscreen_window(&self) -> WebDriverResult<()> {
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
    async fn get_window_rect(&self) -> WebDriverResult<Rect> {
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
    async fn set_window_rect(&self, rect: OptionRect) -> WebDriverResult<()> {
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
    async fn back(&self) -> WebDriverResult<()> {
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
    async fn forward(&self) -> WebDriverResult<()> {
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
    async fn refresh(&self) -> WebDriverResult<()> {
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
    async fn get_timeouts(&self) -> WebDriverResult<TimeoutConfiguration> {
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
    async fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
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
    async fn set_implicit_wait_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
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
    async fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
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
    async fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
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
    fn action_chain(&self) -> ActionChain {
        ActionChain::new(self.session())
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
    async fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
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
    async fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie> {
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
    async fn delete_cookie(&self, name: &str) -> WebDriverResult<()> {
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
    async fn delete_all_cookies(&self) -> WebDriverResult<()> {
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
    async fn add_cookie(&self, cookie: Cookie) -> WebDriverResult<()> {
        self.cmd(Command::AddCookie(cookie)).await.map(|_| ())
    }

    /// Take a screenshot of the current window and return it as a
    /// base64-encoded String.
    async fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::TakeScreenshot).await?;
        convert_json(&v["value"])
    }

    /// Take a screenshot of the current window and return it as PNG bytes.
    async fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64().await?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of the current window and write it to the specified
    /// filename.
    #[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
    async fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png().await?;
        let mut file = File::create(path).await?;
        file.write_all(&png).await?;
        Ok(())
    }

    /// Return a SwitchTo struct for switching to another window or frame.
    fn switch_to(&self) -> SwitchTo {
        SwitchTo::new(self.session())
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
    async fn set_window_name(&self, window_name: &str) -> WebDriverResult<()> {
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
    /// #[derive(Serialize)]
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
    ///
    ///         Ok(())
    ///     })
    /// }
    ///
    /// ```
    async fn extension_command<T: ExtensionCommand + Send + Sync + 'static>(
        &self,
        ext_cmd: T,
    ) -> WebDriverResult<serde_json::Value> {
        let response = self.cmd(Command::ExtensionCommand(Box::new(ext_cmd))).await?;
        Ok(response["value"].clone())
    }
}

/// Helper struct for getting return values from scripts.
/// See the examples for [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
/// and [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script).
pub struct ScriptRet<'a> {
    session: &'a WebDriverSession,
    value: serde_json::Value,
}

impl<'a> ScriptRet<'a> {
    /// Create a new ScriptRet. This is typically done automatically via
    /// [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
    /// or [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script)
    pub fn new(session: &'a WebDriverSession, value: serde_json::Value) -> Self {
        ScriptRet {
            session,
            value,
        }
    }

    /// Get the raw JSON value.
    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }

    pub fn convert<T>(&self) -> WebDriverResult<T>
    where
        T: DeserializeOwned,
    {
        let v: T = serde_json::from_value(self.value.clone())?;
        Ok(v)
    }

    /// Get a single WebElement return value.
    /// Your script must return only a single element for this to work.
    pub fn get_element(&self) -> WebDriverResult<WebElement> {
        convert_element_async(self.session, &self.value)
    }

    /// Get a vec of WebElements from the return value.
    /// Your script must return an array of elements for this to work.
    pub fn get_elements(&self) -> WebDriverResult<Vec<WebElement>> {
        convert_elements_async(self.session, &self.value)
    }
}
