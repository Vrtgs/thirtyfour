use crate::Capabilities;
use serde::Serialize;
use serde_json::json;
use std::ops::{Deref, DerefMut};

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

impl From<InternetExplorerCapabilities> for Capabilities {
    fn from(caps: InternetExplorerCapabilities) -> Capabilities {
        caps.capabilities
    }
}

impl Deref for InternetExplorerCapabilities {
    type Target = Capabilities;

    fn deref(&self) -> &Self::Target {
        &self.capabilities
    }
}

impl DerefMut for InternetExplorerCapabilities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.capabilities
    }
}
