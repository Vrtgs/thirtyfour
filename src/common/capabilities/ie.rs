use serde::Serialize;
use serde_json::{json, Value};

use crate::common::capabilities::desiredcapabilities::Capabilities;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct InternetExplorerCapabilities {
    capabilities: Value,
}

impl Default for InternetExplorerCapabilities {
    fn default() -> Self {
        InternetExplorerCapabilities {
            capabilities: json!({
                "browserName": "internet explorer"
            }),
        }
    }
}

impl InternetExplorerCapabilities {
    pub fn new() -> Self {
        InternetExplorerCapabilities::default()
    }
}

impl Capabilities for InternetExplorerCapabilities {
    fn get(&self) -> &Value {
        &self.capabilities
    }

    fn get_mut(&mut self) -> &mut Value {
        &mut self.capabilities
    }
}
