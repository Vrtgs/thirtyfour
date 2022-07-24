use crate::error::WebDriverResult;
use crate::SessionId;
use fantoccini::wd::Capabilities;
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
struct InnerConfig {
    pub session_id: SessionId,
    pub capabilities: Capabilities,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct WebDriverConfig {
    config: Arc<RwLock<InnerConfig>>,
}

impl WebDriverConfig {
    pub fn new(session_id: SessionId, capabilities: Capabilities) -> Self {
        Self {
            config: Arc::new(RwLock::new(InnerConfig {
                session_id,
                capabilities,
                custom_settings: HashMap::default(),
            })),
        }
    }

    pub fn get_session_id(&self) -> SessionId {
        let cfg = self.config.read();
        cfg.session_id.clone()
    }

    pub fn get_capabilities(&self) -> Capabilities {
        let cfg = self.config.read();
        cfg.capabilities.clone()
    }

    pub fn get<V>(&self, key: &str) -> Option<V>
    where
        V: DeserializeOwned,
    {
        let cfg = self.config.read();
        cfg.custom_settings.get(key).and_then(|v| serde_json::from_value::<V>(v.clone()).ok())
    }

    pub fn set<V>(&mut self, key: &str, value: V) -> WebDriverResult<()>
    where
        V: Serialize,
    {
        let mut cfg = self.config.write();
        cfg.custom_settings.insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }
}
