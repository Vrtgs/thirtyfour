use crate::error::WebDriverResult;
use crate::Capabilities;
use crate::CapabilitiesHelper;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_value, json, to_value};
use std::ops::{Deref, DerefMut};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct ChromeCapabilities {
    capabilities: Capabilities,
}

impl Default for ChromeCapabilities {
    fn default() -> Self {
        let mut capabilities = Capabilities::new();
        capabilities.insert("browserName".to_string(), json!("chrome"));
        ChromeCapabilities {
            capabilities,
        }
    }
}

impl ChromeCapabilities {
    /// Create a new ChromeCapabilities struct.
    pub fn new() -> Self {
        ChromeCapabilities::default()
    }

    /// Add the specified Chrome option. This is a helper method for `add_chrome_arg()`.
    pub fn add_chrome_option<T>(&mut self, key: &str, value: T) -> WebDriverResult<()>
    where
        T: Serialize,
    {
        self.add_subkey("goog:chromeOptions", key, value)
    }

    /// Get the specified Chrome option.
    pub fn get_chrome_option<T>(&self, key: &str) -> T
    where
        T: DeserializeOwned + Default,
    {
        self.capabilities
            .get("goog:chromeOptions")
            .and_then(|options| options.get(key))
            .and_then(|option| from_value(option.clone()).ok())
            .unwrap_or_default()
    }

    /// Get the current list of command-line arguments to `chromedriver` as a vec.
    pub fn get_args(&self) -> Vec<String> {
        self.get_chrome_option("args")
    }

    /// Get the current list of Chrome extensions as a vec.
    /// Each item is a base64-encoded string containing the .CRX extension file contents.
    /// Use `add_extension()` to add a new extension file.
    pub fn get_extensions(&self) -> Vec<String> {
        self.get_chrome_option("extensions")
    }

    /// Get the path to the chrome binary (if one was previously set).
    pub fn get_binary(&self) -> String {
        self.get_chrome_option("binary")
    }

    /// Set the path to chrome binary to use.
    pub fn set_binary(&mut self, path: &str) -> WebDriverResult<()> {
        self.add_chrome_option("binary", path)
    }

    /// Get the current debugger address (if one was previously set).
    pub fn get_debugger_address(&self) -> String {
        self.get_chrome_option("debuggerAddress")
    }

    /// Set the debugger address.
    pub fn set_debugger_address(&mut self, address: &str) -> WebDriverResult<()> {
        self.add_chrome_option("debuggerAddress", address)
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

    /// Remove the specified Chrome command-line argument if it had been added previously.
    pub fn remove_chrome_arg(&mut self, arg: &str) -> WebDriverResult<()> {
        let mut args = self.get_args();
        if args.is_empty() {
            Ok(())
        } else {
            args.retain(|v| v != arg);
            self.add_chrome_option("args", to_value(args)?)
        }
    }

    /// Add a base64-encoded extension.
    pub fn add_encoded_extension(&mut self, extension_base64: &str) -> WebDriverResult<()> {
        let mut extensions = self.get_extensions();
        let ext_string = extension_base64.to_string();
        if !extensions.contains(&ext_string) {
            extensions.push(ext_string);
        }
        self.add_chrome_option("extensions", to_value(extensions)?)
    }

    /// Remove the specified base64-encoded extension if it had been added previously.
    pub fn remove_encoded_extension(&mut self, extension_base64: &str) -> WebDriverResult<()> {
        let mut extensions = self.get_extensions();
        if extensions.is_empty() {
            Ok(())
        } else {
            extensions.retain(|v| v != extension_base64);
            self.add_chrome_option("extensions", to_value(extensions)?)
        }
    }

    /// Add Chrome extension file. This will be a file with a .CRX extension.
    pub fn add_extension(&mut self, crx_file: &Path) -> WebDriverResult<()> {
        let contents = std::fs::read(crx_file)?;
        let b64_contents = base64::encode(contents);
        self.add_encoded_extension(&b64_contents)
    }

    /// Remove the specified Chrome extension file if an identical extension had been added
    /// previously.
    pub fn remove_extension(&mut self, crx_file: &Path) -> WebDriverResult<()> {
        let contents = std::fs::read(crx_file)?;
        let b64_contents = base64::encode(contents);
        self.remove_encoded_extension(&b64_contents)
    }

    /// Set the browser to run headless.
    pub fn set_headless(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--headless")
    }

    /// Unset the headless option.
    pub fn unset_headless(&mut self) -> WebDriverResult<()> {
        self.remove_chrome_arg("--headless")
    }

    /// Set disable web security.
    pub fn set_disable_web_security(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--disable-web-security")
    }

    /// Unset disable web security.
    pub fn unset_disable_web_security(&mut self) -> WebDriverResult<()> {
        self.remove_chrome_arg("--disable-web-security")
    }

    /// Set ignore certificate errors.
    pub fn set_ignore_certificate_errors(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--ignore-certificate-errors")
    }

    /// Unset ignore certificate errors.
    pub fn unset_ignore_certificate_errors(&mut self) -> WebDriverResult<()> {
        self.remove_chrome_arg("--ignore-certificate-errors")
    }

    pub fn set_no_sandbox(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--no-sandbox")
    }

    pub fn unset_no_sandbox(&mut self) -> WebDriverResult<()> {
        self.remove_chrome_arg("--no-sandbox")
    }

    pub fn set_disable_gpu(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--disable-gpu")
    }

    pub fn unset_disable_gpu(&mut self) -> WebDriverResult<()> {
        self.remove_chrome_arg("--disable-gpu")
    }

    pub fn set_disable_dev_shm_usage(&mut self) -> WebDriverResult<()> {
        self.add_chrome_arg("--disable-dev-shm-usage")
    }

    pub fn unset_disable_dev_shm_usage(&mut self) -> WebDriverResult<()> {
        self.remove_chrome_arg("--disable-dev-shm-usage")
    }
}

impl From<ChromeCapabilities> for Capabilities {
    fn from(caps: ChromeCapabilities) -> Capabilities {
        caps.capabilities
    }
}

impl Deref for ChromeCapabilities {
    type Target = Capabilities;

    fn deref(&self) -> &Self::Target {
        &self.capabilities
    }
}

impl DerefMut for ChromeCapabilities {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.capabilities
    }
}
