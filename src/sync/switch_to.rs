use crate::sync::webdriver::{WebDriverCommands, WebDriverSession};
use crate::{
    common::command::Command,
    error::{WebDriverError, WebDriverResult},
    sync::{webelement::unwrap_element_sync, Alert, WebElement},
    WindowHandle,
};

/// Struct for switching between frames/windows/alerts.
pub struct SwitchTo<'a> {
    driver: WebDriverSession<'a>,
}

impl<'a> SwitchTo<'a> {
    /// Create a new SwitchTo struct. This is typically created internally
    /// via a call to `WebDriver::switch_to()`.
    pub fn new(driver: WebDriverSession<'a>) -> Self {
        SwitchTo { driver }
    }

    ///Convenience wrapper for executing a WebDriver command.
    fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.driver.cmd(command)
    }

    /// Return the element with focus, or the `<body>` element if nothing has focus.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
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
    pub fn active_element(self) -> WebDriverResult<WebElement<'a>> {
        let v = self.cmd(Command::GetActiveElement)?;
        unwrap_element_sync(self.driver, &v["value"])
    }

    /// Return Alert struct for processing the active alert on the page.
    ///
    /// See [Alert](struct.Alert.html) documentation for examples.
    pub fn alert(self) -> Alert<'a> {
        Alert::new(self.driver)
    }

    /// Switch to the default frame.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
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
    pub fn default_content(self) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToFrameDefault).map(|_| ())
    }

    /// Switch to an iframe by index. The first iframe on the page has index 0.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
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
    pub fn frame_number(self, frame_number: u16) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToFrameNumber(frame_number))
            .map(|_| ())
    }

    /// Switch to the specified iframe element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
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
    pub fn frame_element(self, frame_element: &WebElement) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToFrameElement(&frame_element.element_id))
            .map(|_| ())
    }

    /// Switch to the parent frame.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
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
    pub fn parent_frame(self) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToParentFrame).map(|_| ())
    }

    /// Switch to the specified window.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
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
    pub fn window(self, handle: &WindowHandle) -> WebDriverResult<()> {
        self.cmd(Command::SwitchToWindow(handle)).map(|_| ())
    }

    /// Switch to the window with the specified name. This uses the `window.name` property.
    /// You can set a window name via `WebDriver::set_window_name("someName")?`.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
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
    pub fn window_name(self, name: &str) -> WebDriverResult<()> {
        let original_handle = self.driver.current_window_handle()?;
        let this = &self;
        let handles = this.driver.window_handles()?;
        for handle in &handles {
            this.driver.switch_to().window(handle)?;
            let ret = this.driver.execute_script(r#"return window.name;"#)?;
            let current_name: String = ret.convert()?;
            if current_name == name {
                return Ok(());
            }
        }

        self.window(&original_handle)?;
        Err(WebDriverError::NotFoundError(format!(
            "No window handle found matching '{}'",
            name
        )))
    }
}
