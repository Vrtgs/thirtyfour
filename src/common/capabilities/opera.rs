use crate::Capabilities;
use serde::Serialize;
use serde_json::json;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct OperaCapabilities {
    capabilities: Capabilities,
}

impl Default for OperaCapabilities {
    fn default() -> Self {
        let mut capabilities = Capabilities::new();
        capabilities.insert("browserName".to_string(), json!("opera"));
        OperaCapabilities {
            capabilities,
        }
    }
}

impl OperaCapabilities {
    pub fn new() -> Self {
        OperaCapabilities::default()
    }
}

impl From<OperaCapabilities> for Capabilities {
    fn from(caps: OperaCapabilities) -> Capabilities {
        caps.capabilities
    }
}

impl Deref for OperaCapabilities {
    type Target = Capabilities;

    fn deref(&self) -> &Self::Target {
        &self.capabilities
    }
}

impl DerefMut for OperaCapabilities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.capabilities
    }
}
