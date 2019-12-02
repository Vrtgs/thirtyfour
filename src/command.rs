use std::fmt;
use std::time::Duration;

use serde::Serialize;
use serde_json::json;

use crate::keys::TypingData;

pub struct SessionId {
    _id: String,
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self._id)
    }
}

pub struct ElementId {
    _id: String,
}

impl fmt::Display for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self._id)
    }
}

pub struct WindowHandle {
    _handle: String,
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

#[derive(Debug, Clone, Serialize)]
pub struct OptionRect {
    _x: Option<i32>,
    _y: Option<i32>,
    _width: Option<i32>,
    _height: Option<i32>,
}

impl OptionRect {
    pub fn new() -> Self {
        OptionRect {
            _x: None,
            _y: None,
            _width: None,
            _height: None,
        }
    }

    pub fn x(mut self, value: i32) -> Self {
        self._x = Some(value);
        self
    }

    pub fn y(mut self, value: i32) -> Self {
        self._y = Some(value);
        self
    }

    pub fn width(mut self, value: i32) -> Self {
        self._width = Some(value);
        self
    }

    pub fn height(mut self, value: i32) -> Self {
        self._height = Some(value);
        self
    }

    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "x": self._x,
            "y": self._y,
            "width": self._width,
            "height": self._height
        })
    }
}

pub struct TimeoutConfiguration {
    pub script: Option<Duration>,
    pub page_load: Option<Duration>,
    pub implicit: Option<Duration>,
}

impl TimeoutConfiguration {
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "script": self.script.map(|x| x.as_millis()),
            "pageLoad": self.page_load.map(|x| x.as_millis()),
            "implicit": self.implicit.map(|x| x.as_millis())
        })
    }
}

pub enum RequestMethod {
    Get,
    Post,
    Delete,
}

pub struct RequestData {
    pub method: RequestMethod,
    pub url: String,
    pub body: serde_json::Value,
}

impl RequestData {
    pub fn new<S: Into<String>>(method: RequestMethod, url: S) -> Self {
        RequestData {
            method,
            url: url.into(),
            body: serde_json::Value::Null,
        }
    }

    pub fn add_body(mut self, body: serde_json::Value) -> Self {
        self.body = body;
        self
    }
}

// TODO: These can become actual types later.
type W3CCapabilities = serde_json::Value;
type DesiredCapabilities = serde_json::Value;
type CookieData = serde_json::Value;
type Actions = serde_json::Value;

pub enum By {
    Id(String),
    XPath(String),
    LinkText(String),
    PartialLinkText(String),
    Name(String),
    Tag(String),
    ClassName(String),
    Css(String),
}

impl By {
    pub fn get_w3c_selector(&self) -> (String, String) {
        match self {
            By::Id(x) => (String::from("css selector"), format!("[id=\"{}\"]", x)),
            By::XPath(x) => (String::from("xpath"), x.clone()),
            By::LinkText(x) => (String::from("link text"), x.clone()),
            By::PartialLinkText(x) => (String::from("partial link text"), x.clone()),
            By::Name(x) => (String::from("css selector"), format!("[name=\"{}\"]", x)),
            By::Tag(x) => (String::from("css selector"), x.clone()),
            By::ClassName(x) => (String::from("css selector"), format!(".{}", x)),
            By::Css(x) => (String::from("css selector"), x.clone()),
        }
    }
}

