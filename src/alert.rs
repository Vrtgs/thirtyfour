use crate::webdrivercommands::{WebDriverCommands, WebDriverSession};
use crate::{
    common::{command::Command, connection_common::convert_json, keys::TypingData},
    error::WebDriverResult,
};

/// Struct for managing alerts.
pub struct Alert<'a> {
    driver: WebDriverSession<'a>,
}

impl<'a> Alert<'a> {
    /// Create a new Alert struct. This is typically created internally
    /// via a call to `WebDriver::switch_to().alert()`.
    pub fn new(driver: WebDriverSession<'a>) -> Self {
        Alert {
            driver,
        }
    }

    ///Convenience wrapper for executing a WebDriver command.
    async fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.driver.cmd(command).await
    }

    /// Get the active alert text.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #     driver.find_element(By::Id("alertbutton1")).await?.click().await?;
    /// let alert = driver.switch_to().alert();
    /// let text = alert.text().await?;
    /// #     assert_eq!(text, "Alert 1 showing");
    /// #     alert.dismiss().await?;
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn text(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetAlertText).await?;
        convert_json::<String>(&v["value"])
    }

    /// Dismiss the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #     driver.find_element(By::Id("alertbutton2")).await?.click().await?;
    /// driver.switch_to().alert().dismiss().await?;
    /// #     let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #     assert_eq!(elem.text().await?, "Alert 2 clicked false");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn dismiss(&self) -> WebDriverResult<()> {
        self.cmd(Command::DismissAlert).await.map(|_| ())
    }

    /// Accept the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #     driver.find_element(By::Id("alertbutton2")).await?.click().await?;
    /// driver.switch_to().alert().accept().await?;
    /// #     let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #     assert_eq!(elem.text().await?, "Alert 2 clicked true");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn accept(&self) -> WebDriverResult<()> {
        self.cmd(Command::AcceptAlert).await.map(|_| ())
    }

    /// Send the specified keys to the active alert.
    ///
    /// # Example:
    /// You can specify anything that implements `Into<TypingData>`. This
    /// includes &str and String.
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #     driver.find_element(By::Id("alertbutton3")).await?.click().await?;
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys("selenium").await?;
    /// alert.accept().await?;
    /// #     let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #     assert_eq!(elem.text().await?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// You can also send special key combinations like this:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use tokio;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #     driver.get("http://webappdemo").await?;
    /// #     driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #     driver.find_element(By::Id("alertbutton3")).await?.click().await?;
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys("selenium").await?;
    /// alert.send_keys(Keys::Control + "a").await?;
    /// alert.send_keys("thirtyfour").await?;
    /// #     alert.accept().await?;
    /// #     let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #     assert_eq!(elem.text().await?, "thirtyfour");
    /// #     Ok(())
    /// # }
    /// ```
    pub async fn send_keys<S>(&self, keys: S) -> WebDriverResult<()>
    where
        S: Into<TypingData>,
    {
        self.cmd(Command::SendAlertText(keys.into())).await.map(|_| ())
    }
}
