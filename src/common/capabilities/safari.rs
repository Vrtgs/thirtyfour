use crate::CapabilitiesHelper;
use fantoccini::wd::Capabilities;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct SafariCapabilities {
    capabilities: Capabilities,
}

impl Default for SafariCapabilities {
    fn default() -> Self {
        let mut capabilities = Capabilities::new();
        capabilities.insert("browserName".to_string(), json!("safari"));
        SafariCapabilities {
            capabilities,
        }
    }
}

impl SafariCapabilities {
    pub fn new() -> Self {
        SafariCapabilities::default()
    }
}

impl CapabilitiesHelper for SafariCapabilities {
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

impl From<SafariCapabilities> for Capabilities {
    fn from(caps: SafariCapabilities) -> Capabilities {
        caps.capabilities
    }
}
