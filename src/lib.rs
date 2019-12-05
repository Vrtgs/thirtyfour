//! Selenium client for working with W3C-compatible WebDriver implmentations.
//!
//! The API is roughly modeled after the python selenium library.
//!
//! **NOTE:** This project is still WIP and not yet ready for production.
//!
//! Both synchronous and asynchronous APIs are provided.
//!
//! ### Async example:
//!
//! ```rust
//! use std::thread;
//! use std::time::Duration;
//! use thirtyfour::error::WebDriverResult;
//! use thirtyfour::{By, WebDriver};
//!
//! #[tokio::main]
//! async fn main() {
//!     webtest().await.expect("Something went wrong");
//! }
//!
//! async fn webtest() -> WebDriverResult<()> {
//!     info!("Launching new browser session");
//!     let caps = serde_json::json!({
//!         "browserName": "chrome",
//!         "version": "",
//!         "platform": "any"
//!     });
//!     let driver = WebDriver::new("http://localhost:4444/wd/hub", caps).await?;
//!     driver.get("https://google.com.au").await?;
//!     let elems = driver.find_elements(By::Tag("div")).await?;
//!     println!("Got: {:?}", elems);
//!     thread::sleep(Duration::new(3, 0));
//!     info!("Closing browser (implicit)");
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
//! use thirtyfour::By;
//! use thirtyfour::sync::WebDriver;
//!
//! fn main() {
//!     webtest().expect("Something went wrong");
//! }
//!
//! fn webtest() -> WebDriverResult<()> {
//!     info!("Launching new browser session");
//!     let caps = serde_json::json!({
//!         "browserName": "chrome",
//!         "version": "",
//!         "platform": "any"
//!     });
//!     let driver = WebDriver::new("http://localhost:4444/wd/hub", caps)?;
//!     driver.get("https://google.com.au")?;
//!     let elems = driver.find_elements(By::Tag("div"))?;
//!     println!("Got: {:?}", elems);
//!     thread::sleep(Duration::new(3, 0));
//!     info!("Closing browser (implicit)");
//!
//!     Ok(())
//! }
//! ```
pub use common::command::By;
pub use connection_async::*;
pub use webdriver::WebDriver;
pub use webelement::WebElement;

mod connection_async;
mod webdriver;
mod webelement;

pub mod common {
    pub mod capabilities;
    pub mod command;
    pub mod connection_common;
    pub mod constant;
    pub mod keys;
}
pub mod sync {
    pub use connection_sync::*;
    pub use webdriver::WebDriver;
    pub use webelement::WebElement;

    mod connection_sync;
    mod webdriver;
    mod webelement;
}

pub mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
