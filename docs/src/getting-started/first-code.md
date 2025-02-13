## Writing Your First Browser Automation Code

Before we begin, you'll need to install Rust. You can do so by using the [rustup](https://rustup.rs/) tool.

Let's start a new project. Open your terminal application and navigate to the directory 
where you usually put your source code. Then run these commands:

    cargo new --bin my-automation-project
    cd my-automation-project

You will see a `Cargo.toml` file and a `src/` directory there already.

First, let's edit the `Cargo.toml` file in your editor (e.g. Visual Studio Code) and add some dependencies:

    [dependencies]
    thirtyfour = "THIRTYFOUR_CRATE_VERSION"
    tokio = { version = "1", features = ["full"] }

Great! Now let's open `src/main.rs` and  add the following code.

> **NOTE:** Make sure you remove any existing code from `main.rs`.

Don't worry, we'll go through what it does soon.

> */src/main.rs*
```rust
use std::error::Error;

use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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
    driver.query(By::ClassName("firstHeading")).first().await?;
    assert_eq!(driver.title().await?, "Selenium – Wikipedia");

    // Always explicitly close the browser.
    driver.quit().await?;

    Ok(())
}
```

Next we need to make sure our webdriver is running.

Open your terminal application and run:

    chromedriver

> **NOTE:** This tutorial is currently set up for Chrome.
> If you'd prefer to run on Firefox instead, this is explained below.

Now open a new tab in your terminal and run your code:

    cargo run

If everything worked correctly you should have seen a Chrome browser window open up,
navigate to the "Selenium" article on Wikipedia, and then close again.

## Running on Firefox

To run the code using Firefox instead, we first need to tell `thirtyfour` to use the 
configuration for Firefox. To do this, change the first to lines of your `main` function to this:

```rust
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;
```

Now, instead of running `chromedriver` in your terminal, we'll run `geckodriver` instead:

    geckodriver

And again, in the other tab, run your code again:

    cargo run

If everything worked correctly, you should have seen the Wikipedia page open up on Firefox this time.

Congratulations! You successfully automated a web browser.
