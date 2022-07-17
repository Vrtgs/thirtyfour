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
    /// #         driver.get("http://webappdemo").await?;
    /// #         // Wait for page load.
    /// #         driver.find_element(By::Id("button1")).await?;
    /// // If no element has focus, active_element() will return the body tag.
    /// let elem = driver.switch_to().active_element().await?;
    /// assert_eq!(elem.tag_name().await?, "body");
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// // Now let's manually focus an element and try active_element() again.
    /// driver.execute_script(r#"document.getElementsByName("input1")[0].focus();"#, Vec::new()).await?;
    /// let elem = driver.switch_to().active_element().await?;
    /// elem.send_keys("selenium").await?;
    /// #         let elem = driver.find_element(By::Name("input1")).await?;
    /// #         assert_eq!(elem.value().await?, Some("selenium".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn active_element(self) -> WebDriverResult<WebElement> {
        let elem = self.handle.client.active_element().await?;
        Ok(WebElement::new(elem, self.handle.clone()))
    }

    /// Return Alert struct for processing the active alert on the page.
    ///
    /// See [Alert](struct.Alert.html) documentation for examples.
    pub fn alert(self) -> Alert {
        Alert::new(self.handle)
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// driver.switch_to().frame_number(0).await?;
    /// // We are now inside the iframe.
    /// driver.find_element(By::Id("button1")).await?;
    /// driver.switch_to().default_content().await?;
    /// // We are now back in the original window.
    /// #         driver.find_element(By::Id("iframeid1")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn default_content(self) -> WebDriverResult<()> {
        self.handle.client.enter_frame(None).await?;
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// driver.switch_to().frame_number(0).await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn frame_number(self, frame_number: u16) -> WebDriverResult<()> {
        self.handle.client.enter_frame(Some(frame_number)).await?;
        Ok(())
    }

    /// Switch to the specified iframe element.
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1")).await?;
    /// driver.switch_to().frame_element(&elem_iframe).await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn frame_element(self, frame_element: &WebElement) -> WebDriverResult<()> {
        let frame = Element::from_element_id(self.handle.client, frame_element.element_id());
        frame.enter_frame().await?;
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pageiframe")).await?.click().await?;
    /// let elem_iframe = driver.find_element(By::Id("iframeid1")).await?;
    /// driver.switch_to().frame_element(&elem_iframe).await?;
    /// // We can now search for elements within the iframe.
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// // Now switch back to the parent frame.
    /// driver.switch_to().parent_frame().await?;
    /// // We are now back in the parent document.
    /// #         driver.find_element(By::Id("iframeid1")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn parent_frame(self) -> WebDriverResult<()> {
        self.handle.client.enter_parent_frame().await?;
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#, Vec::new()).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(handles[1].clone()).await?;
    /// // We are now controlling the new tab.
    /// driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("button1")).await?;
    /// #         driver.switch_to().window(handles[0].clone()).await?;
    /// #         driver.find_element(By::Name("input1")).await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn window(self, handle: WindowHandle) -> WebDriverResult<()> {
        self.handle.client.switch_to_window(handle).await?;
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
    /// #         driver.get("http://webappdemo").await?;
    /// #         assert_eq!(driver.title().await?, "Demo Web App");
    /// // Set main window name so we can switch back easily.
    /// driver.set_window_name("mywindow").await?;
    /// // Open a new tab.
    /// driver.execute_script(r#"window.open("about:blank", target="_blank");"#, Vec::new()).await?;
    /// // Get window handles and switch to the new tab.
    /// let handles = driver.window_handles().await?;
    /// driver.switch_to().window(handles[1].clone()).await?;
    /// // We are now controlling the new tab.
    /// assert_eq!(driver.title().await?, "");
    /// driver.switch_to().window_name("mywindow").await?;
    /// // We are now back in the original tab.
    /// assert_eq!(driver.title().await?, "Demo Web App");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn window_name(self, name: &str) -> WebDriverResult<()> {
        let original_handle = self.handle.current_window_handle().await?;
        let handles = self.handle.window_handles().await?;
        for handle in handles {
            self.handle.switch_to().window(handle).await?;
            let ret = self.handle.execute_script(r#"return window.name;"#, Vec::new()).await?;
            let current_name: String = ret.convert()?;
            if current_name == name {
                return Ok(());
            }
        }

        self.window(original_handle).await?;
        Err(WebDriverError::NoSuchWindow(name.to_string()))
    }
}
