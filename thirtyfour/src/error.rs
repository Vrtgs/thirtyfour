use fantoccini::error::{CmdError, NewSessionError};
use url::ParseError;

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
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("webDriver command error: {0}")]
    CmdError(#[from] CmdError),
    #[error("{0}")]
    CustomError(String),
}

impl From<url::ParseError> for WebDriverError {
    fn from(pe: ParseError) -> Self {
        Self::CustomError(format!("unable to parse url: {}", pe))
    }
}
