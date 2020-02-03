use std::{path::Path, sync::Arc, time::Duration};

use base64::decode;
use futures::executor::block_on;
use log::error;
use serde::Deserialize;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::error::WebDriverError;
use crate::{
    action_chain::ActionChain,
    common::{
        command::{By, Command},
        connection_common::{unwrap, unwrap_vec},
    },
    error::WebDriverResult,
    webelement::{unwrap_element_async, unwrap_elements_async},
    Cookie, DesiredCapabilities, OptionRect, Rect, RemoteConnectionAsync, SessionId, SwitchTo,
    TimeoutConfiguration, WebElement, WindowHandle,
};

/// The WebDriver struct encapsulates an async Selenium WebDriver browser
/// session. For the async driver, see
/// [sync::WebDriver](sync/struct.WebDriver.html).
///
/// # Example:
/// ```rust
/// use thirtyfour::error::WebDriverResult;
/// use thirtyfour::{DesiredCapabilities, WebDriver};
/// use tokio;
///
/// #[tokio::main]
/// async fn main() -> WebDriverResult<()> {
///     let caps = DesiredCapabilities::chrome();
///     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
///     driver.get("http://webappdemo").await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WebDriver {
    session_id: SessionId,
    capabilities: serde_json::Value,
    conn: Arc<RemoteConnectionAsync>,
}

