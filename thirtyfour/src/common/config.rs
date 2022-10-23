use crate::extensions::query::{ElementPollerWithTimeout, IntoElementPoller};
use std::sync::Arc;

/// Configuration options used by a `WebDriver` instance and the related `SessionHandle`.
///
/// The configuration of a `WebDriver` will be shared by all elements found via that instance.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct WebDriverConfig {
    /// The default poller to use when performing element queries or waits.
    pub poller: Arc<dyn IntoElementPoller + Send + Sync>,
}

impl Default for WebDriverConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl WebDriverConfig {
    /// Create new `WebDriverConfigBuilder`.
    pub fn builder() -> WebDriverConfigBuilder {
        WebDriverConfigBuilder::new()
    }
}

/// Builder for `WebDriverConfig`.
#[derive(Debug, Clone)]
pub struct WebDriverConfigBuilder {
    poller: Option<Arc<dyn IntoElementPoller + Send + Sync>>,
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
            poller: None,
        }
    }

    /// Set the specified element poller.
    pub fn poller(mut self, poller: Arc<dyn IntoElementPoller + Send + Sync>) -> Self {
        self.poller = Some(poller);
        self
    }

    /// Build `WebDriverConfig` using builder options.
    pub fn build(self) -> WebDriverConfig {
        WebDriverConfig {
            poller: self.poller.unwrap_or_else(|| Arc::new(ElementPollerWithTimeout::default())),
        }
    }
}
