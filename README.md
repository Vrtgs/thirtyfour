[![Crates.io](https://img.shields.io/crates/v/thirtyfour.svg?style=for-the-badge)](https://crates.io/crates/thirtyfour)
[![docs.rs](https://img.shields.io/badge/docs.rs-thirtyfour-blue?style=for-the-badge)](https://docs.rs/thirtyfour)
[![Build Status](https://img.shields.io/github/workflow/status/stevepryde/thirtyfour/build-check/main?style=for-the-badge)](https://github.com/stevepryde/thirtyfour/actions)

Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.

It supports the full W3C WebDriver spec. Tested with Chrome and Firefox although any W3C-compatible WebDriver should work.

## UPDATE ANNOUNCEMENT - v0.29.0

The `thirtyfour` crate has switched to `fantoccini` as the `WebDriver` client backend 
from Version 0.29 and onwards, with the goal of reducing duplication of effort and 
creating a more stable ecosystem around Web Browser automation in Rust.

The update aims to maintain broad API compatibility with previous versions, however there 
are some breaking changes (see the section on breaking changes for v0.29.x below).

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
- Chrome DevTools Protocol (CDP) support
- Advanced query interface including explicit waits and various predicates

## Why 'thirtyfour' ?

It is named after the atomic number for the Selenium chemical element (Se).

## Feature Flags

- `rusttls-tls`: (Default) Use rusttls to provide TLS support (via fantoccini/hyper).
- `native-tls`: Use native TLS (via fantoccini/hyper).

## Examples

The examples assume you have a WebDriver running at localhost:4444.

You can use Selenium (see instructions below) or you can use chromedriver 
directly by downloading the chromedriver that matches your Chrome version,
from here: [https://chromedriver.chromium.org/downloads](https://chromedriver.chromium.org/downloads)

Then run it like this:

    chromedriver --port=4444

### Example (async):

To run this example:

    cargo run --example tokio_async

```rust
use thirtyfour::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
     let caps = DesiredCapabilities::chrome();
     let driver = WebDriver::new("http://localhost:4444", caps).await?;

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
    
     // Always explicitly close the browser. There are no async destructors.
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

## Breaking changes in v0.29 

- Tokio is now the only supported async runtime (via `fantoccini`)
- `WebDriver` and all `WebElement` instances no longer contain a reference to the underlying `WebDriverSession`.
  This makes some things easier since you no longer need to worry about element lifetimes.
  However it also introduces the potential for your code to issue `WebElement::*` commands after the browser has
  been closed, which would lead to runtime errors.
- `WebDriver::new()` now takes ownership of `DesiredCapabilities` rather than taking a reference to it.
- WebDriver timeouts are currently not supported. `WebDriver::new_with_timeout()` will still create a `WebDriver`
  instance, however the timeouts will not take effect.
- The `WebDriverError` enum now mostly wraps `fantoccini::CmdError` with some variants split out into top-level
  variants for ease-of-use
- `TypingData` has been removed. You can now just use `&str` and/or the `Key` enum directly
- Cookie support is now provided by `cookie-rs`, for compatibility with `fantoccini`
- `ScriptArgs` has been removed. You can provide arguments to scripts as `Vec<serde_json::Value>`. 
  Use `WebElement::to_json()?` to convert a `WebElement` to a `serde_json::Value`.
- Likewise, `WebDriver::execute_script_with_args()` has been removed. `WebDriver::execute_script()` now requires
  a `Vec<serde_json::Value>`, for which you can specify `Vec::new()` or `vec![]` if your script does not require args.
- `WebDriver::execute_async_script*()` has been renamed to `WebDriver::execute_script_async()` and requires a
  `Vec<serde_json::Value>` similar to `WebDriver::execute_script()`.
- `WebElement::screenshot_as_base64()` has been removed. Use `WebElement::screenshot_as_png()` to get the PNG data.
- `WebDriver::extension_command()` has been removed. Extension commands can be executed by implementing 
  `WebDriverCompatibleCommand` and passing it to `WebDriver::handle.client.issue_cmd()`
  (see `ChromeCommand` and `FirefoxCommand` for example of how to do this)

If there are other changes I've missed that should be on this list, please let me know and I'll add them.

## Running against selenium

**NOTE:** To run the selenium example, start selenium (instructions below) then run:

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

    docker run --rm -d -p 4444:4444 -p 5900:5900 --name selenium-server -v /dev/shm:/dev/shm selenium/standalone-chrome-debug:3.141.59-zinc

We will use the `-debug` container because it starts a VNC server, allowing us to view the browser session in real time.
To connect to the VNC server you will need to use a VNC viewer application. Point it at `localhost:5900` (the default password is `secret`).
If you don't want to use a VNC viewer you don't have to, but then you won't see what is happening when you run the examples.

If you want to run on Firefox instead, or customize the browser setup, see
[docker-selenium](https://github.com/SeleniumHQ/docker-selenium) on GitHub for more options

## Running the tests for `thirtyfour`, including doctests

You generally only need to run the tests if you plan on contributing to the development of `thirtyfour`. If you just want to use the crate in your own project, you can skip this section.

Just like the examples above, the tests in this crate require a running instance of Selenium at `http://localhost:4444`.

If you still have the docker container running from the examples above, you will need to bring it down (otherwise the next step will complain that port 4444 is already in use):

    docker stop selenium-server 

The tests also require a small web app called `webappdemo` that was purpose-built for testing the `thirtyfour` crate.

Both can be run easily using docker-compose. To install docker-compose, see [https://docs.docker.com/compose/install/](https://docs.docker.com/compose/install/)

Once you have docker-compose installed, you can start the required containers, as follows:

    docker-compose up -d

Then, to run the tests:

    cargo test -- --test-threads=1

We need to limit the tests to a single thread because the selenium server only supports 1 browser instance at a time.
(You can increase this limit in the `docker-compose.yml` file if you want. Remember to restart the containers afterwards)

If you need to restart the docker containers:

    docker-compose restart 

And finally, to remove them:

    docker-compose down

## LICENSE

This work is dual-licensed under MIT or Apache 2.0.
You can choose either license if you use this work.

See the NOTICE file for more details.

`SPDX-License-Identifier: MIT OR Apache-2.0`
