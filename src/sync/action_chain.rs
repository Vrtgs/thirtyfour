use std::sync::Arc;

use crate::{
    common::{
        action::{ActionSource, KeyAction, PointerAction, PointerActionType},
        command::{Actions, Command},
        keys::TypingData,
        types::SessionId,
    },
    error::WebDriverResult,
    sync::{RemoteConnectionSync, WebElement},
};

/// The ActionChain struct allows you to perform multiple input actions in
/// a sequence, including drag-and-drop, send keystrokes to an element, and
/// hover the mouse over an element.
///
/// The easiest way to construct an ActionChain struct is via the WebDriver
/// struct.
///
/// # Example:
/// ```ignore
/// driver.action_chain().drag_and_drop_element(elem_src, elem_target).perform()?;
/// ```
pub struct ActionChain {
    conn: Arc<RemoteConnectionSync>,
    session_id: SessionId,
    key_actions: ActionSource<KeyAction>,
    pointer_actions: ActionSource<PointerAction>,
}

impl ActionChain {
    /// Create a new ActionChain struct.
    ///
    /// See [WebDriver::action_chain()](../struct.WebDriver.html#method.action_chain)
    /// for more details.
    pub fn new(conn: Arc<RemoteConnectionSync>, session_id: SessionId) -> Self {
        ActionChain {
            conn,
            session_id,
            key_actions: ActionSource::<KeyAction>::new("key"),
            pointer_actions: ActionSource::<PointerAction>::new(
                "pointer",
                PointerActionType::Mouse,
            ),
        }
    }

    /// Perform the action sequence. No actions are actually performed until
    /// this method is called.
    pub fn perform(&self) -> WebDriverResult<()> {
        let actions = Actions::from(serde_json::json!([self.key_actions, self.pointer_actions]));
        self.conn
            .execute(Command::PerformActions(&self.session_id, actions))?;
        Ok(())
    }

    /// Click and release the left mouse button.
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
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().move_to_element_center(&elem).click().perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn click(mut self) -> Self {
        self.pointer_actions.click();
        // Click = 2 actions (PointerDown + PointerUp).
        self.key_actions.pause();
        self.key_actions.pause();
        self
    }

