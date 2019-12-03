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

pub fn make_w3c_caps(caps: serde_json::Value) -> serde_json::Value {
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

        if W3C_CAPABILITY_NAMES.contains(&k.as_str()) || k.contains(":") {
            always_match[k] = v.clone();
        }
    }

    serde_json::json!({
        "firstMatch": [{}], "alwaysMatch": always_match
    })
}
