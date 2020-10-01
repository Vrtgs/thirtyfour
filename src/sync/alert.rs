use crate::sync::webdrivercommands::WebDriverCommands;
use crate::sync::WebDriverSession;
use crate::{
    common::{command::Command, connection_common::convert_json, keys::TypingData},
    error::WebDriverResult,
};

/// Struct for managing alerts.
pub struct Alert<'a> {
    session: &'a WebDriverSession,
}

impl<'a> Alert<'a> {
    /// Create a new Alert struct. This is typically created internally
    /// via a call to `WebDriver::switch_to().alert()`.
    pub fn new(session: &'a WebDriverSession) -> Self {
        Alert {
            session,
        }
    }

    ///Convenience wrapper for executing a WebDriver command.
    fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.session.cmd(command)
    }

    /// Get the active alert text.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagealerts"))?.click()?;
    /// #     driver.find_element(By::Id("alertbutton1"))?.click()?;
    /// let alert = driver.switch_to().alert();
    /// let text = alert.text()?;
    /// #     assert_eq!(text, "Alert 1 showing");
    /// #     alert.dismiss()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn text(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetAlertText)?;
        convert_json::<String>(&v["value"])
    }

    /// Dismiss the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagealerts"))?.click()?;
    /// #     driver.find_element(By::Id("alertbutton2"))?.click()?;
    /// driver.switch_to().alert().dismiss()?;
    /// #     let elem = driver.find_element(By::Id("alert-result"))?;
    /// #     assert_eq!(elem.text()?, "Alert 2 clicked false");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn dismiss(&self) -> WebDriverResult<()> {
        self.cmd(Command::DismissAlert).map(|_| ())
    }

    /// Accept the active alert.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagealerts"))?.click()?;
    /// #     driver.find_element(By::Id("alertbutton2"))?.click()?;
    /// driver.switch_to().alert().accept()?;
    /// #     let elem = driver.find_element(By::Id("alert-result"))?;
    /// #     assert_eq!(elem.text()?, "Alert 2 clicked true");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn accept(&self) -> WebDriverResult<()> {
        self.cmd(Command::AcceptAlert).map(|_| ())
    }

    /// Send the specified keys to the active alert.
    ///
    /// # Example:
    /// You can specify anything that implements `Into<TypingData>`. This
    /// includes &str and String.
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagealerts"))?.click()?;
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
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagealerts"))?.click()?;
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
        self.cmd(Command::SendAlertText(keys.into())).map(|_| ())
    }
}
