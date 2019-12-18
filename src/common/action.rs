use serde::Serialize;
use serde_repr::Serialize_repr;

use crate::common::keys::TypingData;
use crate::common::types::ElementId;

pub trait Action {
    fn get_pause(duration_seconds: Option<u64>) -> Self;
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NullAction {
    Pause {
        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<u64>,
    },
}

impl Action for NullAction {
    fn get_pause(duration_seconds: Option<u64>) -> Self {
        NullAction::Pause {
            duration: duration_seconds,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KeyAction {
    Pause {
        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<u64>,
    },
    KeyUp {
        value: char,
    },
    KeyDown {
        value: char,
    },
}

impl Action for KeyAction {
    fn get_pause(duration_seconds: Option<u64>) -> Self {
        KeyAction::Pause {
            duration: duration_seconds,
        }
    }
}

#[derive(Debug, Clone, Serialize_repr)]
#[repr(u8)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PointerOrigin {
    Viewport,
    Pointer,
    #[serde(rename = "element-6066-11e4-a52e-4f735466cecf")]
    WebElement(ElementId),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PointerAction {
    Pause {
        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<u64>,
    },
    PointerDown {
        button: MouseButton,
    },
    PointerUp {
        button: MouseButton,
    },
    PointerMove {
        #[serde(skip_serializing_if = "Option::is_none")]
        duration: Option<u64>,
        origin: PointerOrigin,
        x: i64,
        y: i64,
    },
    PointerCancel,
}

impl Action for PointerAction {
    fn get_pause(duration_seconds: Option<u64>) -> Self {
        PointerAction::Pause {
            duration: duration_seconds,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointerParameters {
    pointer_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionSource<T: Action + Clone> {
    id: String,
    #[serde(rename(serialize = "type"))]
    action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<PointerParameters>,
    actions: Vec<T>,
}

impl<T> ActionSource<T>
where
    T: Action + Clone,
{
    pub fn add_action(&mut self, action: T) {
        self.actions.push(action);
    }

    pub fn pause(&mut self) {
        self.actions.push(T::get_pause(None));
    }

    pub fn pause_for(&mut self, duration_seconds: Option<u64>) {
        self.actions.push(T::get_pause(duration_seconds));
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl ActionSource<KeyAction> {
    pub fn new(name: &str) -> Self {
        ActionSource {
            id: name.to_owned(),
            action_type: String::from("key"),
            parameters: None,
            actions: Vec::new(),
        }
    }

    pub fn key_down(&mut self, value: char) {
        self.add_action(KeyAction::KeyDown { value });
    }

    pub fn key_up(&mut self, value: char) {
        self.add_action(KeyAction::KeyUp { value });
    }

    pub fn send_keys(&mut self, text: TypingData) {
        for c in text.as_vec() {
            self.key_down(c);
            self.key_up(c);
        }
    }
}

#[derive(Debug)]
pub enum PointerActionType {
    Mouse,
    Pen,
    Touch,
}

impl ActionSource<PointerAction> {
    pub fn new(name: &str, action_type: PointerActionType) -> Self {
        ActionSource {
            id: name.to_owned(),
            action_type: String::from("pointer"),
            parameters: Some(PointerParameters {
                pointer_type: String::from(match action_type {
                    PointerActionType::Mouse => "mouse",
                    PointerActionType::Pen => "pen",
                    PointerActionType::Touch => "touch",
                }),
            }),
            actions: Vec::new(),
        }
    }

    pub fn move_to(&mut self, x: i64, y: i64) {
        self.add_action(PointerAction::PointerMove {
            duration: None,
            origin: PointerOrigin::Viewport,
            x,
            y,
        });
    }

    pub fn move_by(&mut self, x: i64, y: i64) {
        self.add_action(PointerAction::PointerMove {
            duration: None,
            origin: PointerOrigin::Pointer,
            x,
            y,
        });
    }

    pub fn move_to_element(&mut self, element_id: ElementId, x: i64, y: i64) {
        self.add_action(PointerAction::PointerMove {
            duration: None,
            origin: PointerOrigin::WebElement(element_id),
            x,
            y,
        });
    }

    pub fn move_to_element_center(&mut self, element_id: ElementId) {
        self.add_action(PointerAction::PointerMove {
            duration: None,
            origin: PointerOrigin::WebElement(element_id),
            x: 0,
            y: 0,
        });
    }

    pub fn click(&mut self) {
        self.add_action(PointerAction::PointerDown {
            button: MouseButton::Left,
        });
        self.add_action(PointerAction::PointerUp {
            button: MouseButton::Left,
        });
    }

    pub fn context_click(&mut self) {
        self.add_action(PointerAction::PointerDown {
            button: MouseButton::Right,
        });
        self.add_action(PointerAction::PointerUp {
            button: MouseButton::Right,
        });
    }

    pub fn click_and_hold(&mut self) {
        self.add_action(PointerAction::PointerDown {
            button: MouseButton::Left,
        });
    }

    pub fn click_element_and_hold(&mut self, element_id: ElementId) {
        self.move_to_element_center(element_id);
        self.click_and_hold();
    }

    pub fn release(&mut self) {
        self.add_action(PointerAction::PointerUp {
            button: MouseButton::Left,
        });
    }

    pub fn double_click(&mut self) {
        self.click();
        self.click();
    }

    pub fn double_click_element(&mut self, element_id: ElementId) {
        self.move_to_element_center(element_id);
        self.double_click();
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn compare_null_action(action: NullAction, value: serde_json::Value) {
        let actions: Vec<NullAction> = vec![action];
        let source = ActionSource {
            action_type: String::from("none"),
            id: String::from("null"),
            parameters: None,
            actions,
        };

        let value_got = serde_json::to_value(source);
        assert!(value_got.is_ok());
        assert_eq!(
            value_got.unwrap(),
            json!({
                "id": "null",
                "type": "none",
                "actions": [ value ]
            })
        );
    }

    #[test]
    fn test_null_action() {
        compare_null_action(
            NullAction::Pause { duration: None },
            json!({"type": "pause"}),
        );

        compare_null_action(
            NullAction::Pause { duration: Some(4) },
            json!({"type": "pause", "duration": 4}),
        );
    }

    fn compare_key_action(action: KeyAction, value: serde_json::Value) {
        let mut source = ActionSource::<KeyAction>::new("key");
        source.add_action(action);

        let value_got = serde_json::to_value(source);
        assert!(value_got.is_ok());
        assert_eq!(
            value_got.unwrap(),
            json!({
                "id": "key",
                "type": "key",
                "actions": [ value ]
            })
        );
    }

    #[test]
    fn test_key_action_pause() {
        compare_key_action(
            KeyAction::Pause { duration: None },
            json!({"type": "pause"}),
        );

        compare_key_action(
            KeyAction::Pause { duration: Some(3) },
            json!({"type": "pause", "duration": 3}),
        );
    }

    #[test]
    fn test_key_action_updown() {
        compare_key_action(
            KeyAction::KeyDown { value: 'a' },
            json!({"type": "keyDown", "value": 'a'}),
        );

        compare_key_action(
            KeyAction::KeyDown { value: '\u{e004}' },
            json!({
            "type": "keyDown", "value": '\u{e004}'
            }),
        );

        compare_key_action(
            KeyAction::KeyUp { value: 'a' },
            json!({"type": "keyUp", "value": 'a'}),
        );

        compare_key_action(
            KeyAction::KeyUp { value: '\u{e004}' },
            json!({
            "type": "keyUp", "value": '\u{e004}'
            }),
        );
    }

    fn compare_pointer_action(action: PointerAction, value: serde_json::Value) {
        let mut source = ActionSource::<PointerAction>::new("mouse", PointerActionType::Mouse);
        source.add_action(action);

        let value_got = serde_json::to_value(source);
        assert!(value_got.is_ok());
        assert_eq!(
            value_got.unwrap(),
            json!({
                "id": "mouse",
                "type": "pointer",
                "parameters": {
                    "pointerType": "mouse"
                },
                "actions": [ value ]
            })
        );
    }

    #[test]
    fn test_pointer_action_pause() {
        compare_pointer_action(
            PointerAction::Pause { duration: None },
            json!({"type": "pause"}),
        );

        compare_pointer_action(
            PointerAction::Pause { duration: Some(2) },
            json!({"type": "pause", "duration": 2}),
        );
    }

    #[test]
    fn test_pointer_action_button() {
        compare_pointer_action(
            PointerAction::PointerDown {
                button: MouseButton::Left,
            },
            json!({"type": "pointerDown", "button": 0}),
        );

        compare_pointer_action(
            PointerAction::PointerDown {
                button: MouseButton::Middle,
            },
            json!({"type": "pointerDown", "button": 1 }),
        );

        compare_pointer_action(
            PointerAction::PointerDown {
                button: MouseButton::Right,
            },
            json!({"type": "pointerDown", "button": 2}),
        );

        compare_pointer_action(
            PointerAction::PointerUp {
                button: MouseButton::Left,
            },
            json!({"type": "pointerUp", "button": 0}),
        );

        compare_pointer_action(
            PointerAction::PointerUp {
                button: MouseButton::Middle,
            },
            json!({"type": "pointerUp", "button": 1 }),
        );

        compare_pointer_action(
            PointerAction::PointerUp {
                button: MouseButton::Right,
            },
            json!({"type": "pointerUp", "button": 2}),
        );
    }

    #[test]
    fn test_pointer_action_pointermove() {
        compare_pointer_action(
            PointerAction::PointerMove {
                duration: None,
                x: 0,
                y: 0,
                origin: PointerOrigin::Viewport,
            },
            json!({
            "type": "pointerMove", "origin": "viewport", "x": 0, "y": 0
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: None,
                x: 0,
                y: 0,
                origin: PointerOrigin::Pointer,
            },
            json!({
            "type": "pointerMove", "origin": "pointer", "x": 0, "y": 0
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: None,
                x: 0,
                y: 0,
                origin: PointerOrigin::WebElement(ElementId::from("id1234")),
            },
            json!({
            "type": "pointerMove", "origin": {"element-6066-11e4-a52e-4f735466cecf": "id1234"}, "x": 0, "y": 0
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: Some(1),
                x: 100,
                y: 200,
                origin: PointerOrigin::Viewport,
            },
            json!({
                "type": "pointerMove",
                "x": 100,
                "y": 200,
                "duration": 1,
                "origin": "viewport"
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: Some(1),
                x: 100,
                y: 200,
                origin: PointerOrigin::Pointer,
            },
            json!({
                "type": "pointerMove",
                "x": 100,
                "y": 200,
                "duration": 1,
                "origin": "pointer"
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: Some(1),
                x: 100,
                y: 200,
                origin: PointerOrigin::WebElement(ElementId::from("someid")),
            },
            json!({
                "type": "pointerMove",
                "x": 100,
                "y": 200,
                "duration": 1,
                "origin": {"element-6066-11e4-a52e-4f735466cecf": "someid"}
            }),
        );
    }

    #[test]
    fn test_pointer_action_cancel() {
        compare_pointer_action(
            PointerAction::PointerCancel,
            json!({"type": "pointerCancel"}),
        );
    }
}
