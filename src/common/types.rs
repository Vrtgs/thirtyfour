use crate::ElementRef;
use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ElementRect {
    pub fn icenter(&self) -> (i64, i64) {
        (self.x as i64 + (self.width / 2.0) as i64, self.y as i64 + (self.height / 2.0) as i64)
    }

    pub fn center(&self) -> (f64, f64) {
        (self.x + (self.width / 2.0), self.y + (self.height / 2.0))
    }
}

/// Helper to Deserialize ElementRef from JSON Value.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ElementRefHelper {
    Element {
        #[serde(rename(deserialize = "element-6066-11e4-a52e-4f735466cecf"))]
        id: String,
    },
    ShadowElement {
        #[serde(rename(deserialize = "shadow-6066-11e4-a52e-4f735466cecf"))]
        id: String,
    },
}

impl ElementRefHelper {
    pub fn id(&self) -> &str {
        match &self {
            ElementRefHelper::Element {
                id,
            } => id,
            ElementRefHelper::ShadowElement {
                id,
            } => id,
        }
    }
}

impl From<ElementRefHelper> for ElementRef {
    fn from(element_ref: ElementRefHelper) -> Self {
        let id = match element_ref {
            ElementRefHelper::Element {
                id,
            } => id,
            ElementRefHelper::ShadowElement {
                id,
            } => id,
        };
        ElementRef::from(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct SessionId {
    id: String,
}

impl SessionId {
    pub fn null() -> Self {
        SessionId {
            id: String::new(),
        }
    }
}

impl Deref for SessionId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl<S> From<S> for SessionId
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        SessionId {
            id: value.into(),
        }
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Clone)]
pub enum WindowType {
    Tab,
    Window,
}

impl fmt::Display for WindowType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WindowType::Tab => "tab",
                WindowType::Window => "window",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

impl Rect {
    pub fn new(x: i64, y: i64, width: i64, height: i64) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}
