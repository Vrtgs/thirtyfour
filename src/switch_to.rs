use std::sync::Arc;

use crate::common::command::Command;
use crate::common::connection_common::{unwrap, unwrap_vec};
use crate::error::{WebDriverError, WebDriverResult};
use crate::webelement::unwrap_element_async;
use crate::{Alert, RemoteConnectionAsync, WebElement};
use crate::{By, SessionId, WindowHandle};

/// Struct for switching between frames/windows/alerts.
pub struct SwitchTo {
    session_id: SessionId,
    conn: Arc<RemoteConnectionAsync>,
}

impl SwitchTo {
    /// Create a new SwitchTo struct. This is typically created internally
    /// via a call to `WebDriver::switch_to()`.
    pub fn new(session_id: SessionId, conn: Arc<RemoteConnectionAsync>) -> Self {
        SwitchTo { session_id, conn }
    }

    /// Return the element with focus, or BODY if nothing has focus.
    pub async fn active_element(&self) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::GetActiveElement(&self.session_id))
            .await?;
        unwrap_element_async(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    /// Return Alert struct for processing the active alert on the page.
    pub fn alert(&self) -> Alert {
        Alert::new(self.session_id.clone(), self.conn.clone())
    }

    /// Switch focus to the default frame.
    pub async fn default_content(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameDefault(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Switch focus to the frame by index.
    pub async fn frame_number(&self, frame_number: u16) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameNumber(&self.session_id, frame_number))
            .await
            .map(|_| ())
    }

    /// Switch focus to the element with the specified Id or Name.
    ///
    /// This will attempt to find the element first by Id, and if that fails,
    /// by Name.
    pub async fn frame_name(&self, frame_name: &str) -> WebDriverResult<()> {
        let v = match self
            .conn
            .execute(Command::FindElement(&self.session_id, By::Id(frame_name)))
            .await
        {
            Ok(elem) => elem,
            Err(WebDriverError::NoSuchElement(_)) => {
                self.conn
                    .execute(Command::FindElement(&self.session_id, By::Name(frame_name)))
                    .await?
            }
            Err(e) => return Err(e),
        };
        let elem = unwrap_element_async(self.conn.clone(), self.session_id.clone(), &v["value"])?;
        self.frame_element(elem).await
    }

    /// Switch focus to the specified element.
    pub async fn frame_element(&self, frame_element: WebElement) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameElement(
                &self.session_id,
                &frame_element.element_id,
            ))
            .await
            .map(|_| ())
    }

    /// Switch focus to the parent frame.
    pub async fn parent_frame(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToParentFrame(&self.session_id))
            .await
            .map(|_| ())
    }

    /// Switch focus to the specified window.
    pub async fn window(&self, handle: &WindowHandle) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToWindow(&self.session_id, handle))
            .await
            .map(|_| ())
    }

    /// Switch focus to the window with the specified name.
    pub async fn window_name(&self, name: &str) -> WebDriverResult<()> {
        let original_handle = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id))
            .await
            .map(|v| unwrap::<String>(&v["value"]))??;

        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))
            .await?;
        let handles: Vec<String> = unwrap_vec(&v["value"])?;
        for handle in handles {
            self.window(&WindowHandle::from(handle)).await?;
            let current_name = self
                .conn
                .execute(Command::ExecuteScript(
                    &self.session_id,
                    String::from("return window.name"),
                    Vec::new(),
                ))
                .await
                .map(|v| v["value"].clone())?;

            if let Some(x) = current_name.as_str() {
                if x == name {
                    return Ok(());
                }
            }
        }

        self.window(&WindowHandle::from(original_handle)).await?;
        Err(WebDriverError::NotFoundError(format!(
            "No window handle found matching '{}'",
            name
        )))
    }
}
