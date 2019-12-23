use std::sync::Arc;

use crate::common::command::Command;
use crate::common::connection_common::{unwrap, unwrap_vec};
use crate::error::{WebDriverError, WebDriverResult};
use crate::sync::webelement::unwrap_element_sync;
use crate::sync::{Alert, RemoteConnectionSync, WebElement};
use crate::{By, SessionId, WindowHandle};

/// Struct for switching between frames/windows/alerts.
pub struct SwitchTo {
    session_id: SessionId,
    conn: Arc<RemoteConnectionSync>,
}

impl SwitchTo {
    /// Create a new SwitchTo struct. This is typically created internally
    /// via a call to `WebDriver::switch_to()`.
    pub fn new(session_id: SessionId, conn: Arc<RemoteConnectionSync>) -> Self {
        SwitchTo { session_id, conn }
    }

    /// Return the element with focus, or BODY if nothing has focus.
    pub fn active_element(&self) -> WebDriverResult<WebElement> {
        let v = self
            .conn
            .execute(Command::GetActiveElement(&self.session_id))?;
        unwrap_element_sync(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    /// Return Alert struct for processing the active alert on the page.
    pub fn alert(&self) -> Alert {
        Alert::new(self.session_id.clone(), self.conn.clone())
    }

    /// Switch focus to the default frame.
    pub fn default_content(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameDefault(&self.session_id))
            .map(|_| ())
    }

    /// Switch focus to the frame by index.
    pub fn frame_number(&self, frame_number: u16) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameNumber(&self.session_id, frame_number))
            .map(|_| ())
    }

    /// Switch focus to the element with the specified Id or Name.
    ///
    /// This will attempt to find the element first by Id, and if that fails,
    /// by Name.
    pub fn frame_name(&self, frame_name: &str) -> WebDriverResult<()> {
        let v = match self
            .conn
            .execute(Command::FindElement(&self.session_id, By::Id(frame_name)))
        {
            Ok(elem) => elem,
            Err(WebDriverError::NoSuchElement(_)) => self
                .conn
                .execute(Command::FindElement(&self.session_id, By::Name(frame_name)))?,
            Err(e) => return Err(e),
        };
        let elem = unwrap_element_sync(self.conn.clone(), self.session_id.clone(), &v["value"])?;
        self.frame_element(elem)
    }

    /// Switch focus to the specified element.
    pub fn frame_element(&self, frame_element: WebElement) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToFrameElement(
                &self.session_id,
                &frame_element.element_id,
            ))
            .map(|_| ())
    }

    /// Switch focus to the parent frame.
    pub fn parent_frame(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToParentFrame(&self.session_id))
            .map(|_| ())
    }

    /// Switch focus to the specified window.
    pub fn window(&self, handle: &WindowHandle) -> WebDriverResult<()> {
        self.conn
            .execute(Command::SwitchToWindow(&self.session_id, handle))
            .map(|_| ())
    }

    /// Switch focus to the window with the specified name.
    pub fn window_name(&self, name: &str) -> WebDriverResult<()> {
        let original_handle = self
            .conn
            .execute(Command::GetWindowHandle(&self.session_id))
            .map(|v| unwrap::<String>(&v["value"]))??;

        let v = self
            .conn
            .execute(Command::GetWindowHandles(&self.session_id))?;
        let handles: Vec<String> = unwrap_vec(&v["value"])?;
        for handle in handles {
            self.window(&WindowHandle::from(handle))?;
            let current_name = self
                .conn
                .execute(Command::ExecuteScript(
                    &self.session_id,
                    String::from("return window.name"),
                    Vec::new(),
                ))
                .map(|v| v["value"].clone())?;

            if let Some(x) = current_name.as_str() {
                if x == name {
                    return Ok(());
                }
            }
        }

        self.window(&WindowHandle::from(original_handle))?;
        Err(WebDriverError::NotFoundError(format!(
            "No window handle found matching '{}'",
            name
        )))
    }
}