    /// Click on the specified element using the left mouse button and release.
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
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().click_element(&elem).perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).click()
    }

    /// Click the left mouse button and hold it down.
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
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "None");
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().move_to_element_center(&elem).click_and_hold().perform()?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 down");
    /// #     driver.action_chain().release().perform()?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn click_and_hold(mut self) -> Self {
        self.pointer_actions.click_and_hold();
        self.key_actions.pause();
        self
    }

    /// Click on the specified element using the left mouse button and
    /// hold the button down.
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
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "None");
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().click_and_hold_element(&elem).perform()?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 down");
    /// #     driver.action_chain().release().perform()?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn click_and_hold_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).click_and_hold()
    }

    /// Click and release the right mouse button.
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
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().move_to_element_center(&elem).context_click().perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 right-clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn context_click(mut self) -> Self {
        self.pointer_actions.context_click();
        // Click = 2 actions (PointerDown + PointerUp).
        self.key_actions.pause();
        self.key_actions.pause();
        self
    }

    /// Click on the specified element using the right mouse button and release.
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
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().context_click_element(&elem).perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 right-clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn context_click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).context_click()
    }

    /// Double-click the left mouse button.
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
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().move_to_element_center(&elem).double_click().perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 double-clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn double_click(mut self) -> Self {
        self.pointer_actions.double_click();
        // Each click = 2 actions (PointerDown + PointerUp).
        for _ in 0..4 {
            self.key_actions.pause();
        }
        self
    }

    /// Double-click on the specified element.
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
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// driver.action_chain().double_click_element(&elem).perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 double-clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn double_click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).double_click()
    }

    /// Drag the mouse cursor from the center of the source element to the
    /// center of the target element.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagedragdrop"))?.click()?;
    /// let elem_src = driver.find_element(By::Id("buttondrag1"))?;
    /// let elem_tgt = driver.find_element(By::Id("dragdrop-result"))?;
    /// driver.action_chain()
    ///     .drag_and_drop_element(&elem_src, &elem_tgt)
    ///     .perform()?;
    /// #     assert_eq!(elem_tgt.text()?, "Dropped BUTTON 1");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn drag_and_drop_element(self, source: &WebElement, target: &WebElement) -> Self {
        self.click_and_hold_element(source)
            .release_on_element(target)
    }

    /// Drag the mouse cursor by the specified X and Y offsets.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagedragdrop"))?.click()?;
    /// // Drag source element by offset.
    /// let elem_src = driver.find_element(By::Id("buttondrag1"))?;
    /// let elem_tgt = driver.find_element(By::Id("dragdrop-result"))?;
    /// let offset = elem_tgt.rect()?.icenter().0 - elem_src.rect()?.icenter().0;
    /// driver.action_chain()
    ///     .move_to_element_center(&elem_src)
    ///     .drag_and_drop_by_offset(offset, 0)
    ///     .perform()?;
    /// #     assert_eq!(elem_tgt.text()?, "Dropped BUTTON 1");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn drag_and_drop_by_offset(self, x_offset: i32, y_offset: i32) -> Self {
        self.click_and_hold()
            .move_by_offset(x_offset, y_offset)
            .release()
    }

    /// Drag the mouse cursor by the specified X and Y offsets, starting
    /// from the center of the specified element.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagedragdrop"))?.click()?;
    /// // Drag source element by offset.
    /// let elem_src = driver.find_element(By::Id("buttondrag1"))?;
    /// let elem_tgt = driver.find_element(By::Id("dragdrop-result"))?;
    /// let offset = elem_tgt.rect()?.icenter().0 - elem_src.rect()?.icenter().0;
    /// driver.action_chain()
    ///     .drag_and_drop_element_by_offset(&elem_src, offset, 0)
    ///     .perform()?;
    /// #     assert_eq!(elem_tgt.text()?, "Dropped BUTTON 1");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn drag_and_drop_element_by_offset(
        self,
        element: &WebElement,
        x_offset: i32,
        y_offset: i32,
    ) -> Self {
        self.click_and_hold_element(element)
            .move_by_offset(x_offset, y_offset)
    }

    /// Press the specified key down.
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
    /// let elem = driver.find_element(By::Name("input1"))?;
    /// #     assert_eq!(elem.text()?, "");
    /// driver.action_chain().click_element(&elem).key_down('a').perform()?;
    /// #     assert_eq!(elem.text()?, "a");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn key_down<T>(mut self, value: T) -> Self
    where
        T: Into<char>,
    {
        self.key_actions.key_down(value.into());
        self.pointer_actions.pause();
        self
    }

    /// Click the specified element and then press the specified key down.
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
    /// let elem = driver.find_element(By::Name("input1"))?;
    /// #     assert_eq!(elem.text()?, "");
    /// driver.action_chain().key_down_on_element(&elem, 'a').perform()?;
    /// #     assert_eq!(elem.text()?, "a");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn key_down_on_element<T>(self, element: &WebElement, value: T) -> Self
    where
        T: Into<char>,
    {
        self.click_element(element).key_down(value)
    }

    /// Release the specified key. This usually follows a `key_down()` action.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::Keys;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem = driver.find_element(By::Name("input1"))?;
    /// #     assert_eq!(elem.text()?, "");
    /// elem.send_keys("selenium")?;
    /// assert_eq!(elem.text()?, "selenium");
    /// driver.action_chain()
    ///     .key_down_on_element(&elem, Keys::Control).key_down('a')
    ///     .key_up(Keys::Control).key_up('a')
    ///     .key_down('b')
    ///     .perform()?;
    /// assert_eq!(elem.text()?, "b");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn key_up<T>(mut self, value: T) -> Self
    where
        T: Into<char>,
    {
        self.key_actions.key_up(value.into());
        self.pointer_actions.pause();
        self
    }

    /// Click the specified element and release the specified key.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::Keys;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem = driver.find_element(By::Name("input1"))?;
    /// #     assert_eq!(elem.text()?, "");
    /// elem.send_keys("selenium")?;
    /// assert_eq!(elem.text()?, "selenium");
    /// driver.action_chain()
    ///     .key_down_on_element(&elem, Keys::Control).key_down('a')
    ///     .key_up_on_element(&elem, 'a').key_up_on_element(&elem, Keys::Control)
    ///     .key_down('b')
    ///     .perform()?;
    /// assert_eq!(elem.text()?, "b");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn key_up_on_element<T>(self, element: &WebElement, value: T) -> Self
    where
        T: Into<char>,
    {
        self.click_element(element).key_up(value)
    }

    /// Move the mouse cursor to the specified X and Y coordinates.
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
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// let center = elem.rect()?.icenter();
    /// driver.action_chain()
    ///     .move_to(center.0, center.1)
    ///     .click()
    ///     .perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn move_to(mut self, x: i32, y: i32) -> Self {
        self.pointer_actions.move_to(x, y);
        self.key_actions.pause();
        self
    }

    /// Move the mouse cursor by the specified X and Y offsets.
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
    /// let elem1 = driver.find_element(By::Id("button1"))?;
    /// let elem2 = driver.find_element(By::Id("button2"))?;
    /// // We will calculate the distance between the two center points and then
    /// // use action_chain() to move to the second button before clicking.
    /// let offset = elem2.rect()?.center().0 as i32 - elem1.rect()?.center().0 as i32;
    /// driver.action_chain()
    ///     .move_to_element_center(&elem1)
    ///     .move_by_offset(offset, 0)
    ///     .click()
    ///     .perform()?;
    /// #     let elem_result = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Button 2 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn move_by_offset(mut self, x_offset: i32, y_offset: i32) -> Self {
        self.pointer_actions.move_by(x_offset, y_offset);
        self.key_actions.pause();
        self
    }

    /// Move the mouse cursor to the center of the specified element.
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
    /// #     driver.find_element(By::Id("pagedropdown"))?.click()?;
    /// // First we will hover over the dropdown parent and then select the
    /// // second item in the list.
    /// let elem_dropdown = driver.find_element(By::Id("dropdown-outer"))?;
    /// let elem_option = driver.find_element(By::Id("option2"))?;
    /// driver.action_chain()
    ///     .move_to_element_center(&elem_dropdown)
    ///     .click_element(&elem_option)
    ///     .perform()?;
    /// #     let elem_result = driver.find_element(By::Id("dropdown-result"))?;
    /// #     assert_eq!(elem_result.text()?, "Option 2 selected");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn move_to_element_center(mut self, element: &WebElement) -> Self {
        self.pointer_actions
            .move_to_element_center(element.element_id.clone());
        self.key_actions.pause();
        self
    }

    /// Move the mouse cursor to the specified offsets relative to the specified
    /// element's center position.
    ///
    /// # Example:
    ///
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, Keys, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("button1"))?.click()?;
    /// // Select the text in the source element and copy it to the clipboard.
    /// let elem = driver.find_element(By::Id("button-result"))?;
    /// let width = elem.rect()?.width;
    /// driver.action_chain()
    ///     .move_to_element_with_offset(&elem, (-width / 2.0) as i32, 0)
    ///     .drag_and_drop_by_offset(width as i32, 0)
    ///     .key_down(Keys::Control)
    ///     .key_down('c').key_up('c')
    ///     .key_up(Keys::Control)
    ///     .perform()?;
    ///
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// // Now paste the text into the input field.
    /// let elem_tgt = driver.find_element(By::Name("input1"))?;
    /// elem_tgt.send_keys(Keys::Control + "v")?;
    /// #     assert_eq!(elem_tgt.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn move_to_element_with_offset(
        mut self,
        element: &WebElement,
        x_offset: i32,
        y_offset: i32,
    ) -> Self {
        self.pointer_actions
            .move_to_element(element.element_id.clone(), x_offset, y_offset);
        self.key_actions.pause();
        self
    }

    /// Release the left mouse button.
    pub fn release(mut self) -> Self {
        self.pointer_actions.release();
        self.key_actions.pause();
        self
    }

    /// Move the mouse to the specified element and release the mouse button.
    pub fn release_on_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).release()
    }

    /// Send the specified keystrokes to the active element.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::Keys;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem = driver.find_element(By::Name("input1"))?;
    /// let button = driver.find_element(By::Id("button-set"))?;
    /// #     assert_eq!(elem.text()?, "");
    /// driver.action_chain()
    ///     .click_element(&elem)
    ///     .send_keys("selenium")
    ///     .click_element(&button)
    ///     .perform()?;
    /// #     let elem_result = driver.find_element(By::Name("input-result"))?;
    /// #     assert_eq!(elem_result.text()?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send_keys<S>(mut self, text: S) -> Self
    where
        S: Into<TypingData>,
    {
        let typing: TypingData = text.into();
        for c in typing.as_vec() {
            self = self.key_down(c).key_up(c);
        }
        self
    }

    /// Click on the specified element and send the specified keystrokes.
    ///
    /// # Example:
    /// ```rust
    /// use thirtyfour::Keys;
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem = driver.find_element(By::Name("input1"))?;
    /// let button = driver.find_element(By::Id("button-set"))?;
    /// #     assert_eq!(elem.text()?, "");
    /// driver.action_chain()
    ///     .send_keys_to_element(&elem, "selenium")
    ///     .click_element(&button)
    ///     .perform()?;
    /// #     let elem_result = driver.find_element(By::Name("input-result"))?;
    /// #     assert_eq!(elem_result.text()?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send_keys_to_element<S>(self, element: &WebElement, text: S) -> Self
    where
        S: Into<TypingData>,
    {
        self.click_element(element).send_keys(text)
    }
}
