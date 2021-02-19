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

    /// Get the cookie name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the cookie name. This will not modify the actual cookie in the browser.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Get the cookie value.
    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }

    /// Set the cookie value. This will not modify the actual cookie in the browser.
    pub fn set_value(&mut self, value: serde_json::Value) {
        self.value = value;
    }

    /// Get the cookie path.
    pub fn path(&self) -> &Option<String> {
        &self.path
    }

    /// Set the cookie path. This will not modify the actual cookie in the browser.
    pub fn set_path(&mut self, path: Option<String>) {
        self.path = path;
    }

    /// Get the cookie domain.
    pub fn domain(&self) -> &Option<String> {
        &self.domain
    }

    /// Set the cookie domain. This will not modify the actual cookie in the browser.
    pub fn set_domain(&mut self, domain: Option<String>) {
        self.domain = domain;
    }

    /// Get the cookie secure flag (if set).
    pub fn secure(&self) -> &Option<bool> {
        &self.secure
    }

    /// Set the cookie secure flag. This will not modify the actual cookie in the browser.
    pub fn set_secure(&mut self, secure: Option<bool>) {
        self.secure = secure;
    }

    /// Get the cookie expiry date/time.
    pub fn expiry(&self) -> Option<DateTime<Utc>> {
        self.expiry.map(|x| Utc.timestamp(x, 0))
    }

    /// Set the cookie expiry date/time. This will not modify the actual cookie in the browser.
    pub fn set_expiry<TZ: TimeZone>(&mut self, expiry: Option<DateTime<TZ>>) {
        self.expiry = expiry.map(|x| x.timestamp());
    }
}
