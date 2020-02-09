use serde_json::{json, to_value, Value};

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
    // TODO: support proxy and firefox profile.

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

pub struct DesiredCapabilities {
    pub capabilities: serde_json::Value,
}

impl DesiredCapabilities {
    pub fn firefox() -> Self {
        DesiredCapabilities {
            capabilities: json!({
                "browserName": "firefox",
                "acceptInsecureCerts": true,
            }),
        }
    }

    pub fn internet_explorer() -> Self {
        DesiredCapabilities {
            capabilities: json!({
                "browserName": "internet explorer",
                "version": "",
                "platform": "WINDOWS"
            }),
        }
    }

    pub fn edge() -> Self {
        DesiredCapabilities {
            capabilities: json!({
                "browserName": "MicrosoftEdge",
                "version": "",
                "platform": "WINDOWS"
            }),
        }
    }

    pub fn chrome() -> Self {
        DesiredCapabilities {
            capabilities: json!({
                "browserName": "chrome",
                "version": "",
                "platform": "ANY"
            }),
        }
    }

    pub fn opera() -> Self {
        DesiredCapabilities {
            capabilities: json!({
                "browserName": "opera",
                "version": "",
                "platform": "ANY"
            }),
        }
    }

    pub fn safari() -> Self {
        DesiredCapabilities {
            capabilities: json!({
                "browserName": "safari",
                "version": "",
                "platform": "MAC"
            }),
        }
    }

    pub fn set_version(&mut self, version: &str) -> WebDriverResult<()> {
        self.capabilities["version"] = to_value(version)?;
        Ok(())
    }

    pub fn set_platform(&mut self, platform: &str) -> WebDriverResult<()> {
        self.capabilities["platform"] = to_value(platform)?;
        Ok(())
    }

    pub fn add(&mut self, key: &str, value: Value) {
        self.capabilities[key] = value;
    }

    pub fn update(&mut self, value: Value) {
        merge(&mut self.capabilities, value);
    }
}
