use std::path::Path;
use std::sync::Arc;

use base64::Engine;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use super::FirefoxCommand;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use crate::error::{WebDriverError, WebDriverResult};
use crate::session::handle::SessionHandle;
use crate::upstream::CmdError;

/// Provider of Firefox-specific commands.
#[derive(Debug, Clone)]
pub struct FirefoxTools {
    /// The underlying session handle.
    pub handle: Arc<SessionHandle>,
}

impl FirefoxTools {
    /// Create a new FirefoxTools struct.
    ///
    /// # Example:
    /// ```no_run
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::extensions::addons::firefox::FirefoxTools;
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
    pub fn new(handle: Arc<SessionHandle>) -> Self {
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
            let decoded = BASE64.decode(src)?;
            Ok(decoded)
        } else {
            Err(WebDriverError::Cmd(CmdError::NotW3C(src)))
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
