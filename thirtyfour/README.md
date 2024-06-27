
# thirtyfour

Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.

## ** Unmaintained **

This crate is no longer maintained.

Feel free to create a fork. Otherwise, you could check out these related crates:

- For Selenium/WebDriver, try [fantoccini](https://github.com/jonhoo/fantoccini)
- For using Chrome Devtools Protocol (CDP) directly, try [chromiumoxide](https://github.com/mattsse/chromiumoxide)



## Features

- All W3C WebDriver and WebElement methods supported
- Create new browser session directly via WebDriver (e.g. chromedriver)
- Create new browser session via Selenium Standalone or Grid
- Find elements (via all common selectors e.g. Id, Class, CSS, Tag, XPath)
- Send keys to elements, including key-combinations
- Execute Javascript
- Action Chains
- Get and set cookies
- Switch to frame/window/element/alert
- Shadow DOM support
- Alert support
- Capture / Save screenshot of browser or individual element as PNG
- Chrome DevTools Protocol (CDP) support (limited)
- Advanced query interface including explicit waits and various predicates
- Component Wrappers (similar to `Page Object Model`)

## Feature Flags

- `rustls-tls`: (Default) Use rustls to provide TLS support (via reqwest).
- `native-tls`: Use native TLS (via reqwest).
- `component`: (Default) Enable the `Component` derive macro (via thirtyfour_macros).

## Examples

The examples assume you have `chromedriver` running on your system.

You can use Selenium (see instructions below) or you can use chromedriver 
directly by downloading the chromedriver that matches your Chrome version,
from here: [https://chromedriver.chromium.org/downloads](https://chromedriver.chromium.org/downloads)

Then run it like this:

    chromedriver

### Example (async):

To run this example:

    cargo run --example tokio_async

```rust
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
     let caps = DesiredCapabilities::chrome();
     let driver = WebDriver::new("http://localhost:9515", caps).await?;

     // Navigate to https://wikipedia.org.
     driver.goto("https://wikipedia.org").await?;
     let elem_form = driver.find(By::Id("search-form")).await?;

     // Find element from element.
     let elem_text = elem_form.find(By::Id("searchInput")).await?;

     // Type in the search terms.
     elem_text.send_keys("selenium").await?;

     // Click the search button.
     let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
     elem_button.click().await?;

     // Look for header to implicitly wait for the page to load.
     driver.find(By::ClassName("firstHeading")).await?;
     assert_eq!(driver.title().await?, "Selenium - Wikipedia");
    
     // Always explicitly close the browser.
     driver.quit().await?;

     Ok(())
}
```

## Minimum Supported Rust Version

The MSRV for `thirtyfour` is currently 1.75 and will be updated as needed by dependencies.

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

See the NOTICE file for more details.

`SPDX-License-Identifier: MIT OR Apache-2.0`
