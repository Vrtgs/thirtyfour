use std::ops::Deref;

use serde_json::json;

use crate::common::{
    capabilities::{make_w3c_caps, DesiredCapabilities},
    cookie::Cookie,
    keys::TypingData,
    types::{ElementId, OptionRect, SessionId, TimeoutConfiguration, WindowHandle},
};

pub const MAGIC_ELEMENTID: &str = "element-6066-11e4-a52e-4f735466cecf";

#[derive(Debug, Clone)]
pub enum RequestMethod {
    Get,
    Post,
    Delete,
}

#[derive(Debug, Clone)]
pub struct RequestData {
    pub method: RequestMethod,
    pub url: String,
    pub body: Option<serde_json::Value>,
}

impl RequestData {
    pub fn new<S: Into<String>>(method: RequestMethod, url: S) -> Self {
        RequestData {
            method,
            url: url.into(),
            body: None,
        }
    }

    pub fn add_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

pub struct Actions(serde_json::Value);

impl From<serde_json::Value> for Actions {
    fn from(value: serde_json::Value) -> Self {
        Actions(value)
    }
}

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

impl<'a> By<'a> {
    pub fn get_w3c_selector(&self) -> (String, String) {
        match self {
            By::Id(x) => (String::from("css selector"), format!("[id=\"{}\"]", x)),
            By::XPath(x) => (String::from("xpath"), x.deref().to_string()),
            By::LinkText(x) => (String::from("link text"), x.deref().to_string()),
            By::PartialLinkText(x) => (String::from("partial link text"), x.deref().to_string()),
            By::Name(x) => (String::from("css selector"), format!("[name=\"{}\"]", x)),
            By::Tag(x) => (String::from("css selector"), x.deref().to_string()),
            By::ClassName(x) => (String::from("css selector"), format!(".{}", x)),
            By::Css(x) => (String::from("css selector"), x.deref().to_string()),
        }
    }
}

pub enum Command<'a> {
    NewSession(&'a DesiredCapabilities),
    DeleteSession(&'a SessionId),
    Status,
    GetTimeouts(&'a SessionId),
    SetTimeouts(&'a SessionId, TimeoutConfiguration),
    NavigateTo(&'a SessionId, String),
    GetCurrentUrl(&'a SessionId),
    Back(&'a SessionId),
    Forward(&'a SessionId),
    Refresh(&'a SessionId),
    GetTitle(&'a SessionId),
    GetWindowHandle(&'a SessionId),
    CloseWindow(&'a SessionId),
    SwitchToWindow(&'a SessionId, &'a WindowHandle),
    GetWindowHandles(&'a SessionId),
    SwitchToFrameDefault(&'a SessionId),
    SwitchToFrameNumber(&'a SessionId, u16),
    SwitchToFrameElement(&'a SessionId, &'a ElementId),
    SwitchToParentFrame(&'a SessionId),
    GetWindowRect(&'a SessionId),
    SetWindowRect(&'a SessionId, OptionRect),
    MaximizeWindow(&'a SessionId),
    MinimizeWindow(&'a SessionId),
    FullscreenWindow(&'a SessionId),
    GetActiveElement(&'a SessionId),
    FindElement(&'a SessionId, By<'a>),
    FindElements(&'a SessionId, By<'a>),
    FindElementFromElement(&'a SessionId, &'a ElementId, By<'a>),
    FindElementsFromElement(&'a SessionId, &'a ElementId, By<'a>),
    IsElementSelected(&'a SessionId, &'a ElementId),
    GetElementAttribute(&'a SessionId, &'a ElementId, String),
    GetElementProperty(&'a SessionId, &'a ElementId, String),
    GetElementCSSValue(&'a SessionId, &'a ElementId, String),
    GetElementText(&'a SessionId, &'a ElementId),
    GetElementTagName(&'a SessionId, &'a ElementId),
    GetElementRect(&'a SessionId, &'a ElementId),
    IsElementEnabled(&'a SessionId, &'a ElementId),
    ElementClick(&'a SessionId, &'a ElementId),
    ElementClear(&'a SessionId, &'a ElementId),
    ElementSendKeys(&'a SessionId, &'a ElementId, TypingData),
    GetPageSource(&'a SessionId),
    ExecuteScript(&'a SessionId, String, Vec<serde_json::Value>),
    ExecuteAsyncScript(&'a SessionId, String, Vec<serde_json::Value>),
    GetAllCookies(&'a SessionId),
    GetNamedCookie(&'a SessionId, &'a str),
    AddCookie(&'a SessionId, Cookie),
    DeleteCookie(&'a SessionId, &'a str),
    DeleteAllCookies(&'a SessionId),
    PerformActions(&'a SessionId, Actions),
    ReleaseActions(&'a SessionId),
    DismissAlert(&'a SessionId),
    AcceptAlert(&'a SessionId),
    GetAlertText(&'a SessionId),
    SendAlertText(&'a SessionId, TypingData),
    TakeScreenshot(&'a SessionId),
    TakeElementScreenshot(&'a SessionId, &'a ElementId),
}

impl<'a> Command<'a> {
    pub fn format_request(&self) -> RequestData {
        match self {
            Command::NewSession(caps) => {
                let w3c_caps = make_w3c_caps(&caps.capabilities);
                RequestData::new(RequestMethod::Post, "/session").add_body(json!({
                    "capabilities": w3c_caps,
                    "desiredCapabilities": caps.capabilities
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
            .add_body(json!(timeout_configuration)),
            Command::NavigateTo(session_id, url) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/url", session_id))
                    .add_body(json!({ "url": url }))
            }
            Command::GetCurrentUrl(session_id) => {
                RequestData::new(RequestMethod::Get, format!("/session/{}/url", session_id))
            }
            Command::Back(session_id) => {
                RequestData::new(RequestMethod::Post, format!("/session/{}/back", session_id))
                    .add_body(json!({}))
            }
            Command::Forward(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/forward", session_id),
            )
            .add_body(json!({})),
            Command::Refresh(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/refresh", session_id),
            )
            .add_body(json!({})),
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
            Command::SwitchToFrameDefault(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/frame", session_id),
            )
            .add_body(json!({ "id": serde_json::Value::Null })),
            Command::SwitchToFrameNumber(session_id, frame_number) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/frame", session_id),
            )
            .add_body(json!({ "id": frame_number })),
            Command::SwitchToFrameElement(session_id, element_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/frame", session_id),
            )
            .add_body(json!({
                "ELEMENT": element_id.to_string(),
                MAGIC_ELEMENTID: element_id.to_string()
            })),
            Command::SwitchToParentFrame(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/frame/parent", session_id),
            )
            .add_body(json!({})),
            Command::GetWindowRect(session_id) => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/window/rect", session_id),
            ),
            Command::SetWindowRect(session_id, option_rect) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/rect", session_id),
            )
            .add_body(json!(option_rect)),
            Command::MaximizeWindow(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/maximize", session_id),
            )
            .add_body(json!({})),
            Command::MinimizeWindow(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/minimize", session_id),
            )
            .add_body(json!({})),
            Command::FullscreenWindow(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/window/fullscreen", session_id),
            )
            .add_body(json!({})),
            Command::GetActiveElement(session_id) => RequestData::new(
                RequestMethod::Get,
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
            )
            .add_body(json!({})),
            Command::ElementClear(session_id, element_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/element/{}/clear", session_id, element_id),
            )
            .add_body(json!({})),
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
            .add_body(json!({ "cookie": cookie })),
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
            .add_body(json!({"actions": actions.0})),
            Command::ReleaseActions(session_id) => RequestData::new(
                RequestMethod::Delete,
                format!("/session/{}/actions", session_id),
            ),
            Command::DismissAlert(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/alert/dismiss", session_id),
            )
            .add_body(json!({})),
            Command::AcceptAlert(session_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/alert/accept", session_id),
            )
            .add_body(json!({})),
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
