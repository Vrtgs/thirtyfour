use crate::upstream::{CmdError, ErrorStatus, NewSessionError};
use serde::Serialize;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;

pub type WebDriverResult<T> = Result<T, WebDriverError>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum WebDriverError {
    #[error("error creating new session: {0}")]
    NewSessionError(#[from] NewSessionError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("decode error: {0}")]
    DecodeError(String),
    #[error("webdriver command error: {0}")]
    Cmd(CmdError),
    #[error("{0}")]
    CustomError(String),
    // The following errors are returned from the WebDriver, according to the spec.
    #[error(
        "the element's shadow root is not attached to the active document, or \
        the reference is stale: {0}"
    )]
    DetachedShadowRoot(WebDriverErrorDetails),
    #[error(
        "the element click command could not be completed because the \
        element receiving the events is obscuring the element that was requested \
        clicked: {0}"
    )]
    ElementClickIntercepted(WebDriverErrorDetails),
    #[error(
        "a command could not be completed because the element is not pointer or \
        keyboard interactable: {0}"
    )]
    ElementNotInteractable(WebDriverErrorDetails),
    #[error("an attempt was made to select an element that cannot be selected: {0}")]
    ElementNotSelectable(WebDriverErrorDetails),
    #[error(
        "navigation caused the user agent to hit a certificate warning, which is \
        usually the result of an expired or invalid TLS certificate: {0}"
    )]
    InsecureCertificate(WebDriverErrorDetails),
    #[error("the arguments passed to a command are either invalid or malformed: {0}")]
    InvalidArgument(WebDriverErrorDetails),
    #[error(
        "an illegal attempt was made to set a cookie under a different domain \
        than the current page: {0}"
    )]
    InvalidCookieDomain(WebDriverErrorDetails),
    #[error("the coordinates provided to an interactions operation are invalid: {0}")]
    InvalidCoordinates(WebDriverErrorDetails),
    #[error(
        "a command could not be completed because the element is an invalid \
        state, e.g. attempting to click an element that is no longer attached \
        to the document: {0}"
    )]
    InvalidElementState(WebDriverErrorDetails),
    #[error("argument was an invalid selector: {0}")]
    InvalidSelector(WebDriverErrorDetails),
    #[error(
        "the given session ID is not in the list of active sessions, \
        meaning the session either does not exist or that it’s not active: {0}"
    )]
    InvalidSessionId(WebDriverErrorDetails),
    #[error("an error occurred while executing JavaScript supplied by the user: {0}")]
    JavascriptError(WebDriverErrorDetails),
    #[error(
        "the target for mouse interaction is not in the browser’s viewport and \
        cannot be brought into that viewport: {0}"
    )]
    MoveTargetOutOfBounds(WebDriverErrorDetails),
    #[error(
        "an attempt was made to operate on a modal dialogue when one was not \
        open: {0}"
    )]
    NoSuchAlert(WebDriverErrorDetails),
    #[error(
        "no cookie matching the given path name was found amongst the associated \
        cookies of the current browsing context’s active document: {0}"
    )]
    NoSuchCookie(WebDriverErrorDetails),
    #[error(
        "an element could not be located on the page using the given search \
        parameters: {0}"
    )]
    NoSuchElement(WebDriverErrorDetails),
    #[error(
        "a command to switch to a frame could not be satisfied because the \
        frame could not be found: {0}"
    )]
    NoSuchFrame(WebDriverErrorDetails),
    #[error("an element's shadow root was not found attached to the element: {0}")]
    NoSuchShadowRoot(WebDriverErrorDetails),
    #[error(
        "a command to switch to a window could not be satisfied because the \
        window could not be found: {0}"
    )]
    NoSuchWindow(WebDriverErrorDetails),
    #[error("a script did not complete before its timeout expired: {0}")]
    ScriptTimeout(WebDriverErrorDetails),
    #[error("a new session could not be created: {0}")]
    SessionNotCreated(WebDriverErrorDetails),
    #[error(
        "a command failed because the referenced element is no longer attached to the DOM: {0}"
    )]
    StaleElementReference(WebDriverErrorDetails),
    #[error("an operation did not complete before its timeout expired: {0}")]
    Timeout(WebDriverErrorDetails),
    #[error("a screen capture was made impossible: {0}")]
    UnableToCaptureScreen(WebDriverErrorDetails),
    #[error("setting the cookie’s value could not be done: {0}")]
    UnableToSetCookie(WebDriverErrorDetails),
    #[error("a modal dialogue was open, blocking this operation: {0}")]
    UnexpectedAlertOpen(WebDriverErrorDetails),
    #[error("the requested command could not be executed because it does not exist: {0}")]
    UnknownCommand(WebDriverErrorDetails),
    #[error("an unknown error occurred in the remote end whilst processing the command: {0}")]
    UnknownError(WebDriverErrorDetails),
    #[error(
        "the requested command matched a known endpoint, but did not match a \
        method for that endpoint: {0}"
    )]
    UnknownMethod(WebDriverErrorDetails),
    #[error("unknown WebDriver command: {0}")]
    UnknownPath(WebDriverErrorDetails),
    #[error("a command that should have executed properly is not currently supported: {0}")]
    UnsupportedOperation(WebDriverErrorDetails),
}

