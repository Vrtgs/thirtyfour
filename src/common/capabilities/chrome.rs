use serde::Serialize;
use serde_json::{from_value, json, to_value, Value};

use crate::common::capabilities::desiredcapabilities::Capabilities;
use crate::error::WebDriverResult;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct ChromeCapabilities {
    capabilities: Value,
}

impl Default for ChromeCapabilities {
    fn default() -> Self {
        ChromeCapabilities {
            capabilities: json!({
                "browserName": "chrome",
                "version": "",
                "platform": "ANY"
            }),
        }
    }
}

impl ChromeCapabilities {
    /// Create a new ChromeCapabilities struct.
    pub fn new() -> Self {
        ChromeCapabilities::default()
    }

    /// Add the specified command-line argument to `chromedriver`. Eg. "--disable-local-storage"
    /// The full list of switches can be found here:
    /// [https://chromium.googlesource.com/chromium/src/+/master/chrome/common/chrome_switches.cc](https://chromium.googlesource.com/chromium/src/+/master/chrome/common/chrome_switches.cc)
    pub fn add_chrome_arg(&mut self, arg: &str) -> WebDriverResult<()> {
        let mut args = self.get_args();
        let arg_string = arg.to_string();
        if !args.contains(&arg_string) {
            args.push(arg_string);
        }
        self.add_chrome_option("args", to_value(args)?)
    }

    /// Add the specified chrome option. This is a helper method for `add_chrome_arg()`.
    pub fn add_chrome_option<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self.add_subkey("goog:chromeOptions", key, value)
    }

    /// Get the current list of command-line arguments to `chromedriver` as a vec.
    pub fn get_args(&self) -> Vec<String> {
        from_value(self.capabilities["goog:chromeOptions"]["args"].clone()).unwrap_or_default()
    }

    /// Set the browser to run headless.
    pub fn set_headless(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--headless")
    }

    /// Disable web security.
    pub fn set_disable_web_security(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--disable-web-security")
    }
}

impl Capabilities for ChromeCapabilities {
    fn get(&self) -> &Value {
        &self.capabilities
    }

    fn get_mut(&mut self) -> &mut Value {
        &mut self.capabilities
    }
}
