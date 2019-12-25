use crate::{
    common::{command::Command, connection_common::unwrap, keys::TypingData},
    error::WebDriverResult,
    sync::RemoteConnectionSync,
    SessionId,
};
use std::sync::Arc;

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
    pub fn text(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetAlertText(&self.session_id))?;
        unwrap::<String>(&v["value"])
    }

    /// Dismiss the active alert.
    pub fn dismiss(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DismissAlert(&self.session_id))
            .map(|_| ())
    }

    /// Accept the active alert.
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
    /// ```ignore
    /// driver.switch_to().alert().send_keys("username")?;
    /// ```
    ///
    /// You can also send special key combinations like this:
    /// ```ignore
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys(Keys::Control + "a")?;
    /// alert.send_keys(TypingData::from("selenium") + Keys::Enter)?;
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
