use crate::error::WebDriverResult;
use crate::query::ElementPoller;
use crate::SessionId;
use parking_lot::RwLock;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
struct InnerConfig {
    pub session_id: SessionId,
    pub query_poller: ElementPoller,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct WebDriverConfig {
    config: Arc<RwLock<InnerConfig>>,
}

impl WebDriverConfig {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            config: Arc::new(RwLock::new(InnerConfig {
                session_id,
                query_poller: ElementPoller::default(),
                custom_settings: HashMap::default(),
            })),
        }
    }

    pub fn get_session_id(&self) -> SessionId {
        let cfg = self.config.read();
        cfg.session_id.clone()
    }

    pub fn get_query_poller(&self) -> ElementPoller {
        let cfg = self.config.read();
        cfg.query_poller.clone()
    }

    pub fn set_query_poller(&self, poller: ElementPoller) {
        let mut cfg = self.config.write();
        cfg.query_poller = poller;
    }

    pub fn get<V>(&self, key: &str) -> Option<V>
    where
        V: DeserializeOwned,
    {
        let cfg = self.config.read();
        cfg.custom_settings.get(key).map(|v| serde_json::from_value::<V>(v.clone()).ok()).flatten()
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
