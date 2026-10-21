/// Types used with action chains.
pub mod action;
/// Support for desired capabilities.
pub mod capabilities;
/// Helpers for webdriver commands.
pub mod command;
/// Configuration options for a `WebDriver` instance.
pub mod config;
/// Cookie type.
pub mod cookie;
/// Types for working with keyboard input.
pub mod keys;
/// Browser log entry types and logging-preferences capability values.
pub mod log;
/// Types used with print commands.
pub mod print;
#[cfg(any(feature = "cdp", feature = "bidi"))]
pub(crate) mod protocol;
/// Type for request method and body.
pub mod requestdata;
/// Common types used within thirtyfour.
pub mod types;
#[cfg(any(feature = "cdp-events", feature = "bidi"))]
pub(crate) mod ws;
