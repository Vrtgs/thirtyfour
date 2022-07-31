use crate::Capabilities;
use serde::Serialize;
use serde_json::json;
use std::ops::{Deref, DerefMut};

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

impl Deref for EdgeCapabilities {
    type Target = Capabilities;

    fn deref(&self) -> &Self::Target {
        &self.capabilities
    }
}

impl DerefMut for EdgeCapabilities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.capabilities
    }
}
