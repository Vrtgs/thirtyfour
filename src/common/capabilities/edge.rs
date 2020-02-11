use serde::Serialize;
use serde_json::{json, Value};

use crate::common::capabilities::desiredcapabilities::Capabilities;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct EdgeCapabilities {
    capabilities: Value,
}

impl Default for EdgeCapabilities {
    fn default() -> Self {
        EdgeCapabilities {
            capabilities: json!({
                "browserName": "MicrosoftEdge",
                "version": "",
                "platform": "WINDOWS"
            }),
        }
    }
}

impl EdgeCapabilities {
    pub fn new() -> Self {
        EdgeCapabilities::default()
    }
}

impl Capabilities for EdgeCapabilities {
    fn get(&self) -> &Value {
        &self.capabilities
    }

    fn get_mut(&mut self) -> &mut Value {
        &mut self.capabilities
    }
}
