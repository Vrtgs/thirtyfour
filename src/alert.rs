use crate::error::WebDriverResult;
use crate::session::handle::SessionHandle;

/// Struct for managing alerts.
pub struct Alert {
    handle: SessionHandle,
}

impl Alert {
    /// Create a new Alert struct. This is typically created internally
    /// via a call to `WebDriver::switch_to().alert()`.
    pub fn new(handle: SessionHandle) -> Self {
        Self {
            handle,
        }
    }

    /// Get the active alert text.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #         driver.find_element(By::Id("alertbutton1")).await?.click().await?;
    /// let alert = driver.switch_to().alert();
    /// let text = alert.text().await?;
    /// #         assert_eq!(text, "Alert 1 showing");
    /// #         alert.dismiss().await?;
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn text(&self) -> WebDriverResult<String> {
        Ok(self.handle.client.get_alert_text().await?)
    }

    /// Dismiss the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #         driver.find_element(By::Id("alertbutton2")).await?.click().await?;
    /// driver.switch_to().alert().dismiss().await?;
    /// #         let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #         assert_eq!(elem.text().await?, "Alert 2 clicked false");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn dismiss(&self) -> WebDriverResult<()> {
        self.handle.client.dismiss_alert().await?;
        Ok(())
    }

    /// Accept the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #         driver.find_element(By::Id("alertbutton2")).await?.click().await?;
    /// driver.switch_to().alert().accept().await?;
    /// #         let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #         assert_eq!(elem.text().await?, "Alert 2 clicked true");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn accept(&self) -> WebDriverResult<()> {
        self.handle.client.accept_alert().await?;
        Ok(())
    }

    /// Send the specified keys to the active alert.
    ///
    /// # Example:
    /// You can specify anything that implements `Into<TypingData>`. This
    /// includes &str and String.
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #         driver.find_element(By::Id("alertbutton3")).await?.click().await?;
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys("selenium").await?;
    /// alert.accept().await?;
    /// #         let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #         assert_eq!(elem.text().await?, "selenium");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    ///
    /// You can also send special key combinations like this:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444", caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagealerts")).await?.click().await?;
    /// #         driver.find_element(By::Id("alertbutton3")).await?.click().await?;
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys("selenium").await?;
    /// alert.send_keys(Key::Control + "a".to_string()).await?;
    /// alert.send_keys("thirtyfour").await?;
    /// #         alert.accept().await?;
    /// #         let elem = driver.find_element(By::Id("alert-result")).await?;
    /// #         assert_eq!(elem.text().await?, "thirtyfour");
    /// #         driver.quit().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn send_keys(&self, keys: impl AsRef<str>) -> WebDriverResult<()> {
        self.handle.client.send_alert_text(keys.as_ref()).await?;
        Ok(())
    }
}
