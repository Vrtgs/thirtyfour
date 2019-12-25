use crate::{
    common::{
        action::{ActionSource, KeyAction, PointerAction, PointerActionType},
        command::{Actions, Command},
        keys::TypingData,
        types::SessionId,
    },
    error::WebDriverResult,
    RemoteConnectionAsync, WebElement,
};
use std::sync::Arc;

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
    conn: Arc<RemoteConnectionAsync>,
    session_id: SessionId,
    key_actions: ActionSource<KeyAction>,
    pointer_actions: ActionSource<PointerAction>,
}

impl ActionChain {
    /// Create a new ActionChain struct.
    ///
    /// See [WebDriver::action_chain()](../struct.WebDriver.html#method.action_chain)
    /// for more details.
    pub fn new(conn: Arc<RemoteConnectionAsync>, session_id: SessionId) -> Self {
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
    pub async fn perform(&self) -> WebDriverResult<()> {
        let actions = Actions::from(serde_json::json!([self.key_actions, self.pointer_actions]));
        self.conn
            .execute(Command::PerformActions(&self.session_id, actions))
            .await?;
        Ok(())
    }

    /// Click and release the left mouse button.
    pub fn click(mut self) -> Self {
        self.pointer_actions.click();
        // Click = 2 actions (PointerDown + PointerUp).
        self.key_actions.pause();
        self.key_actions.pause();
        self
    }

    /// Click on the specified element using the left mouse button and release.
    pub fn click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).click()
    }

    /// Click the left mouse button and hold it down.
    pub fn click_and_hold(mut self) -> Self {
        self.pointer_actions.click_and_hold();
        self.key_actions.pause();
        self
    }

    /// Click on the specified element using the left mouse button and
    /// hold the button down.
    pub fn click_and_hold_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).click_and_hold()
    }

    /// Click and release the right mouse button.
    pub fn context_click(mut self) -> Self {
        self.pointer_actions.context_click();
        // Click = 2 actions (PointerDown + PointerUp).
        self.key_actions.pause();
        self.key_actions.pause();
        self
    }

    /// Click on the specified element using the right mouse button and release.
    pub fn context_click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).context_click()
    }

    /// Double-click the left mouse button.
    pub fn double_click(mut self) -> Self {
        self.pointer_actions.double_click();
        // Each click = 2 actions (PointerDown + PointerUp).
        for _ in 0..4 {
            self.key_actions.pause();
        }
        self
    }

    /// Double-click on the specified element.
    pub fn double_click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).double_click()
    }

    /// Drag the mouse cursor from the center of the source element to the
    /// center of the target element.
    pub fn drag_and_drop_element(self, source: &WebElement, target: &WebElement) -> Self {
        self.click_and_hold_element(source)
            .release_on_element(target)
    }

    /// Drag the mouse cursor by the specified X and Y offsets.
    pub fn drag_and_drop_by_offset(self, x_offset: i64, y_offset: i64) -> Self {
        self.click_and_hold().move_by_offset(x_offset, y_offset)
    }

    /// Drag the mouse cursor by the specified X and Y offsets, starting
    /// from the center of the specified element.
    pub fn drag_and_drop_element_by_offset(
        self,
        element: &WebElement,
        x_offset: i64,
        y_offset: i64,
    ) -> Self {
        self.click_and_hold_element(element)
            .move_by_offset(x_offset, y_offset)
    }

    /// Press the specified key down.
    pub fn key_down(mut self, value: char) -> Self {
        self.key_actions.key_down(value);
        self.pointer_actions.pause();
        self
    }

    /// Click the specified element and then press the specified key down.
    pub fn key_down_on_element(self, element: &WebElement, value: char) -> Self {
        self.click_element(element).key_down(value)
    }

    /// Release the specified key. This usually follows a `key_down()` action.
    pub fn key_up(mut self, value: char) -> Self {
        self.key_actions.key_up(value);
        self.pointer_actions.pause();
        self
    }

    /// Click the specified element and release the specified key.
    pub fn key_up_on_element(self, element: &WebElement, value: char) -> Self {
        self.click_element(element).key_up(value)
    }

    /// Move the mouse cursor by the specified X and Y offsets.
    pub fn move_by_offset(mut self, x_offset: i64, y_offset: i64) -> Self {
        self.pointer_actions.move_by(x_offset, y_offset);
        self.key_actions.pause();
        self
    }

    /// Move the mouse cursor to the center of the specified element.
    pub fn move_to_element_center(mut self, element: &WebElement) -> Self {
        self.pointer_actions
            .move_to_element_center(element.element_id.clone());
        self.key_actions.pause();
        self
    }

    /// Move the mouse cursor to the specified offsets from the specified
    /// element.
    pub fn move_to_element_with_offset(
        mut self,
        element: &WebElement,
        x_offset: i64,
        y_offset: i64,
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

    /// Send the specified keystrokes.
    pub fn send_keys(mut self, text: TypingData) -> Self {
        for c in text.as_vec() {
            self = self.key_down(c).key_up(c);
        }
        self
    }

    /// Click on the specified element and send the specified keystrokes.
    pub fn send_keys_to_element(self, element: &WebElement, text: TypingData) -> Self {
        self.click_element(element).send_keys(text)
    }
}
