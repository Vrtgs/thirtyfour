use serde_json::json;

use crate::{common::command::FormatRequestData, RequestData, RequestMethod};

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

impl FormatRequestData for FirefoxCommand {
    fn format_request(&self, session_id: &crate::SessionId) -> RequestData {
        match &self {
            FirefoxCommand::InstallAddon {
                path,
                temporary,
            } => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/moz/addon/install", session_id),
            )
            .add_body(json!({
                "path": path,
                "temporary": temporary
            })),
            FirefoxCommand::FullScreenshot {} => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/moz/screenshot/full", session_id),
            ),
        }
    }
}
