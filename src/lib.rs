//! Thirtyfour is a full-featured Selenium library for Rust,
//! inspired by the Python Selenium library.
//!
//! It supports the W3C WebDriver spec.
//! Tested with Chrome and Firefox although any W3C-compatible WebDriver
//! should work.
//!
//! Both synchronous and asynchronous APIs are provided (see examples below).
//!
//! ## Features
//!
//! - Async / await support
//! - Synchronous support
//! - Create new browser session via Selenium Standalone or Grid
//! - Automatically close browser session on drop
//! - All W3C WebDriver and WebElement methods supported
//! - Find elements (via all common selectors)
//! - Send keys to elements, including key-combinations
//! - Execute Javascript
//! - Action Chains
//! - Cookies
//! - Switch to frame/window/element/alert
//! - Alert support
//! - Capture / Save screenshot of browser or individual element
//!
//! ## Examples
//!
//! The following examples assume you have a selenium server running
//! at localhost:4444.
//!
//! i.e.
//!
//! ```ignore
//! java -jar selenium-server-standalone-3.141.59.jar
//! ```
//!
//! ### Async example:
//!
//! ```rust
//! use thirtyfour::error::WebDriverResult;
//! use thirtyfour::{By, DesiredCapabilities, WebDriver};
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> WebDriverResult<()> {
//!     let caps = DesiredCapabilities::chrome();
//!     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
//!
//!     // Navigate to https://wikipedia.org.
//!     driver.get("https://wikipedia.org").await?;
//!     let elem_form = driver.find_element(By::Id("search-form")).await?;
//!
//!     // Find element from element.
//!     let elem_text = elem_form.find_element(By::Id("searchInput")).await?;
//!
//!     // Type in the search terms.
//!     elem_text.send_keys("selenium").await?;
//!
//!     // Click the search button.
//!     let elem_button = elem_form.find_element(By::Css("button[type='submit']")).await?;
//!     elem_button.click().await?;
//!
//!     // Look for header to implicitly wait for the page to load.
//!     driver.find_element(By::ClassName("firstHeading")).await?;
//!     assert_eq!(driver.title().await?, "Selenium - Wikipedia");
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Sync example:
//!
//! ```rust
//! use thirtyfour::error::WebDriverResult;
//! use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
//!
//! fn main() -> WebDriverResult<()> {
//!     let caps = DesiredCapabilities::chrome();
//!     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
//!
//!     // Navigate to https://wikipedia.org.
//!     driver.get("https://wikipedia.org")?;
//!     let elem_form = driver.find_element(By::Id("search-form"))?;
//!
//!     // Find element from element.
//!     let elem_text = elem_form.find_element(By::Id("searchInput"))?;
//!
//!     // Type in the search terms.
//!     elem_text.send_keys("selenium")?;
//!
//!     // Click the search button.
//!     let elem_button = elem_form.find_element(By::Css("button[type='submit']"))?;
//!     elem_button.click()?;
//!
//!     // Look for header to implicitly wait for the page to load.
//!     driver.find_element(By::ClassName("firstHeading"))?;
//!     assert_eq!(driver.title()?, "Selenium - Wikipedia");
//!
//!     Ok(())
//! }
//! ```

#![allow(clippy::needless_doctest_main)]

pub use alert::Alert;
pub use common::{capabilities::DesiredCapabilities, command::By, cookie::Cookie, types::*};
pub use connection_async::*;
pub use switch_to::SwitchTo;
pub use webdriver::WebDriver;
pub use webelement::WebElement;

pub mod action_chain;
mod alert;
mod connection_async;
mod switch_to;
mod webdriver;
mod webelement;

pub mod common {
    pub mod action;
    pub mod capabilities;
    pub mod command;
    pub mod connection_common;
    pub mod cookie;
    pub mod keys;
    pub mod types;
}
pub mod sync {
    pub use alert::Alert;
    pub use connection_sync::*;
    pub use switch_to::SwitchTo;
    pub use webdriver::WebDriver;
    pub use webelement::WebElement;

    pub mod action_chain;
    mod alert;
    mod connection_sync;
    mod switch_to;
    mod webdriver;
    mod webelement;
}

pub mod error;
