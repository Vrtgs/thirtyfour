use serde::Serialize;
use serde_json::{json, Value};

use crate::{BrowserCapabilitiesHelper, Capabilities, CapabilitiesHelper};

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

impl From<EdgeCapabilities> for Capabilities {
    fn from(caps: EdgeCapabilities) -> Capabilities {
        caps.capabilities
    }
}

impl CapabilitiesHelper for EdgeCapabilities {
    fn _get(&self, key: &str) -> Option<&Value> {
        self.capabilities._get(key)
    }

    fn _get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.capabilities._get_mut(key)
    }

    fn insert_base_capability(&mut self, key: String, value: Value) {
        self.capabilities.insert_base_capability(key, value);
    }
}

impl BrowserCapabilitiesHelper for EdgeCapabilities {
    const KEY: &'static str = "ms:edgeOptions";
}
