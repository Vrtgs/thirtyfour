use crate::{ElementRef, WebElement};
use futures::future::BoxFuture;
use std::fmt;

use crate::error::WebDriverResult;
use serde::{Deserialize, Serialize};

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
pub enum ElementRefHelper {
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

impl ElementRefHelper {
    /// The element id, as returned by the webdriver.
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
