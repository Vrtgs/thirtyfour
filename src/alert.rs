use std::sync::Arc;

use crate::common::command::Command;
use crate::common::connection_common::unwrap;
use crate::common::keys::TypingData;
use crate::error::WebDriverResult;
use crate::{RemoteConnectionAsync, SessionId};

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
    pub async fn text(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetAlertText(&self.session_id))
            .await?;
        unwrap::<String>(&v["value"])
    }

    /// Dismiss the active alert.
    pub async fn dismiss(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::DismissAlert(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Accept the active alert.
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
    /// ```ignore
    /// driver.switch_to().alert().send_keys("username").await?;
    /// ```
    ///
    /// You can also send special key combinations like this:
    /// ```ignore
    /// let alert = driver.switch_to().alert();
    /// alert.send_keys(Keys::Control + "a").await?;
    /// alert.send_keys(TypingData::from("selenium") + Keys::Enter).await?;
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
