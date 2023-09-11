use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, to_value, Value};

use crate::common::capabilities::chrome::ChromeCapabilities;
use crate::common::capabilities::edge::EdgeCapabilities;
use crate::common::capabilities::firefox::FirefoxCapabilities;
use crate::common::capabilities::ie::InternetExplorerCapabilities;
use crate::common::capabilities::opera::OperaCapabilities;
use crate::common::capabilities::safari::SafariCapabilities;
use crate::error::WebDriverResult;
use crate::{Capabilities, ChromiumCapabilities};

/// Provides static methods for constructing browser-specific capabilities.
///
/// ## Example
/// ```no_run
/// use thirtyfour::{DesiredCapabilities, WebDriver};
/// let caps = DesiredCapabilities::chrome();
/// let driver = WebDriver::new("http://localhost:4444", caps);
/// ```
#[derive(Debug)]
pub struct DesiredCapabilities;

impl DesiredCapabilities {
    /// Create a ChromeCapabilities struct.
    pub fn chrome() -> ChromeCapabilities {
        ChromeCapabilities::new()
    }

    /// Create a ChromiumCapabilities struct.
    pub fn chromium() -> ChromiumCapabilities {
        ChromiumCapabilities::new()
    }

    /// Create an EdgeCapabilities struct.
    pub fn edge() -> EdgeCapabilities {
        EdgeCapabilities::new()
    }

    /// Create a FirefoxCapabilities struct.
    pub fn firefox() -> FirefoxCapabilities {
        FirefoxCapabilities::new()
    }

    /// Create an InternetExplorerCapabilities struct.
    pub fn internet_explorer() -> InternetExplorerCapabilities {
        InternetExplorerCapabilities::new()
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

/// Provides common features for all Capabilities structs.
pub trait CapabilitiesHelper {
    /// Get an immutable reference to the underlying serde_json::Value.
    fn _get(&self, key: &str) -> Option<&Value>;

    /// Get a mutable reference to the underlying serde_json::Value.
    fn _get_mut(&mut self, key: &str) -> Option<&mut Value>;

    /// Set the specified capability at the root level.
    fn insert_base_capability(&mut self, key: String, value: Value);

    /// Add any Serialize-able object to the capabilities under the specified key.
    fn set_base_capability<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self.insert_base_capability(key.to_string(), to_value(value)?);
        Ok(())
    }

    /// Set the desired browser version.
    fn set_version(&mut self, version: &str) -> WebDriverResult<()> {
        self.set_base_capability("version", version)
    }

    /// Set the desired browser platform.
    fn set_platform(&mut self, platform: &str) -> WebDriverResult<()> {
        self.set_base_capability("platform", platform)
    }

    /// Set whether the session supports executing user-supplied Javascript.
    fn set_javascript_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("javascriptEnabled", enabled)
    }

    /// Set whether the session can interact with database storage.
    fn set_database_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("databaseEnabled", enabled)
    }

    /// Set whether the session can set and query the browser's location context.
    fn set_location_context_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("locationContextEnabled", enabled)
    }

    /// Set whether the session can interact with the application cache.
    fn set_application_cache_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("applicationCacheEnabled", enabled)
    }

    /// Set whether the session can query for the browser's connectivity and disable it if desired.
    fn set_browser_connection_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("browserConnectionEnabled", enabled)
    }

    /// Set whether the session supports interactions with local storage.
    fn set_web_storage_enabled(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("webStorageEnabled", enabled)
    }

    /// Set whether the session should accept all SSL certificates by default.
    #[deprecated(since = "0.32.0-rc.5", note = "please use `accept_insecure_certs` instead")]
    fn accept_ssl_certs(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("acceptSslCerts", enabled)
    }

    /// Set whether the session should accept insecure SSL certificates by default.
    fn accept_insecure_certs(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("acceptInsecureCerts", enabled)
    }

    /// Set whether the session can rotate the current page's layout between portrait and landscape
    /// orientations. Only applies to mobile platforms.
    fn set_rotatable(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("rotatable", enabled)
    }

    /// Set whether the session is capable of generating native events when simulating user input.
    fn set_native_events(&mut self, enabled: bool) -> WebDriverResult<()> {
        self.set_base_capability("nativeEvents", enabled)
    }

    /// Set the proxy to use.
    fn set_proxy(&mut self, proxy: Proxy) -> WebDriverResult<()> {
        self.set_base_capability("proxy", proxy)
    }

    /// Set the behaviour to be followed when an unexpected alert is encountered.
    fn set_unexpected_alert_behaviour(&mut self, behaviour: AlertBehaviour) -> WebDriverResult<()> {
        self.set_base_capability("unexpectedAlertBehaviour", behaviour)
    }

    /// Set whether elements are scrolled into the viewport for interation to align with the top
    /// or the bottom of the viewport. The default is to align with the top.
    fn set_element_scroll_behaviour(&mut self, behaviour: ScrollBehaviour) -> WebDriverResult<()> {
        self.set_base_capability("elementScrollBehavior", behaviour)
    }

    /// Get whether the session can interact with modal popups such as `window.alert`.
    fn handles_alerts(&self) -> Option<bool> {
        self._get("handlesAlerts").and_then(|x| x.as_bool())
    }

    /// Get whether the session supports CSS selectors when searching for elements.
    fn css_selectors_enabled(&self) -> Option<bool> {
        self._get("cssSelectorsEnabled").and_then(|x| x.as_bool())
    }

    /// Get the current page load strategy.
    fn page_load_strategy(&self) -> WebDriverResult<PageLoadStrategy> {
        let strategy = self._get("pageLoadStrategy").map(|x| from_value(x.clone())).transpose()?;
        Ok(strategy.unwrap_or_default())
    }

    /// Set the page load strategy to use.
    fn set_page_load_strategy(&mut self, strategy: PageLoadStrategy) -> WebDriverResult<()> {
        self.set_base_capability("pageLoadStrategy", strategy)
    }
}

