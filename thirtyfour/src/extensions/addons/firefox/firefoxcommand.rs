use crate::upstream::WebDriverCompatibleCommand;
use http::Method;
use serde_json::json;
use url::{ParseError, Url};

/// Extra commands specific to Firefox.
#[derive(Debug)]
pub enum FirefoxCommand {
    /// Install the specified add-on.
    InstallAddon {
        /// Path to the add-on.
        path: String,
        /// True if the add-on is temporary.
        temporary: Option<bool>,
    },
    /// Take a full screenshot.
    FullScreenshot {},
}

impl WebDriverCompatibleCommand for FirefoxCommand {
    fn endpoint(&self, base_url: &Url, session_id: Option<&str>) -> Result<Url, ParseError> {
        let base = { base_url.join(&format!("session/{}/", session_id.as_ref().unwrap()))? };
        match &self {
            FirefoxCommand::InstallAddon {
                ..
            } => base.join("moz/addon/install"),
            FirefoxCommand::FullScreenshot {} => base.join("moz/screenshot/full"),
        }
    }

    fn method_and_body(&self, _request_url: &Url) -> (Method, Option<String>) {
        match &self {
            FirefoxCommand::InstallAddon {
                path,
                temporary,
            } => (Method::POST, Some(json!({"path": path, "temporary": temporary}).to_string())),
            FirefoxCommand::FullScreenshot {} => (Method::GET, None),
        }
    }
}
