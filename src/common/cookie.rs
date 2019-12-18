use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    name: String,
    value: serde_json::Value,
    path: Option<String>,
    domain: Option<String>,
    secure: Option<bool>,
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
}
