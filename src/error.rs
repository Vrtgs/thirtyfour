use base64::DecodeError;
use serde::Deserialize;

pub type WebDriverResult<T> = Result<T, WebDriverError>;

#[derive(Debug, Deserialize, Clone)]
pub struct WebDriverErrorValue {
    message: String,
    error: Option<String>,
    stacktrace: Option<String>,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebDriverErrorInfo {
    #[serde(skip)]
    status: u16,
    #[serde(default, rename(deserialize = "state"))]
    error: String,
    value: WebDriverErrorValue,
}

/// WebDriverError is the main error type
#[derive(Debug, Clone)]
pub enum WebDriverError {
    ConnectionError(RemoteConnectionError),
    JsonError(String),
    DecodeError(String),
    IOError(String),
    RequestFailed(String),
    NotInSpec(WebDriverErrorInfo),
    ElementClickIntercepted(WebDriverErrorInfo),
    ElementNotInteractable(WebDriverErrorInfo),
    InsecureCertificate(WebDriverErrorInfo),
    InvalidArgument(WebDriverErrorInfo),
    InvalidCookieDomain(WebDriverErrorInfo),
    InvalidElementState(WebDriverErrorInfo),
    InvalidSelector(WebDriverErrorInfo),
    InvalidSessionId(WebDriverErrorInfo),
    JavascriptError(WebDriverErrorInfo),
    MoveTargetOutOfBounds(WebDriverErrorInfo),
    NoSuchAlert(WebDriverErrorInfo),
    NoSuchCookie(WebDriverErrorInfo),
    NoSuchElement(WebDriverErrorInfo),
    NoSuchFrame(WebDriverErrorInfo),
    NoSuchWindow(WebDriverErrorInfo),
    ScriptTimeout(WebDriverErrorInfo),
    SessionNotCreated(WebDriverErrorInfo),
    StaleElementReference(WebDriverErrorInfo),
    Timeout(WebDriverErrorInfo),
    UnableToSetCookie(WebDriverErrorInfo),
    UnableToCaptureScreen(WebDriverErrorInfo),
    UnexpectedAlertOpen(WebDriverErrorInfo),
    UnknownCommand(WebDriverErrorInfo),
    UnknownError(WebDriverErrorInfo),
    UnknownMethod(WebDriverErrorInfo),
    UnsupportedOperation(WebDriverErrorInfo),
}

impl WebDriverError {
    pub fn parse(status: u16, body: serde_json::Value) -> Self {
        let mut payload: WebDriverErrorInfo = match serde_json::from_value(body.clone()) {
            Ok(x) => x,
            Err(e) => {
                return WebDriverError::JsonError(e.to_string());
            }
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
            "timeout" => WebDriverError::Timeout(payload),
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

impl From<serde_json::error::Error> for WebDriverError {
    fn from(value: serde_json::error::Error) -> Self {
        WebDriverError::JsonError(value.to_string())
    }
}

impl From<RemoteConnectionError> for WebDriverError {
    fn from(value: RemoteConnectionError) -> Self {
        WebDriverError::ConnectionError(value)
    }
}

impl From<DecodeError> for WebDriverError {
    fn from(value: DecodeError) -> Self {
        WebDriverError::DecodeError(value.to_string())
    }
}

impl From<std::io::Error> for WebDriverError {
    fn from(value: std::io::Error) -> Self {
        WebDriverError::IOError(value.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum RemoteConnectionError {
    InvalidUrl(String),
    ClientError(String),
}

impl From<reqwest::header::InvalidHeaderValue> for RemoteConnectionError {
    fn from(value: reqwest::header::InvalidHeaderValue) -> Self {
        RemoteConnectionError::InvalidUrl(value.to_string())
    }
}

impl From<reqwest::Error> for RemoteConnectionError {
    fn from(value: reqwest::Error) -> Self {
        RemoteConnectionError::ClientError(value.to_string())
    }
}
