use std::sync::Arc;

use crate::{RemoteConnectionAsync, WebElement};
use crate::common::action::{ActionSource, KeyAction, PointerAction, PointerActionType};
use crate::common::command::{Command, SessionId};
use crate::common::keys::TypingData;
use crate::error::WebDriverResult;

pub struct ActionChain {
    conn: Arc<RemoteConnectionAsync>,
    session_id: SessionId,
    key_actions: ActionSource<KeyAction>,
    pointer_actions: ActionSource<PointerAction>,
}

impl ActionChain {
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

    pub async fn perform(&self) -> WebDriverResult<()> {
        let mut sources: Vec<serde_json::Value> = Vec::new();
        sources.push(serde_json::to_value(self.key_actions.clone())?);
        sources.push(serde_json::to_value(self.pointer_actions.clone())?);
        let actions = serde_json::to_value(sources)?;
        self.conn
            .execute(Command::PerformActions(&self.session_id, actions))
            .await?;
        Ok(())
    }

    pub fn click(mut self) -> Self {
        self.pointer_actions.click();
        // Click = 2 actions (PointerDown + PointerUp).
        self.key_actions.pause();
        self.key_actions.pause();
        self
    }

    pub fn click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).click()
    }

    pub fn click_and_hold(mut self) -> Self {
        self.pointer_actions.click_and_hold();
        self.key_actions.pause();
        self
    }

    pub fn click_and_hold_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).click_and_hold()
    }

    pub fn context_click(mut self) -> Self {
        self.pointer_actions.context_click();
        // Click = 2 actions (PointerDown + PointerUp).
        self.key_actions.pause();
        self.key_actions.pause();
        self
    }

    pub fn context_click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).context_click()
    }

    pub fn double_click(mut self) -> Self {
        self.pointer_actions.double_click();
        // Each click = 2 actions (PointerDown + PointerUp).
        for _ in 0..4 {
            self.key_actions.pause();
        }
        self
    }

    pub fn double_click_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).double_click()
    }

    pub fn drag_and_drop_element(self, source: &WebElement, target: &WebElement) -> Self {
        self.click_and_hold_element(source)
            .release_on_element(target)
    }

    pub fn drag_and_drop_by_offset(self, x_offset: i64, y_offset: i64) -> Self {
        self.click_and_hold().move_by_offset(x_offset, y_offset)
    }

    pub fn drag_and_drop_element_by_offset(
        self,
        element: &WebElement,
        x_offset: i64,
        y_offset: i64,
    ) -> Self {
        self.click_and_hold_element(element)
            .move_by_offset(x_offset, y_offset)
    }

    pub fn key_down(mut self, value: char) -> Self {
        self.key_actions.key_down(value);
        self.pointer_actions.pause();
        self
    }

    pub fn key_down_on_element(self, element: &WebElement, value: char) -> Self {
        self.click_element(element).key_down(value)
    }

    pub fn key_up(mut self, value: char) -> Self {
        self.key_actions.key_up(value);
        self.pointer_actions.pause();
        self
    }

    pub fn key_up_on_element(self, element: &WebElement, value: char) -> Self {
        self.click_element(element).key_up(value)
    }

    pub fn move_by_offset(mut self, x_offset: i64, y_offset: i64) -> Self {
        self.pointer_actions.move_by(x_offset, y_offset);
        self.key_actions.pause();
        self
    }

    pub fn move_to_element_center(mut self, element: &WebElement) -> Self {
        self.pointer_actions
            .move_to_element_center(element.element_id.clone());
        self.key_actions.pause();
        self
    }

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

    pub fn release(mut self) -> Self {
        self.pointer_actions.release();
        self.key_actions.pause();
        self
    }

    pub fn release_on_element(self, element: &WebElement) -> Self {
        self.move_to_element_center(element).release()
    }

    pub fn send_keys(mut self, text: TypingData) -> Self {
        for c in text.as_vec() {
            self = self.key_down(c).key_up(c);
        }
        self
    }

    pub fn send_keys_to_element(self, element: &WebElement, text: TypingData) -> Self {
        self.click_element(element).send_keys(text)
    }
}
