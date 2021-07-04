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
    preferences: serde_json::Map<String, Value>,
}

impl Default for FirefoxPreferences {
    fn default() -> Self {
        FirefoxPreferences {
            preferences: serde_json::Map::<String, Value>::default(),
        }
    }
}

impl FirefoxPreferences {
    pub fn new() -> Self {
        FirefoxPreferences::default()
    }

    /// Sets the specified firefox preference. This is a helper method for the various
    /// specific option methods.
    pub fn set<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self.preferences[key] = to_value(value)?;
        Ok(())
    }

    /// Resets the specified firefox preference. This is a helper method for the various
    /// specific option methods.
    pub fn reset(&mut self, key: &str) -> WebDriverResult<()> {
        self.preferences.remove(key);
        Ok(())
    }

    /// Sets accept untrusted certs
    pub fn set_accept_untrusted_certs(&mut self, value: bool) -> WebDriverResult<()> {
        self.set("webdriver_accept_untrusted_certs", value)
    }

    /// Resets accept untrusted certs
    pub fn reset_accept_untrusted_certs(&mut self) -> WebDriverResult<()> {
        self.reset("webdriver_accept_untrusted_certs")
    }

    /// Sets assume untrusted issuer
    pub fn set_assume_untrusted_issuer(&mut self, value: bool) -> WebDriverResult<()> {
        self.set("webdriver_assume_untrusted_issuer", value)
    }

    /// Resets assume untrusted issuer
    pub fn reset_assume_untrusted_issuer(&mut self) -> WebDriverResult<()> {
        self.reset("webdriver_assume_untrusted_issuer")
    }

    /// Sets the log driver
    pub fn set_log_driver(&mut self, value: FirefoxProfileLogDriver) -> WebDriverResult<()> {
        self.set("webdriver.log.driver", value)
    }

    /// Resets the log driver
    pub fn reset_log_driver(&mut self) -> WebDriverResult<()> {
        self.reset("webdriver.log.driver")
    }

    /// Sets the log file
    pub fn set_log_file(&mut self, value: String) -> WebDriverResult<()> {
        self.set("webdriver.log.file", value)
    }

    /// Resets the log file
    pub fn reset_log_file(&mut self) -> WebDriverResult<()> {
        self.reset("webdriver.log.file")
    }

    /// Sets the load strategy
    pub fn set_load_strategy(&mut self, value: String) -> WebDriverResult<()> {
        self.set("webdriver.load.strategy", value)
    }

    /// Resets the load strategy
    pub fn reset_load_strategy(&mut self) -> WebDriverResult<()> {
        self.reset("webdriver.load.strategy")
    }

    /// Sets the webdriver port
    pub fn set_webdriver_port(&mut self, value: u16) -> WebDriverResult<()> {
        self.set("webdriver_firefox_port", value)
    }

    /// Resets the webdriver port
    pub fn reset_webdriver_port(&mut self) -> WebDriverResult<()> {
        self.reset("webdriver_firefox_port")
    }

    /// Sets the user agent
    pub fn set_user_agent(&mut self, value: String) -> WebDriverResult<()> {
        self.set("general.useragent.override", value)
    }

    /// Resets the user agent
    pub fn reset_user_agent(&mut self) -> WebDriverResult<()> {
        self.reset("general.useragent.override")
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
