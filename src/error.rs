use base64::DecodeError;
use displaydoc::Display;
use serde::Deserialize;
use std::fmt::Formatter;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

pub type WebDriverResult<T> = Result<T, WebDriverError>;

fn indent_lines(message: &str, indent: usize) -> String {
    let lines: Vec<String> =
        message.split('\n').map(|s| format!("{0:i$}{1}", " ", s, i = indent)).collect();
    lines.join("\n")
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebDriverErrorValue {
    pub message: String,
    pub error: Option<String>,
    // This stacktrace is returned from the WebDriver.
    pub stacktrace: Option<String>,
    pub data: Option<serde_json::Value>,
}

impl WebDriverErrorValue {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            error: None,
            stacktrace: None,
            data: None,
        }
    }
}

impl std::fmt::Display for WebDriverErrorValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stacktrace = self
            .stacktrace
            .as_ref()
            .map(|x| format!("Stacktrace:\n{}", indent_lines(x, 4)))
            .unwrap_or_default();
        let error = self.error.as_ref().map(|x| format!("Error: {}", x)).unwrap_or_default();
        let data = self
            .data
            .as_ref()
            .map(|x| format!("Data:\n{}", indent_lines(&format!("{:#?}", x), 4)))
            .unwrap_or_default();
        let lines: Vec<String> = vec![self.message.clone(), error, data, stacktrace]
            .into_iter()
            .filter(|l| !l.is_empty())
            .collect();
        write!(f, "{}", lines.join("\n"))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebDriverErrorInfo {
    #[serde(skip)]
    pub status: u16,
    #[serde(default, rename(deserialize = "state"))]
    pub error: String,
    pub value: WebDriverErrorValue,
}

impl WebDriverErrorInfo {
    pub fn new(message: &str) -> Self {
        Self {
            status: 0,
            error: message.to_string(),
            value: WebDriverErrorValue::new(message),
        }
    }
}

impl std::fmt::Display for WebDriverErrorInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::new();
        if self.status != 0 {
            msg.push_str(&format!("\nStatus: {}\n", self.status));
        }
        if !self.error.is_empty() {
            msg.push_str(&format!("State: {}\n", self.error));
        }
        let additional_info =
            format!("Additional info:\n{}", indent_lines(&self.value.to_string(), 4),);
        msg.push_str(&additional_info);

        write!(f, "{}", indent_lines(&msg, 4))
    }
}

