use fantoccini::wd::WebDriverCompatibleCommand;
use http::Method;
use serde_json::json;
use url::{ParseError, Url};

#[derive(Debug)]
pub enum FirefoxCommand {
    InstallAddon {
        path: String,
        temporary: Option<bool>,
    },
}

impl WebDriverCompatibleCommand for FirefoxCommand {
    fn endpoint(&self, base_url: &Url, session_id: Option<&str>) -> Result<Url, ParseError> {
        let base = { base_url.join(&format!("session/{}/", session_id.as_ref().unwrap()))? };
        match &self {
            FirefoxCommand::InstallAddon {
                ..
            } => base.join("moz/addon/install"),
        }
    }

    fn method_and_body(&self, _request_url: &Url) -> (Method, Option<String>) {
        match &self {
            FirefoxCommand::InstallAddon {
                path,
                temporary,
            } => (Method::POST, Some(json!({"path": path, "temporary": temporary}).to_string())),
        }
    }
}
