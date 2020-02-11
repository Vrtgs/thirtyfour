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
    pub fn new() -> Self {
        ChromeCapabilities::default()
    }

    pub fn add_chrome_option<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self.add_subkey("goog:chromeOptions", key, value)
    }

    pub fn get_args(&self) -> Vec<String> {
        from_value(self.capabilities["goog:chromeOptions"]["args"].clone()).unwrap_or_default()
    }

    pub fn add_arg(&mut self, arg: &str) -> WebDriverResult<()> {
        let mut args = self.get_args();
        let arg_string = arg.to_string();
        if !args.contains(&arg_string) {
            args.push(arg_string);
        }
        self.add_chrome_option("args", to_value(args)?)
    }

    pub fn set_headless(&mut self) -> WebDriverResult<()> {
        self.add_arg("--headless")
    }

    pub fn set_disable_web_security(&mut self) -> WebDriverResult<()> {
        self.add_arg("--disable-web-security")
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
