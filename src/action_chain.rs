use crate::session::handle::SessionHandle;
use crate::{error::WebDriverResult, WebElement};
use fantoccini::actions::{
    ActionSequence, InputSource, KeyAction, KeyActions, MouseActions, PointerAction,
    MOUSE_BUTTON_LEFT, MOUSE_BUTTON_RIGHT,
};

use std::time::Duration;

/// The ActionChain struct allows you to perform multiple input actions in
/// a sequence, including drag-and-drop, send keystrokes to an element, and
/// hover the mouse over an element.
///
/// The easiest way to construct an ActionChain struct is via the WebDriver
/// struct.
///
/// # Example:
/// ```ignore
/// driver.action_chain().drag_and_drop_element(elem_src, elem_target).perform().await?;
/// ```
pub struct ActionChain {
    handle: SessionHandle,
    key_actions: Option<KeyActions>,
    mouse_actions: Option<MouseActions>,
}

impl ActionChain {
    /// Create a new ActionChain struct.
    ///
    /// See [WebDriver::action_chain()](../struct.WebDriver.html#method.action_chain)
    /// for more details.
    pub fn new(handle: SessionHandle) -> Self {
        ActionChain {
            handle,
            key_actions: Some(KeyActions::new("key".to_string())),
            mouse_actions: Some(MouseActions::new("mouse".to_string())),
        }
    }

    /// Add a pause for the key sequence. Usually required after adding a mouse event,
    /// to keep the key sequence in sync with the mouse sequence.
    fn add_key_pause(&mut self) {
        self.key_actions = Some(self.key_actions.take().unwrap().pause(Duration::from_millis(0)));
    }

    fn add_key_down(&mut self, key: char) {
        self.key_actions = Some(self.key_actions.take().unwrap().then(KeyAction::Down {
            value: key,
        }));
        self.add_mouse_pause();
    }

    fn add_key_up(&mut self, key: char) {
        self.key_actions = Some(self.key_actions.take().unwrap().then(KeyAction::Up {
            value: key,
        }));
        self.add_mouse_pause();
    }

    /// Add a pause for the mouse sequence. Usually required after adding a key event,
    /// to keep the mouse sequence in sync with the key sequence.
    fn add_mouse_pause(&mut self) {
        self.mouse_actions =
            Some(self.mouse_actions.take().unwrap().pause(Duration::from_millis(0)));
    }

    fn add_mouse_down(&mut self, button: u64) {
        self.mouse_actions = Some(self.mouse_actions.take().unwrap().then(PointerAction::Down {
            button,
        }));
        self.add_key_pause();
    }

    fn add_mouse_up(&mut self, button: u64) {
        self.mouse_actions = Some(self.mouse_actions.take().unwrap().then(PointerAction::Up {
            button,
        }));
        self.add_key_pause();
    }

    fn add_move_to_element(&mut self, element: &WebElement, x_offset: i64, y_offset: i64) {
        self.mouse_actions =
            Some(self.mouse_actions.take().unwrap().then(PointerAction::MoveToElement {
                element: element.element.clone(),
                duration: None,
                x: x_offset,
                y: y_offset,
            }));
        self.add_key_pause();
    }

    fn add_move_to(&mut self, x: i64, y: i64) {
        self.mouse_actions = Some(self.mouse_actions.take().unwrap().then(PointerAction::MoveTo {
            duration: None,
            x,
            y,
        }));
        self.add_key_pause();
    }

    fn add_move_by(&mut self, x: i64, y: i64) {
        self.mouse_actions = Some(self.mouse_actions.take().unwrap().then(PointerAction::MoveBy {
            duration: None,
            x,
            y,
        }));
        self.add_key_pause();
    }

