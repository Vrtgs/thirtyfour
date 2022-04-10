use crate::error::WebDriverResult;
use crate::extensions::firefox::FirefoxCommand;
use crate::session::handle::SessionHandle;

#[derive(Clone)]
pub struct FirefoxTools {
    pub handle: SessionHandle,
}

impl FirefoxTools {
    /// Create a new FirefoxTools struct.
    ///
    /// # Example:
    /// ```
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::extensions::firefox::FirefoxTools;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         // NOTE: this only tests creation of FirefoxTools, so chrome is fine.
    /// #         //       The test environment currently doesn't support firefox.
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// let tools = FirefoxTools::new(driver.handle.clone());
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn new(handle: SessionHandle) -> Self {
        Self {
            handle,
        }
    }

    /// Install the specified firefox add-on.
    pub async fn install_addon(&self, path: &str, temporary: Option<bool>) -> WebDriverResult<()> {
        self.handle
            .client
            .issue_cmd(FirefoxCommand::InstallAddon {
                path: path.to_string(),
                temporary,
            })
            .await?;
        Ok(())
    }
}
