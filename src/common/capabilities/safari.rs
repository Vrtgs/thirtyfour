use serde::Serialize;
use serde_json::{json, Value};

use crate::common::capabilities::desiredcapabilities::Capabilities;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct SafariCapabilities {
    capabilities: Value,
}

impl Default for SafariCapabilities {
    fn default() -> Self {
        SafariCapabilities {
            capabilities: json!({
                "browserName": "safari",
                "version": "",
                "platform": "MAC"
            }),
        }
    }
}

impl SafariCapabilities {
    pub fn new() -> Self {
        SafariCapabilities::default()
    }
}

impl Capabilities for SafariCapabilities {
    fn get(&self) -> &Value {
        &self.capabilities
    }

    fn get_mut(&mut self) -> &mut Value {
        &mut self.capabilities
    }
}
