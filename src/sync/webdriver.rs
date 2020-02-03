use std::{fs::File, io::Write, path::Path, sync::Arc, time::Duration};

use base64::decode;
use log::error;
use serde::Deserialize;

use crate::error::WebDriverError;
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
    By, Cookie, DesiredCapabilities, OptionRect, Rect, SessionId, TimeoutConfiguration,
    WindowHandle,
};

/// This WebDriver struct encapsulates a synchronous Selenium WebDriver browser
/// session. For the async driver, see [WebDriver](../struct.WebDriver.html).
///
/// # Example:
/// ```rust
/// use thirtyfour::error::WebDriverResult;
/// use thirtyfour::{DesiredCapabilities, sync::WebDriver};
///
/// fn main() -> WebDriverResult<()> {
///     let caps = DesiredCapabilities::chrome();
///     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
///     driver.get("http://webappdemo")?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WebDriver {
    session_id: SessionId,
    capabilities: serde_json::Value,
    conn: Arc<RemoteConnectionSync>,
}

impl WebDriver {
    /// Create a new synchronous WebDriver struct.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
    /// #
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)
    ///     .expect("Error starting browser");
    /// ```
    pub fn new(
        remote_server_addr: &str,
        capabilities: &DesiredCapabilities,
    ) -> WebDriverResult<Self> {
        let conn = Arc::new(RemoteConnectionSync::new(remote_server_addr)?);
        let v = match conn.execute(Command::NewSession(capabilities)) {
            Ok(x) => Ok(x),
            Err(e) => {
                // Selenium sometimes gives a bogus 500 error "Chrome failed to start".
                // Retry if we get a 500. If it happens twice in a row then the second error
                // will be returned.
                if let WebDriverError::UnknownError(x) = &e {
                    if x.status == 500 {
                        conn.execute(Command::NewSession(capabilities))
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
            capabilities: actual_capabilities,
            conn,
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

    /// Return the actual capabilities as returned by Selenium.
    pub fn capabilities(&self) -> &serde_json::Value {
        &self.capabilities
    }

    /// Close the current window.
    pub fn close(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::CloseWindow(&self.session_id))
            .map(|_| ())
    }

    /// End the webdriver session.
    pub fn quit(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteSession(&self.session_id))
            .map(|_| ())
    }

    /// Navigate to the specified URL.
    pub fn get<S: Into<String>>(&self, url: S) -> WebDriverResult<()> {
        self.conn
            .execute(Command::NavigateTo(&self.session_id, url.into()))
            .map(|_| ())
    }

    /// Get the current URL as a String.
    pub fn current_url(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetCurrentUrl(&self.session_id))?;
        unwrap(&v["value"])
    }

    /// Get the page source as a String.
    pub fn page_source(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetPageSource(&self.session_id))?;
        unwrap(&v["value"])
    }

    /// Get the page title as a String.
    pub fn title(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetTitle(&self.session_id))?;
        unwrap(&v["value"])
    }

    /// Search for an element on the current page using the specified selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
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
    pub fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::FindElement(&self.session_id, by))?;
        unwrap_element_sync(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    /// Search for all elements on the current page that match the specified
    /// selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
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
    pub fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self
            .conn
            .execute(Command::FindElements(&self.session_id, by))?;
        unwrap_elements_sync(&self.conn, &self.session_id, &v["value"])
    }

    /// Execute the specified Javascript synchronously and return the result
    /// as a serde_json::Value.
    pub fn execute_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self.conn.execute(Command::ExecuteScript(
            &self.session_id,
            script.to_owned(),
            args,
        ))?;
        Ok(v["value"].clone())
    }

    /// Execute the specified Javascrypt asynchronously and return the result
    /// as a serde_json::Value.
    pub fn execute_async_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self.conn.execute(Command::ExecuteAsyncScript(
            &self.session_id,
            script.to_owned(),
            args,
        ))?;
        Ok(v["value"].clone())
    }

    /// Get the current window handle.
    pub fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        let v = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id))?;
        unwrap::<String>(&v["value"]).map(WindowHandle::from)
    }

    /// Get all window handles for the current session.
    pub fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))?;
        let strings: Vec<String> = unwrap_vec(&v["value"])?;
        Ok(strings.iter().map(WindowHandle::from).collect())
    }

    /// Maximize the current window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// driver.maximize_window()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn maximize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MaximizeWindow(&self.session_id))
            .map(|_| ())
    }

    /// Minimize the current window.
    pub fn minimize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MinimizeWindow(&self.session_id))
            .map(|_| ())
    }

    /// Make the current window fullscreen.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// driver.fullscreen_window()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::FullscreenWindow(&self.session_id))
            .map(|_| ())
    }

    /// Get the current window rectangle, in pixels.
    ///
    /// The returned Rect struct has members `x`, `y`, `width`, `height`,
    /// all i32.
    pub fn get_window_rect(&self) -> WebDriverResult<Rect> {
        let v = self
            .conn
            .execute(Command::GetWindowRect(&self.session_id))?;
        unwrap(&v["value"])
    }

    /// Set the current window rectangle, in pixels.
    ///
    /// This requires an OptionRect, which is similar to Rect except all
    /// members are wrapped in Option.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn set_window_rect(&self, rect: OptionRect) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SetWindowRect(&self.session_id, rect))
            .map(|_| ())
    }

    /// Go back. This is equivalent to clicking the browser's back button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn back(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Back(&self.session_id))
            .map(|_| ())
    }

    /// Go forward. This is equivalent to clicking the browser's forward button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn forward(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Forward(&self.session_id))
            .map(|_| ())
    }

    /// Refresh the current page.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn refresh(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Refresh(&self.session_id))
            .map(|_| ())
    }

    /// Get all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
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
    pub fn get_timeouts(&self) -> WebDriverResult<TimeoutConfiguration> {
        let v = self.conn.execute(Command::GetTimeouts(&self.session_id))?;
        unwrap(&v["value"])
    }

    /// Set all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
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
    pub fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SetTimeouts(&self.session_id, timeouts))
            .map(|_| ())
    }

    /// Set the implicit wait timeout.
    pub fn implicitly_wait(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.set_timeouts(timeouts)
    }

    /// Set the script timeout.
    pub fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.set_timeouts(timeouts)
    }

    /// Set the page load timeout.
    pub fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.set_timeouts(timeouts)
    }

    /// Create a new action chain for this session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
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
    pub fn action_chain(&self) -> ActionChain {
        ActionChain::new(self.conn.clone(), self.session_id.clone())
    }

    /// Get all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
        let v = self
            .conn
            .execute(Command::GetAllCookies(&self.session_id))?;
        unwrap_vec::<Cookie>(&v["value"])
    }

    /// Get the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie> {
        let v = self
            .conn
            .execute(Command::GetNamedCookie(&self.session_id, name))?;
        unwrap::<Cookie>(&v["value"])
    }

    /// Delete the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn delete_cookie(&self, name: &str) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteCookie(&self.session_id, name))
            .map(|_| ())
    }

    /// Delete all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn delete_all_cookies(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteAllCookies(&self.session_id))
            .map(|_| ())
    }

    /// Add the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, sync::WebDriver};
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
    pub fn add_cookie(&self, cookie: Cookie) -> WebDriverResult<()> {
        self.conn
            .execute(Command::AddCookie(&self.session_id, cookie))
            .map(|_| ())
    }

    /// Take a screenshot of the current window and return it as a
    /// base64-encoded String.
    pub fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::TakeScreenshot(&self.session_id))?;
        unwrap(&v["value"])
    }

    /// Take a screenshot of the current window and return it as PNG bytes.
    pub fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64()?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of the current window and write it to the specified
    /// filename.
    pub fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png()?;
        let mut file = File::create(path)?;
        file.write_all(&png)?;
        Ok(())
    }

    /// Return a SwitchTo struct for switching to another window or frame.
    pub fn switch_to(&self) -> SwitchTo {
        SwitchTo::new(self.session_id.clone(), self.conn.clone())
    }
}

impl Drop for WebDriver {
    /// Close the current session when the WebDriver struct goes out of scope.
    fn drop(&mut self) {
        if !(*self.session_id).is_empty() {
            if let Err(e) = self.quit() {
                error!("Failed to close session: {:?}", e);
            }
        }
    }
}
