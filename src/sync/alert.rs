use std::sync::Arc;

use crate::{
    common::{command::Command, connection_common::unwrap, keys::TypingData},
    error::WebDriverResult,
    sync::RemoteConnectionSync,
    SessionId,
};

/// Struct for managing alerts.
pub struct Alert {
    session_id: SessionId,
    conn: Arc<RemoteConnectionSync>,
}

impl Alert {
    /// Create a new Alert struct. This is typically created internally
    /// via a call to `WebDriver::switch_to().alert()`.
    pub fn new(session_id: SessionId, conn: Arc<RemoteConnectionSync>) -> Self {
        Alert { session_id, conn }
    }

    /// Get the active alert text.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("alertbutton1"))?.click()?;
    /// let alert = driver.switch_to().alert();
    /// let text = alert.text()?;
    /// #     assert_eq!(text, "Alert 1 showing");
    /// #     alert.dismiss()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn text(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetAlertText(&self.session_id))?;
        unwrap::<String>(&v["value"])
    }

    /// Dismiss the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("alertbutton2"))?.click()?;
    /// driver.switch_to().alert().dismiss()?;
    /// #     let elem = driver.find_element(By::Id("alert-result"))?;
    /// #     assert_eq!(elem.text()?, "Alert 2 clicked false");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn dismiss(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DismissAlert(&self.session_id))
            .map(|_| ())
    }

    /// Accept the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("alertbutton2"))?.click()?;
    /// driver.switch_to().alert().accept()?;
    /// #     let elem = driver.find_element(By::Id("alert-result"))?;
    /// #     assert_eq!(elem.text()?, "Alert 2 clicked true");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn accept(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::AcceptAlert(&self.session_id))
            .map(|_| ())
    }

    /// Send the specified keys to the active alert.
    ///
    /// # Example:
    /// You can specify anything that implements `Into<TypingData>`. This
    /// includes &str and String.
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("alertbutton3"))?.click()?;
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys("selenium")?;
    /// alert.accept()?;
    /// #     let elem = driver.find_element(By::Id("alert-result"))?;
    /// #     assert_eq!(elem.text()?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// You can also send special key combinations like this:
    /// ```rust
    /// # use thirtyfour::error::WebDriverResult;
    /// # use thirtyfour::{By, DesiredCapabilities, Keys, TypingData, sync::WebDriver};
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("alertbutton3"))?.click()?;
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys("selenium")?;
    /// alert.send_keys(Keys::Control + "a")?;
    /// alert.send_keys("thirtyfour")?;
    /// #     alert.accept()?;
    /// #     let elem = driver.find_element(By::Id("alert-result"))?;
    /// #     assert_eq!(elem.text()?, "thirtyfour");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send_keys<S>(&self, keys: S) -> WebDriverResult<()>
    where
        S: Into<TypingData>,
    {
        self.conn
            .execute(Command::SendAlertText(&self.session_id, keys.into()))
            .map(|_| ())
    }
}
