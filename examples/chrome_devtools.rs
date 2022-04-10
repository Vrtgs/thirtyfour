//! Requires chromedriver running on port 4444:
//!
//!     chromedriver --port=4444
//!
//! Run as follows:
//!
//!     cargo run --example chrome_devtools

use thirtyfour::extensions::chrome::{ChromeDevTools, NetworkConditions};
use thirtyfour::prelude::*;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // The use of color_eyre gives much nicer error reports, including making
    // it much easier to locate where the error occurred.
    color_eyre::install()?;

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // Use Chrome Devtools Protocol (CDP).
    let dev_tools = ChromeDevTools::new(driver.handle.clone());
    let mut conditions = NetworkConditions::new();
    conditions.download_throughput = 20;
    conditions.upload_throughput = 10;
    dev_tools.set_network_conditions(&conditions).await?;
    let conditions = dev_tools.get_network_conditions().await?;
    assert_eq!(conditions.download_throughput, 20);
    assert_eq!(conditions.upload_throughput, 10);
    println!("Conditions: {:?}", conditions);

    // Execute CDP command.
    let version_info = dev_tools.execute_cdp("Browser.getVersion").await?;
    println!("Chrome Version: {:?}", version_info);

    // Always explicitly close the browser. There are no async destructors.
    driver.quit().await?;

    Ok(())
}