pub enum Command {
    NewSession(W3CCapabilities, DesiredCapabilities),
    DeleteSession(SessionId),
    Status,
    GetTimeouts(SessionId),
    SetTimeouts(SessionId, TimeoutConfiguration),
    NavigateTo(SessionId, String),
    GetCurrentUrl(SessionId),
    Back(SessionId),
    Forward(SessionId),
    Refresh(SessionId),
    GetTitle(SessionId),
    GetWindowHandle(SessionId),
    CloseWindow(SessionId),
    SwitchToWindow(SessionId, WindowHandle),
    GetWindowHandles(SessionId),
    NewWindow(SessionId, Option<WindowType>),
    SwitchToFrame(SessionId, ElementId),
    SwitchToParentFrame(SessionId),
    GetWindowRect(SessionId),
    SetWindowRect(SessionId, OptionRect),
    MaximizeWindow(SessionId),
    MinimizeWindow(SessionId),
    FullscreenWindow(SessionId),
    GetActiveElement(SessionId),
    FindElement(SessionId, By),
    FindElements(SessionId, By),
    FindElementFromElement(SessionId, ElementId, By),
    FindElementsFromElement(SessionId, ElementId, By),
    IsElementSelected(SessionId, ElementId),
    GetElementAttribute(SessionId, ElementId, String),
    GetElementProperty(SessionId, ElementId, String),
    GetElementCSSValue(SessionId, ElementId, String),
    GetElementText(SessionId, ElementId),
    GetElementTagName(SessionId, ElementId),
    GetElementRect(SessionId, ElementId),
    IsElementEnabled(SessionId, ElementId),
    ElementClick(SessionId, ElementId),
    ElementClear(SessionId, ElementId),
    ElementSendKeys(SessionId, ElementId, TypingData),
    GetPageSource(SessionId),
    ExecuteScript(SessionId, String, Vec<String>),
    ExecuteAsyncScript(SessionId, String, Vec<String>),
    GetAllCookies(SessionId),
    GetNamedCookie(SessionId, String),
    AddCookie(SessionId, CookieData),
    DeleteCookie(SessionId, String),
    DeleteAllCookies(SessionId),
    PerformActions(SessionId, Actions),
    ReleaseActions(SessionId),
    DismissAlert(SessionId),
    AcceptAlert(SessionId),
    GetAlertText(SessionId),
    SendAlertText(SessionId, TypingData),
    TakeScreenshot(SessionId),
    TakeElementScreenshot(SessionId, ElementId),
}

