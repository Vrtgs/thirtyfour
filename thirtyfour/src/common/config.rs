use crate::error::WebDriverError;
use crate::{
    extensions::query::{ElementPollerWithTimeout, IntoElementPoller},
    prelude::WebDriverResult,
};
use http::HeaderValue;
use std::sync::{Arc, OnceLock};

/// Configuration options used by a `WebDriver` instance and the related `SessionHandle`.
///
/// The configuration of a `WebDriver` will be shared by all elements found via that instance.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WebDriverConfig {
    /// If true, send the "Connection: keep-alive" header with all requests.
    pub keep_alive: bool,
    /// The default poller to use when performing element queries or waits.
    pub poller: Arc<dyn IntoElementPoller + Send + Sync>,
    /// The user agent to use when sending commands to the webdriver server.
    pub user_agent: HeaderValue,
}

impl Default for WebDriverConfig {
    fn default() -> Self {
        Self::builder().build().expect("default values failed")
    }
}

impl WebDriverConfig {
    /// Create new `WebDriverConfigBuilder`.
    pub fn builder() -> WebDriverConfigBuilder {
        WebDriverConfigBuilder::new()
    }

    /// Get the default user agent.
    pub fn default_user_agent() -> HeaderValue {
        static DEFAULT: OnceLock<HeaderValue> = OnceLock::new();

        DEFAULT
            .get_or_init(|| {
                format!(
                    "thirtyfour/{} (rust/{}; {})",
                    crate::VERSION,
                    std::env::var("RUSTC_VERSION").as_deref().unwrap_or("unknown"),
                    std::env::consts::OS
                )
                .try_into()
                .expect("bad default user agent")
            })
            .clone()
    }
}

/// Builder for `WebDriverConfig`.
#[derive(Debug)]
pub struct WebDriverConfigBuilder {
    keep_alive: bool,
    poller: Option<Arc<dyn IntoElementPoller + Send + Sync>>,
    user_agent: Option<WebDriverResult<HeaderValue>>,
}

impl Default for WebDriverConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WebDriverConfigBuilder {
    /// Create a new `WebDriverConfigBuilder`.
    pub fn new() -> Self {
        Self {
            keep_alive: true,
            poller: None,
            user_agent: None,
        }
    }

    /// Set the keep_alive option.
    pub fn keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = keep_alive;
        self
    }

    /// Set the specified element poller.
    pub fn poller(mut self, poller: Arc<dyn IntoElementPoller + Send + Sync>) -> Self {
        self.poller = Some(poller);
        self
    }

    /// Set the user agent.
    pub fn user_agent<V>(mut self, user_agent: V) -> Self
    where
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<WebDriverError>,
    {
        self.user_agent = Some(user_agent.try_into().map_err(Into::into));
        self
    }

    /// Build `WebDriverConfig` using builder options.
    pub fn build(self) -> WebDriverResult<WebDriverConfig> {
        Ok(WebDriverConfig {
            keep_alive: self.keep_alive,
            poller: self.poller.unwrap_or_else(|| Arc::new(ElementPollerWithTimeout::default())),
            user_agent: self
                .user_agent
                .unwrap_or_else(|| Ok(WebDriverConfig::default_user_agent()))?,
        })
    }
}
