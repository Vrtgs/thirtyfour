Selenium webdriver client for Rust, inspired by the python selenium library.

Only the W3C WebDriver spec is supported (including Chrome and Firefox, possibly others).

Both synchronous and async / await APIs are provided (see examples below).

## Status
 
**This project is still WIP and has not yet been thoroughly tested.**

Any help with testing and creating tickets for any issues found would be greatly appreciated.

Most basic functionality is already implemented including:

- Creating browser session
- Automatically closing browser session on drop
- Most WebDriver and WebElement methods
- Finding elements (via all common selectors)
- Sending keys to elements, including key-combinations
- Executing Javascript
- Action Chains
- Cookies
- Saving screenshots of browser or individual elements
- Synchronous support
- Async / await support

More to come...

## Why 'thirtyfour' ?

It is named after the atomic number for the Selenium chemical element (Se).

## Examples

The following examples assume you have a selenium server running
at localhost:4444.

i.e.

```ignore
java -jar selenium-server-standalone-3.141.59.jar
```

### Async example:

```rust
use std::thread;
use std::time::Duration;
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, WebDriver, common::keys::TypingData};
use tokio;

#[tokio::main]
async fn main() {
     webtest().await.expect("Something went wrong");
}

async fn webtest() -> WebDriverResult<()> {
     let caps = serde_json::json!({
         "browserName": "chrome",
         "version": "",
         "platform": "any"
     });

     let driver = WebDriver::new("http://localhost:4444/wd/hub", caps).await?;

     // Navigate to https://wikipedia.org.
     driver.get("https://wikipedia.org").await?;
     let elem_form = driver.find_element(By::Id("search-form")).await?;

     // Find element from element.
     let elem_text = elem_form.find_element(By::Id("searchInput")).await?;

     // Type in the search terms.
     elem_text.send_keys(TypingData::from("selenium")).await?;

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
use std::thread;
use std::time::Duration;
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, sync::WebDriver, common::keys::TypingData};

fn main() {
     webtest().expect("Something went wrong");
}

fn webtest() -> WebDriverResult<()> {
     let caps = serde_json::json!({
         "browserName": "chrome",
         "version": "",
         "platform": "any"
     });

     let driver = WebDriver::new("http://localhost:4444/wd/hub", caps)?;

     // Navigate to https://wikipedia.org.
     driver.get("https://wikipedia.org")?;
     let elem_form = driver.find_element(By::Id("search-form"))?;

     // Find element from element.
     let elem_text = elem_form.find_element(By::Id("searchInput"))?;

     // Type in the search terms.
     elem_text.send_keys(TypingData::from("selenium"))?;

     // Click the search button.
     let elem_button = elem_form.find_element(By::Css("button[type='submit']"))?;
     elem_button.click()?;

     // Look for header to implicitly wait for the page to load.
     driver.find_element(By::ClassName("firstHeading"))?;
     assert_eq!(driver.title()?, "Selenium - Wikipedia");

     Ok(())
}
```
