# thirtyfour

[![Crates.io](https://img.shields.io/crates/v/thirtyfour.svg)](https://crates.io/crates/thirtyfour)
[![docs.rs](https://docs.rs/thirtyfour/badge.svg)](https://docs.rs/thirtyfour)
[![Build Status](https://img.shields.io/github/workflow/status/stevepryde/thirtyfour/build-check/main)](https://github.com/stevepryde/thirtyfour/actions)
[![codecov](https://codecov.io/gh/stevepryde/thirtyfour/branch/main/graph/badge.svg?token=OVORQE9PZK)](https://codecov.io/gh/stevepryde/thirtyfour)

Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.

It supports the full [W3C WebDriver spec](https://www.w3.org/TR/webdriver1/). Tested with Chrome and Firefox although any W3C-compatible WebDriver should work.

## Why "thirtyfour" ?

34 is the atomic number for the Selenium chemical element (Se).

## Built on top of fantoccini

The thirtyfour crate uses [fantoccini](https://docs.rs/fantoccini/latest/fantoccini/) as the backend
for interacting with the underlying WebDriver (chromedriver, geckodriver, etc). `Fantoccini` aims 
to stick fairly close to the WebDriver specification, whereas `thirtyfour` builds on top of that
foundation and adds several high-level features as well as exploring ways to improve the 
ergonomics of browser automation in Rust.

## Important changes in v0.30.0

A number of methods in the `thirtyfour` API have been renamed to more closely align 
with `fantoccini`, as part of the move towards greater compatibility.
The existing method names remain but have been deprecated.

The deprecated methods will remain for the time being, however you should 
aim to migrate code away from the deprecated methods as soon as is practical.

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

- `rustls-tls`: (Default) Use rustls to provide TLS support (via fantoccini/hyper).
- `native-tls`: Use native TLS (via fantoccini/hyper).
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

### The browser will not close automatically

Rust does not have [async destructors](https://boats.gitlab.io/blog/post/poll-drop/),
which means there is no reliable way to execute an async HTTP request on Drop and wait for
it to complete. This means you are in charge of closing the browser at the end of your code,
via a call to `WebDriver::quit()` as in the above example.

If you do not call `WebDriver::quit()` then the browser will stay open until it is 
either explicitly closed later outside your code, or the session times out.

### Advanced element queries

#### ElementQuery

The `WebDriver::query()` and `WebElement::query()` methods return an `ElementQuery` struct.

Using `ElementQuery`, you can do things like:

```rust
let elem_text =
    driver.query(By::Css("match.this")).or(By::Id("orThis")).first().await?;
```

This will execute both queries once per poll iteration and return the first one that matches.
You can also filter on one or both query branches like this:

```rust
driver.query(By::Css("branch.one")).with_text("testing")
    .or(By::Id("branchTwo")).with_class("search").and_not_enabled()
    .first().await?;
```

The `all()` method will return an empty Vec if no elements were found.
In order to return an error in this scenario, use the `all_required()` method instead.

`ElementQuery` also allows the use of custom predicates that take a `&WebElement` argument
and return a `WebDriverResult<bool>`.

As noted above, the `query()` method is also available on `WebElement` structs as well for querying elements
in relation to a particular element in the DOM.

#### ElementWaiter

The `WebElement::wait_until()` method returns an `ElementWaiter` struct.

Using `ElementWaiter` you can do things like this:

```rust
elem.wait_until().displayed().await?;
// You can optionally provide a nicer error message like this.
elem.wait_until().error("Timed out waiting for element to disappear").not_displayed().await?;

elem.wait_until().enabled().await?;
elem.wait_until().clickable().await?;
```

And so on. See the `ElementWaiter` docs for the full list of predicates available.

`ElementWaiter` also allows the use of custom predicates that take a `&WebElement` argument
and return a `WebDriverResult<bool>`.

A range of pre-defined predicates are also supplied for convenience in the
`thirtyfour::query::conditions` module.

```rust
use thirtyfour::query::conditions;

elem.wait_until().conditions(vec![
    conditions::element_is_displayed(true),
    conditions::element_is_clickable(true)
]).await?;
```

These predicates (or your own) can also be supplied as filters to `ElementQuery`.

### Components

Version 0.31.0 introduces `Component`, a derive macro and mechanisms for wrapping web components.

This approach may seem familiar to anyone who has used a 
[Page Object Model](https://www.selenium.dev/documentation/test_practices/encouraged/page_object_models/) before. 
However a `Component` can wrap any node in the DOM, not just "pages".

It uses smart element resolvers that can lazily resolve elements within the component and cache them for subsequent 
uses. You can also nest components, making them an extremely powerful feature for automating any modern web app.

#### Example

Given the following HTML structure:

```html
<div id="checkbox-section">
    <label>
        <input type="checkbox" id="checkbox-option-1" />
        Option 1
    </label>

    <label>
        <input type="checkbox" id="checkbox-disabled" disabled />
        Option 2
    </label>

    <label>
        <input type="checkbox" id="checkbox-hidden" style="display: none;" />
        Option 3
    </label>
</div>
```

```rust
/// This component shows how to wrap a simple web component.
#[derive(Debug, Clone, Component)]
pub struct CheckboxComponent {
    base: WebElement, // This is the <label> element
    #[by(css = "input[type='checkbox']", first)]
    input: ElementResolver<WebElement>, // This is the <input /> element
}

impl CheckboxComponent {
    /// Return true if the checkbox is ticked.
    pub async fn is_ticked(&self) -> WebDriverResult<bool> {
        let elem = self.input.resolve().await?;
        let prop = elem.prop("checked").await?;
        Ok(prop.unwrap_or_default() == "true")
    }

    /// Tick the checkbox if it is clickable and isn't already ticked.
    pub async fn tick(&self) -> WebDriverResult<()> {
        // This checks that the element is present before returning the element.
        // If the element had become stale, this would implicitly re-query the element.
        let elem = self.input.resolve_present().await?;
        if elem.is_clickable().await? && !self.is_ticked().await? {
            elem.click().await?;
            // Now make sure it's ticked.
            assert!(self.is_ticked().await?);
        }

        Ok(())
    }
}

/// This component shows how to nest components inside others.
#[derive(Debug, Clone, Component)]
pub struct CheckboxSectionComponent {
    base: WebElement, // This is the outer <div>
    #[by(tag = "label", allow_empty)]
    boxes: ElementResolver<Vec<CheckboxComponent>>, // ElementResolver works with Components too.
    // Other fields will be initialised with Default::default().
    my_field: bool,
}
```

So how do you construct a Component?

Simple. The `Component` derive automatically implements `From<WebElement>`.

```rust
let elem = driver.query(By::Id("checkbox-section")).await?;
let component = CheckboxSectionComponent::from(elem);

// Now you can get the checkbox components easily like this.
let checkboxes = component.boxes.resolve().await?;
for checkbox in checkboxes {
    checkbox.tick().await?;
}
```

This allows you to wrap any component using `ElementResolver` to resolve elements and nested components easily.

See the documentation on the `Component` derive macro for more details.

## Running against selenium

**NOTE:** To run the selenium example, start selenium server and then run:

    cargo run --example selenium_example

Below you can find my recommended development environment for running selenium tests.

Essentially you need 3 main components as a minimum:

1. Selenium standalone running on some server, usually localhost at port 4444.

    For example, `http://localhost:4444`

2. The webdriver for your browser somewhere in your PATH, e.g. chromedriver (Chrome) or geckodriver (Firefox)
3. Your code, that imports this library

If you want you can download selenium and the webdriver manually, copy the webdriver
to somewhere in your path, then run selenium manually using `java -jar selenium.jar`.

However, this is a lot of messing around and you'll need to do it all again any
time either selenium or the webdriver gets updated. A better solution is to run
both selenium and webdriver in a docker container, following the instructions below.

### Setting up Docker and Selenium

To install docker, see [https://docs.docker.com/install/](https://docs.docker.com/install/) (follow the SERVER section if you're on Linux, then look for the Community Edition)

Once you have docker installed, you can start the selenium server, as follows:

    docker run --rm -d -p 4444:4444 -p 5900:5900 --name selenium-server -v /dev/shm:/dev/shm selenium/standalone-chrome:4.1.0-20211123

For more information on running selenium in docker, visit
[docker-selenium](https://github.com/SeleniumHQ/docker-selenium)

## Running the tests for `thirtyfour`

You generally only need to run the tests if you plan on contributing to the development of `thirtyfour`. If you just want to use the crate in your own project, you can skip this section.

Make sure selenium is not still running (or anything else that might use port 4444 or port 9515).

To run the tests, you need to have an instance of `geckodriver` and an instance of `chromedriver` running in the background, perhaps in separate tabs in your terminal.

Download links for these are here:

* chromedriver: https://chromedriver.chromium.org/downloads
* geckodriver: https://github.com/mozilla/geckodriver/releases

In separate terminal tabs, run the following:

* Tab 1:

      chromedriver

* Tab 2:

      geckodriver

* Tab 3 (navigate to the root of this repository):

      cargo test

## Minimum Supported Rust Version

The MSRV for `thirtyfour` is 1.57.

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

See the NOTICE file for more details.

`SPDX-License-Identifier: MIT OR Apache-2.0`
