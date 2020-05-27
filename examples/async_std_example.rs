//! Requires chromedriver running on port 4444:
//!
//!     chromedriver --port=4444
//!
//! Run as follows:
//!
//!     cargo run --example async_std_example --features async-std-runtime --no-default-features

use async_std::task;

use thirtyfour::prelude::*;

fn main() -> WebDriverResult<()> {
    task::block_on(async {
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
    })
}
