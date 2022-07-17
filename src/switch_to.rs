use crate::session::handle::SessionHandle;
use crate::{
    error::{WebDriverError, WebDriverResult},
    Alert, WebElement,
};
use fantoccini::elements::Element;
use fantoccini::wd::WindowHandle;

/// Struct for switching between frames/windows/alerts.
pub struct SwitchTo {
    handle: SessionHandle,
}

impl SwitchTo {
    /// Create a new SwitchTo struct. This is typically created internally
    /// via a call to `WebDriver::switch_to()`.
    pub fn new(handle: SessionHandle) -> Self {
        Self {
            handle,
        }
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been moved to WebDriver::active_element()"
    )]
    pub async fn active_element(self) -> WebDriverResult<WebElement> {
        let elem = self.handle.client.active_element().await?;
        Ok(WebElement::new(elem, self.handle.clone()))
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been deprecated. See the `Alert` module for new method names"
    )]
    pub fn alert(self) -> Alert {
        Alert::new(self.handle)
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been moved to WebDriver::enter_default_frame()"
    )]
    pub async fn default_content(self) -> WebDriverResult<()> {
        self.handle.client.enter_frame(None).await?;
        Ok(())
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been moved to WebDriver::enter_frame()"
    )]
    pub async fn frame_number(self, frame_number: u16) -> WebDriverResult<()> {
        self.handle.client.enter_frame(Some(frame_number)).await?;
        Ok(())
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been moved to WebElement::enter_frame()"
    )]
    pub async fn frame_element(self, frame_element: &WebElement) -> WebDriverResult<()> {
        let frame = Element::from_element_id(self.handle.client, frame_element.element_id());
        frame.enter_frame().await?;
        Ok(())
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been moved to WebDriver::enter_parent_frame()"
    )]
    pub async fn parent_frame(self) -> WebDriverResult<()> {
        self.handle.client.enter_parent_frame().await?;
        Ok(())
    }

    #[deprecated(since = "v0.30.0", note = "This method has been moved to WebDriver::new_window()")]
    pub async fn new_window(self) -> WebDriverResult<WindowHandle> {
        let response = self.handle.client.new_window(false).await?;
        Ok(response.handle)
    }

    #[deprecated(since = "v0.30.0", note = "This method has been moved to WebDriver::new_tab()")]
    pub async fn new_tab(self) -> WebDriverResult<WindowHandle> {
        let response = self.handle.client.new_window(true).await?;
        Ok(response.handle)
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been moved to WebDriver::switch_to_window()"
    )]
    pub async fn window(self, handle: WindowHandle) -> WebDriverResult<()> {
        self.handle.client.switch_to_window(handle).await?;
        Ok(())
    }

    #[deprecated(
        since = "v0.30.0",
        note = "This method has been moved to WebDriver::switch_to_named_window()"
    )]
    pub async fn window_name(self, name: &str) -> WebDriverResult<()> {
        let original_handle = self.handle.window().await?;
        let handles = self.handle.windows().await?;
        for handle in handles {
            self.handle.switch_to_window(handle).await?;
            let ret = self.handle.execute(r#"return window.name;"#, Vec::new()).await?;
            let current_name: String = ret.convert()?;
            if current_name == name {
                return Ok(());
            }
        }

        self.handle.switch_to_window(original_handle).await?;
        Err(WebDriverError::NoSuchWindow(name.to_string()))
    }
}