    /// Reset all actions, reverting all input devices back to default states.
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
    /// // Hold mouse button down on element.
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().click_and_hold_element(&elem).perform().await?;
    /// let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// assert_eq!(elem_result.text().await?, "Button 1 down");
    /// // Now reset all actions.
    /// driver.action_chain().reset_actions().await?;
    /// // Mouse button is now released.
    /// assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn reset_actions(&self) -> WebDriverResult<()> {
        self.handle.client.release_actions().await?;
        Ok(())
    }

    /// Perform the action sequence. No actions are actually performed until
    /// this method is called.
    pub async fn perform(self) -> WebDriverResult<()> {
        self.handle
            .client
            .perform_actions(vec![
                ActionSequence::from(self.key_actions.unwrap()),
                ActionSequence::from(self.mouse_actions.unwrap()),
            ])
            .await?;
        Ok(())
    }

    /// Click and release the left mouse button.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().move_to_element_center(&elem).click().perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn click(mut self) -> Self {
        self.add_mouse_down(MOUSE_BUTTON_LEFT);
        self.add_mouse_up(MOUSE_BUTTON_LEFT);
        self
    }

    /// Click on the specified element using the left mouse button and release.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().click_element(&elem).perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn click_element(mut self, element: &WebElement) -> Self {
        self.add_move_to_element(element, 0, 0);
        self.click()
    }

    /// Click the left mouse button and hold it down.
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
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "None");
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().move_to_element_center(&elem).click_and_hold().perform().await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 down");
    /// #         driver.action_chain().release().perform().await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn click_and_hold(mut self) -> Self {
        self.add_mouse_down(MOUSE_BUTTON_LEFT);
        self
    }

    /// Click on the specified element using the left mouse button and
    /// hold the button down.
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
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "None");
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().click_and_hold_element(&elem).perform().await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 down");
    /// #         driver.action_chain().release().perform().await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn click_and_hold_element(mut self, element: &WebElement) -> Self {
        self.add_move_to_element(element, 0, 0);
        self.click_and_hold()
    }

    /// Click and release the right mouse button.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().move_to_element_center(&elem).context_click().perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 right-clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn context_click(mut self) -> Self {
        self.add_mouse_down(MOUSE_BUTTON_RIGHT);
        self.add_mouse_up(MOUSE_BUTTON_RIGHT);
        self
    }

    /// Click on the specified element using the right mouse button and release.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().context_click_element(&elem).perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 right-clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn context_click_element(mut self, element: &WebElement) -> Self {
        self.add_move_to_element(element, 0, 0);
        self.context_click()
    }

    /// Double-click the left mouse button.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().move_to_element_center(&elem).double_click().perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 double-clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn double_click(self) -> Self {
        self.click().click()
    }

    /// Double-click on the specified element.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain().double_click_element(&elem).perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 double-clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn double_click_element(mut self, element: &WebElement) -> Self {
        self.add_move_to_element(element, 0, 0);
        self.double_click()
    }

    /// Drag the mouse cursor from the center of the source element to the
    /// center of the target element.
    ///
    /// ## This method is not working correctly due to a selenium bug.
    /// It appears selenium has a bug in the drag and drop feature
    /// causing it to start the drag but not perform the drop.
    /// See [https://github.com/SeleniumHQ/selenium/issues/8003](https://github.com/SeleniumHQ/selenium/issues/8003)
    ///
    /// This method has been confirmed to produce identical JSON output
    /// compared to the python selenium library (which also fails due to
    /// the same bug).
    pub fn drag_and_drop_element(self, source: &WebElement, target: &WebElement) -> Self {
        self.click_and_hold_element(source).release_on_element(target)
    }

    /// Drag the mouse cursor by the specified X and Y offsets.
    ///
    /// ## This method is not working correctly due to a selenium bug.
    /// It appears selenium has a bug in the drag and drop feature
    /// causing it to start the drag but not perform the drop.
    /// See [https://github.com/SeleniumHQ/selenium/issues/8003](https://github.com/SeleniumHQ/selenium/issues/8003)
    ///
    /// This method has been confirmed to produce identical JSON output
    /// compared to the python selenium library (which also fails due to
    /// the same bug).
    pub fn drag_and_drop_by_offset(self, x_offset: i64, y_offset: i64) -> Self {
        self.click_and_hold().move_by_offset(x_offset, y_offset)
    }

    /// Drag the mouse cursor by the specified X and Y offsets, starting
    /// from the center of the specified element.
    ///
    /// ## This method is not working correctly due to a selenium bug.
    /// It appears selenium has a bug in the drag and drop feature
    /// causing it to start the drag but not perform the drop.
    /// See [https://github.com/SeleniumHQ/selenium/issues/8003](https://github.com/SeleniumHQ/selenium/issues/8003)
    ///
    /// This method has been confirmed to produce identical JSON output
    /// compared to the python selenium library (which also fails due to
    /// the same bug).
    pub fn drag_and_drop_element_by_offset(
        self,
        element: &WebElement,
        x_offset: i64,
        y_offset: i64,
    ) -> Self {
        self.click_and_hold_element(element).move_by_offset(x_offset, y_offset)
    }

    /// Press the specified key down.
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
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// #         assert_eq!(elem.value().await?, Some("".to_string()));
    /// driver.action_chain().click_element(&elem).key_down('a').perform().await?;
    /// #         assert_eq!(elem.value().await?, Some("a".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn key_down(mut self, value: impl Into<char>) -> Self {
        self.add_key_down(value.into());
        self
    }

    /// Click the specified element and then press the specified key down.
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
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// #         assert_eq!(elem.value().await?, Some("".to_string()));
    /// driver.action_chain().key_down_on_element(&elem, 'a').perform().await?;
    /// #         assert_eq!(elem.value().await?, Some("a".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn key_down_on_element(self, element: &WebElement, value: impl Into<char>) -> Self {
        self.click_element(element).key_down(value)
    }

    /// Release the specified key. This usually follows a `key_down()` action.
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
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// #         assert_eq!(elem.value().await?, Some("".to_string()));
    /// elem.send_keys("selenium").await?;
    /// assert_eq!(elem.value().await?, Some("selenium".to_string()));
    /// driver.action_chain()
    ///     .key_down_on_element(&elem, Key::Control).key_down('a')
    ///     .key_up(Key::Control).key_up('a')
    ///     .key_down('b')
    ///     .perform().await?;
    /// assert_eq!(elem.value().await?, Some("b".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn key_up(mut self, value: impl Into<char>) -> Self {
        self.add_key_up(value.into());
        self
    }

    /// Click the specified element and release the specified key.
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
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// #         assert_eq!(elem.value().await?, Some("".to_string()));
    /// elem.send_keys("selenium").await?;
    /// assert_eq!(elem.value().await?, Some("selenium".to_string()));
    /// driver.action_chain()
    ///     .key_down_on_element(&elem, Key::Control).key_down('a')
    ///     .key_up_on_element(&elem, 'a').key_up_on_element(&elem, Key::Control)
    ///     .key_down('b')
    ///     .perform().await?;
    /// assert_eq!(elem.value().await?, Some("b".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn key_up_on_element(self, element: &WebElement, value: impl Into<char>) -> Self {
        self.click_element(element).key_up(value)
    }

    /// Move the mouse cursor to the specified X and Y coordinates.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let center = elem.rect().await?.icenter();
    /// driver.action_chain()
    ///     .move_to(center.0, center.1)
    ///     .click()
    ///     .perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn move_to(mut self, x: i64, y: i64) -> Self {
        self.add_move_to(x, y);
        self
    }

    /// Move the mouse cursor by the specified X and Y offsets.
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
    /// let elem1 = driver.find_element(By::Id("button1")).await?;
    /// let elem2 = driver.find_element(By::Id("button2")).await?;
    /// // We will calculate the distance between the two center points and then
    /// // use action_chain() to move to the second button before clicking.
    /// let offset = elem2.rect().await?.center().0 as i64 - elem1.rect().await?.center().0 as i64;
    /// driver.action_chain()
    ///     .move_to_element_center(&elem1)
    ///     .move_by_offset(offset, 0)
    ///     .click()
    ///     .perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 2 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn move_by_offset(mut self, x_offset: i64, y_offset: i64) -> Self {
        self.add_move_by(x_offset, y_offset);
        self
    }

    /// Move the mouse cursor to the center of the specified element.
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
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// driver.action_chain()
    ///     .move_to_element_center(&elem)
    ///     .click()
    ///     .perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn move_to_element_center(mut self, element: &WebElement) -> Self {
        self.add_move_to_element(element, 0, 0);
        self
    }

    /// Move the mouse cursor to the specified offsets relative to the specified
    /// element's center position.
    ///
    /// # Example:
    ///
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("button1")).await?.click().await?;
    /// // Select the text in the source element and copy it to the clipboard.
    /// let elem = driver.find_element(By::Id("button-result")).await?;
    /// let width = elem.rect().await?.width;
    /// driver.action_chain()
    ///     .move_to_element_with_offset(&elem, (-(width as f64) / 2.0) as i64, 0)
    ///     .drag_and_drop_by_offset(width as i64, 0)
    ///     .key_down(Key::Control)
    ///     .key_down('c').key_up('c')
    ///     .key_up(Key::Control)
    ///     .perform().await?;
    ///
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// // Now paste the text into the input field.
    /// let elem_tgt = driver.find_element(By::Name("input1")).await?;
    /// elem_tgt.send_keys(Key::Control + "v").await?;
    /// #         assert_eq!(elem_tgt.value().await?, Some("Button 1 clicked".to_string()));
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn move_to_element_with_offset(
        mut self,
        element: &WebElement,
        x_offset: i64,
        y_offset: i64,
    ) -> Self {
        self.add_move_to_element(element, x_offset, y_offset);
        self
    }

    /// Release the left mouse button.
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
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// #         driver.action_chain().click_and_hold_element(&elem).perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 down");
    /// driver.action_chain().release().perform().await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn release(mut self) -> Self {
        self.add_mouse_up(MOUSE_BUTTON_LEFT);
        self
    }

    /// Move the mouse to the specified element and release the mouse button.
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
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// #         driver.action_chain().click_and_hold_element(&elem).perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 down");
    /// driver.action_chain().release_on_element(&elem).perform().await?;
    /// #         assert_eq!(elem_result.text().await?, "Button 1 clicked");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn release_on_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).release()
    }

    /// Send the specified keystrokes to the active element.
    ///
    /// # Example:
    /// ```no_run
    /// use thirtyfour::Key;
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// let button = driver.find_element(By::Id("button-set")).await?;
    /// #         assert_eq!(elem.value().await?, Some("".to_string()));
    /// driver.action_chain()
    ///     .click_element(&elem)
    ///     .send_keys("selenium")
    ///     .click_element(&button)
    ///     .perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("input-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "selenium");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn send_keys(mut self, text: impl AsRef<str>) -> Self {
        for c in text.as_ref().chars() {
            self = self.key_down(c).key_up(c);
        }
        self
    }

    /// Click on the specified element and send the specified keystrokes.
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
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// let button = driver.find_element(By::Id("button-set")).await?;
    /// #         assert_eq!(elem.value().await?, Some("".to_string()));
    /// driver.action_chain()
    ///     .send_keys_to_element(&elem, "selenium")
    ///     .click_element(&button)
    ///     .perform().await?;
    /// #         let elem_result = driver.find_element(By::Id("input-result")).await?;
    /// #         assert_eq!(elem_result.text().await?, "selenium");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn send_keys_to_element(self, element: &WebElement, text: impl AsRef<str>) -> Self {
        self.click_element(element).send_keys(text)
    }
}
