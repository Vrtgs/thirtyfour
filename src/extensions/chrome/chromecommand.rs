use crate::common::command::FormatRequestData;
use crate::extensions::chrome::NetworkConditions;
use crate::{RequestData, RequestMethod, SessionId};
use serde_json::{json, Value};

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

impl FormatRequestData for ChromeCommand {
    fn format_request(&self, session_id: &SessionId) -> RequestData {
        match self {
            ChromeCommand::LaunchApp(app_id) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/chromium/launch_app", session_id),
            )
            .add_body(json!({ "id": app_id })),
            ChromeCommand::GetNetworkConditions => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/chromium/network_conditions", session_id),
            ),
            ChromeCommand::SetNetworkConditions(conditions) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/chromium/network_conditions", session_id),
            )
            .add_body(json!({ "network_conditions": conditions })),
            ChromeCommand::ExecuteCdpCommand(command, params) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/goog/cdp/execute", session_id),
            )
            .add_body(json!({ "cmd": command, "params": params })),
            ChromeCommand::GetSinks => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/goog/cast/get_sinks", session_id),
            ),
            ChromeCommand::GetIssueMessage => RequestData::new(
                RequestMethod::Get,
                format!("/session/{}/goog/cast/get_issue_message", session_id),
            ),
            ChromeCommand::SetSinkToUse(sink_name) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/goog/cast/set_sink_to_use", session_id),
            )
            .add_body(json!({ "sinkName": sink_name })),
            ChromeCommand::StartTabMirroring(sink_name) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/goog/cast/start_tab_mirroring", session_id),
            )
            .add_body(json!({ "sinkName": sink_name })),
            ChromeCommand::StopCasting(sink_name) => RequestData::new(
                RequestMethod::Post,
                format!("/session/{}/goog/cast/stop_casting", session_id),
            )
            .add_body(json!({ "sinkName": sink_name })),
        }
    }
}
