[![Crates.io](https://img.shields.io/crates/v/thirtyfour.svg?style=for-the-badge)](https://crates.io/crates/thirtyfour)
[![docs.rs](https://img.shields.io/badge/docs.rs-thirtyfour-blue?style=for-the-badge)](https://docs.rs/thirtyfour)
[![Build Status](https://img.shields.io/github/workflow/status/stevepryde/thirtyfour/build-check/main?style=for-the-badge)](https://github.com/stevepryde/thirtyfour/actions)

Thirtyfour is a Selenium / WebDriver library for Rust, for automated website UI testing.

It supports the full W3C WebDriver spec. Tested with Chrome and Firefox although any W3C-compatible WebDriver should work.

Async only (`tokio` and `async-std` runtimes supported via feature flags).
For synchronous support, use the [thirtyfour_sync](https://docs.rs/thirtyfour_sync) crate instead.

## Features

- All W3C WebDriver and WebElement methods supported
- Async / await support (both **tokio** and **async-std** runtimes supported via feature flags)
- Create new browser session directly via WebDriver (e.g. chromedriver)
- Create new browser session via Selenium Standalone or Grid
- Automatically close browser session on drop
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

## Why 'thirtyfour' ?

It is named after the atomic number for the Selenium chemical element (Se).

## Feature Flags

- `tokio-runtime`: (Default) Use the **tokio** async runtime with the [reqwest](https://docs.rs/reqwest) http client.
- `async-std-runtime`: Use the **async-std** runtime with the [surf](https://docs.rs/surf) http client.

  **NOTE**: You cannot combine `async-std-runtime` with `tokio-runtime`

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
     let driver = WebDriver::new("http://localhost:4444", &caps).await?;

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

### Advanced element queries

See the (experimental) [thirtyfour_query](https://github.com/stevepryde/thirtyfour_query) crate for 
more advanced element queries and polling options.

## Running against selenium

*NOTE:* To run the selenium example, start selenium (instructions below) then run:

    cargo run --example selenium_example

Below you can find my recommended development environment for running selenium tests.

Essentially you need 3 main components as a minimum:

1. Selenium standalone running on some server, usually localhost at port 4444.

    When using selenium, remember to include `/wd/hub` after the server address.
    For example, when running against localhost:4444, your selenium url will be `http://localhost:4444/wd/hub`

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

### Choosing between Sync and Async

The `thirtyfour` crate provides an async API whereas `thirtyfour_sync` provides sync. Which one you should use really depends on your personal preference and the nature of your application.

For a more in-depth introduction to async programming in rust, see [The Rust Async Book](https://rust-lang.github.io/async-book/01_getting_started/01_chapter.html).

All interactions with selenium involve sending requests to the selenium server, and then waiting for a response.
The difference between sync and async for `thirtyfour` essentially comes down to what actually happens (or doesn't happen) while waiting for that response.

With the synchronous version, requests to selenium will block the calling thread until they return. While this is obviously inefficient, the code is often a little bit simpler and easier to follow.
You can alleviate some of the inefficiency by using multiple threads if/when concurrency is required, but note that threads have some overhead as well. If you're thinking of spawning hundreds (or even tens) of threads, you probably want to use async instead.

In contrast, async method calls will immediately return a "future", which is a bit like a task that can be executed later (typically via `.await`), and will eventually complete and return a value.
While it is `.await`ing a response from selenium, the "future" will yield control of the current thread, allowing other "futures" to run.
This allows multiple operations to be performed concurrently on the same thread, rather than just sitting and blocking the thread completely for each call to selenium.
For more information on running multiple tasks concurrently, see chapter 6 of [The Rust Async Book](https://rust-lang.github.io/async-book/06_multiple_futures/01_chapter.html).

In short, if you need to do a lot of I/O (network requests, reading/writing files, handling network connections), and especially if you want to do I/O operations concurrently, choose async. If not, it's really up to your personal preference.

## WebDriverSession Lifetime

All WebElement structs share a reference to the WebDriverSession which 
provdes a compile-time guarantee that no element or alert struct (for example)
will outlast the browser session. This should prevent issues where something 
attempts to send a command to a browser session that has already been closed.

Note also that the WebDriver struct will attempt to close the
session / browser on Drop, and hence this struct cannot be cloned.
There are ways to allow cloning, for example, by making sure none of
the clones will attempt to close the session on Drop. I'm not yet
convinced this is a requirement. Please raise an issue if you need this
functionality, and explain your use case.

By embedding a reference to the actual WebDriverSession inside each
struct such as WebElement, this also enables things such as easily adding
WebElement methods that run JavaScript internally.

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

`SPDX-License-Identifier: MIT OR Apache-2.0`
