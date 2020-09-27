use crate::error::WebDriverResult;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct WebDriverConfig {
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for WebDriverConfig {
    fn default() -> Self {
        Self {
            custom_settings: HashMap::new(),
        }
    }
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
