use crate::CapabilitiesHelper;
use fantoccini::wd::Capabilities;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct InternetExplorerCapabilities {
    capabilities: Capabilities,
}

impl Default for InternetExplorerCapabilities {
    fn default() -> Self {
        let mut capabilities = Capabilities::new();
        capabilities.insert("browserName".to_string(), json!("internet explorer"));
        InternetExplorerCapabilities {
            capabilities,
        }
    }
}

impl InternetExplorerCapabilities {
    pub fn new() -> Self {
        InternetExplorerCapabilities::default()
    }
}

impl CapabilitiesHelper for InternetExplorerCapabilities {
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

impl From<InternetExplorerCapabilities> for Capabilities {
    fn from(caps: InternetExplorerCapabilities) -> Capabilities {
        caps.capabilities
    }
}