/// WebDriverError is the main error type for thirtyfour
#[non_exhaustive]
#[derive(Debug, Error, Display)]
pub enum WebDriverError {
    /// The WebDriver server returned an unrecognised response: {0} :: {1}
    UnknownResponse(u16, String),
    /// Failed to send request to webdriver: {0}
    RequestFailed(String),
    /// The requested item '{0}' was not found: {1}
    NotFound(String, String),
    /// operation timed out: {0}
    Timeout(String),
    /// Unable to parse JSON: {0}
    JsonError(#[from] serde_json::error::Error),
    /// Unable to decode base64: {0}
    DecodeError(#[from] DecodeError),
    /// IO Error: {0}
    IoError(#[from] std::io::Error),
    #[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
    /// The WebDriver request returned an error: {0}
    HttpError(#[from] reqwest::Error),
    #[cfg(feature = "async-std-runtime")]
    /// The WebDriver request returned an error: {0}
    HttpError(surf::Error),
    /// The WebDriver response does not conform to the W3C WebDriver spec: {0}
    NotInSpec(WebDriverErrorInfo),
    /// The click event was intercepted by another element: {0}
    ElementClickIntercepted(WebDriverErrorInfo),
    /// The element is not interactable: {0}
    ElementNotInteractable(WebDriverErrorInfo),
    /// The certificate is insecure: {0}
    InsecureCertificate(WebDriverErrorInfo),
    /// An argument passed to the WebDriver server was invalid: {0}
    InvalidArgument(WebDriverErrorInfo),
    /// Invalid cookie domain: {0}
    InvalidCookieDomain(WebDriverErrorInfo),
    /// The element is in an invalid state: {0}
    InvalidElementState(WebDriverErrorInfo),
    /// The specified element selector is invalid: {0}
    InvalidSelector(WebDriverErrorInfo),
    /// The WebDriver session id is invalid: {0}
    InvalidSessionId(WebDriverErrorInfo),
    /// The Javascript code returned an error: {0}
    JavascriptError(WebDriverErrorInfo),
    /// Unable to scroll the element into the viewport: {0}
    MoveTargetOutOfBounds(WebDriverErrorInfo),
    /// Alert not found: {0}
    NoSuchAlert(WebDriverErrorInfo),
    /// Cookie not found: {0}
    NoSuchCookie(WebDriverErrorInfo),
    /// Element not found: {0}
    NoSuchElement(WebDriverErrorInfo),
    /// Frame not found: {0}
    NoSuchFrame(WebDriverErrorInfo),
    /// Window not found: {0}
    NoSuchWindow(WebDriverErrorInfo),
    /// The Javascript code did not complete within the script timeout (see WebDriver::set_script_timeout()): {0}
    ScriptTimeout(WebDriverErrorInfo),
    /// Unable to create WebDriver session: {0}
    SessionNotCreated(WebDriverErrorInfo),
    /// Element is stale: {0}
    StaleElementReference(WebDriverErrorInfo),
    /// Operation timed out: {0}
    WebDriverTimeout(WebDriverErrorInfo),
    /// Unable to set cookie: {0}
    UnableToSetCookie(WebDriverErrorInfo),
    /// Unable to capture screenshot: {0}
    UnableToCaptureScreen(WebDriverErrorInfo),
    /// An unexpected alert is currently open: {0}
    UnexpectedAlertOpen(WebDriverErrorInfo),
    /// Unknown command: {0}
    UnknownCommand(WebDriverErrorInfo),
    /// Unknown error: {0}
    UnknownError(WebDriverErrorInfo),
    /// Unknown method: {0}
    UnknownMethod(WebDriverErrorInfo),
    /// Unsupport operation: {0}
    UnsupportedOperation(WebDriverErrorInfo),
    /// Something caused the session to terminate.
    FatalError(String),
    /// Failed to receive command: {0}
    CommandRecvError(String),
    /// The command could not be sent to the session: {0}
    CommandSendError(String),
    /// Could not create session: {0}
    SessionCreateError(String),
}

impl WebDriverError {
    pub fn parse(status: u16, body: String) -> Self {
        let body_json = match serde_json::from_str(&body) {
            Ok(x) => x,
            Err(_) => {
                return WebDriverError::UnknownResponse(status, body);
            }
        };

        let mut payload: WebDriverErrorInfo = match serde_json::from_value(body_json) {
            Ok(x) => x,
            Err(_) => return WebDriverError::UnknownResponse(status, body),
        };

        payload.status = status;
        let mut error = payload.error.clone();
        if error.is_empty() {
            error = payload.value.error.clone().unwrap_or_default();
            if error.is_empty() {
                return WebDriverError::NotInSpec(payload);
            }
        }

        match error.as_str() {
            "element click intercepted" => WebDriverError::ElementClickIntercepted(payload),
            "element not interactable" => WebDriverError::ElementNotInteractable(payload),
            "insecure certificate" => WebDriverError::InsecureCertificate(payload),
            "invalid argument" => WebDriverError::InvalidArgument(payload),
            "invalid cookie domain" => WebDriverError::InvalidCookieDomain(payload),
            "invalid element state" => WebDriverError::InvalidElementState(payload),
            "invalid selector" => WebDriverError::InvalidSelector(payload),
            "invalid session id" => WebDriverError::InvalidSessionId(payload),
            "javascript error" => WebDriverError::JavascriptError(payload),
            "move target out of bounds" => WebDriverError::MoveTargetOutOfBounds(payload),
            "no such alert" => WebDriverError::NoSuchAlert(payload),
            "no such cookie" => WebDriverError::NoSuchCookie(payload),
            "no such element" => WebDriverError::NoSuchElement(payload),
            "no such frame" => WebDriverError::NoSuchFrame(payload),
            "no such window" => WebDriverError::NoSuchWindow(payload),
            "script timeout" => WebDriverError::ScriptTimeout(payload),
            "session not created" => WebDriverError::SessionNotCreated(payload),
            "stale element reference" => WebDriverError::StaleElementReference(payload),
            "timeout" => WebDriverError::WebDriverTimeout(payload),
            "unable to set cookie" => WebDriverError::UnableToSetCookie(payload),
            "unable to capture screen" => WebDriverError::UnableToCaptureScreen(payload),
            "unexpected alert open" => WebDriverError::UnexpectedAlertOpen(payload),
            "unknown command" => WebDriverError::UnknownCommand(payload),
            "unknown error" => WebDriverError::UnknownError(payload),
            "unknown method" => WebDriverError::UnknownMethod(payload),
            "unsupported operation" => WebDriverError::UnsupportedOperation(payload),
            _ => WebDriverError::NotInSpec(payload),
        }
    }
}

#[cfg(feature = "async-std-runtime")]
impl From<surf::Error> for WebDriverError {
    fn from(err: surf::Error) -> Self {
        Self::HttpError(err)
    }
}

impl From<oneshot::error::RecvError> for WebDriverError {
    fn from(err: oneshot::error::RecvError) -> Self {
        Self::CommandRecvError(err.to_string())
    }
}

impl<T> From<mpsc::error::SendError<T>> for WebDriverError {
    fn from(err: mpsc::error::SendError<T>) -> Self {
        Self::CommandSendError(err.to_string())
    }
}

/// Convenience function to construct a simulated NoSuchElement error.
pub fn no_such_element(message: &str) -> WebDriverError {
    WebDriverError::NoSuchElement(WebDriverErrorInfo {
        status: 400,
        error: message.to_string(),
        value: WebDriverErrorValue {
            message: message.to_string(),
            error: None,
            stacktrace: None,
            data: None,
        },
    })
}