impl SessionHandle {
    /// Return the element with focus, or the `<body>` element if nothing has focus.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         // Wait for page load.
    /// #         driver.find(By::Id("button1")).await?;
    /// // If no element has focus, active_element() will return the body tag.
    /// let elem = driver.active_element().await?;
    /// assert_eq!(elem.tag_name().await?, "body");
    /// #         driver.find(By::Id("pagetextinput")).await?.click().await?;
    /// // Now let's manually focus an element and try active_element() again.
    /// driver.execute(r#"document.getElementsByName("input1")[0].focus();"#, Vec::new()).await?;
    /// let elem = driver.active_element().await?;
    /// elem.send_keys("selenium").await?;
    /// #         let elem = driver.find(By::Name("input1")).await?;
    /// #         assert_eq!(elem.value().await?, Some("selenium".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn active_element(&self) -> WebDriverResult<WebElement> {
        let elem = self.client.active_element().await?;
        Ok(WebElement::new(elem, self.clone()))
    }

    /// Switch to the default frame.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         driver.find(By::Id("pageiframe")).await?.click().await?;
    /// driver.enter_frame(0).await?;
    /// // We are now inside the iframe.
    /// driver.find(By::Id("button1")).await?;
    /// driver.enter_default_frame().await?;
    /// // We are now back in the original window.
    /// #         driver.find(By::Id("iframeid1")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn enter_default_frame(&self) -> WebDriverResult<()> {
        self.client.enter_frame(None).await?;
        Ok(())
    }

    /// Switch to an iframe by index. The first iframe on the page has index 0.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         driver.find(By::Id("pageiframe")).await?.click().await?;
    /// driver.enter_frame(0).await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #         let elem_result = driver.find(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn enter_frame(&self, frame_number: u16) -> WebDriverResult<()> {
        self.client.enter_frame(Some(frame_number)).await?;
        Ok(())
    }

    /// Switch to the parent frame.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         driver.find(By::Id("pageiframe")).await?.click().await?;
    /// let elem_iframe = driver.find(By::Id("iframeid1")).await?;
    /// elem_iframe.enter_frame().await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #         let elem_result = driver.find(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// // Now switch back to the parent frame.
    /// driver.enter_parent_frame().await?;
    /// // We are now back in the parent document.
    /// #         driver.find(By::Id("iframeid1")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn enter_parent_frame(&self) -> WebDriverResult<()> {
        self.client.enter_parent_frame().await?;
        Ok(())
    }

    /// Switch to the specified window.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         driver.find(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Open a new tab.
    /// driver.new_tab().await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.windows().await?;
    /// driver.switch_to_window(handles[1].clone()).await?;
    /// // We are now controlling the new tab.
    /// driver.goto("http://webappdemo").await?;
    /// #         driver.find(By::Id("button1")).await?;
    /// #         driver.switch_to_window(handles[0].clone()).await?;
    /// #         driver.find(By::Name("input1")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn switch_to_window(&self, handle: WindowHandle) -> WebDriverResult<()> {
        self.client.switch_to_window(handle).await?;
        Ok(())
    }

    /// Switch to the window with the specified name. This uses the `window.name` property.
    /// You can set a window name via `WebDriver::set_window_name("someName").await?`.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Set main window name so we can switch back easily.
    /// driver.set_window_name("mywindow").await?;
    /// // Open a new tab.
    /// driver.new_tab().await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.windows().await?;
    /// driver.switch_to_window(handles[1].clone()).await?;
    /// // We are now controlling the new tab.
    /// assert_eq!(driver.title().await?, "");
    /// driver.switch_to_named_window("mywindow").await?;
    /// // We are now back in the original tab.
    /// assert_eq!(driver.title().await?, "Demo Web App");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn switch_to_named_window(&self, name: &str) -> WebDriverResult<()> {
        let original_handle = self.window().await?;
        let handles = self.windows().await?;
        for handle in handles {
            self.switch_to_window(handle).await?;
            let ret = self.execute(r#"return window.name;"#, Vec::new()).await?;
            let current_name: String = ret.convert()?;
            if current_name == name {
                return Ok(());
            }
        }

        self.switch_to_window(original_handle).await?;
        Err(WebDriverError::NoSuchWindow(name.to_string()))
    }

    /// Switch to a new window.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         driver.find(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Open a new window.
    /// let handle = driver.new_window().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn new_window(&self) -> WebDriverResult<WindowHandle> {
        let response = self.client.new_window(false).await?;
        Ok(response.handle)
    }

    /// Switch to a new tab.
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
    /// #         driver.goto("http://webappdemo").await?;
    /// #         driver.find(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Open a new window.
    /// let handle = driver.new_tab().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn new_tab(&self) -> WebDriverResult<WindowHandle> {
        let response = self.client.new_window(true).await?;
        Ok(response.handle)
    }
}
