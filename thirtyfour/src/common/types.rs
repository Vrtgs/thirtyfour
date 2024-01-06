use std::{fmt, time::Duration};

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

use crate::error::WebDriverResult;
use crate::WebElement;

/// Rectangle representing the dimensions of an element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    /// The x coordinate of the top-left corner.
    pub x: f64,
    /// The y coordinate of the top-left corner.
    pub y: f64,
    /// The element width.
    pub width: f64,
    /// The element height.
    pub height: f64,
}

impl ElementRect {
    /// The coordinates of the rectangle center point, rounded to integers.
    pub fn icenter(&self) -> (i64, i64) {
        let (x, y) = self.center();
        (x as i64, y as i64)
    }

    /// The coordinates of the rectangle center point.
    pub fn center(&self) -> (f64, f64) {
        (self.x + (self.width / 2.0), self.y + (self.height / 2.0))
    }
}

/// Helper to Deserialize ElementRef from JSON Value.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ElementRef {
    /// Reference for a regular element.
    Element {
        /// Element id, as returned by the webdriver.
        #[serde(rename(deserialize = "element-6066-11e4-a52e-4f735466cecf"))]
        id: String,
    },
    /// Reference for a shadow element.
    ShadowElement {
        /// Element id, as returned by the webdriver.
        #[serde(rename(deserialize = "shadow-6066-11e4-a52e-4f735466cecf"))]
        id: String,
    },
}

impl ElementRef {
    /// The element id, as returned by the webdriver.
    pub fn id(&self) -> &str {
        match &self {
            ElementRef::Element {
                id,
            } => id,
            ElementRef::ShadowElement {
                id,
            } => id,
        }
    }
}

/// Newtype for the session id.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct SessionId {
    id: String,
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

impl SessionId {
    /// Create a dummy SessionId for cases where it's not used.
    ///
    /// E.g. session creation.
    pub fn null() -> Self {
        SessionId {
            id: String::new(),
        }
    }
}

/// Newtype for the element id.
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

/// Newtype for the window handle.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
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

/// The window type. The webdriver spec treats tabs and windows as windows.
#[derive(Debug, Clone)]
pub enum WindowType {
    /// A browser tab.
    Tab,
    /// A browser window.
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

/// Rectangle position and dimensions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    /// The x coordinate of the top-left corner.
    pub x: i64,
    /// The y coordinate of the top-left corner.
    pub y: i64,
    /// The rectangle width.
    pub width: i64,
    /// The rectangle height.
    pub height: i64,
}

impl Rect {
    /// Create a new `Rect`.
    pub fn new(x: i64, y: i64, width: i64, height: i64) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

/// Generic element query function that returns some type T.
pub type ElementQueryFn<T> =
    Box<dyn Fn(&WebElement) -> BoxFuture<WebDriverResult<T>> + Send + Sync + 'static>;

/// Function signature for element predicates.
pub type ElementPredicate = ElementQueryFn<bool>;

/// Rect struct with optional fields.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize)]
pub struct OptionRect {
    /// The x coordinate of the top-left corner.
    pub x: Option<i64>,
    /// The y coordinate of the top-left corner.
    pub y: Option<i64>,
    /// The rectangle width.
    pub width: Option<i64>,
    /// The rectangle height.
    pub height: Option<i64>,
}

impl OptionRect {
    /// Create a new `OptionRect`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the x coordinate of the top-left corner.
    pub fn with_x(mut self, value: i64) -> Self {
        self.x = Some(value);
        self
    }

    /// Set the y coordinate of the top-left corner.
    pub fn with_y(mut self, value: i64) -> Self {
        self.y = Some(value);
        self
    }

    /// Set the rectangle width.
    pub fn with_width(mut self, value: i64) -> Self {
        self.width = Some(value);
        self
    }

    /// Set the rectangle height.
    pub fn with_height(mut self, value: i64) -> Self {
        self.height = Some(value);
        self
    }

    /// Set the rectangle position.
    pub fn with_pos(mut self, x: i64, y: i64) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// Set the rectangle size.
    pub fn with_size(mut self, width: i64, height: i64) -> Self {
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

/// The timeout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfiguration {
    /// The script timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    script: Option<u64>,
    /// The page load timeout.
    #[serde(rename = "pageLoad", skip_serializing_if = "Option::is_none")]
    page_load: Option<u64>,
    /// The implicit wait timeout.
    #[serde(skip_serializing_if = "Option::is_none")]
    implicit: Option<u64>,
}

impl Default for TimeoutConfiguration {
    fn default() -> Self {
        TimeoutConfiguration::new(
            Some(Duration::from_secs(60)),
            Some(Duration::from_secs(60)),
            // NOTE: Implicit wait must default to zero in order to support ElementQuery.
            Some(Duration::from_secs(0)),
        )
    }
}

impl TimeoutConfiguration {
    /// Create a new `TimeoutConfiguration`.
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

    /// Get the script timeout, if set.
    pub fn script(&self) -> Option<Duration> {
        self.script.map(Duration::from_millis)
    }

    /// Set the script timeout.
    pub fn set_script(&mut self, timeout: Option<Duration>) {
        self.script = timeout.map(|x| x.as_millis() as u64);
    }

    /// Get the page load timeout, if set.
    pub fn page_load(&self) -> Option<Duration> {
        self.page_load.map(Duration::from_millis)
    }

    /// Set the page load timeout.
    pub fn set_page_load(&mut self, timeout: Option<Duration>) {
        self.page_load = timeout.map(|x| x.as_millis() as u64);
    }

    /// Get the implicit wait timeout, if set.
    pub fn implicit(&self) -> Option<Duration> {
        self.implicit.map(Duration::from_millis)
    }

    /// Set the implicit wait timeout.
    pub fn set_implicit(&mut self, timeout: Option<Duration>) {
        self.implicit = timeout.map(|x| x.as_millis() as u64);
    }
}

/// The WebDriver status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDriverStatus {
    /// Whether the webdriver is ready to accept new sessions.
    pub ready: bool,
    /// The current status message.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use serde_json::json;

    #[test]
    fn test_element_ref() {
        let id = "daaea226-43aa-400f-896c-210e5af2ac62";
        let value = json!({ "element-6066-11e4-a52e-4f735466cecf": id });
        let elem_ref: ElementRef = serde_json::from_value(value).unwrap();
        assert_matches!(&elem_ref, ElementRef::Element { id: x} if x == id);
        assert_eq!(elem_ref.id(), id);
    }

    #[test]
    fn test_shadow_element_ref() {
        let id = "daaea226-43aa-400f-896c-210e5af2ac62";
        let value = json!({ "shadow-6066-11e4-a52e-4f735466cecf": id });
        let elem_ref: ElementRef = serde_json::from_value(value).unwrap();
        assert_matches!(&elem_ref, ElementRef::ShadowElement { id: x} if x == id);
        assert_eq!(elem_ref.id(), id);
    }
}
