use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::write;

use base64::decode;

use crate::common::command::{Command, ElementId, SessionId};
use crate::common::connection_common::unwrap;
use crate::common::keys::TypingData;
use crate::error::WebDriverResult;
use crate::sync::RemoteConnectionSync;
use crate::types::{ElementRect, ElementRef};
use crate::By;

pub fn unwrap_element_sync(
    conn: Arc<RemoteConnectionSync>,
    session_id: SessionId,
    value: &serde_json::Value,
) -> WebDriverResult<WebElement> {
    let elem_id: ElementRef = serde_json::from_value(value.clone())?;
    Ok(WebElement::new(
        conn,
        session_id,
        ElementId::from(elem_id.id),
    ))
}

pub fn unwrap_elements_sync(
    conn: &Arc<RemoteConnectionSync>,
    session_id: &SessionId,
    value: &serde_json::Value,
) -> WebDriverResult<Vec<WebElement>> {
    let values: Vec<ElementRef> = serde_json::from_value(value.clone())?;
    Ok(values
        .into_iter()
        .map(|x| WebElement::new(conn.clone(), session_id.clone(), ElementId::from(x.id)))
        .collect())
}

#[derive(Debug, Clone)]
pub struct WebElement {
    pub element_id: ElementId,
    session_id: SessionId,
    conn: Arc<RemoteConnectionSync>,
}

impl WebElement {
    pub fn new(
        conn: Arc<RemoteConnectionSync>,
        session_id: SessionId,
        element_id: ElementId,
    ) -> Self {
        WebElement {
            conn,
            session_id,
            element_id,
        }
    }

    pub fn rect(&self) -> WebDriverResult<ElementRect> {
        let v = self
            .conn
            .execute(Command::GetElementRect(&self.session_id, &self.element_id))?;
        let r: ElementRect = serde_json::from_value((&v["value"]).clone())?;
        Ok(r)
    }

    pub fn tag_name(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementTagName(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    pub fn text(&self) -> WebDriverResult<String> {
        let v = self
            .conn
            .execute(Command::GetElementText(&self.session_id, &self.element_id))?;
        unwrap(&v["value"])
    }

    pub fn click(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::ElementClick(&self.session_id, &self.element_id))?;
        Ok(())
    }

    pub fn clear(&self) -> WebDriverResult<()> {
        self.conn
            .execute(Command::ElementClear(&self.session_id, &self.element_id))?;
        Ok(())
    }

    pub fn get_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementProperty(
            &self.session_id,
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    pub fn get_attribute(&self, name: &str) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementAttribute(
            &self.session_id,
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    pub fn get_css_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::GetElementCSSValue(
            &self.session_id,
            &self.element_id,
            name.to_owned(),
        ))?;
        unwrap(&v["value"])
    }

    pub fn is_selected(&self) -> WebDriverResult<bool> {
        let v = self.conn.execute(Command::IsElementSelected(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    pub fn is_enabled(&self) -> WebDriverResult<bool> {
        let v = self.conn.execute(Command::IsElementEnabled(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    pub fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self.conn.execute(Command::FindElementFromElement(
            &self.session_id,
            &self.element_id,
            by,
        ))?;
        unwrap_element_sync(self.conn.clone(), self.session_id.clone(), &v["value"])
    }

    pub fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self.conn.execute(Command::FindElementsFromElement(
            &self.session_id,
            &self.element_id,
            by,
        ))?;
        unwrap_elements_sync(&self.conn, &self.session_id, &v["value"])
    }

    pub fn send_keys(&self, keys: TypingData) -> WebDriverResult<()> {
        self.conn.execute(Command::ElementSendKeys(
            &self.session_id,
            &self.element_id,
            keys,
        ))?;
        Ok(())
    }

    pub fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.conn.execute(Command::TakeElementScreenshot(
            &self.session_id,
            &self.element_id,
        ))?;
        unwrap(&v["value"])
    }

    pub fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64()?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    pub fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png()?;
        let mut file = File::create(path)?;
        file.write_all(&png)?;
        Ok(())
    }
}

impl fmt::Display for WebElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"(session="{}", element="{}")"#,
            self.session_id, self.element_id
        )
    }
}
