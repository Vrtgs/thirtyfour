//! Selenium client for working with W3C-compatible WebDriver implementations.
//!
//! Both synchronous and asynchronous APIs are provided (see examples below).
//!
//! Currently supported (but may not yet be fully tested):
//!
//! - Create remote browser session
//! - Automatically close browser session on drop
//! - Most WebDriver and WebElement methods
//! - Find elements (via all common selectors)
//! - Send keys to elements, including key-combinations
//! - Execute Javascript
//! - Action Chains
//! - Cookies
//! - Switch to frame/window/element/alert
//! - Alert support
//! - Save screenshot of browser or individual element
//! - Synchronous support
//! - Async / await support
//!
//! **Any help with testing and creating tickets for any issues found would be greatly appreciated.**
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
//! use std::thread;
//! use std::time::Duration;
//! use thirtyfour::error::WebDriverResult;
//! use thirtyfour::{By, WebDriver};
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() {
//!     webtest().await.expect("Something went wrong");
//! }
//!
//! async fn webtest() -> WebDriverResult<()> {
//!     let caps = serde_json::json!({
//!         "browserName": "chrome",
//!         "version": "",
//!         "platform": "any"
//!     });
//!
//!     let driver = WebDriver::new("http://localhost:4444/wd/hub", caps).await?;
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
//! use std::thread;
//! use std::time::Duration;
//! use thirtyfour::error::WebDriverResult;
//! use thirtyfour::{By, sync::WebDriver};
//!
//! fn main() {
//!     webtest().expect("Something went wrong");
//! }
//!
//! fn webtest() -> WebDriverResult<()> {
//!     let caps = serde_json::json!({
//!         "browserName": "chrome",
//!         "version": "",
//!         "platform": "any"
//!     });
//!
//!     let driver = WebDriver::new("http://localhost:4444/wd/hub", caps)?;
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
pub use alert::Alert;
pub use common::command::By;
pub use common::cookie::Cookie;
pub use common::types::*;
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
