use serde_json::{json, Value};

use crate::common::connection_common::convert_json;
use crate::error::WebDriverResult;
use crate::extensions::chrome::networkconditions::NetworkConditions;
use crate::extensions::chrome::ChromeCommand;
use crate::WebDriverSession;

/// The ChromeDevTools struct allows you to interact with Chromium-based browsers via
/// the Chrome Devtools Protocol (CDP).
///
/// You can find documentation for the available commands here:
/// [https://chromedevtools.github.io/devtools-protocol/](https://chromedevtools.github.io/devtools-protocol/])
///
/// # Example
/// ```rust
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// # use thirtyfour::extensions::chrome::ChromeDevTools;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// let caps = DesiredCapabilities::chrome();
/// let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
///
/// // Create a ChromeDevTools struct like this.
/// let dev_tools = ChromeDevTools::new(driver.session());
/// dev_tools.execute_cdp("Network.clearBrowserCache").await?;
/// #         Ok(())
/// #     })
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ChromeDevTools<'a> {
    pub session: &'a WebDriverSession,
}

impl<'a> ChromeDevTools<'a> {
    /// Create a new ChromeDevTools struct.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::extensions::chrome::ChromeDevTools;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// let dev_tools = ChromeDevTools::new(driver.session());
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub fn new(session: &'a WebDriverSession) -> Self {
        Self {
            session,
        }
    }

    /// Convenience method to execute a ChromeCommand.
    async fn cmd(&self, command: ChromeCommand) -> WebDriverResult<serde_json::Value> {
        self.session.execute(Box::new(command)).await
    }

    /// Launch the Chrome app with the specified id.
    pub async fn launch_app(&self, app_id: &str) -> WebDriverResult<()> {
        self.cmd(ChromeCommand::LaunchApp(app_id.to_string())).await?;
        Ok(())
    }

    /// Get the current network conditions. You must set the conditions first.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::extensions::chrome::{ChromeDevTools, NetworkConditions};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// // Create ChromeDevTools struct.
    /// let dev_tools = ChromeDevTools::new(driver.session());
    ///
    /// // First we need to set the network conditions.
    /// let mut conditions = NetworkConditions::new();
    /// conditions.download_throughput = 20;
    /// dev_tools.set_network_conditions(&conditions).await?;
    ///
    /// // Now we can get the network conditions.
    /// let conditions_out = dev_tools.get_network_conditions().await?;
    /// assert_eq!(conditions_out.download_throughput, conditions.download_throughput);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_network_conditions(&self) -> WebDriverResult<NetworkConditions> {
        let v = self.cmd(ChromeCommand::GetNetworkConditions).await?;
        convert_json(&v["value"])
    }

    /// Set the network conditions.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::extensions::chrome::{ChromeDevTools, NetworkConditions};
    ///
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// // Create ChromeDevTools struct.
    /// let dev_tools = ChromeDevTools::new(driver.session());
    ///
    /// // Now we can set the network conditions. You do not need to set all parameters.
    /// let mut conditions = NetworkConditions::new();
    /// conditions.download_throughput = 20;
    /// conditions.upload_throughput = 10;
    /// conditions.offline = false;
    /// conditions.latency = 200;
    ///
    /// dev_tools.set_network_conditions(&conditions).await?;
    /// #         let conditions_out = dev_tools.get_network_conditions().await?;
    /// #         assert_eq!(conditions_out.download_throughput, conditions.download_throughput);
    /// #         assert_eq!(conditions_out.upload_throughput, conditions.upload_throughput);
    /// #         assert_eq!(conditions_out.latency, conditions.latency);
    /// #         assert_eq!(conditions_out.offline, conditions.offline);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn set_network_conditions(
        &self,
        conditions: &NetworkConditions,
    ) -> WebDriverResult<()> {
        self.cmd(ChromeCommand::SetNetworkConditions(conditions.clone())).await?;
        Ok(())
    }

    /// Execute the specified command without parameters.
    /// For commands that require parameters, use `execute_cdp_with_params()` instead.
    ///
    /// You can find documentation for the available commands here:
    /// [https://chromedevtools.github.io/devtools-protocol/](https://chromedevtools.github.io/devtools-protocol/])
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::extensions::chrome::ChromeDevTools;
    ///
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// let dev_tools = ChromeDevTools::new(driver.session());
    /// dev_tools.execute_cdp("Network.clearBrowserCache").await?;
    ///
    /// // execute_cdp() can also return values as well.
    /// let version_info = dev_tools.execute_cdp("Browser.getVersion").await?;
    /// let user_agent = version_info["userAgent"].as_str().unwrap();
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute_cdp(&self, cmd: &str) -> WebDriverResult<Value> {
        self.execute_cdp_with_params(cmd, json!({})).await
    }

    /// Execute the specified command with the specified parameter(s).
    ///
    /// You can find documentation for the available commands here:
    /// [https://chromedevtools.github.io/devtools-protocol/](https://chromedevtools.github.io/devtools-protocol/])
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// use thirtyfour::extensions::chrome::ChromeDevTools;
    /// use serde_json::json;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// let dev_tools = ChromeDevTools::new(driver.session());
    /// dev_tools.execute_cdp_with_params("Network.setCacheDisabled", json!({"cacheDisabled": true})).await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn execute_cdp_with_params(
        &self,
        cmd: &str,
        cmd_args: Value,
    ) -> WebDriverResult<Value> {
        let v = self.cmd(ChromeCommand::ExecuteCdpCommand(cmd.to_string(), cmd_args)).await?;
        Ok(v["value"].clone())
    }

    /// Get the list of sinks available for cast.
    pub async fn get_sinks(&self) -> WebDriverResult<Value> {
        let v = self.cmd(ChromeCommand::GetSinks).await?;
        Ok(v["value"].clone())
    }

    /// Get the issue message for any issue in a cast session.
    pub async fn get_issue_message(&self) -> WebDriverResult<Value> {
        let v = self.cmd(ChromeCommand::GetIssueMessage).await?;
        Ok(v["value"].clone())
    }

    /// Set the specified sink as the cast session receiver target.
    pub async fn set_sink_to_use(&self, sink_name: &str) -> WebDriverResult<()> {
        self.cmd(ChromeCommand::SetSinkToUse(sink_name.to_string())).await?;
        Ok(())
    }

    /// Start a tab mirroring session on the specified receiver target.
    pub async fn start_tab_mirroring(&self, sink_name: &str) -> WebDriverResult<()> {
        self.cmd(ChromeCommand::StartTabMirroring(sink_name.to_string())).await?;
        Ok(())
    }

    /// Stop the existing cast session on the specified receiver target.
    pub async fn stop_casting(&self, sink_name: &str) -> WebDriverResult<()> {
        self.cmd(ChromeCommand::StopCasting(sink_name.to_string())).await?;
        Ok(())
    }
}
