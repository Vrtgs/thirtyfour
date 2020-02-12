use std::path::Path;

use serde::Serialize;
use serde_json::{json, Value};

use crate::common::capabilities::desiredcapabilities::Capabilities;
use crate::error::WebDriverResult;
use crate::PageLoadStrategy;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct FirefoxCapabilities {
    capabilities: Value,
}

impl Default for FirefoxCapabilities {
    fn default() -> Self {
        FirefoxCapabilities {
            capabilities: json!({
                "browserName": "firefox",
                "version": "",
                "platform": "ANY"
            }),
        }
    }
}

impl FirefoxCapabilities {
    pub fn new() -> Self {
        FirefoxCapabilities::default()
    }

    /// Add the specified firefox option. This is a helper method for the various
    /// specific option methods.
    pub fn add_firefox_option<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self.add_subkey("moz:firefoxOptions", key, value)
    }

    /// Set the selenium logging preferences. To set the `geckodriver` log level,
    /// use `set_log_level()` instead.
    pub fn set_logging_prefs(&mut self, component: String, log_level: LoggingPrefsLogLevel) {
        self.update(json!({"loggingPrefs": {component: log_level}}));
    }

    /// Set the `geckodriver` log level.
    pub fn set_log_level(&mut self, log_level: LogLevel) -> WebDriverResult<()> {
        self.add_firefox_option("log", json!({ "level": log_level }))
    }

    /// Set the path to the firefox binary.
    pub fn set_firefox_binary(&mut self, path: &Path) -> WebDriverResult<()> {
        self.add("firefox_binary", path.to_string_lossy().to_string())
    }

    /// Set the page load strategy to use.
    /// Valid values are: `normal` (the default)
    pub fn set_page_load_strategy(&mut self, strategy: PageLoadStrategy) -> WebDriverResult<()> {
        self.add("pageLoadingStrategy", strategy)
    }

    /// Set the firefox profile settings to use.
    pub fn set_profile(&mut self, profile: FirefoxProfile) -> WebDriverResult<()> {
        self.add_firefox_option("profile", profile)
    }
}

impl Capabilities for FirefoxCapabilities {
    fn get(&self) -> &Value {
        &self.capabilities
    }

    fn get_mut(&mut self) -> &mut Value {
        &mut self.capabilities
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Config,
    Info,
    Warn,
    Error,
    Fatal,
    Default,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LoggingPrefsLogLevel {
    Off,
    Severe,
    Warning,
    Info,
    Config,
    Fine,
    Finer,
    Finest,
    All,
}

#[derive(Debug, Clone, Serialize)]
pub struct FirefoxProfile {
    #[serde(
        rename = "webdriver_accept_untrusted_certs",
        skip_serializing_if = "Option::is_none"
    )]
    pub accept_untrusted_certs: Option<bool>,
    #[serde(
        rename = "webdriver_assume_untrusted_issuer",
        skip_serializing_if = "Option::is_none"
    )]
    pub assume_untrusted_issuer: Option<bool>,
    #[serde(
        rename = "webdriver.log.driver",
        skip_serializing_if = "Option::is_none"
    )]
    pub log_driver: Option<FirefoxProfileLogDriver>,
    #[serde(rename = "webdriver.log.file", skip_serializing_if = "Option::is_none")]
    pub log_file: Option<String>,
    #[serde(
        rename = "webdriver.load.strategy",
        skip_serializing_if = "Option::is_none"
    )]
    pub load_strategy: Option<String>,
    #[serde(
        rename = "webdriver_firefox_port",
        skip_serializing_if = "Option::is_none"
    )]
    pub webdriver_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FirefoxProfileLogDriver {
    Debug,
    Info,
    Warning,
    Error,
    Off,
}
