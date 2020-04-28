use std::{fs::File, io::Write, path::Path, sync::Arc, time::Duration};

use base64::decode;
use log::error;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, to_value, Value};

use crate::error::WebDriverError;
use crate::sync::ReqwestDriverSync;
use crate::{
    common::{
        command::Command,
        connection_common::{unwrap, unwrap_vec},
    },
    error::WebDriverResult,
    sync::{
        action_chain::ActionChain,
        webelement::{unwrap_element_sync, unwrap_elements_sync},
        RemoteConnectionSync, SwitchTo, WebElement,
    },
    By, Cookie, DesiredCapabilities, OptionRect, Rect, ScriptArgs, SessionId, TimeoutConfiguration,
    WindowHandle,
};

/// This WebDriver struct encapsulates a synchronous Selenium WebDriver browser
/// session. For the async driver, see [WebDriver](../struct.WebDriver.html).
///
/// See the [WebDriverCommands](trait.WebDriverCommands.html) trait for WebDriver methods.
///
/// # Example:
/// ```rust
/// use thirtyfour::sync::prelude::*;
///
/// fn main() -> WebDriverResult<()> {
///     let caps = DesiredCapabilities::chrome();
///     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
///     driver.get("http://webappdemo")?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct WebDriver {
    pub session_id: SessionId,
    conn: Arc<dyn RemoteConnectionSync>,
    capabilities: Value,
    quit_on_drop: bool,
}

impl WebDriver {
    /// Create a new synchronous WebDriver struct.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)
    ///     .expect("Error starting browser");
    /// ```
    pub fn new<T>(remote_server_addr: &str, capabilities: T) -> WebDriverResult<Self>
    where
        T: Serialize,
    {
        let conn = Arc::new(ReqwestDriverSync::new(remote_server_addr)?);
        let caps = to_value(capabilities)?;
        let v = match conn.execute(&SessionId::null(), Command::NewSession(&caps)) {
            Ok(x) => Ok(x),
            Err(e) => {
                // Selenium sometimes gives a bogus 500 error "Chrome failed to start".
                // Retry if we get a 500. If it happens twice in a row then the second error
                // will be returned.
                if let WebDriverError::UnknownError(x) = &e {
                    if x.status == 500 {
                        conn.execute(&SessionId::null(), Command::NewSession(&caps))
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
            capabilities: Value,
        }

        #[derive(Debug, Deserialize)]
        struct ConnectionResp {
            #[serde(default)]
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
        let actual_capabilities = data.capabilities;

        let driver = WebDriver {
            session_id,
            conn,
            capabilities: actual_capabilities,
            quit_on_drop: true,
        };

        // Set default timeouts.
        let timeout_config = TimeoutConfiguration::new(
            Some(Duration::new(60, 0)),
            Some(Duration::new(60, 0)),
            Some(Duration::new(30, 0)),
        );
        driver.set_timeouts(timeout_config)?;
        Ok(driver)
    }

    /// Return a clone of the capabilities as returned by Selenium.
    pub fn capabilities(&self) -> DesiredCapabilities {
        DesiredCapabilities::new(self.capabilities.clone())
    }

    /// End the webdriver session.
    pub fn quit(mut self) -> WebDriverResult<()> {
        self.cmd(Command::DeleteSession)?;
        self.quit_on_drop = false;
        Ok(())
    }
}

impl WebDriverCommands for WebDriver {
    fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.conn.execute(&self.session_id, command)
    }

    fn session(&self) -> WebDriverSession {
        WebDriverSession {
            session_id: &self.session_id,
            conn: self.conn.clone(),
        }
    }
}

impl Drop for WebDriver {
    /// Close the current session when the WebDriver struct goes out of scope.
    fn drop(&mut self) {
        if self.quit_on_drop && !(*self.session_id).is_empty() {
            if let Err(e) = self.cmd(Command::DeleteSession) {
                error!("Failed to close session: {:?}", e);
            }
        }
    }
}

#[derive(Debug)]
pub struct WebDriverSession<'a> {
    pub session_id: &'a SessionId,
    conn: Arc<dyn RemoteConnectionSync>,
}

impl<'a> Clone for WebDriverSession<'a> {
    fn clone(&self) -> Self {
        WebDriverSession {
            session_id: self.session_id,
            conn: self.conn.clone(),
        }
    }
}

impl<'a> WebDriverCommands for WebDriverSession<'a> {
    fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.conn.execute(&self.session_id, command)
    }

    fn session(&self) -> WebDriverSession {
        self.clone()
    }
}

/// All browser-level W3C WebDriver commands are implemented under this trait.
pub trait WebDriverCommands {
    /// Convenience wrapper for running WebDriver commands.
    fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value>;

