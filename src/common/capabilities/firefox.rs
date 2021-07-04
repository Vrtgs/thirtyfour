use std::path::Path;

use serde::Serialize;
use serde_json::{from_value, json, to_value, Value};

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
                "browserName": "firefox"
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

    /// Set the firefox preferences to use.
    pub fn set_preferences(&mut self, preferences: FirefoxPreferences) -> WebDriverResult<()> {
        self.add_firefox_option("prefs", preferences)
    }

    /// Add the specified command-line argument to `geckodriver`.
    pub fn add_firefox_arg(&mut self, arg: &str) -> WebDriverResult<()> {
        let mut args = self.get_args();
        let arg_string = arg.to_string();
        if !args.contains(&arg_string) {
            args.push(arg_string);
        }
        self.add_firefox_option("args", to_value(args)?)
    }

    /// Get the current list of command-line arguments to `geckodriver` as a vec.
    pub fn get_args(&self) -> Vec<String> {
        from_value(self.capabilities["moz:firefoxOptions"]["args"].clone()).unwrap_or_default()
    }

    /// Set the browser to run headless.
    pub fn set_headless(&mut self) -> WebDriverResult<()> {
        self.add_firefox_arg("--headless")
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
#[serde(transparent)]
pub struct FirefoxPreferences {
    preferences: Value,
}

impl Default for FirefoxPreferences {
    fn default() -> Self {
        FirefoxPreferences {
            preferences: json!({}),
        }
    }
}

impl FirefoxPreferences {
    pub fn get(&self) -> &Value {
        &self.preferences
    }

    pub fn get_mut(&mut self) -> &mut Value {
        &mut self.preferences
    }
}

impl FirefoxPreferences {
    pub fn new() -> Self {
        FirefoxPreferences::default()
    }

    pub fn set<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        let v = self.get_mut();
        v[key] = to_value(value)?;
        Ok(())
    }
    pub fn unset(&mut self, key: &str) -> WebDriverResult<()> {
        let v = self.get_mut().as_object_mut().unwrap(); // This is safe because it should allways be an object
        v.remove(key);
        Ok(())
    }

    pub fn set_accept_untrusted_certs(&mut self, value: bool) -> WebDriverResult<()> {
        self.set("webdriver_accept_untrusted_certs", value)
    }

    pub fn unset_accept_untrusted_certs(&mut self) -> WebDriverResult<()> {
        self.unset("webdriver_accept_untrusted_certs")
    }

    pub fn set_assume_untrusted_issuer(&mut self, value: bool) -> WebDriverResult<()> {
        self.set("webdriver_assume_untrusted_issuer", value)
    }

    pub fn unset_assume_untrusted_issuer(&mut self) -> WebDriverResult<()> {
        self.unset("webdriver_assume_untrusted_issuer")
    }

    pub fn set_log_driver(&mut self, value: FirefoxProfileLogDriver) -> WebDriverResult<()> {
        self.set("webdriver.log.driver", value)
    }

    pub fn unset_log_driver(&mut self) -> WebDriverResult<()> {
        self.unset("webdriver.log.driver")
    }

    pub fn set_log_file(&mut self, value: String) -> WebDriverResult<()> {
        self.set("webdriver.log.file", value)
    }

    pub fn unset_log_file(&mut self) -> WebDriverResult<()> {
        self.unset("webdriver.log.file")
    }

    pub fn set_load_strategy(&mut self, value: String) -> WebDriverResult<()> {
        self.set("webdriver.load.strategy", value)
    }

    pub fn unset_load_strategy(&mut self) -> WebDriverResult<()> {
        self.unset("webdriver.load.strategy")
    }

    pub fn set_webdriver_port(&mut self, value: u16) -> WebDriverResult<()> {
        self.set("webdriver_firefox_port", value)
    }

    pub fn unset_webdriver_port(&mut self) -> WebDriverResult<()> {
        self.unset("webdriver_firefox_port")
    }

    pub fn set_user_agent(&mut self, value: String) -> WebDriverResult<()> {
        self.set("general.useragent.override", value)
    }

    pub fn unset_user_agent(&mut self) -> WebDriverResult<()> {
        self.unset("general.useragent.override")
    }
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
