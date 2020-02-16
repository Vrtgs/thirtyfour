use std::sync::Arc;

use crate::{
    common::{
        command::Command,
        connection_common::{unwrap, unwrap_vec},
    },
    error::{WebDriverError, WebDriverResult},
    sync::{webelement::unwrap_element_sync, Alert, RemoteConnectionSync, WebElement},
    SessionId, WindowHandle,
};

/// Struct for switching between frames/windows/alerts.
pub struct SwitchTo {
    session_id: SessionId,
    conn: Arc<RemoteConnectionSync>,
}

impl SwitchTo {
    /// Create a new SwitchTo struct. This is typically created internally
    /// via a call to `WebDriver::switch_to()`.
    pub fn new(session_id: SessionId, conn: Arc<RemoteConnectionSync>) -> Self {
        SwitchTo { session_id, conn }
    }

    /// Return the element with focus, or the `<body>` element if nothing has focus.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     // Wait for page load.
    /// #     driver.find_element(By::Id("button1"))?;
    /// // If no element has focus, active_element() will return the body tag.
    /// let elem = driver.switch_to().active_element()?;
    /// assert_eq!(elem.tag_name()?, "body");
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// // Now let's manually focus an element and try active_element() again.
    /// driver.execute_script(r#"document.getElementsByName("input1")[0].focus();"#)?;
    /// let elem = driver.switch_to().active_element()?;
    /// elem.send_keys("selenium")?;
    /// #     let elem = driver.find_element(By::Name("input1"))?;
    /// #     assert_eq!(elem.text()?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn active_element(&self) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::GetActiveElement(&self.session_id))?;
        unwrap_element_sync(self.conn.clone(), self.session_id.clone(), &v["value"])
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
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// driver.switch_to().frame_number(0)?;
    /// // We are now inside the iframe.
    /// driver.find_element(By::Id("button1"))?;
    /// driver.switch_to().default_content()?;
    /// // We are now back in the original window.
    /// #     driver.find_element(By::Id("iframeid1"))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn default_content(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameDefault(&self.session_id))
            .map(|_| ())
    }

    /// Switch to an iframe by index. The first iframe on the page has index 0.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// driver.switch_to().frame_number(0)?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.click()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn frame_number(&self, frame_number: u16) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameNumber(&self.session_id, frame_number))
            .map(|_| ())
    }

    /// Switch to the specified iframe element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1"))?;
    /// driver.switch_to().frame_element(&elem_iframe)?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.click()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn frame_element(&self, frame_element: &WebElement) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameElement(
                &self.session_id,
                &frame_element.element_id,
            ))
            .map(|_| ())
    }

    /// Switch to the parent frame.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pageiframe"))?.click()?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1"))?;
    /// driver.switch_to().frame_element(&elem_iframe)?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.click()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// // Now switch back to the parent frame.
    /// driver.switch_to().parent_frame()?;
    /// // We are now back in the parent document.
    /// #     driver.find_element(By::Id("iframeid1"))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn parent_frame(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToParentFrame(&self.session_id))
            .map(|_| ())
    }

    /// Switch to the specified window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// driver.switch_to().window(&handles[1])?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("button1"))?;
    /// #     driver.switch_to().window(&handles[0])?;
    /// #     driver.find_element(By::Name("input1"))?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn window(&self, handle: &WindowHandle) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToWindow(&self.session_id, handle))
            .map(|_| ())
    }

    /// Switch to the window with the specified name. This uses the `window.name` property.
    /// You can set a window name via `WebDriver::set_window_name("someName")?`.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     assert_eq!(driver.title()?, "Demo Web App");
    /// // Set main window name so we can switch back easily.
    /// driver.set_window_name("mywindow")?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#)?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles()?;
    /// driver.switch_to().window(&handles[1])?;
    /// // We are now controlling the new tab.
    /// assert_eq!(driver.title()?, "");
    /// driver.switch_to().window_name("mywindow")?;
    /// // We are now back in the original tab.
    /// assert_eq!(driver.title()?, "Demo Web App");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn window_name(&self, name: &str) -> WebDriverResult<()> {
        let original_handle = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id))
            .map(|v| unwrap::<String>(&v["value"]))??;

        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))?;
        let handles: Vec<String> = unwrap_vec(&v["value"])?;
        for handle in handles {
            self.window(&WindowHandle::from(handle))?;
            let current_name = self
                .conn
                .execute(Command::ExecuteScript(
                    &self.session_id,
                    String::from("return window.name;"),
                    Vec::new(),
                ))
                .map(|v| v["value"].clone())?;

            if let Some(x) = current_name.as_str() {
                if x == name {
                    return Ok(());
                }
            }
        }

        self.window(&WindowHandle::from(original_handle))?;
        Err(WebDriverError::NotFoundError(format!(
            "No window handle found matching '{}'",
            name
        )))
    }
}
