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
use base64::decode;
use futures::executor::block_on;
use log::error;
use serde::Deserialize;
use std::{path::Path, sync::Arc, time::Duration};
use tokio::{fs::File, io::AsyncWriteExt};

/// The WebDriver struct encapsulates an async Selenium WebDriver browser
/// session. For the async driver, see
/// [sync::WebDriver](sync/struct.WebDriver.html).
///
/// # Example:
/// ```ignore
/// use thirtyfour::{DesiredCapabilities, WebDriver};
///
/// let caps = DesiredCapabilities::chrome();
/// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
///
/// // Navigate to mozilla.org.
/// driver.get("https://mozilla.org").await?;
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
    /// ```ignore
    /// use thirtyfour::{DesiredCapabilities, WebDriver};
    ///
    /// let caps = DesiredCapabilities::chrome();
    /// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// ```
    pub async fn new(
        remote_server_addr: &str,
        capabilities: &DesiredCapabilities,
    ) -> WebDriverResult<Self> {
        let conn = Arc::new(RemoteConnectionAsync::new(remote_server_addr)?);
        let v = conn.execute(Command::NewSession(capabilities)).await?;

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
        Ok(WebDriver {
            session_id,
            capabilities: actual_capabilities,
            conn,
        })
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
    /// ```ignore
    /// use thirtyfour::By;
    ///
    /// let elem = driver.find_element(By::Id("theElementId")).await?;
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
    /// ```ignore
    /// let elems = driver.find_elements(By::Class("some-class")).await?;
    /// for elem in elems {
    ///     println!("Found element: {}", elem);
    /// }
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
    pub async fn mazimize_window(&self) -> WebDriverResult<()> {
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
    /// ```ignore
    /// use thirtyfour::OptionRect;
    /// let r = OptionRect::new().with_size(1280, 720);
    /// driver.set_window_rect(r).await?;
    /// ```
    ///
    /// You can also convert from a Rect if you want to get the window size
    /// and modify it before setting it again.
    /// ```ignore
    /// let rect = driver.get_window_rect().await?;
    /// let option_rect = OptionRect::from(rect);
    /// driver.set_window_rect(option_rect.with_width(1024)).await?;
    /// ```
    pub async fn set_window_rect(&self, rect: OptionRect) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SetWindowRect(&self.session_id, rect))
            .await
            .map(|_| ())
    }

    /// Go back. This is equivalent to clicking the browser's back button.
    pub async fn back(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Back(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Go forward. This is equivalent to clicking the browser's forward button.
    pub async fn forward(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Forward(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Refresh the current page.
    pub async fn refresh(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::Refresh(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Set all timeouts for the current session.
    ///
    /// # Example:
    /// ```ignore
    /// use thirtyfour::TimeoutConfiguration;
    /// use std::time::Duration;
    /// let timeouts = TimeoutConfiguration::new(None, Some(Duration::new(30, 0)), None);
    /// driver.set_timeouts(timeouts).await?;
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
    /// ```ignore
    /// driver.action_chain().drag_and_drop_element(elem_src, elem_target).perform().await?;
    /// ```
    pub fn action_chain(&self) -> ActionChain {
        ActionChain::new(self.conn.clone(), self.session_id.clone())
    }

    /// Get all cookies.
    pub async fn get_cookies(&self) -> WebDriverResult<Vec<Cookie>> {
        let v = self
            .conn
            .execute(Command::GetAllCookies(&self.session_id))
            .await?;
        unwrap_vec::<Cookie>(&v["value"])
    }

    /// Get the specified cookie.
    pub async fn get_cookie(&self, name: &str) -> WebDriverResult<Cookie> {
        let v = self
            .conn
            .execute(Command::GetNamedCookie(&self.session_id, name))
            .await?;
        unwrap::<Cookie>(&v["value"])
    }

    /// Delete the specified cookie.
    pub async fn delete_cookie(&self, name: &str) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteCookie(&self.session_id, name))
            .await
            .map(|_| ())
    }

    /// Delete all cookies.
    pub async fn delete_all_cookies(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DeleteAllCookies(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Add the specified cookie.
    ///
    /// # Example:
    /// ```ignore
    /// use thirtyfour::Cookie;
    /// let cookie = Cookie::new("key", serde_json::json!("value"));
    /// driver.add_cookie(cookie).await?;
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