impl WebDriver {
    /// Create a new async WebDriver struct.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn new(
        remote_server_addr: &str,
        capabilities: &DesiredCapabilities,
    ) -> WebDriverResult<Self> {
        let conn = Arc::new(RemoteConnectionAsync::new(remote_server_addr)?);
        let v = match conn.execute(Command::NewSession(capabilities)).await {
            Ok(x) => Ok(x),
            Err(e) => {
                // Selenium sometimes gives a bogus 500 error "Chrome failed to start".
                // Retry if we get a 500. If it happens twice in a row then the second error
                // will be returned.
                if let WebDriverError::UnknownError(x) = &e {
                    if x.status == 500 {
                        conn.execute(Command::NewSession(capabilities)).await
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
        driver.set_timeouts(timeout_config).await?;
        Ok(driver)
    }

    /// Return the actual capabilities as returned by Selenium.
    pub fn capabilities(&self) -> &serde_json::Value {
        &self.capabilities
    }

    /// Close the current window.
    pub async fn close(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::CloseWindow(&self.session_id))
            .await
            .map(|_| ())
    }

    /// End the webdriver session.
    pub async fn quit(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteSession(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Navigate to the specified URL.
    pub async fn get<S: Into<String>>(&self, url: S) -> WebDriverResult<()> {
        self.conn
            .execute(Command::NavigateTo(&self.session_id, url.into()))
            .await
            .map(|_| ())
    }

    /// Get the current URL as a String.
    pub async fn current_url(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetCurrentUrl(&self.session_id))
            .await?;
        unwrap(&v["value"])
    }

    /// Get the page source as a String.
    pub async fn page_source(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetPageSource(&self.session_id))
            .await?;
        unwrap(&v["value"])
    }

    /// Get the page title as a String.
    pub async fn title(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetTitle(&self.session_id))
            .await?;
        Ok(v["value"].as_str().unwrap_or_default().to_owned())
    }

    /// Search for an element on the current page using the specified selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem_text = driver.find_element(By::Name("input1")).await?;
    /// let elem_button = driver.find_element(By::Id("button-set")).await?;
    /// let elem_result = driver.find_element(By::Name("input-result")).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn find_element(&self, by: By<'_>) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::FindElement(&self.session_id, by))
            .await?;
        unwrap_element_async(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    /// Search for all elements on the current page that match the specified
    /// selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// let elems = driver.find_elements(By::ClassName("section")).await?;
    /// for elem in elems {
    ///     assert!(elem.get_attribute("class").await?.contains("section"));
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn find_elements(&self, by: By<'_>) -> WebDriverResult<Vec<WebElement>> {
        let v = self
            .conn
            .execute(Command::FindElements(&self.session_id, by))
            .await?;
        unwrap_elements_async(&self.conn, &self.session_id, &v["value"])
    }

    /// Execute the specified Javascript synchronously and return the result
    /// as a serde_json::Value.
    pub async fn execute_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self
            .conn
            .execute(Command::ExecuteScript(
                &self.session_id,
                script.to_owned(),
                args,
            ))
            .await?;
        Ok(v["value"].clone())
    }

    /// Execute the specified Javascrypt asynchronously and return the result
    /// as a serde_json::Value.
    pub async fn execute_async_script(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> WebDriverResult<serde_json::Value> {
        let v = self
            .conn
            .execute(Command::ExecuteAsyncScript(
                &self.session_id,
                script.to_owned(),
                args,
            ))
            .await?;
        Ok(v["value"].clone())
    }

    /// Get the current window handle.
    pub async fn current_window_handle(&self) -> WebDriverResult<WindowHandle> {
        let v = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id))
            .await?;
        unwrap::<String>(&v["value"]).map(WindowHandle::from)
    }

    /// Get all window handles for the current session.
    pub async fn window_handles(&self) -> WebDriverResult<Vec<WindowHandle>> {
        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))
            .await?;
        let strings: Vec<String> = unwrap_vec(&v["value"])?;
        Ok(strings.iter().map(WindowHandle::from).collect())
    }

    /// Maximize the current window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// driver.maximize_window().await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn maximize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MaximizeWindow(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Minimize the current window.
    pub async fn minimize_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::MinimizeWindow(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Make the current window fullscreen.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// driver.fullscreen_window().await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn fullscreen_window(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::FullscreenWindow(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Get the current window rectangle, in pixels.
    ///
    /// The returned Rect struct has members `x`, `y`, `width`, `height`,
    /// all i32.
    pub async fn get_window_rect(&self) -> WebDriverResult<Rect> {
        let v = self
            .conn
            .execute(Command::GetWindowRect(&self.session_id))
            .await?;
        unwrap(&v["value"])
    }

    /// Set the current window rectangle, in pixels.
    ///
    /// This requires an OptionRect, which is similar to Rect except all
    /// members are wrapped in Option.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::OptionRect;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// let r = OptionRect::new().with_size(1280, 720);
    /// driver.set_window_rect(r).await?;
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// You can also convert from a Rect if you want to get the window size
    /// and modify it before setting it again.
    /// ```rust
    /// use thirtyfour::OptionRect;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// let rect = driver.get_window_rect().await?;
    /// let option_rect = OptionRect::from(rect);
    /// driver.set_window_rect(option_rect.with_width(1024)).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn set_window_rect(&self, rect: OptionRect) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SetWindowRect(&self.session_id, rect))
            .await
            .map(|_| ())
    }

    /// Go back. This is equivalent to clicking the browser's back button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     assert_eq!(driver.title().await?, "Demo Web App");
    /// driver.back().await?;
    /// #     assert_eq!(driver.title().await?, "");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn back(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Back(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Go forward. This is equivalent to clicking the browser's forward button.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     assert_eq!(driver.title().await?, "Demo Web App");
    /// #     driver.back().await?;
    /// #     assert_eq!(driver.title().await?, "");
    /// driver.forward().await?;
    /// #     assert_eq!(driver.title().await?, "Demo Web App");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn forward(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Forward(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Refresh the current page.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     assert_eq!(driver.title().await?, "Demo Web App");
    /// driver.refresh().await?;
    /// #     assert_eq!(driver.title().await?, "Demo Web App");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn refresh(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Refresh(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Get all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     let set_timeouts = TimeoutConfiguration::new(
    /// #         Some(Duration::new(1, 0)),
    /// #         Some(Duration::new(2, 0)),
    /// #         Some(Duration::new(3, 0))
    /// #     );
    /// #     driver.set_timeouts(set_timeouts.clone()).await?;
    /// let timeouts = driver.get_timeouts().await?;
    /// println!("Page load timeout = {:?}", timeouts.page_load());
    /// #     assert_eq!(timeouts.script(), Some(Duration::new(1, 0)));
    /// #     assert_eq!(timeouts.page_load(), Some(Duration::new(2, 0)));
    /// #     assert_eq!(timeouts.implicit(), Some(Duration::new(3, 0)));
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn get_timeouts(&self) -> WebDriverResult<TimeoutConfiguration> {
        let v = self
            .conn
            .execute(Command::GetTimeouts(&self.session_id))
            .await?;
        unwrap(&v["value"])
    }

    /// Set all timeouts for the current session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// // Setting timeouts to None means those timeout values will not be updated.
    /// let timeouts = TimeoutConfiguration::new(None, Some(Duration::new(11, 0)), None);
    /// driver.set_timeouts(timeouts.clone()).await?;
    /// #     let got_timeouts = driver.get_timeouts().await?;
    /// #     assert_eq!(timeouts.page_load(), Some(Duration::new(11, 0)));
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn set_timeouts(&self, timeouts: TimeoutConfiguration) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SetTimeouts(&self.session_id, timeouts))
            .await
            .map(|_| ())
    }

    /// Set the implicit wait timeout.
    pub async fn implicitly_wait(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, None, Some(time_to_wait));
        self.set_timeouts(timeouts).await
    }

    /// Set the script timeout.
    pub async fn set_script_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(Some(time_to_wait), None, None);
        self.set_timeouts(timeouts).await
    }

    /// Set the page load timeout.
    pub async fn set_page_load_timeout(&self, time_to_wait: Duration) -> WebDriverResult<()> {
        let timeouts = TimeoutConfiguration::new(None, Some(time_to_wait), None);
        self.set_timeouts(timeouts).await
    }

    /// Create a new action chain for this session.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem_text = driver.find_element(By::Name("input1")).await?;
    /// let elem_button = driver.find_element(By::Id("button-set")).await?;
    ///
    /// driver.action_chain()
    ///     .send_keys_to_element(&elem_text, "thirtyfour")
    ///     .move_to_element_center(&elem_button)
    ///     .click()
    ///     .perform().await?;
    /// #     let elem_result = driver.find_element(By::Name("input-result")).await?;
    /// #     assert_eq!(elem_result.text().await?, "thirtyfour");
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
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie).await?;
    /// let cookies = driver.get_cookies().await?;
    /// for cookie in &cookies {
    ///     println!("Got cookie: {}", cookie.value());
    /// }
    /// #     assert_eq!(
    /// #         cookies.iter().filter(|x| x.value() == &serde_json::json!("value")).count(), 1);
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
        let v = self
            .conn
            .execute(Command::GetAllCookies(&self.session_id))
            .await?;
        unwrap_vec::<Cookie>(&v["value"])
    }

    /// Get the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie).await?;
    /// let cookie = driver.get_cookie("key").await?;
    /// println!("Got cookie: {}", cookie.value());
    /// #     assert_eq!(cookie.value(), &serde_json::json!("value"));
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie> {
        let v = self
            .conn
            .execute(Command::GetNamedCookie(&self.session_id, name))
            .await?;
        unwrap::<Cookie>(&v["value"])
    }

    /// Delete the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie).await?;
    /// #     assert!(driver.get_cookie("key").await.is_ok());
    /// driver.delete_cookie("key").await?;
    /// #     assert!(driver.get_cookie("key").await.is_err());
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn delete_cookie(&self, name: &str) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteCookie(&self.session_id, name))
            .await
            .map(|_| ())
    }

    /// Delete all cookies.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     let set_cookie = Cookie::new("key", serde_json::json!("value"));
    /// #     driver.add_cookie(set_cookie).await?;
    /// #     assert!(driver.get_cookie("key").await.is_ok());
    /// driver.delete_all_cookies().await?;
    /// #     assert!(driver.get_cookie("key").await.is_err());
    /// #     assert!(driver.get_cookies().await?.is_empty());
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn delete_all_cookies(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteAllCookies(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Add the specified cookie.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::Cookie;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// # use tokio;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// let cookie = Cookie::new("key", serde_json::json!("value"));
    /// driver.add_cookie(cookie).await?;
    /// #     let got_cookie = driver.get_cookie("key").await?;
    /// #     assert_eq!(got_cookie.value(), &serde_json::json!("value"));
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn add_cookie(&self, cookie: Cookie) -> WebDriverResult<()> {
        self.conn
            .execute(Command::AddCookie(&self.session_id, cookie))
            .await
            .map(|_| ())
    }

    /// Take a screenshot of the current window and return it as a
    /// base64-encoded String.
    pub async fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::TakeScreenshot(&self.session_id))
            .await?;
        unwrap(&v["value"])
    }

    /// Take a screenshot of the current window and return it as PNG bytes.
    pub async fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64().await?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of the current window and write it to the specified
    /// filename.
    pub async fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png().await?;
        let mut file = File::create(path).await?;
        file.write_all(&png).await?;
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
            if let Err(e) = block_on(self.quit()) {
                error!("Failed to close session: {:?}", e);
            }
        }
    }
}
