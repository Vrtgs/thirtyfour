use serde::{Deserialize, Serialize};
use std::{fmt, ops::Deref, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize)]
pub struct ElementRef {
    #[serde(rename(deserialize = "element-6066-11e4-a52e-4f735466cecf"))]
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct SessionId {
    _id: String,
}

impl Deref for SessionId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self._id
    }
}

impl<S> From<S> for SessionId
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        SessionId { _id: value.into() }
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self._id)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct ElementId {
    _id: String,
}

impl<S> From<S> for ElementId
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        ElementId { _id: value.into() }
    }
}

impl fmt::Display for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self._id)
    }
}

#[derive(Debug, Clone)]
pub struct WindowHandle {
    _handle: String,
}

impl<S> From<S> for WindowHandle
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        WindowHandle {
            _handle: value.into(),
        }
    }
}

impl fmt::Display for WindowHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self._handle)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct OptionRect {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl OptionRect {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_x(mut self, value: i32) -> Self {
        self.x = Some(value);
        self
    }

    pub fn with_y(mut self, value: i32) -> Self {
        self.y = Some(value);
        self
    }

    pub fn with_width(mut self, value: i32) -> Self {
        self.width = Some(value);
        self
    }

    pub fn with_height(mut self, value: i32) -> Self {
        self.height = Some(value);
        self
    }

    pub fn with_pos(mut self, x: i32, y: i32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    pub fn with_size(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

impl From<Rect> for OptionRect {
    fn from(value: Rect) -> Self {
        OptionRect {
            x: Some(value.x),
            y: Some(value.y),
            width: Some(value.width),
            height: Some(value.height),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimeoutConfiguration {
    pub script: Option<Duration>,
    pub page_load: Option<Duration>,
    pub implicit: Option<Duration>,
}

impl TimeoutConfiguration {
    pub fn new(
        script: Option<Duration>,
        page_load: Option<Duration>,
        implicit: Option<Duration>,
    ) -> Self {
        TimeoutConfiguration {
            script,
            page_load,
            implicit,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "script": self.script.map(|x| x.as_millis()),
            "pageLoad": self.page_load.map(|x| x.as_millis()),
            "implicit": self.implicit.map(|x| x.as_millis())
        })
    }
}
