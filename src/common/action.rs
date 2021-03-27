use serde::Serialize;
use serde_repr::Serialize_repr;

use crate::common::{keys::TypingData, types::ElementId};

pub trait Action {
    fn get_pause(duration_ms: u64) -> Self;
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NullAction {
    Pause {
        duration: u64,
    },
}

impl Action for NullAction {
    fn get_pause(duration_ms: u64) -> Self {
        NullAction::Pause {
            duration: duration_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum KeyAction {
    Pause {
        duration: u64,
    },
    KeyUp {
        value: char,
    },
    KeyDown {
        value: char,
    },
}

impl Action for KeyAction {
    fn get_pause(duration_ms: u64) -> Self {
        KeyAction::Pause {
            duration: duration_ms,
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
        duration: u64,
    },
    PointerDown {
        button: MouseButton,
        duration: u64,
    },
    PointerUp {
        button: MouseButton,
        duration: u64,
    },
    PointerMove {
        duration: u64,
        origin: PointerOrigin,
        x: i32,
        y: i32,
    },
    PointerCancel,
}

impl Action for PointerAction {
    fn get_pause(duration_ms: u64) -> Self {
        PointerAction::Pause {
            duration: duration_ms,
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
    #[serde(skip_serializing)]
    duration: u64,
}

impl<T> ActionSource<T>
where
    T: Action + Clone,
{
    pub fn add_action(&mut self, action: T) {
        self.actions.push(action);
    }

    pub fn pause(&mut self) {
        self.actions.push(T::get_pause(0));
    }

    pub fn pause_for(&mut self, duration_ms: u64) {
        self.actions.push(T::get_pause(duration_ms));
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
            duration: 0,
        }
    }

    pub fn key_down(&mut self, value: char) {
        self.add_action(KeyAction::KeyDown {
            value,
        });
    }

    pub fn key_up(&mut self, value: char) {
        self.add_action(KeyAction::KeyUp {
            value,
        });
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
            duration: 250,
        }
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.add_action(PointerAction::PointerMove {
            duration: self.duration,
            origin: PointerOrigin::Viewport,
            x,
            y,
        });
    }

    pub fn move_by(&mut self, x: i32, y: i32) {
        self.add_action(PointerAction::PointerMove {
            duration: self.duration,
            origin: PointerOrigin::Pointer,
            x,
            y,
        });
    }

    pub fn move_to_element(&mut self, element_id: ElementId, x: i32, y: i32) {
        self.add_action(PointerAction::PointerMove {
            duration: self.duration,
            origin: PointerOrigin::WebElement(element_id),
            x,
            y,
        });
    }

    pub fn move_to_element_center(&mut self, element_id: ElementId) {
        self.add_action(PointerAction::PointerMove {
            duration: self.duration,
            origin: PointerOrigin::WebElement(element_id),
            x: 0,
            y: 0,
        });
    }

    pub fn click(&mut self) {
        self.add_action(PointerAction::PointerDown {
            button: MouseButton::Left,
            duration: 0,
        });
        self.add_action(PointerAction::PointerUp {
            button: MouseButton::Left,
            duration: 0,
        });
    }

    pub fn context_click(&mut self) {
        self.add_action(PointerAction::PointerDown {
            button: MouseButton::Right,
            duration: 0,
        });
        self.add_action(PointerAction::PointerUp {
            button: MouseButton::Right,
            duration: 0,
        });
    }

    pub fn click_and_hold(&mut self) {
        self.add_action(PointerAction::PointerDown {
            button: MouseButton::Left,
            duration: 0,
        });
    }

    pub fn click_element_and_hold(&mut self, element_id: ElementId) {
        self.move_to_element_center(element_id);
        self.click_and_hold();
    }

    pub fn release(&mut self) {
        self.add_action(PointerAction::PointerUp {
            button: MouseButton::Left,
            duration: 0,
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
            duration: 0,
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
            NullAction::Pause {
                duration: 0,
            },
            json!({"type": "pause", "duration": 0}),
        );

        compare_null_action(
            NullAction::Pause {
                duration: 4,
            },
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
            KeyAction::Pause {
                duration: 0,
            },
            json!({"type": "pause", "duration": 0}),
        );

        compare_key_action(
            KeyAction::Pause {
                duration: 3,
            },
            json!({"type": "pause", "duration": 3}),
        );
    }

    #[test]
    fn test_key_action_updown() {
        compare_key_action(
            KeyAction::KeyDown {
                value: 'a',
            },
            json!({"type": "keyDown", "value": 'a'}),
        );

        compare_key_action(
            KeyAction::KeyDown {
                value: '\u{e004}',
            },
            json!({
            "type": "keyDown", "value": '\u{e004}'
            }),
        );

        compare_key_action(
            KeyAction::KeyUp {
                value: 'a',
            },
            json!({"type": "keyUp", "value": 'a'}),
        );

        compare_key_action(
            KeyAction::KeyUp {
                value: '\u{e004}',
            },
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
            PointerAction::Pause {
                duration: 0,
            },
            json!({"type": "pause", "duration": 0}),
        );

        compare_pointer_action(
            PointerAction::Pause {
                duration: 2,
            },
            json!({"type": "pause", "duration": 2}),
        );
    }

    #[test]
    fn test_pointer_action_button() {
        compare_pointer_action(
            PointerAction::PointerDown {
                button: MouseButton::Left,
                duration: 0,
            },
            json!({"type": "pointerDown", "button": 0, "duration": 0}),
        );

        compare_pointer_action(
            PointerAction::PointerDown {
                button: MouseButton::Middle,
                duration: 0,
            },
            json!({"type": "pointerDown", "button": 1, "duration": 0}),
        );

        compare_pointer_action(
            PointerAction::PointerDown {
                button: MouseButton::Right,
                duration: 0,
            },
            json!({"type": "pointerDown", "button": 2, "duration": 0}),
        );

        compare_pointer_action(
            PointerAction::PointerUp {
                button: MouseButton::Left,
                duration: 0,
            },
            json!({"type": "pointerUp", "button": 0, "duration": 0}),
        );

        compare_pointer_action(
            PointerAction::PointerUp {
                button: MouseButton::Middle,
                duration: 0,
            },
            json!({"type": "pointerUp", "button": 1, "duration": 0}),
        );

        compare_pointer_action(
            PointerAction::PointerUp {
                button: MouseButton::Right,
                duration: 0,
            },
            json!({"type": "pointerUp", "button": 2, "duration": 0}),
        );
    }

    #[test]
    fn test_pointer_action_pointermove() {
        compare_pointer_action(
            PointerAction::PointerMove {
                duration: 0,
                x: 0,
                y: 0,
                origin: PointerOrigin::Viewport,
            },
            json!({
            "type": "pointerMove", "origin": "viewport", "x": 0, "y": 0, "duration": 0
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: 0,
                x: 0,
                y: 0,
                origin: PointerOrigin::Pointer,
            },
            json!({
            "type": "pointerMove", "origin": "pointer", "x": 0, "y": 0, "duration": 0
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: 0,
                x: 0,
                y: 0,
                origin: PointerOrigin::WebElement(ElementId::from("id1234")),
            },
            json!({
            "type": "pointerMove", "origin": {"element-6066-11e4-a52e-4f735466cecf": "id1234"}, "x": 0, "y": 0, "duration": 0
            }),
        );

        compare_pointer_action(
            PointerAction::PointerMove {
                duration: 1,
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
                duration: 1,
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
                duration: 1,
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
        compare_pointer_action(PointerAction::PointerCancel, json!({"type": "pointerCancel"}));
    }
}
