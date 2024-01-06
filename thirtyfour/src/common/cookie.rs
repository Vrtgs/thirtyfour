use serde::{Deserialize, Serialize};

/// Enum representing the SameSite attribute of a cookie.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SameSite {
    /// Strict SameSite attribute.
    Strict,
    /// Lax SameSite attribute.
    Lax,
    /// No SameSite attribute.
    None,
}

/// Cookie struct used to create new cookies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    /// The name of the cookie.
    pub name: String,
    /// The value of the cookie.
    pub value: serde_json::Value,
    /// The path of the cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The domain of the cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Whether the cookie is secure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<bool>,
    /// The expiry date of the cookie.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<i64>,
    /// The sameSite attribute of the cookie.
    #[serde(skip_serializing_if = "Option::is_none", rename = "sameSite")]
    pub same_site: Option<SameSite>,
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
            same_site: None,
        }
    }
}
