//! Requires chromedriver running on port 4444:
//!
//!     chromedriver --port=4444
//!
//! Run as follows:
//!
//!     cargo run --example sync --features blocking

use thirtyfour::sync::prelude::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", &caps)?;

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
