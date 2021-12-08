use crate::error::WebDriverResult;
use crate::query::ElementPoller;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct WebDriverConfig {
    pub query_poller: ElementPoller,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl WebDriverConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<V>(&self, key: &str) -> Option<V>
    where
        V: DeserializeOwned,
    {
        self.custom_settings.get(key).map(|v| serde_json::from_value::<V>(v.clone()).ok()).flatten()
    }

    pub fn set<V>(&mut self, key: &str, value: V) -> WebDriverResult<()>
    where
        V: Serialize,
    {
        self.custom_settings.insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }
}
