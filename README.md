[![Crates.io](https://img.shields.io/crates/v/thirtyfour.svg)](https://crates.io/crates/thirtyfour)
[![Documentation](https://docs.rs/thirtyfour/badge.svg)](https://docs.rs/thirtyfour/)
[![Build Status](https://travis-ci.org/stevepryde/thirtyfour.svg?branch=master)](https://travis-ci.org/stevepryde/thirtyfour)

Thirtyfour is a full-featured Selenium library for Rust, inspired by the Python Selenium library.

It supports the W3C WebDriver spec. Tested with Chrome and Firefox although any W3C-compatible WebDriver should work.

Both synchronous and async / await APIs are provided (see examples below).

## Features

- Async / await support
- Synchronous support
- Create new browser session via Selenium Standalone or Grid
- Automatically close browser session on drop
- All W3C WebDriver and WebElement methods supported
- Find elements (via all common selectors)
- Send keys to elements, including key-combinations
- Execute Javascript
- Action Chains
- Cookies
- Switch to frame/window/element/alert
- Alert support
- Capture / Save screenshot of browser or individual element

## Why 'thirtyfour' ?

It is named after the atomic number for the Selenium chemical element (Se).

## Examples

The following examples assume you have a selenium server running
at localhost:4444.

You can set this up using docker, as follows:

    docker run --rm -d --network host --name selenium-server -v /dev/shm:/dev/shm selenium/standalone-chrome:3.141.59-zinc

Alternatively you can download selenium from the web ([https://selenium.dev/downloads](https://selenium.dev/downloads)) and run it manually:

    java -jar selenium-server-standalone-3.141.59.jar

### Async example:

```rust
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, DesiredCapabilities, WebDriver};
use tokio;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
     let caps = DesiredCapabilities::chrome();
     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;

     // Navigate to https://wikipedia.org.
     driver.get("https://wikipedia.org").await?;
     let elem_form = driver.find_element(By::Id("search-form")).await?;

     // Find element from element.
     let elem_text = elem_form.find_element(By::Id("searchInput")).await?;

     // Type in the search terms.
     elem_text.send_keys("selenium").await?;

     // Click the search button.
     let elem_button = elem_form.find_element(By::Css("button[type='submit']")).await?;
     elem_button.click().await?;

     // Look for header to implicitly wait for the page to load.
     driver.find_element(By::ClassName("firstHeading")).await?;
     assert_eq!(driver.title().await?, "Selenium - Wikipedia");

     Ok(())
}
```

### Sync example:

```rust
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};

fn main() -> WebDriverResult<()> {
     let caps = DesiredCapabilities::chrome();
     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;

     // Navigate to https://wikipedia.org.
     driver.get("https://wikipedia.org")?;
     let elem_form = driver.find_element(By::Id("search-form"))?;

     // Find element from element.
     let elem_text = elem_form.find_element(By::Id("searchInput"))?;

     // Type in the search terms.
     elem_text.send_keys("selenium")?;

     // Click the search button.
     let elem_button = elem_form.find_element(By::Css("button[type='submit']"))?;
     elem_button.click()?;

     // Look for header to implicitly wait for the page to load.
     driver.find_element(By::ClassName("firstHeading"))?;
     assert_eq!(driver.title()?, "Selenium - Wikipedia");

     Ok(())
}
```

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