/// Error details returned by WebDriver.
#[derive(Debug, Serialize)]
pub struct WebDriverErrorDetails {
    /// Description of this error provided by WebDriver.
    pub message: Cow<'static, str>,
    /// Stacktrace of this error provided by WebDriver.
    pub stacktrace: String,
    /// Optional [error data], populated by some commands.
    ///
    /// [error data]: https://www.w3.org/TR/webdriver1/#dfn-error-data
    pub data: Option<serde_json::Value>,
}

impl WebDriverErrorDetails {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
            stacktrace: String::new(),
            data: None,
        }
    }
}

impl fmt::Display for WebDriverErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for WebDriverErrorDetails {}

impl From<fantoccini::error::WebDriver> for WebDriverErrorDetails {
    fn from(e: fantoccini::error::WebDriver) -> Self {
        Self {
            message: e.message,
            stacktrace: e.stacktrace,
            data: e.data,
        }
    }
}

impl From<CmdError> for WebDriverError {
    fn from(e: CmdError) -> Self {
        match e {
            CmdError::Standard(mut s) => {
                let error_status = std::mem::replace(&mut s.error, ErrorStatus::UnknownError);
                let details = WebDriverErrorDetails::from(s);
                match error_status {
                    ErrorStatus::DetachedShadowRoot => WebDriverError::DetachedShadowRoot(details),
                    ErrorStatus::ElementClickIntercepted => {
                        WebDriverError::ElementClickIntercepted(details)
                    }
                    ErrorStatus::ElementNotInteractable => {
                        WebDriverError::ElementNotInteractable(details)
                    }
                    ErrorStatus::ElementNotSelectable => {
                        WebDriverError::ElementNotSelectable(details)
                    }
                    ErrorStatus::InsecureCertificate => {
                        WebDriverError::InsecureCertificate(details)
                    }
                    ErrorStatus::InvalidArgument => WebDriverError::InvalidArgument(details),
                    ErrorStatus::InvalidCookieDomain => {
                        WebDriverError::InvalidCookieDomain(details)
                    }
                    ErrorStatus::InvalidCoordinates => WebDriverError::InvalidCoordinates(details),
                    ErrorStatus::InvalidElementState => {
                        WebDriverError::InvalidElementState(details)
                    }
                    ErrorStatus::InvalidSelector => WebDriverError::InvalidSelector(details),
                    ErrorStatus::InvalidSessionId => WebDriverError::InvalidSessionId(details),
                    ErrorStatus::JavascriptError => WebDriverError::JavascriptError(details),
                    ErrorStatus::MoveTargetOutOfBounds => {
                        WebDriverError::MoveTargetOutOfBounds(details)
                    }
                    ErrorStatus::NoSuchAlert => WebDriverError::NoSuchAlert(details),
                    ErrorStatus::NoSuchCookie => WebDriverError::NoSuchCookie(details),
                    ErrorStatus::NoSuchElement => WebDriverError::NoSuchElement(details),
                    ErrorStatus::NoSuchFrame => WebDriverError::NoSuchFrame(details),
                    ErrorStatus::NoSuchShadowRoot => WebDriverError::NoSuchShadowRoot(details),
                    ErrorStatus::NoSuchWindow => WebDriverError::NoSuchWindow(details),
                    ErrorStatus::ScriptTimeout => WebDriverError::ScriptTimeout(details),
                    ErrorStatus::SessionNotCreated => WebDriverError::SessionNotCreated(details),
                    ErrorStatus::StaleElementReference => {
                        WebDriverError::StaleElementReference(details)
                    }
                    ErrorStatus::Timeout => WebDriverError::Timeout(details),
                    ErrorStatus::UnableToCaptureScreen => {
                        WebDriverError::UnableToCaptureScreen(details)
                    }
                    ErrorStatus::UnableToSetCookie => WebDriverError::UnableToSetCookie(details),
                    ErrorStatus::UnexpectedAlertOpen => {
                        WebDriverError::UnexpectedAlertOpen(details)
                    }
                    ErrorStatus::UnknownCommand => WebDriverError::UnknownCommand(details),
                    ErrorStatus::UnknownError => WebDriverError::UnknownError(details),
                    ErrorStatus::UnknownMethod => WebDriverError::UnknownMethod(details),
                    ErrorStatus::UnknownPath => WebDriverError::UnknownPath(details),
                    ErrorStatus::UnsupportedOperation => {
                        WebDriverError::UnsupportedOperation(details)
                    }
                    _ => WebDriverError::UnknownError(details),
                }
            }
            x => WebDriverError::Cmd(x),
        }
    }
}

impl From<url::ParseError> for WebDriverError {
    fn from(pe: url::ParseError) -> Self {
        Self::CustomError(format!("unable to parse url: {}", pe))
    }
}

impl From<base64::DecodeError> for WebDriverError {
    fn from(e: base64::DecodeError) -> Self {
        Self::DecodeError(e.to_string())
    }
}