impl Command {
    pub fn format_request(&self) -> RequestData {
        match self {
            Command::NewSession(w3c_caps, caps) => {
                RequestData::new(RequestMethod::Post, "/session").add_body(json!({
                    "capabilties": w3c_caps,
                    "desiredCapabilities": caps
                }))
            }
            Command::DeleteSession(session_id) => {
                RequestData::new(RequestMethod::Delete, format!("/session/{}", session_id))
            }
            Command::Status => RequestData::new(RequestMethod::Get, "/status"),
            Command::GetTimeouts(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/timeouts", session_id),
            ),
            Command::SetTimeouts(session_id, timeout_configuration) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/timeouts", session_id),
            )
            .add_body(timeout_configuration.to_json()),
            Command::NavigateTo(session_id, url) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/url", session_id))
                    .add_body(json!({ "url": url }))
            }
            Command::GetCurrentUrl(session_id) => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/url", session_id))
            }
            Command::Back(session_id) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/back", session_id))
            }
            Command::Forward(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/forward", session_id),
            ),
            Command::Refresh(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/refresh", session_id),
            ),
            Command::GetTitle(session_id) => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/title", session_id))
            }
            Command::GetWindowHandle(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/window", session_id),
            ),
            Command::CloseWindow(session_id) => RequestData::new(
                RequestMethod::Delete,
                format!("/session/{}/window", session_id),
            ),
            Command::SwitchToWindow(session_id, window_handle) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window", session_id),
            )
            .add_body(json!({ "handle": window_handle.to_string() })),
            Command::GetWindowHandles(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/window/handles", session_id),
            ),
            Command::NewWindow(session_id, type_hint) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/new", session_id),
            )
            .add_body(json!({
                "type": type_hint.clone().map(|x| x.to_string())
            })),
            Command::SwitchToFrame(session_id, element_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/frame", session_id),
            )
            .add_body(json!({
                "ELEMENT": element_id.to_string(),
                "element-6066-11e4-a52e-4f735466cecf": element_id.to_string()
            })),
            Command::SwitchToParentFrame(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/frame/parent", session_id),
            ),
            Command::GetWindowRect(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/window/rect", session_id),
            ),
            Command::SetWindowRect(session_id, option_rect) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/rect", session_id),
            )
            .add_body(option_rect.to_json()),
            Command::MaximizeWindow(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/maximize", session_id),
            ),
            Command::MinimizeWindow(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/minimize", session_id),
            ),
            Command::FullscreenWindow(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/fullscreen", session_id),
            ),
            Command::GetActiveElement(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/active", session_id),
            ),
            Command::FindElement(session_id, by) => {
                let (selector, value) = by.get_w3c_selector();
                RequestData::new(
                    RequestMethod::Post,
                    format!("/session/{}/element", session_id),
                )
                .add_body(json!({"using": selector, "value": value}))
            }
            Command::FindElements(session_id, by) => {
                let (selector, value) = by.get_w3c_selector();
                RequestData::new(
                    RequestMethod::Post,
                    format!("/session/{}/elements", session_id),
                )
                .add_body(json!({"using": selector, "value": value}))
            }
            Command::FindElementFromElement(session_id, element_id, by) => {
                let (selector, value) = by.get_w3c_selector();
                RequestData::new(
                    RequestMethod::Post,
                    format!("/session/{}/element/{}/element", session_id, element_id),
                )
                .add_body(json!({"using": selector, "value": value}))
            }
            Command::FindElementsFromElement(session_id, element_id, by) => {
                let (selector, value) = by.get_w3c_selector();
                RequestData::new(
                    RequestMethod::Post,
                    format!("/session/{}/element/{}/elements", session_id, element_id),
                )
                .add_body(json!({"using": selector, "value": value}))
            }
            Command::IsElementSelected(session_id, element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/selected", session_id, element_id),
            ),
            Command::GetElementAttribute(session_id, element_id, attribute_name) => {
                RequestData::new(
                    RequestMethod::Get,
                    format!(
                        "/session/{}/element/{}/attribute/{}",
                        session_id, element_id, attribute_name
                    ),
                )
            }
            Command::GetElementProperty(session_id, element_id, property_name) => RequestData::new(
                RequestMethod::Get,
                format!(
                    "/session/{}/element/{}/proprty/{}",
                    session_id, element_id, property_name
                ),
            ),
            Command::GetElementCSSValue(session_id, element_id, property_name) => RequestData::new(
                RequestMethod::Get,
                format!(
                    "/session/{}/element/{}/css/{}",
                    session_id, element_id, property_name
                ),
            ),
            Command::GetElementText(session_id, element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/text", session_id, element_id),
            ),

            Command::GetElementTagName(session_id, element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/name", session_id, element_id),
            ),
            Command::GetElementRect(session_id, element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/rect", session_id, element_id),
            ),
            Command::IsElementEnabled(session_id, element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/enabled", session_id, element_id),
            ),
            Command::ElementClick(session_id, element_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/click", session_id, element_id),
            ),
            Command::ElementClear(session_id, element_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/clear", session_id, element_id),
            ),
            Command::ElementSendKeys(session_id, element_id, typing_data) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/value", session_id, element_id),
            )
            .add_body(json!({"text": typing_data.to_string(), "value": typing_data.as_vec() })),
            Command::GetPageSource(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/source", session_id),
            ),
            Command::ExecuteScript(session_id, script, args) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/execute/sync", session_id),
            )
            .add_body(json!({"script": script, "args": args})),
            Command::ExecuteAsyncScript(session_id, script, args) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/execute/async", session_id),
            )
            .add_body(json!({"script": script, "args": args})),
            Command::GetAllCookies(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/cookie", session_id),
            ),
            Command::GetNamedCookie(session_id, cookie_name) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/cookie/{}", session_id, cookie_name),
            ),
            Command::AddCookie(session_id, cookie) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/cookie", session_id),
            )
            .add_body(cookie.clone()),
            Command::DeleteCookie(session_id, cookie_name) => RequestData::new(
                RequestMethod::Delete,
                format!("/session/{}/cookie/{}", session_id, cookie_name),
            ),
            Command::DeleteAllCookies(session_id) => RequestData::new(
                RequestMethod::Delete,
                format!("/session/{}/cookie", session_id),
            ),
            Command::PerformActions(session_id, actions) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/actions", session_id),
            )
            .add_body(actions.clone()),
            Command::ReleaseActions(session_id) => RequestData::new(
                RequestMethod::Delete,
                format!("/session/{}/actions", session_id),
            ),
            Command::DismissAlert(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/alert/dismiss", session_id),
            ),
            Command::AcceptAlert(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/alert/accept", session_id),
            ),
            Command::GetAlertText(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/alert/text", session_id),
            ),
            Command::SendAlertText(session_id, typing_data) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/alert/text", session_id),
            )
            .add_body(json!({
                "value": typing_data.as_vec(), "text": typing_data.to_string()
            })),
            Command::TakeScreenshot(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/screenshot", session_id),
            ),
            Command::TakeElementScreenshot(session_id, element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/screenshot", session_id, element_id),
            ),
        }
    }
}
