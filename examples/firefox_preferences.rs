//! Requires geckodriver running on port 4444:
//!
//!     geckodriver --port=4444
//!
//! Run as follows:
//!
//!     cargo run --example firefox_preferences

use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use thirtyfour::{FirefoxCapabilities, WebDriver};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // The use of color_eyre gives much nicer error reports, including making
    // it much easier to locate where the error occurred.
    color_eyre::install()?;

    let user_agent = "Custom";

    // Set user agent via Firefox preferences.
    let mut prefs = FirefoxPreferences::new();
    prefs.set_user_agent(user_agent.to_string())?;

    let mut caps = FirefoxCapabilities::new();
    caps.set_preferences(prefs)?;

    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    driver.goto("https://www.google.com").await?;

    // Get the user agent and verify.
    let js_user_agent: String =
        driver.execute(r#"return navigator.userAgent;"#, Vec::new()).await?.convert()?;
    assert_eq!(&js_user_agent, user_agent);

    // Always explicitly close the browser. There are no async destructors.
    driver.close_window().await?;
    Ok(())
}
