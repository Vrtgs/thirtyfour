use serde::Serialize;
use serde_json::{json, to_value, Value};

use crate::common::capabilities::chrome::ChromeCapabilities;
use crate::common::capabilities::edge::EdgeCapabilities;
use crate::common::capabilities::firefox::FirefoxCapabilities;
use crate::common::capabilities::ie::InternetExplorerCapabilities;
use crate::common::capabilities::opera::OperaCapabilities;
use crate::common::capabilities::safari::SafariCapabilities;
use crate::error::WebDriverResult;
use crate::Capabilities;

const W3C_CAPABILITY_NAMES: &[&str] = &[
    "acceptInsecureCerts",
    "browserName",
    "browserVersion",
    "platformName",
    "pageLoadStrategy",
    "proxy",
    "setWindowRect",
    "timeouts",
    "unhandledPromptBehavior",
    "strictFileInteractability",
];

const OSS_W3C_CONVERSION: &[(&str, &str)] = &[
    ("acceptSslCerts", "acceptInsecureCerts"),
    ("version", "browserVersion"),
    ("platform", "platformName"),
];

pub fn make_w3c_caps(caps: &Value) -> Value {
    let mut always_match = json!({});

    if let Some(caps_map) = caps.as_object() {
        for (k, v) in caps_map.iter() {
            if !v.is_null() {
                for (k_from, k_to) in OSS_W3C_CONVERSION {
                    if k_from == k {
                        always_match[k_to] = v.clone();
                    }
                }
            }

            if W3C_CAPABILITY_NAMES.contains(&k.as_str()) || k.contains(':') {
                always_match[k] = v.clone();
            }
        }
    }

    json!({
        "firstMatch": [{}], "alwaysMatch": always_match
    })
}

/// Merge two serde_json::Value structs.
///
/// From https://stackoverflow.com/questions/47070876/how-can-i-merge-two-json-objects-with-rust
fn merge(a: &mut Value, b: Value) {
    match (a, b) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

/// The DesiredCapabilities struct provides a generic way to construct capabilities as well as
/// helper methods that create specific capabilities structs for the various browsers.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct DesiredCapabilities {
    capabilities: Capabilities,
}

impl Default for DesiredCapabilities {
    fn default() -> Self {
        Self {
            capabilities: serde_json::Map::new(),
        }
    }
}

impl DesiredCapabilities {
    /// Create a new custom DesiredCapabilities struct. Generally you should use the
    /// browser-specific functions instead, such as `DesiredCapabilities::firefox()`,
    /// but you can use `DesiredCapabilities::new` if you need to create capabilities
    /// for a browser not listed here.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a FirefoxCapabilities struct.
    pub fn firefox() -> FirefoxCapabilities {
        FirefoxCapabilities::new()
    }

    /// Create an InternetExplorerCapabilities struct.
    pub fn internet_explorer() -> InternetExplorerCapabilities {
        InternetExplorerCapabilities::new()
    }

    /// Create an EdgeCapabilities struct.
    pub fn edge() -> EdgeCapabilities {
        EdgeCapabilities::new()
    }

    /// Create a ChromeCapabilities struct.
    pub fn chrome() -> ChromeCapabilities {
        ChromeCapabilities::new()
    }

    /// Create an OperaCapabilities struct.
    pub fn opera() -> OperaCapabilities {
        OperaCapabilities::new()
    }

    /// Create a SafariCapabilities struct.
    pub fn safari() -> SafariCapabilities {
        SafariCapabilities::new()
    }
}

impl From<DesiredCapabilities> for Capabilities {
    fn from(caps: DesiredCapabilities) -> Capabilities {
        caps.capabilities
    }
}

pub trait CapabilitiesHelper {
    /// Get an immutable reference to the underlying serde_json::Value.
    fn _get(&self, key: &str) -> Option<&Value>;

    /// Get a mutable reference to the underlying serde_json::Value.
    fn _get_mut(&mut self, key: &str) -> Option<&mut Value>;

    fn _set(&mut self, key: String, value: Value);

