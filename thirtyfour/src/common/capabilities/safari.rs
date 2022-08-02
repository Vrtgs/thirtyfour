use crate::Capabilities;
use serde::Serialize;
use serde_json::json;
use std::ops::{Deref, DerefMut};

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

impl From<SafariCapabilities> for Capabilities {
    fn from(caps: SafariCapabilities) -> Capabilities {
        caps.capabilities
    }
}

impl Deref for SafariCapabilities {
    type Target = Capabilities;

    fn deref(&self) -> &Self::Target {
        &self.capabilities
    }
}

impl DerefMut for SafariCapabilities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.capabilities
    }
}
