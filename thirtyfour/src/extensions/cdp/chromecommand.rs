use super::NetworkConditions;
use crate::upstream::WebDriverCompatibleCommand;
use http::Method;
use serde_json::{json, Value};
use url::{ParseError, Url};

#[derive(Debug)]
pub enum ChromeCommand {
    LaunchApp(String),
    GetNetworkConditions,
    SetNetworkConditions(NetworkConditions),
    ExecuteCdpCommand(String, Value),
    GetSinks,
    GetIssueMessage,
    SetSinkToUse(String),
    StartTabMirroring(String),
    StopCasting(String),
}

impl WebDriverCompatibleCommand for ChromeCommand {
    fn endpoint(&self, base_url: &Url, session_id: Option<&str>) -> Result<Url, ParseError> {
        let base = { base_url.join(&format!("session/{}/", session_id.as_ref().unwrap()))? };
        match &self {
            ChromeCommand::LaunchApp(_) => base.join("chromium/launch_app"),
            ChromeCommand::GetNetworkConditions | ChromeCommand::SetNetworkConditions(_) => {
                base.join("chromium/network_conditions")
            }
            ChromeCommand::ExecuteCdpCommand(..) => base.join("goog/cdp/execute"),
            ChromeCommand::GetSinks => base.join("goog/cast/get_sinks"),
            ChromeCommand::GetIssueMessage => base.join("goog/cast/get_issue_message"),
            ChromeCommand::SetSinkToUse(_) => base.join("goog/cast/set_sink_to_use"),
            ChromeCommand::StartTabMirroring(_) => base.join("goog/cast/start_tab_mirroring"),
            ChromeCommand::StopCasting(_) => base.join("goog/cast/stop_casting"),
        }
    }

    fn method_and_body(&self, _request_url: &Url) -> (Method, Option<String>) {
        let mut method = Method::GET;
        let mut body = None;

        match &self {
            ChromeCommand::LaunchApp(app_id) => {
                method = Method::POST;
                body = Some(json!({ "id": app_id }).to_string())
            }
            ChromeCommand::SetNetworkConditions(conditions) => {
                method = Method::POST;
                body = Some(json!({ "network_conditions": conditions }).to_string())
            }
            ChromeCommand::ExecuteCdpCommand(command, params) => {
                method = Method::POST;
                body = Some(json!({"cmd": command, "params": params }).to_string())
            }
            ChromeCommand::SetSinkToUse(sink_name)
            | ChromeCommand::StartTabMirroring(sink_name)
            | ChromeCommand::StopCasting(sink_name) => {
                method = Method::POST;
                body = Some(json!({ "sinkName": sink_name }).to_string())
            }
            _ => {}
        }

        (method, body)
    }
}
