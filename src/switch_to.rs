use std::sync::Arc;

use crate::{
    common::{
        command::Command,
        connection_common::{unwrap, unwrap_vec},
    },
    error::{WebDriverError, WebDriverResult},
    webelement::unwrap_element_async,
    Alert, RemoteConnectionAsync, SessionId, WebElement, WindowHandle,
};

/// Struct for switching between frames/windows/alerts.
pub struct SwitchTo {
    session_id: SessionId,
    conn: Arc<RemoteConnectionAsync>,
}

impl SwitchTo {
    /// Create a new SwitchTo struct. This is typically created internally
    /// via a call to `WebDriver::switch_to()`.
    pub fn new(session_id: SessionId, conn: Arc<RemoteConnectionAsync>) -> Self {
        SwitchTo { session_id, conn }
    }

    /// Return the element with focus, or the `<body>` element if nothing has focus.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     // Wait for page load.
    /// #     driver.find_element(By::Id("button1")).await?;
    /// // If no element has focus, active_element() will return the body tag.
    /// let elem = driver.switch_to().active_element().await?;
    /// assert_eq!(elem.tag_name().await?, "body");
    /// #     driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// // Now let's manually focus an element and try active_element() again.
    /// driver.execute_script(r#"document.getElementsByName("input1")[0].focus();"#).await?;
    /// let elem = driver.switch_to().active_element().await?;
    /// elem.send_keys("selenium").await?;
    /// #     let elem = driver.find_element(By::Name("input1")).await?;
    /// #     assert_eq!(elem.text().await?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn active_element(&self) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::GetActiveElement(&self.session_id))
            .await?;
        unwrap_element_async(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    /// Return Alert struct for processing the active alert on the page.
    ///
    /// See [Alert](struct.Alert.html) documentation for examples.
    pub fn alert(&self) -> Alert {
        Alert::new(self.session_id.clone(), self.conn.clone())
    }

    /// Switch to the default frame.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// driver.switch_to().frame_number(0).await?;
    /// // We are now inside the iframe.
    /// driver.find_element(By::Id("button1")).await?;
    /// driver.switch_to().default_content().await?;
    /// // We are now back in the original window.
    /// #     driver.find_element(By::Id("iframeid1")).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn default_content(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameDefault(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Switch to an iframe by index. The first iframe on the page has index 0.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// driver.switch_to().frame_number(0).await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #     let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #     assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn frame_number(&self, frame_number: u16) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameNumber(&self.session_id, frame_number))
            .await
            .map(|_| ())
    }

    /// Switch to the specified iframe element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1")).await?;
    /// driver.switch_to().frame_element(&elem_iframe).await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #     let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #     assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn frame_element(&self, frame_element: &WebElement) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameElement(
                &self.session_id,
                &frame_element.element_id,
            ))
            .await
            .map(|_| ())
    }

    /// Switch to the parent frame.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1")).await?;
    /// driver.switch_to().frame_element(&elem_iframe).await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #     let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #     assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// // Now switch back to the parent frame.
    /// driver.switch_to().parent_frame().await?;
    /// // We are now back in the parent document.
    /// #     driver.find_element(By::Id("iframeid1")).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn parent_frame(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToParentFrame(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Switch to the specified window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #     assert_eq!(driver.title().await?, "Demo Web App");
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(&handles[1]).await?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("button1")).await?;
    /// #     driver.switch_to().window(&handles[0]).await?;
    /// #     driver.find_element(By::Name("input1")).await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn window(&self, handle: &WindowHandle) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToWindow(&self.session_id, handle))
            .await
            .map(|_| ())
    }

    /// Switch to the window with the specified name. This uses the `window.name` property.
    /// You can set a window name via `WebDriver::set_window_name("someName").await?`.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     assert_eq!(driver.title().await?, "Demo Web App");
    /// // Set main window name so we can switch back easily.
    /// driver.set_window_name("mywindow").await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(&handles[1]).await?;
    /// // We are now controlling the new tab.
    /// assert_eq!(driver.title().await?, "");
    /// driver.switch_to().window_name("mywindow").await?;
    /// // We are now back in the original tab.
    /// assert_eq!(driver.title().await?, "Demo Web App");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn window_name(&self, name: &str) -> WebDriverResult<()> {
        let original_handle = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id))
            .await
            .map(|v| unwrap::<String>(&v["value"]))??;

        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))
            .await?;
        let handles: Vec<String> = unwrap_vec(&v["value"])?;
        for handle in handles {
            self.window(&WindowHandle::from(handle)).await?;
            let current_name = self
                .conn
                .execute(Command::ExecuteScript(
                    &self.session_id,
                    String::from("return window.name;"),
                    Vec::new(),
                ))
                .await
                .map(|v| v["value"].clone())?;

            if let Some(x) = current_name.as_str() {
                if x == name {
                    return Ok(());
                }
            }
        }

        self.window(&WindowHandle::from(original_handle)).await?;
        Err(WebDriverError::NotFoundError(format!(
            "No window handle found matching '{}'",
            name
        )))
    }
}
