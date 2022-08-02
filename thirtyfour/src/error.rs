use fantoccini::error::{CmdError, NewSessionError};
use thiserror::Error;
use url::ParseError;

pub type WebDriverResult<T> = Result<T, WebDriverError>;

#[derive(Debug, Error)]
pub enum WebDriverError {
    #[error("error creating new session: {0}")]
    NewSessionError(#[from] NewSessionError),
    // NOTE: This error is also returned for stale elements.
    #[error("no such element: {0}")]
    NoSuchElement(String),
    #[error("no such window: {0}")]
    NoSuchWindow(String),
    #[error("no such alert: {0}")]
    NoSuchAlert(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("webDriver command error: {0}")]
    CmdError(CmdError),
    #[error("{0}")]
    CustomError(String),
}

impl From<CmdError> for WebDriverError {
    fn from(ce: CmdError) -> Self {
        let ce_string = ce.to_string();
        match ce {
            CmdError::NoSuchElement(..) => WebDriverError::NoSuchElement(ce_string),
            CmdError::NoSuchWindow(..) => WebDriverError::NoSuchWindow(ce_string),
            CmdError::NoSuchAlert(..) => WebDriverError::NoSuchAlert(ce_string),
            CmdError::Json(x) => WebDriverError::Json(x),
            x => WebDriverError::CmdError(x),
        }
    }
}

impl From<url::ParseError> for WebDriverError {
    fn from(pe: ParseError) -> Self {
        Self::CustomError(format!("unable to parse url: {}", pe))
    }
}
