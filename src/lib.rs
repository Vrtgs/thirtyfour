//! Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.
//!
//! It supports the full W3C WebDriver spec.
//! Tested with Chrome and Firefox although any W3C-compatible WebDriver
//! should work.
//!
//! ## Features
//!
//! - All W3C WebDriver and WebElement methods supported
//! - Async / await support (both **tokio** and **async-std** runtimes supported via feature flags)
//! - Synchronous support (use the `thirtyfour_sync` crate)
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
//! - Chrome DevTools Protocol (CDP) support
//! - Advanced query interface including explicit waits and various predicates
//!
//! ## Feature Flags
//!
//! * `rusttls-tls`: (Default) Use rusttls to provide TLS support (via fantoccini/hyper).
//! * `native-tls`: Use native TLS (via fantoccini/hyper).
//!
//! ## Examples
//!
//! The following examples assume you have a selenium server running
//! at localhost:4444, and a demo web app running at http://webappdemo
//!
//! You can set these up using docker-compose, as follows:
//!
//! ```ignore
//! docker-compose up -d
//! ```
//!
//! The included web app demo is purely for demonstration / unit testing
//! purposes and is not required in order to use this library in other projects.
//!
//! ### Example (async):
//!
//! ```rust
//! use thirtyfour::prelude::*;
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> WebDriverResult<()> {
//!     let caps = DesiredCapabilities::chrome();
//!     let driver = WebDriver::new("http://localhost:4444", caps).await?;
//!
//!     // Navigate to URL.
//!     driver.get("http://webappdemo").await?;
//!
//!     // Navigate to page, by chaining futures together and awaiting the result.
//!     driver.find_element(By::Id("pagetextinput")).await?.click().await?;
//!
//!     // Find element.
//!     let elem_div = driver.find_element(By::Css("div[data-section='section-input']")).await?;
//!
//!     // Find element from element.
//!     let elem_text = elem_div.find_element(By::Name("input1")).await?;
//!
//!     // Type in the search terms.
//!     elem_text.send_keys("selenium").await?;
//!
//!     // Click the button.
//!     let elem_button = elem_div.find_element(By::Tag("button")).await?;
//!     elem_button.click().await?;
//!
//!     // Get text value of element.
//!     let elem_result = driver.find_element(By::Id("input-result")).await?;
//!     assert_eq!(elem_result.text().await?, "selenium");
//!
//!     // Always explicitly close the browser. There are no async destructors.
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
//! via a call to `WebDriver::quit().await` as in the above example.
//!
//! If you do not call `WebDriver::quit().await` then the browser will stay open until it is
//! either explicitly closed later outside your code, or the session times out.
//!
//! ### Advanced element queries and explicit waits
//!
//! You can use `WebDriver::query()` and `WebElement::query()` to perform more advanced queries
//! including polling and filtering. Custom filter functions are also supported.
//!
//! Also the `WebElement::wait_until()` method provides additional support for explicit waits
//! using a variety of built-in predicates. You can also provide your own custom predicate if
//! desired.
//!
//! See the [query](query/index.html) module documentation for more details.

#![forbid(unsafe_code)]
#![allow(clippy::needless_doctest_main)]

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

pub use cookie;
pub use fantoccini::wd::{TimeoutConfiguration, WindowHandle};
pub use switch_to::SwitchTo;
pub use webdriver::WebDriver;
pub use webelement::WebElement;

/// Allow importing the common async structs via `use thirtyfour::prelude::*`.
pub mod prelude {
    pub use crate::alert::Alert;
    pub use crate::error::WebDriverResult;
    pub use crate::query::{ElementQueryable, ElementWaitable};
    pub use crate::session::scriptret::ScriptRet;
    pub use crate::switch_to::SwitchTo;
    pub use crate::webdriver::WebDriver;
    pub use crate::webelement::WebElement;
    pub use crate::{By, DesiredCapabilities};
    pub use fantoccini::cookies::Cookie;
    pub use fantoccini::key::Key;
}

/// Action chains allow for more complex user interactions with the keyboard and mouse.
pub mod action_chain;
mod alert;
/// Everything related to driving the underlying WebDriver session.
pub mod session {
    pub mod handle;
    pub mod scriptret;
}

/// Miscellaneous support functions for `thirtyfour` tests.
pub mod support;
mod switch_to;
mod webdriver;
mod webelement;

/// Common types used by both async and sync implementations.
pub mod common {
    pub mod capabilities {
        pub mod chrome;
        pub mod desiredcapabilities;
        pub mod edge;
        pub mod firefox;
        pub mod ie;
        pub mod opera;
        pub mod safari;
    }
    pub mod command;
    pub mod config;
    pub mod types;
}

/// Extensions for specific browsers.
pub mod extensions {
    /// Extensions for working with Chromium-based browsers.
    pub mod chrome {
        mod chromecommand;
        mod devtools;
        mod networkconditions;

        pub use chromecommand::ChromeCommand;
        pub use devtools::ChromeDevTools;
        pub use networkconditions::NetworkConditions;
    }

    /// Extensions for working with Firefox.
    pub mod firefox {
        mod firefoxcommand;
        mod firefoxtools;

        pub use firefoxcommand::FirefoxCommand;
        pub use firefoxtools::FirefoxTools;
    }
}

/// Wrappers for specific component types.
pub mod components {
    /// Wrapper for `<select>` elements.
    pub mod select;
}

/// Error types.
pub mod error;

// ElementQuery and ElementWaiter interfaces.
pub mod query;

// Re-export fantoccini types.
pub use fantoccini::key::Key;
