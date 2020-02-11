use serde::Serialize;
use serde_json::{json, Value};

use crate::common::capabilities::desiredcapabilities::Capabilities;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct OperaCapabilities {
    capabilities: Value,
}

impl Default for OperaCapabilities {
    fn default() -> Self {
        OperaCapabilities {
            capabilities: json!({
                "browserName": "opera",
                "version": "",
                "platform": "ANY"
            }),
        }
    }
}

impl OperaCapabilities {
    pub fn new() -> Self {
        OperaCapabilities::default()
    }
}

impl Capabilities for OperaCapabilities {
    fn get(&self) -> &Value {
        &self.capabilities
    }

    fn get_mut(&mut self) -> &mut Value {
        &mut self.capabilities
    }
}
