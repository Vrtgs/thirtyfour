use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use fantoccini::error::CmdError;

use crate::error::{WebDriverError, WebDriverResult};
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
    /// ```no_run
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

    /// Take a full-page screenshot of the current window and return it as PNG bytes.
    pub async fn full_screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let src = self.handle.client.issue_cmd(FirefoxCommand::FullScreenshot {}).await?;
        if let Some(src) = src.as_str() {
            base64::decode(src).map_err(|x| WebDriverError::CmdError(CmdError::ImageDecodeError(x)))
        } else {
            Err(WebDriverError::CmdError(CmdError::NotW3C(src)))
        }
    }

    /// Take a full-page screenshot of the current window and write it to the specified filename.
    pub async fn full_screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.full_screenshot_as_png().await?;
        let mut file = File::create(path).await?;
        file.write_all(&png).await?;
        Ok(())
    }
}