    /// Add any Serialize-able object to the capabilities under the specified key.
    fn add<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self._set(key.to_string(), to_value(value)?);
        Ok(())
    }

    /// Add any Serialize-able object to the capabilities under the specified key and subkey.
    fn add_subkey<T>(&mut self, key: &str, subkey: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        match self._get_mut(key) {
            Some(v) => {
                v[subkey] = to_value(value)?;
            }
            None => self._set(key.to_string(), json!({ subkey: value })),
        }
        Ok(())
    }

    /// Remove a subkey from the specified key, if it exists.
    fn remove_subkey(&mut self, key: &str, subkey: &str) -> WebDriverResult<()> {
        if let Some(Value::Object(v)) = &mut self._get_mut(key) {
            v.remove(subkey);
        }
        Ok(())
    }

    /// Add all keys of the specified object into the capabilities, overwriting any
    /// matching keys that already exist.
    fn update(&mut self, key: &str, value: Value) {
        assert!(value.is_object());
        let merged = match self._get_mut(key) {
            Some(x) => {
                merge(x, value);
                x.clone()
            }
            None => value,
        };
        self._set(key.to_string(), merged);
    }

    /// Set the desired browser version.
    fn set_version(&mut self, version: &str) -> WebDriverResult<()> {
        self.add("version", version)
    }

    /// Set the desired browser platform.
    fn set_platform(&mut self, platform: &str) -> WebDriverResult<()> {
        self.add("platform", platform)
    }

    /// Set whether the session supports executing user-supplied Javascript.
    fn set_javascript_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("javascriptEnabled", enabled)
    }

    /// Set whether the session can interact with database storage.
    fn set_database_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("databaseEnabled", enabled)
    }

    /// Set whether the session can set and query the browser's location context.
    fn set_location_context_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("locationContextEnabled", enabled)
    }

    /// Set whether the session can interact with the application cache.
    fn set_application_cache_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("applicationCacheEnabled", enabled)
    }

    /// Set whether the session can query for the browser's connectivity and disable it if desired.
    fn set_browser_connection_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("browserConnectionEnabled", enabled)
    }

    /// Set whether the session supports interactions with local storage.
    fn set_web_storage_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("webStorageEnabled", enabled)
    }

    /// Set whether the session should accept all SSL certificates by default.
    fn accept_ssl_certs(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("acceptSslCerts", enabled)
    }

    /// Set whether the session can rotate the current page's layout between portrait and landscape
    /// orientations. Only applies to mobile platforms.
    fn set_rotatable(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("rotatable", enabled)
    }

    /// Set whether the session is capable of generating native events when simulating user input.
    fn set_native_events(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("nativeEvents", enabled)
    }

    /// Set the proxy to use.
    fn set_proxy(&mut self, proxy: Proxy) -> WebDriverResult<()> {
        self.add("proxy", proxy)
    }

    /// Set the behaviour to be followed when an unexpected alert is encountered.
    fn set_unexpected_alert_behaviour(&mut self, behaviour: AlertBehaviour) -> WebDriverResult<()> {
        self.add("unexpectedAlertBehaviour", behaviour)
    }

    /// Set whether elements are scrolled into the viewport for interation to align with the top
    /// or the bottom of the viewport. The default is to align with the top.
    fn set_element_scroll_behaviour(&mut self, behaviour: ScrollBehaviour) -> WebDriverResult<()> {
        self.add("elementScrollBehavior", behaviour)
    }

    /// Get whether the session can interact with modal popups such as `window.alert`.
    fn handles_alerts(&self) -> Option<bool> {
        self._get("handlesAlerts").and_then(|x| x.as_bool())
    }

    /// Get whether the session supports CSS selectors when searching for elements.
    fn css_selectors_enabled(&self) -> Option<bool> {
        self._get("cssSelectorsEnabled").and_then(|x| x.as_bool())
    }
}

impl CapabilitiesHelper for Capabilities {
    fn _get(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }

    fn _get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.get_mut(key)
    }

    fn _set(&mut self, key: String, value: Value) {
        self.insert(key, value);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "proxyType", rename_all = "lowercase")]
pub enum Proxy {
    Direct,
    #[serde(rename_all = "camelCase")]
    Manual {
        #[serde(skip_serializing_if = "Option::is_none")]
        ftp_proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http_proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ssl_proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        socks_proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        socks_version: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        socks_username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        socks_password: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        no_proxy: Option<String>,
    },
    #[serde(rename = "pac")]
    AutoConfig {
        #[serde(rename = "proxyAutoconfigUrl")]
        url: String,
    },
    AutoDetect,
    System,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertBehaviour {
    Accept,
    Dismiss,
    Ignore,
}

#[derive(Debug, Clone, Serialize)]
#[repr(u8)]
pub enum ScrollBehaviour {
    Top = 0,
    Bottom = 1,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PageLoadStrategy {
    /// Wait for full page loading (the default).
    Normal,
    /// Wait for the DOMContentLoaded event (html content downloaded and parsed only).
    Eager,
    /// Return immediately after the initial page content is fully received
    /// (html content downloaded).
    None,
}
