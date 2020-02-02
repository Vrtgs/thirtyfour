//! Thirtyfour is a Selenium library for Rust, for automated website UI testing.
//!
//! It supports the full W3C WebDriver spec.
//! Tested with Chrome and Firefox although any W3C-compatible WebDriver
//! should work.
//!
//! Both sync and async APIs are provided (see examples below).
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
//!     // Navigate to URL.
//!     driver.get("http://webappdemo").await?;
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
//!     let elem_result = driver.find_element(By::Name("input-result")).await?;
//!     assert_eq!(elem_result.text().await?, "selenium");
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
//!     // Navigate to URL.
//!     driver.get("http://webappdemo")?;
//!
//!     // Find element.
//!     let elem_div = driver.find_element(By::Css("div[data-section='section-input']"))?;
//!
//!     // Find element from element.
//!     let elem_text = elem_div.find_element(By::Name("input1"))?;
//!
//!     // Type in the search terms.
//!     elem_text.send_keys("selenium")?;
//!
//!     // Click the button.
//!     let elem_button = elem_div.find_element(By::Tag("button"))?;
//!     elem_button.click()?;
//!
//!     // Get text value of element.
//!     let elem_result = driver.find_element(By::Name("input-result"))?;
//!     assert_eq!(elem_result.text()?, "selenium");
//!
//!     Ok(())
//! }
//! ```

#![allow(clippy::needless_doctest_main)]

pub use alert::Alert;
pub use common::{
    capabilities::DesiredCapabilities,
    command::By,
    cookie::Cookie,
    keys::{Keys, TypingData},
    types::*,
};
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
