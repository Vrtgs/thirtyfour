use serde_json::{json, Value};

use crate::common::{
    capabilities::desiredcapabilities::make_w3c_caps,
    cookie::Cookie,
    keys::TypingData,
    types::{ElementId, OptionRect, SessionId, TimeoutConfiguration, WindowHandle},
};
use crate::{RequestData, RequestMethod};
use std::fmt;

pub const MAGIC_ELEMENTID: &str = "element-6066-11e4-a52e-4f735466cecf";

pub struct Actions(serde_json::Value);

impl From<serde_json::Value> for Actions {
    fn from(value: serde_json::Value) -> Self {
        Actions(value)
    }
}

#[derive(Debug, Clone)]
pub struct Selector {
    pub name: String,
    pub query: String,
}

impl Selector {
    pub fn new(name: &str, query: &str) -> Self {
        Self {
            name: name.to_string(),
            query: query.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum By<'a> {
    Id(&'a str),
    XPath(&'a str),
    LinkText(&'a str),
    PartialLinkText(&'a str),
    Name(&'a str),
    Tag(&'a str),
    ClassName(&'a str),
    Css(&'a str),
}

impl fmt::Display for By<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            By::Id(id) => write!(f, "Id({})", id),
            By::XPath(xpath) => write!(f, "XPath({})", xpath),
            By::LinkText(text) => write!(f, "Link Text({})", text),
            By::PartialLinkText(text) => write!(f, "Partial Link Text({})", text),
            By::Name(name) => write!(f, "Name({})", name),
            By::Tag(tag) => write!(f, "Tag({})", tag),
            By::ClassName(cname) => write!(f, "Class({})", cname),
            By::Css(css) => write!(f, "CSS({})", css),
        }
    }
}

impl<'a> By<'a> {
    pub fn get_w3c_selector(&self) -> Selector {
        match self {
            By::Id(x) => Selector::new("css selector", &format!("[id=\"{}\"]", x)),
            By::XPath(x) => Selector::new("xpath", *x),
            By::LinkText(x) => Selector::new("link text", *x),
            By::PartialLinkText(x) => Selector::new("partial link text", *x),
            By::Name(x) => Selector::new("css selector", &format!("[name=\"{}\"]", x)),
            By::Tag(x) => Selector::new("css selector", *x),
            By::ClassName(x) => Selector::new("css selector", &format!(".{}", x)),
            By::Css(x) => Selector::new("css selector", *x),
        }
    }
}

pub trait ExtensionCommand {
    /// Request Body
    fn parameters_json(&self) -> Option<serde_json::Value>;

    /// HTTP method accepting by the webdriver
    fn method(&self) -> RequestMethod;

    /// Endpoint URL without `/session/{sessionId}` prefix
    ///
    /// Example:- `/moz/addon/install`
    fn endpoint(&self) -> String;
}

pub enum Command {
    NewSession(Value),
    DeleteSession,
    Status,
    GetTimeouts,
    SetTimeouts(TimeoutConfiguration),
    NavigateTo(String),
    GetCurrentUrl,
    Back,
    Forward,
    Refresh,
    GetTitle,
    GetWindowHandle,
    CloseWindow,
    SwitchToWindow(WindowHandle),
    GetWindowHandles,
    SwitchToFrameDefault,
    SwitchToFrameNumber(u16),
    SwitchToFrameElement(ElementId),
    SwitchToParentFrame,
    GetWindowRect,
    SetWindowRect(OptionRect),
    MaximizeWindow,
    MinimizeWindow,
    FullscreenWindow,
    GetActiveElement,
    FindElement(Selector),
    FindElements(Selector),
    FindElementFromElement(ElementId, Selector),
    FindElementsFromElement(ElementId, Selector),
    IsElementSelected(ElementId),
    IsElementDisplayed(ElementId),
    GetElementAttribute(ElementId, String),
    GetElementProperty(ElementId, String),
    GetElementCSSValue(ElementId, String),
    GetElementText(ElementId),
    GetElementTagName(ElementId),
    GetElementRect(ElementId),
    IsElementEnabled(ElementId),
    ElementClick(ElementId),
    ElementClear(ElementId),
    ElementSendKeys(ElementId, TypingData),
    GetPageSource,
    ExecuteScript(String, Vec<serde_json::Value>),
    ExecuteAsyncScript(String, Vec<serde_json::Value>),
    GetAllCookies,
    GetNamedCookie(String),
    AddCookie(Cookie),
    DeleteCookie(String),
    DeleteAllCookies,
    PerformActions(Actions),
    ReleaseActions,
    DismissAlert,
    AcceptAlert,
    GetAlertText,
    SendAlertText(TypingData),
    TakeScreenshot,
    TakeElementScreenshot(ElementId),
    ExtensionCommand(Box<dyn ExtensionCommand + Send + Sync>),
}

pub trait FormatRequestData {
    fn format_request(&self, session_id: &SessionId) -> RequestData;
}

impl FormatRequestData for Command {
    fn format_request(&self, session_id: &SessionId) -> RequestData {
        match self {
            Command::NewSession(caps) => {
                let w3c_caps = make_w3c_caps(&caps);
                RequestData::new(RequestMethod::Post, "/session").add_body(json!({
                    "capabilities": w3c_caps,
                    "desiredCapabilities": caps
                }))
            }
            Command::DeleteSession => {
                RequestData::new(RequestMethod::Delete, format!("/session/{}", session_id))
            }
            Command::Status => RequestData::new(RequestMethod::Get, "/status"),
            Command::GetTimeouts => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/timeouts", session_id))
            }
            Command::SetTimeouts(timeout_configuration) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/timeouts", session_id))
                    .add_body(json!(timeout_configuration))
            }
            Command::NavigateTo(url) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/url", session_id))
                    .add_body(json!({ "url": url }))
            }
            Command::GetCurrentUrl => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/url", session_id))
            }
            Command::Back => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/back", session_id))
                    .add_body(json!({}))
            }
            Command::Forward => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/forward", session_id))
                    .add_body(json!({}))
            }
            Command::Refresh => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/refresh", session_id))
                    .add_body(json!({}))
            }
            Command::GetTitle => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/title", session_id))
            }
            Command::GetWindowHandle => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/window", session_id))
            }
            Command::CloseWindow => {
                RequestData::new(RequestMethod::Delete, format!("/session/{}/window", session_id))
            }
            Command::SwitchToWindow(window_handle) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/window", session_id))
                    .add_body(json!({ "handle": window_handle.to_string() }))
            }
            Command::GetWindowHandles => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/window/handles", session_id),
            ),
            Command::SwitchToFrameDefault => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/frame", session_id))
                    .add_body(json!({ "id": serde_json::Value::Null }))
            }
            Command::SwitchToFrameNumber(frame_number) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/frame", session_id))
                    .add_body(json!({ "id": frame_number }))
            }
            Command::SwitchToFrameElement(element_id) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/frame", session_id))
                    .add_body(json!({"id": {
                        "ELEMENT": element_id.to_string(),
                        MAGIC_ELEMENTID: element_id.to_string()
                    }}))
            }
            Command::SwitchToParentFrame => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/frame/parent", session_id),
            )
            .add_body(json!({})),
            Command::GetWindowRect => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/window/rect", session_id))
            }
            Command::SetWindowRect(option_rect) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/rect", session_id),
            )
            .add_body(json!(option_rect)),
            Command::MaximizeWindow => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/maximize", session_id),
            )
            .add_body(json!({})),
            Command::MinimizeWindow => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/minimize", session_id),
            )
            .add_body(json!({})),
            Command::FullscreenWindow => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/fullscreen", session_id),
            )
            .add_body(json!({})),
            Command::GetActiveElement => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/active", session_id),
            ),
            Command::FindElement(selector) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/element", session_id))
                    .add_body(json!({"using": selector.name, "value": selector.query}))
            }
            Command::FindElements(selector) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/elements", session_id))
                    .add_body(json!({"using": selector.name, "value": selector.query}))
            }
            Command::FindElementFromElement(element_id, selector) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/element", session_id, element_id),
            )
            .add_body(json!({"using": selector.name, "value": selector.query})),
            Command::FindElementsFromElement(element_id, selector) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/elements", session_id, element_id),
            )
            .add_body(json!({"using": selector.name, "value": selector.query})),
            Command::IsElementSelected(element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/selected", session_id, element_id),
            ),
            Command::IsElementDisplayed(element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/displayed", session_id, element_id),
            ),
            Command::GetElementAttribute(element_id, attribute_name) => RequestData::new(
                RequestMethod::Get,
                format!(
                    "/session/{}/element/{}/attribute/{}",
                    session_id, element_id, attribute_name
                ),
            ),
            Command::GetElementProperty(element_id, property_name) => RequestData::new(
                RequestMethod::Get,
                format!(
                    "/session/{}/element/{}/property/{}",
                    session_id, element_id, property_name
                ),
            ),
            Command::GetElementCSSValue(element_id, property_name) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/css/{}", session_id, element_id, property_name),
            ),
            Command::GetElementText(element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/text", session_id, element_id),
            ),
            Command::GetElementTagName(element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/name", session_id, element_id),
            ),
            Command::GetElementRect(element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/rect", session_id, element_id),
            ),
            Command::IsElementEnabled(element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/enabled", session_id, element_id),
            ),
            Command::ElementClick(element_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/click", session_id, element_id),
            )
            .add_body(json!({})),
            Command::ElementClear(element_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/clear", session_id, element_id),
            )
            .add_body(json!({})),
            Command::ElementSendKeys(element_id, typing_data) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/value", session_id, element_id),
            )
            .add_body(json!({"text": typing_data.to_string(), "value": typing_data.as_vec() })),
            Command::GetPageSource => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/source", session_id))
            }
            Command::ExecuteScript(script, args) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/execute/sync", session_id),
            )
            .add_body(json!({"script": script, "args": args})),
            Command::ExecuteAsyncScript(script, args) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/execute/async", session_id),
            )
            .add_body(json!({"script": script, "args": args})),
            Command::GetAllCookies => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/cookie", session_id))
            }
            Command::GetNamedCookie(cookie_name) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/cookie/{}", session_id, cookie_name),
            ),
            Command::AddCookie(cookie) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/cookie", session_id))
                    .add_body(json!({ "cookie": cookie }))
            }
            Command::DeleteCookie(cookie_name) => RequestData::new(
                RequestMethod::Delete,
                format!("/session/{}/cookie/{}", session_id, cookie_name),
            ),
            Command::DeleteAllCookies => {
                RequestData::new(RequestMethod::Delete, format!("/session/{}/cookie", session_id))
            }
            Command::PerformActions(actions) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/actions", session_id))
                    .add_body(json!({"actions": actions.0}))
            }
            Command::ReleaseActions => {
                RequestData::new(RequestMethod::Delete, format!("/session/{}/actions", session_id))
            }
            Command::DismissAlert => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/alert/dismiss", session_id),
            )
            .add_body(json!({})),
            Command::AcceptAlert => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/alert/accept", session_id),
            )
            .add_body(json!({})),
            Command::GetAlertText => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/alert/text", session_id))
            }
            Command::SendAlertText(typing_data) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/alert/text", session_id))
                    .add_body(json!({
                        "value": typing_data.as_vec(), "text": typing_data.to_string()
                    }))
            }
            Command::TakeScreenshot => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/screenshot", session_id))
            }
            Command::TakeElementScreenshot(element_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/element/{}/screenshot", session_id, element_id),
            ),
            Command::ExtensionCommand(command) => {
                let request_data = RequestData::new(
                    command.method(),
                    format!("/session/{}{}", session_id, command.endpoint()),
                );
                match command.parameters_json() {
                    Some(param) => request_data.add_body(param),
                    None => request_data,
                }
            }
        }
    }
}
