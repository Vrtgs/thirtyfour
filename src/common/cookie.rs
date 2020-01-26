use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    name: String,
    value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiry: Option<i64>,
}

impl Cookie {
    /// Create a new Cookie struct, specifying the name and the JSON data.
    pub fn new(name: &str, value: serde_json::Value) -> Self {
        Cookie {
            name: String::from(name),
            value,
            path: None,
            domain: None,
            secure: None,
            expiry: None,
        }
    }

    /// Get the cookie expiry date/time.
    pub fn expiry(&self) -> Option<DateTime<Utc>> {
        self.expiry.map(|x| Utc.timestamp(x, 0))
    }

    /// Get the cookie value.
    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }
}
