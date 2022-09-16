//! Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.
//!
//! It supports the full W3C WebDriver spec.
//! Tested with Chrome and Firefox although any W3C-compatible WebDriver
//! should work.
//!
//! ## Features
//!
//! - All W3C WebDriver and WebElement methods supported
//! - Async / await support (tokio only)
//! - Create new browser session directly via WebDriver (e.g. chromedriver)
//! - Create new browser session via Selenium Standalone or Grid
//! - Find elements (via all common selectors e.g. Id, Class, CSS, Tag, XPath)
//! - Send keys to elements, including key-combinations
//! - Execute Javascript
//! - Action Chains
//! - Get and set cookies
//! - Switch to frame/window/element/alert
//! - Shadow DOM support
//! - Alert support
//! - Capture / Save screenshot of browser or individual element as PNG
//! - Some Chrome DevTools Protocol (CDP) support
//! - Advanced query interface including explicit waits and various predicates
//! - Component Wrappers (similar to `Page Object Model`)
//!
//! ## Feature Flags
//!
//! * `rustls-tls`: (Default) Use rustls to provide TLS support (via fantoccini/hyper).
//! * `native-tls`: Use native TLS (via fantoccini/hyper).
//! * `component`: (Default) Enable the `Component` derive macro (via thirtyfour-macros).
//!
//! ## Example
//!
//! The following example assumes you have chromedriver running locally, and
//! a compatible version of Chrome installed.
//!
//! ```no_run
//! use thirtyfour::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> WebDriverResult<()> {
//!     let caps = DesiredCapabilities::chrome();
//!     let driver = WebDriver::new("http://localhost:9515", caps).await?;
//!
//!     // Navigate to https://wikipedia.org.
//!     driver.goto("https://wikipedia.org").await?;
//!     let elem_form = driver.find(By::Id("search-form")).await?;
//!
//!     // Find element from element.
//!     let elem_text = elem_form.find(By::Id("searchInput")).await?;
//!
//!     // Type in the search terms.
//!     elem_text.send_keys("selenium").await?;
//!
//!     // Click the search button.
//!     let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
//!     elem_button.click().await?;
//!
//!     // Look for header to implicitly wait for the page to load.
//!     driver.find(By::ClassName("firstHeading")).await?;
//!     assert_eq!(driver.title().await?, "Selenium - Wikipedia");
//!
//!     // Always explicitly close the browser.
//!     driver.quit().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### The browser will not close automatically
//!
//! Rust does not have [async destructors](https://boats.gitlab.io/blog/post/poll-drop/),
//! which means there is no reliable way to execute an async HTTP request on Drop and wait for
//! it to complete. This means you are in charge of closing the browser at the end of your code,
//! via a call to [`WebDriver::quit`] as in the above example.
//!
//! If you do not call [`WebDriver::quit`] then the browser will stay open until it is
//! either explicitly closed later outside your code, or the session times out.
//!
//! ### Advanced element queries and explicit waits
//!
//! You can use [`WebDriver::query`] to perform more advanced queries
//! including polling and filtering. Custom filter functions are also supported.
//!
//! Also the [`WebElement::wait_until`] method provides additional support for explicit waits
//! using a variety of built-in predicates. You can also provide your own custom predicate if
//! desired.
//!
//! See the [`query`] documentation for more details and examples.
//!
//! [`WebDriver::query`]: crate::extensions::query::ElementQueryable::query
//! [`WebElement::wait_until`]: crate::extensions::query::ElementWaitable::wait_until
//! [`query`]: crate::extensions::query
//!
//! ### Components
//!
//! Components allow you to wrap a web component using smart element resolvers that can
//! automatically re-query stale elements, and much more.
//!
//! ```ignore
//! #[derive(Debug, Clone, Component)]
//! pub struct CheckboxComponent {
//!     base: WebElement,
//!     #[by(tag = "label", first)]
//!     label: ElementResolver<WebElement>,
//!     #[by(css = "input[type='checkbox']")]
//!     input: ElementResolver<WebElement>,
//! }
//!
//! impl CheckBoxComponent {
//!     pub async fn label_text(&self) -> WebDriverResult<String> {
//!         let elem = self.label.resolve().await?;
//!         elem.text().await
//!     }
//!
//!     pub async fn is_ticked(&self) -> WebDriverResult<bool> {
//!         let elem = self.input.resolve().await?;
//!         let prop = elem.prop("checked").await?;
//!         Ok(prop.unwrap_or_default() == "true")
//!     }
//!
//!     pub async fn tick(&self) -> WebDriverResult<()> {
//!         if !self.is_ticked().await? {
//!             let elem = self.input.resolve().await?;
//!             elem.click().await?;
//!             assert!(self.is_ticked().await?);
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! See the [`components`] documentation for more details.
//!
//! ### See Also
//!
//! This crate uses [`fantoccini`] to communicate with the underlying `WebDriver`
//! server, and provides high-level features to extend the capabilities of
//! `fantoccini`. If you want a more lightweight solution and don't need the
//! extra features that `thirtyfour` provides, consider using `fantoccini`
//! directly.
//!

#![forbid(unsafe_code)]
#![allow(clippy::needless_doctest_main)]

// Export types at root level.
pub use alert::Alert;
pub use common::{
    capabilities::{
        chrome::ChromeCapabilities, desiredcapabilities::*, edge::EdgeCapabilities,
        firefox::FirefoxCapabilities, ie::InternetExplorerCapabilities, opera::OperaCapabilities,
        safari::SafariCapabilities,
    },
    command::By,
    types::*,
};
pub use switch_to::SwitchTo;
pub use webdriver::WebDriver;
pub use webelement::WebElement;

/// Allow importing the common types via `use thirtyfour::prelude::*`.
pub mod prelude {
    pub use crate::alert::Alert;
    pub use crate::error::{WebDriverError, WebDriverResult};
    pub use crate::extensions::query::{ElementPoller, ElementQueryable, ElementWaitable};
    pub use crate::session::scriptret::ScriptRet;
    pub use crate::switch_to::SwitchTo;
    pub use crate::WebDriver;
    pub use crate::WebElement;
    pub use crate::{By, Capabilities, DesiredCapabilities};
    pub use crate::{Cookie, Key, TimeoutConfiguration, WindowHandle};
}

/// Action chains allow for more complex user interactions with the keyboard and mouse.
pub mod action_chain;
/// Alert handling.
pub mod alert;
/// Common wrappers used by both async and sync implementations.
pub mod common;
/// Components and component wrappers.
pub mod components;
/// Error wrappers.
pub mod error;
/// Extensions for specific browsers.
pub mod extensions;
/// Everything related to driving the underlying WebDriver session.
pub mod session;
/// Miscellaneous support functions for `thirtyfour` tests.
pub mod support;

mod switch_to;
mod upstream;
mod webdriver;
mod webelement;

// Re-export StringMatch if needed.
pub extern crate stringmatch;

// Re-export common fantoccini types at the root level.
pub use crate::upstream::*;

// Re-export cookie crate.
pub use cookie;
