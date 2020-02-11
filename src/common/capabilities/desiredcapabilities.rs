use serde::Serialize;
use serde_json::{json, to_value, Value};

use crate::common::capabilities::chrome::ChromeCapabilities;
use crate::common::capabilities::edge::EdgeCapabilities;
use crate::common::capabilities::firefox::FirefoxCapabilities;
use crate::common::capabilities::ie::InternetExplorerCapabilities;
use crate::common::capabilities::opera::OperaCapabilities;
use crate::common::capabilities::safari::SafariCapabilities;
use crate::error::WebDriverResult;

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

pub fn make_w3c_caps(caps: &serde_json::Value) -> serde_json::Value {
    let mut always_match = serde_json::json!({});

    for (k, v) in caps.as_object().unwrap().iter() {
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
#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub struct DesiredCapabilities {
    capabilities: Value,
}

impl DesiredCapabilities {
    pub fn new(capabilities: Value) -> Self {
        DesiredCapabilities { capabilities }
    }

    pub fn firefox() -> FirefoxCapabilities {
        FirefoxCapabilities::new()
    }

    pub fn internet_explorer() -> InternetExplorerCapabilities {
        InternetExplorerCapabilities::new()
    }

    pub fn edge() -> EdgeCapabilities {
        EdgeCapabilities::new()
    }

    pub fn chrome() -> ChromeCapabilities {
        ChromeCapabilities::new()
    }

    pub fn opera() -> OperaCapabilities {
        OperaCapabilities::new()
    }

    pub fn safari() -> SafariCapabilities {
        SafariCapabilities::new()
    }
}

/// Add generic Capabilities implementation. This can be used as a convenient way to
/// interact with the returned capabilities from a WebDriver instance, or as a way
/// to construct a custom capabilities JSON object.
impl Capabilities for DesiredCapabilities {
    fn get(&self) -> &Value {
        &self.capabilities
    }

    fn get_mut(&mut self) -> &mut Value {
        &mut self.capabilities
    }
}

pub trait Capabilities {
    fn get(&self) -> &Value;
    fn get_mut(&mut self) -> &mut Value;

    fn add<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self.get_mut()[key] = to_value(value)?;
        Ok(())
    }

    fn add_subkey<T>(&mut self, key: &str, subkey: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        let v = self.get_mut();
        if v[key].is_null() {
            v[key] = json!({ subkey: value });
        } else {
            v[key][subkey] = to_value(value)?;
        }
        Ok(())
    }

    fn update(&mut self, value: Value) {
        merge(&mut self.get_mut(), value);
    }

    fn set_version(&mut self, version: &str) -> WebDriverResult<()> {
        self.add("version", version)
    }

    fn set_platform(&mut self, platform: &str) -> WebDriverResult<()> {
        self.add("platform", platform)
    }

    fn set_javascript_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("javascriptEnabled", enabled)
    }

    fn set_database_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("databaseEnabled", enabled)
    }

    fn set_location_context_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("locationContextEnabled", enabled)
    }

    fn set_application_cache_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("applicationCacheEnabled", enabled)
    }

    fn set_browser_connection_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("browserConnectionEnabled", enabled)
    }

    fn set_web_storage_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("webStorageEnabled", enabled)
    }

    fn accept_ssl_certs(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("acceptSslCerts", enabled)
    }

    fn set_rotatable(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("rotatable", enabled)
    }

    fn set_native_events(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.add("nativeEvents", enabled)
    }

    fn set_proxy(&mut self, proxy: Proxy) -> WebDriverResult<()> {
        self.add("proxy", proxy)
    }

    fn set_unexpected_alert_behaviour(&mut self, behaviour: AlertBehaviour) -> WebDriverResult<()> {
        self.add("unexpectedAlertBehaviour", behaviour)
    }

    fn set_element_scroll_behaviour(&mut self, behaviour: ScrollBehaviour) -> WebDriverResult<()> {
        self.add("elementScrollBehavior", behaviour)
    }

    fn handles_alerts(&self) -> Option<bool> {
        self.get()["handlesAlerts"].as_bool()
    }

    fn css_selectors_enabled(&self) -> Option<bool> {
        self.get()["cssSelectorsEnabled"].as_bool()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "proxyType", rename_all = "lowercase")]
pub enum Proxy {
    Direct,
    #[serde(rename_all = "camelCase")]
    Manual {
        ftp_proxy: Option<String>,
        http_proxy: Option<String>,
        ssl_proxy: Option<String>,
        socks_proxy: Option<String>,
        socks_username: Option<String>,
        socks_password: Option<String>,
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
