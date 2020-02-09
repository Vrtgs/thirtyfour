use std::sync::Arc;

use crate::{
    common::{command::Command, connection_common::unwrap, keys::TypingData},
    error::WebDriverResult,
    RemoteConnectionAsync, SessionId,
};

/// Struct for managing alerts.
pub struct Alert {
    session_id: SessionId,
    conn: Arc<RemoteConnectionAsync>,
}

impl Alert {
    /// Create a new Alert struct. This is typically created internally
    /// via a call to `WebDriver::switch_to().alert()`.
    pub fn new(session_id: SessionId, conn: Arc<RemoteConnectionAsync>) -> Self {
        Alert { session_id, conn }
    }

    /// Get the active alert text.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
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
        let v = self
            .conn
            .execute(Command::GetAlertText(&self.session_id))
            .await?;
        unwrap::<String>(&v["value"])
    }

    /// Dismiss the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
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
        self.conn
            .execute(Command::DismissAlert(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Accept the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
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
        self.conn
            .execute(Command::AcceptAlert(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Send the specified keys to the active alert.
    ///
    /// # Example:
    /// You can specify anything that implements `Into<TypingData>`. This
    /// includes &str and String.
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, WebDriver};
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
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, Keys, WebDriver};
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
        self.conn
            .execute(Command::SendAlertText(&self.session_id, keys.into()))
            .await
            .map(|_| ())
    }
}
