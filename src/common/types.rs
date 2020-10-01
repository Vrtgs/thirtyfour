use std::{fmt, ops::Deref, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ElementRect {
    pub fn center(&self) -> (f32, f32) {
        (self.x + (self.width / 2.0), self.y + (self.height / 2.0))
    }

    pub fn icenter(&self) -> (i32, i32) {
        let c = self.center();
        (c.0 as i32, c.1 as i32)
    }
}

#[derive(Deserialize)]
pub struct ElementRef {
    #[serde(rename(deserialize = "element-6066-11e4-a52e-4f735466cecf"))]
    pub id: String,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(transparent)]
pub struct ElementId {
    id: String,
}

impl<S> From<S> for ElementId
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        ElementId {
            id: value.into(),
        }
    }
}

impl fmt::Display for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WindowHandle {
    handle: String,
}

impl<S> From<S> for WindowHandle
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        WindowHandle {
            handle: value.into(),
        }
    }
}

impl fmt::Display for WindowHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.handle)
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
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    script: Option<u64>,
    #[serde(rename = "pageLoad", skip_serializing_if = "Option::is_none")]
    page_load: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    implicit: Option<u64>,
}

impl TimeoutConfiguration {
    pub fn new(
        script: Option<Duration>,
        page_load: Option<Duration>,
        implicit: Option<Duration>,
    ) -> Self {
        TimeoutConfiguration {
            script: script.map(|x| x.as_millis() as u64),
            page_load: page_load.map(|x| x.as_millis() as u64),
            implicit: implicit.map(|x| x.as_millis() as u64),
        }
    }

    pub fn script(&self) -> Option<Duration> {
        self.script.map(Duration::from_millis)
    }

    pub fn set_script(&mut self, timeout: Option<Duration>) {
        self.script = timeout.map(|x| x.as_millis() as u64);
    }

    pub fn page_load(&self) -> Option<Duration> {
        self.page_load.map(Duration::from_millis)
    }

    pub fn set_page_load(&mut self, timeout: Option<Duration>) {
        self.page_load = timeout.map(|x| x.as_millis() as u64);
    }

    pub fn implicit(&self) -> Option<Duration> {
        self.implicit.map(Duration::from_millis)
    }

    pub fn set_implicit(&mut self, timeout: Option<Duration>) {
        self.implicit = timeout.map(|x| x.as_millis() as u64);
    }
}