/// Helper trait for adding browser-specific capabilities.
///
/// For example, chrome stores capabilities under `goog:chromeOptions` and firefox
/// stores capabilities under `moz:firefoxOptions`.
pub trait BrowserCapabilitiesHelper: CapabilitiesHelper {
    /// The key containing the browser-specific capabilities.
    const KEY: &'static str;

    /// Add any Serialize-able object to the capabilities under the browser's custom key.
    fn insert_browser_option(
        &mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> WebDriverResult<()> {
        match self._get_mut(Self::KEY) {
            Some(Value::Object(v)) => {
                v.insert(key.into(), to_value(value)?);
            }
            _ => self.insert_base_capability(Self::KEY.to_string(), json!({ key: value })),
        }
        Ok(())
    }

    /// Remove the custom browser-specific property, if it exists.
    fn remove_browser_option(&mut self, key: &str) {
        if let Some(Value::Object(v)) = &mut self._get_mut(Self::KEY) {
            v.remove(key);
        }
    }

    /// Get the custom browser-specific property, if it exists.
    fn browser_option<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        self._get(Self::KEY)
            .and_then(|options| options.get(key))
            .and_then(|option| from_value(option.clone()).ok())
    }
}

impl CapabilitiesHelper for Capabilities {
    fn _get(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }

    fn _get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.get_mut(key)
    }

    fn insert_base_capability(&mut self, key: String, value: Value) {
        self.insert(key, value);
    }
}

/// Proxy configuration settings.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "proxyType", rename_all = "lowercase")]
pub enum Proxy {
    /// Direct connection to the webdriver server.
    Direct,
    /// Manual proxy configuration.
    #[serde(rename_all = "camelCase")]
    Manual {
        /// FTP proxy.
        #[serde(skip_serializing_if = "Option::is_none")]
        ftp_proxy: Option<String>,
        /// HTTP proxy.
        #[serde(skip_serializing_if = "Option::is_none")]
        http_proxy: Option<String>,
        /// SSL proxy.
        #[serde(skip_serializing_if = "Option::is_none")]
        ssl_proxy: Option<String>,
        /// SOCKS proxy.
        #[serde(skip_serializing_if = "Option::is_none")]
        socks_proxy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// The SOCKS version.
        socks_version: Option<u8>,
        /// SOCKS username.
        #[serde(skip_serializing_if = "Option::is_none")]
        socks_username: Option<String>,
        /// SOCKS password.
        #[serde(skip_serializing_if = "Option::is_none")]
        socks_password: Option<String>,
        /// Urls to skip the proxy.
        #[serde(skip_serializing_if = "Option::is_none")]
        no_proxy: Option<String>,
    },
    /// Autoconfiguration url.
    #[serde(rename = "pac")]
    AutoConfig {
        /// The autoconfiguration url.
        #[serde(rename = "proxyAutoconfigUrl")]
        url: String,
    },
    /// Auto-detect proxy.
    AutoDetect,
    /// Use the system proxy configuration.
    System,
}

/// The action to take when an alert is encountered.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertBehaviour {
    /// Automatically accept the alert.
    Accept,
    /// Automatically dismiss the alert.
    Dismiss,
    /// Ignore the alert.
    Ignore,
}

/// The automatic scrolling behaviour for this session.
#[derive(Debug, Clone, Serialize)]
#[repr(u8)]
pub enum ScrollBehaviour {
    /// Scroll until the element is at the top of the screen, if possible.
    Top = 0,
    /// Scroll until the element is at the bottom of the screen, if possible.
    Bottom = 1,
}

/// The page load strategy for this session.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for PageLoadStrategy {
    fn default() -> Self {
        PageLoadStrategy::Normal
    }
}
