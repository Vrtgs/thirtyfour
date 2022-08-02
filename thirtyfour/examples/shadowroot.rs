//! Requires chromedriver running on port 9515:
//!
//!     chromedriver --port=9515
//!
//! Run as follows:
//!
//!     cargo run --example shadowroot

use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    std::env::set_var("RUST_BACKTRACE", "1");

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;

    // Navigate to website containing example shadowroot.
    driver.goto("https://web.dev/shadowdom-v1/").await?;

    let elem = driver.query(By::Tag("iframe")).first().await?;
    elem.enter_frame().await?;

    // Get the element containing the shadow root node.
    let elem = driver.query(By::Tag("fancy-tabs")).first().await?;
    // Now get the shadow root node itself.
    let root = elem.get_shadow_root().await?;

    // Now we can search for elements nested below the shadow root node.
    let tabs = root.query(By::Id("tabsSlot")).first().await?;
    let name = tabs.prop("name").await?;
    assert!(name.is_some());
    assert_eq!(name.unwrap(), "title");

    // Always explicitly close the browser. There are no async destructors.
    driver.quit().await?;

    Ok(())
}