    /// Get the current session and connection.
    fn session(&self) -> WebDriverSession;

    /// Close the current window.
    fn close(&self) -> WebDriverResult<()> {
        self.cmd(Command::CloseWindow).map(|_| ())
    }

    /// Navigate to the specified URL.
    fn get<S: Into<String>>(&self, url: S) -> WebDriverResult<()> {
        self.cmd(Command::NavigateTo(url.into())).map(|_| ())
    }

    /// Get the current URL as a String.
    fn current_url(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetCurrentUrl)?;
        unwrap(&v["value"])
    }

    /// Get the page source as a String.
    fn page_source(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetPageSource)?;
        unwrap(&v["value"])
    }

    /// Get the page title as a String.
    fn title(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetTitle)?;
        unwrap(&v["value"])
    }

    /// Search for an element on the current page using the specified selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem_text = driver.find_element(By::Name("input1"))?;
    /// let elem_button = driver.find_element(By::Id("button-set"))?;
    /// let elem_result = driver.find_element(By::Name("input-result"))?;
    /// #     Ok(())
    /// # }
    /// ```
    fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self.cmd(Command::FindElement(by))?;
        unwrap_element_sync(self.session(), &v["value"])
    }

    /// Search for all elements on the current page that match the specified
    /// selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elems = driver.find_elements(By::ClassName("section"))?;
    /// for elem in elems {
    ///     assert!(elem.get_attribute("class")?.contains("section"));
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self.cmd(Command::FindElements(by))?;
        unwrap_elements_sync(self.session(), &v["value"])
    }

    /// Execute the specified Javascript synchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     // Use find_element() to wait for the page to load.
    /// #     driver.find_element(By::Id("button1"))?;
    /// let ret = driver.execute_script(r#"
    ///     let elem = document.getElementById("button1");
    ///     elem.click();
    ///     return elem;
    ///     "#
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.text()?, "BUTTON 1");
    /// let elem = driver.find_element(By::Id("button-result"))?;
    /// assert_eq!(elem.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_script(&self, script: &str) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), Vec::new()))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Execute the specified Javascript synchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// let mut args = ScriptArgs::new();
    /// args.push(elem.clone())?;
    /// args.push("TESTING")?;
    /// let ret = driver.execute_script_with_args(r#"
    ///     arguments[0].innerHTML = arguments[1];
    ///     return arguments[0];
    ///     "#, &args
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id, elem.element_id);
    /// assert_eq!(elem_out.text()?, "TESTING");
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_script_with_args(
        &self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteScript(script.to_owned(), args.get_args()))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     // Use find_element() to wait for the page to load.
    /// #     driver.find_element(By::Id("button1"))?;
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
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.text()?, "BUTTON 1");
    /// let elem = driver.find_element(By::Id("button-result"))?;
    /// assert_eq!(elem.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_async_script(&self, script: &str) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteAsyncScript(script.to_owned(), Vec::new()))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Execute the specified Javascrypt asynchronously and return the result.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
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
    /// )?;
    /// let elem_out = ret.get_element()?;
    /// assert_eq!(elem_out.element_id, elem.element_id);
    /// assert_eq!(elem_out.text()?, "TESTING");
    /// #     Ok(())
    /// # }
    /// ```
    fn execute_async_script_with_args(
        &self,
        script: &str,
        args: &ScriptArgs,
    ) -> WebDriverResult<ScriptRetSync> {
        let v = self.cmd(Command::ExecuteAsyncScript(
            script.to_owned(),
            args.get_args(),
        ))?;
        Ok(ScriptRetSync::new(self.session(), v["value"].clone()))
    }

    /// Get the current window handle.
    fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        let v = self.cmd(Command::GetWindowHandle)?;
        unwrap::<String>(&v["value"]).map(WindowHandle::from)
    }

    /// Get all window handles for the current session.
    fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self.cmd(Command::GetWindowHandles)?;
        let strings: Vec<String> = unwrap_vec(&v["value"])?;
        Ok(strings.iter().map(WindowHandle::from).collect())
    }

    /// Maximize the current window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// driver.maximize_window()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn maximize_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::MaximizeWindow).map(|_| ())
    }

    /// Minimize the current window.
    fn minimize_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::MinimizeWindow).map(|_| ())
    }

    /// Make the current window fullscreen.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// driver.fullscreen_window()?;
    /// #     Ok(())
    /// # }
    /// ```
    fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.cmd(Command::FullscreenWindow).map(|_| ())
    }

    /// Get the current window rectangle, in pixels.
    ///
    /// The returned Rect struct has members `x`, `y`, `width`, `height`,
    /// all i32.
    fn get_window_rect(&self) -> WebDriverResult<Rect> {
        let v = self.cmd(Command::GetWindowRect)?;
        unwrap(&v["value"])
    }

    /// Set the current window rectangle, in pixels.
    ///
    /// This requires an OptionRect, which is similar to Rect except all
    /// members are wrapped in Option.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// use thirtyfour::OptionRect;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// let r = OptionRect::new().with_size(1280, 720);
    /// driver.set_window_rect(r)?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// You can also convert from a Rect if you want to get the window size
    /// and modify it before setting it again.
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// use thirtyfour::OptionRect;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// let rect = driver.get_window_rect()?;
    /// let option_rect = OptionRect::from(rect);
    /// driver.set_window_rect(option_rect.with_width(1024))?;
    /// #     Ok(())
    /// # }
    /// ```
    fn set_window_rect(&self, rect: OptionRect) -> WebDriverResult<()> {
        self.cmd(Command::SetWindowRect(rect)).map(|_| ())
    }

    /// Go back. This is equivalent to clicking the browser's back button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// driver.back()?;
    /// #     assert_eq!(driver.title()?, "");
    /// #     Ok(())
    /// # }
    /// ```
    fn back(&self) -> WebDriverResult<()> {
        self.cmd(Command::Back).map(|_| ())
    }

    /// Go forward. This is equivalent to clicking the browser's forward button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// #     driver.back()?;
    /// #     assert_eq!(driver.title()?, "");
    /// driver.forward()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// #     Ok(())
    /// # }
    /// ```
    fn forward(&self) -> WebDriverResult<()> {
        self.cmd(Command::Forward).map(|_| ())
    }

    /// Refresh the current page.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// driver.refresh()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// #     Ok(())
    /// # }
    /// ```
    fn refresh(&self) -> WebDriverResult<()> {
        self.cmd(Command::Refresh).map(|_| ())
    }

    /// Get all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    ///
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     let set_timeouts = TimeoutConfiguration::new(
    /// #         Some(Duration::new(1, 0)),
    /// #         Some(Duration::new(2, 0)),
    /// #         Some(Duration::new(3, 0))
    /// #     );
    /// #     driver.set_timeouts(set_timeouts.clone())?;
    /// let timeouts = driver.get_timeouts()?;
    /// println!("Page load timeout = {:?}", timeouts.page_load());
    /// #     assert_eq!(timeouts.script(), Some(Duration::new(1, 0)));
    /// #     assert_eq!(timeouts.page_load(), Some(Duration::new(2, 0)));
    /// #     assert_eq!(timeouts.implicit(), Some(Duration::new(3, 0)));
    /// #     Ok(())
    /// # }
    /// ```
    fn get_timeouts(&self) -> WebDriverResult<TimeoutConfiguration> {
        let v = self.cmd(Command::GetTimeouts)?;
        unwrap(&v["value"])
    }

    /// Set all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    ///
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// // Setting timeouts to None means those timeout values will not be updated.
    /// let timeouts = TimeoutConfiguration::new(None, Some(Duration::new(11, 0)), None);
    /// driver.set_timeouts(timeouts.clone())?;
    /// #     let got_timeouts = driver.get_timeouts()?;
    /// #     assert_eq!(timeouts.page_load(), Some(Duration::new(11, 0)));
    /// #     Ok(())
    /// # }
    /// ```
    fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.cmd(Command::SetTimeouts(timeouts)).map(|_| ())
    }

    /// Set the implicit wait timeout.
    fn implicitly_wait(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.set_timeouts(timeouts)
    }

    /// Set the script timeout.
    fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.set_timeouts(timeouts)
    }

    /// Set the page load timeout.
    fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.set_timeouts(timeouts)
    }

    /// Create a new action chain for this session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem_text = driver.find_element(By::Name("input1"))?;
    /// let elem_button = driver.find_element(By::Id("button-set"))?;
    ///
    /// driver.action_chain()
    ///     .send_keys_to_element(&elem_text, "thirtyfour")
    ///     .move_to_element_center(&elem_button)
    ///     .click()
    ///     .perform()?;
    /// #     let elem_result = driver.find_element(By::Name("input-result"))?;
    /// #     assert_eq!(elem_result.text()?, "thirtyfour");
    /// #     Ok(())
    /// # }
    /// ```
    fn action_chain(&self) -> ActionChain {
        ActionChain::new(self.session())
    }

    /// Get all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// let cookies = driver.get_cookies()?;
    /// for cookie in &cookies {
    ///     println!("Got cookie: {}", cookie.value());
    /// }
    /// #     assert_eq!(
    /// #         cookies.iter().filter(|x| x.value() == &serde_json::json!("value")).count(), 1);
    /// #     Ok(())
    /// # }
    /// ```
    fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
        let v = self.cmd(Command::GetAllCookies)?;
        unwrap_vec::<Cookie>(&v["value"])
    }

    /// Get the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// let cookie = driver.get_cookie("key")?;
    /// println!("Got cookie: {}", cookie.value());
    /// #     assert_eq!(cookie.value(), &serde_json::json!("value"));
    /// #     Ok(())
    /// # }
    /// ```
    fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie> {
        let v = self.cmd(Command::GetNamedCookie(name))?;
        unwrap::<Cookie>(&v["value"])
    }

    /// Delete the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// #     assert!(driver.get_cookie("key").is_ok());
    /// driver.delete_cookie("key")?;
    /// #     assert!(driver.get_cookie("key").is_err());
    /// #     Ok(())
    /// # }
    /// ```
    fn delete_cookie(&self, name: &str) -> WebDriverResult<()> {
        self.cmd(Command::DeleteCookie(name)).map(|_| ())
    }

    /// Delete all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie)?;
    /// #     assert!(driver.get_cookie("key").is_ok());
    /// driver.delete_all_cookies()?;
    /// #     assert!(driver.get_cookie("key").is_err());
    /// #     assert!(driver.get_cookies()?.is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    fn delete_all_cookies(&self) -> WebDriverResult<()> {
        self.cmd(Command::DeleteAllCookies).map(|_| ())
    }

    /// Add the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let cookie = Cookie::new("key", serde_json::json!("value"));
    /// driver.add_cookie(cookie)?;
    /// #     let got_cookie = driver.get_cookie("key")?;
    /// #     assert_eq!(got_cookie.value(), &serde_json::json!("value"));
    /// #     Ok(())
    /// # }
    /// ```
    fn add_cookie(&self, cookie: Cookie) -> WebDriverResult<()> {
        self.cmd(Command::AddCookie(cookie)).map(|_| ())
    }

    /// Take a screenshot of the current window and return it as a
    /// base64-encoded String.
    fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::TakeScreenshot)?;
        unwrap(&v["value"])
    }

    /// Take a screenshot of the current window and return it as PNG bytes.
    fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64()?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of the current window and write it to the specified
    /// filename.
    fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png()?;
        let mut file = File::create(path)?;
        file.write_all(&png)?;
        Ok(())
    }

    /// Return a SwitchTo struct for switching to another window or frame.
    fn switch_to(&self) -> SwitchTo {
        SwitchTo::new(self.session())
    }

    /// Set the current window name.
    /// Useful for switching between windows/tabs using `driver.switch_to().window_name(name)`.
    fn set_window_name(&self, window_name: &str) -> WebDriverResult<()> {
        self.execute_script(&format!(r#"window.name = "{}""#, window_name))?;
        Ok(())
    }
}

/// Helper struct for getting return values from scripts.
/// See the examples for [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
/// and [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script).
pub struct ScriptRetSync<'a> {
    driver: WebDriverSession<'a>,
    value: Value,
}

impl<'a> ScriptRetSync<'a> {
    /// Create a new ScriptRetSync. This is typically done automatically via
    /// [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
    /// or [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script)
    pub fn new(driver: WebDriverSession<'a>, value: Value) -> Self {
        ScriptRetSync { driver, value }
    }

    /// Get the raw JSON value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn convert<T>(&self) -> WebDriverResult<T>
    where
        T: DeserializeOwned,
    {
        let v: T = from_value(self.value.clone())?;
        Ok(v)
    }

    /// Get a single WebElement return value.
    /// Your script must return only a single element for this to work.
    pub fn get_element(&self) -> WebDriverResult<WebElement> {
        unwrap_element_sync(self.driver.clone(), &self.value)
    }

    /// Get a vec of WebElements from the return value.
    /// Your script must return an array of elements for this to work.
    pub fn get_elements(&self) -> WebDriverResult<Vec<WebElement>> {
        unwrap_elements_sync(self.driver.clone(), &self.value)
    }
}
