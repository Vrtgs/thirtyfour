use crate::CapabilitiesHelper;
use fantoccini::wd::Capabilities;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct EdgeCapabilities {
    capabilities: Capabilities,
}

impl Default for EdgeCapabilities {
    fn default() -> Self {
        let mut capabilities = Capabilities::new();
        capabilities.insert("browserName".to_string(), json!("MicrosoftEdge"));
        EdgeCapabilities {
            capabilities,
        }
    }
}

impl EdgeCapabilities {
    pub fn new() -> Self {
        EdgeCapabilities::default()
    }
}

impl CapabilitiesHelper for EdgeCapabilities {
    fn get(&self, key: &str) -> Option<&Value> {
        self.capabilities.get(key)
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.capabilities.get_mut(key)
    }

    fn set(&mut self, key: String, value: Value) {
        self.capabilities.insert(key, value);
    }
}

impl From<EdgeCapabilities> for Capabilities {
    fn from(caps: EdgeCapabilities) -> Capabilities {
        caps.capabilities
    }
}
