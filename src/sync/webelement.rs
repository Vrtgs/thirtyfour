use std::{fmt, fs::File, io::Write, path::Path, write};

use base64::decode;
use serde::ser::{Serialize, SerializeMap, Serializer};

use crate::common::command::MAGIC_ELEMENTID;
use crate::sync::webdrivercommands::WebDriverCommands;
use crate::sync::WebDriverSession;
use crate::{
    common::{
        command::Command,
        connection_common::convert_json,
        keys::TypingData,
        types::{ElementId, ElementRect, ElementRef},
    },
    error::WebDriverResult,
    By, ScriptArgs,
};

/// Unwrap the raw JSON into a WebElement struct.
pub fn convert_element_sync<'a>(
    driver: &'a WebDriverSession,
    value: &serde_json::Value,
) -> WebDriverResult<WebElement<'a>> {
    let elem_id: ElementRef = serde_json::from_value(value.clone())?;
    Ok(WebElement::new(driver, ElementId::from(elem_id.id)))
}

/// Unwrap the raw JSON into a Vec of WebElement structs.
pub fn convert_elements_sync<'a>(
    driver: &'a WebDriverSession,
    value: &serde_json::Value,
) -> WebDriverResult<Vec<WebElement<'a>>> {
    let values: Vec<ElementRef> = serde_json::from_value(value.clone())?;
    Ok(values.into_iter().map(|x| WebElement::new(driver, ElementId::from(x.id))).collect())
}

/// The WebElement struct encapsulates a single element on a page.
///
/// WebElement structs are generally not constructed manually, but rather
/// they are returned from a 'find_element()' operation using a WebDriver.
///
/// # Example:
/// ```rust
/// # use thirtyfour::sync::prelude::*;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     let caps = DesiredCapabilities::chrome();
/// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
/// #     driver.get("http://webappdemo")?;
/// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
/// let elem = driver.find_element(By::Id("input-result"))?;
/// #     assert_eq!(elem.get_attribute("id")?, "input-result");
/// #     Ok(())
/// # }
/// ```
///
/// You can also search for a child element of another element as follows:
/// ```rust
/// # use thirtyfour::sync::prelude::*;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     let caps = DesiredCapabilities::chrome();
/// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
/// #     driver.get("http://webappdemo")?;
/// let elem = driver.find_element(By::Css("div[data-section='section-buttons']"))?;
/// let child_elem = elem.find_element(By::Tag("button"))?;
/// #     child_elem.click()?;
/// #     let result_elem = elem.find_element(By::Id("button-result"))?;
/// #     assert_eq!(result_elem.text()?, "Button 1 clicked");
/// #     Ok(())
/// # }
/// ```
///
/// Elements can be clicked using the `click()` method, and you can send
/// input to an element using the `send_keys()` method.
///
#[derive(Debug, Clone)]
pub struct WebElement<'a> {
    pub element_id: ElementId,
    session: &'a WebDriverSession,
}

impl<'a> WebElement<'a> {
    /// Create a new WebElement struct.
    ///
    /// Typically you would not call this directly. WebElement structs are
    /// usually constructed by calling one of the find_element*() methods
    /// either on WebDriver or another WebElement.
    pub fn new(session: &'a WebDriverSession, element_id: ElementId) -> Self {
        WebElement {
            element_id,
            session,
        }
    }

    ///Convenience wrapper for executing a WebDriver command.
    fn cmd(&self, command: Command<'_>) -> WebDriverResult<serde_json::Value> {
        self.session.cmd(command)
    }

    /// Get the bounding rectangle for this WebElement.
    pub fn rect(&self) -> WebDriverResult<ElementRect> {
        let v = self.cmd(Command::GetElementRect(&self.element_id))?;
        let r: ElementRect = serde_json::from_value((&v["value"]).clone())?;
        Ok(r)
    }

    /// Get the tag name for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// assert_eq!(elem.tag_name()?, "button");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn tag_name(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementTagName(&self.element_id))?;
        convert_json(&v["value"])
    }

    /// Get the class name for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// let class_name = elem.class_name()?;
    /// #     assert!(class_name.contains("pure-button"));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn class_name(&self) -> WebDriverResult<String> {
        self.get_attribute("class")
    }

    /// Get the id for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// let id = elem.id()?;
    /// #     assert_eq!(id, "button1");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn id(&self) -> WebDriverResult<String> {
        self.get_attribute("id")
    }

    /// Get the text contents for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("button1"))?.click()?;
    /// let elem = driver.find_element(By::Id("button-result"))?;
    /// let text = elem.text()?;
    /// #     assert_eq!(text, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn text(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementText(&self.element_id))?;
        convert_json(&v["value"])
    }

    /// Convenience method for getting the value attribute of this element.
    pub fn value(&self) -> WebDriverResult<String> {
        self.get_attribute("value")
    }

    /// Click the WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.click()?;
    /// #     let elem = driver.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(elem.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn click(&self) -> WebDriverResult<()> {
        self.cmd(Command::ElementClick(&self.element_id))?;
        Ok(())
    }

    /// Clear the WebElement contents.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input2"))?;
    /// #     elem.clear()?;
    /// # let cleared_text = elem.text()?;
    /// #     assert_eq!(cleared_text, "");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn clear(&self) -> WebDriverResult<()> {
        self.cmd(Command::ElementClear(&self.element_id))?;
        Ok(())
    }

    /// Get the specified property.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input2"))?;
    /// let bool_value = elem.get_property("checked")?;
    /// assert_eq!(bool_value, "true");
    /// let string_value = elem.get_property("name")?;
    /// assert_eq!(string_value, "input2");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementProperty(&self.element_id, name.to_owned()))?;
        if !v["value"].is_string() {
            Ok(v["value"].to_string())
        } else {
            convert_json(&v["value"])
        }
    }

    /// Get the specified attribute.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input2"))?;
    /// let attribute = elem.get_attribute("name")?;
    /// assert_eq!(attribute, "input2");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_attribute(&self, name: &str) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementAttribute(&self.element_id, name.to_owned()))?;
        if !v["value"].is_string() {
            Ok(v["value"].to_string())
        } else {
            convert_json(&v["value"])
        }
    }

    /// Get the specified CSS property.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input2"))?;
    /// let css_color = elem.get_css_property("color")?;
    /// assert_eq!(css_color, r"rgba(0, 0, 0, 1)");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn get_css_property(&self, name: &str) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementCSSValue(&self.element_id, name.to_owned()))?;
        if !v["value"].is_string() {
            Ok(v["value"].to_string())
        } else {
            convert_json(&v["value"])
        }
    }

    /// Return true if the WebElement is currently selected, otherwise false.
    pub fn is_selected(&self) -> WebDriverResult<bool> {
        let v = self.cmd(Command::IsElementSelected(&self.element_id))?;
        convert_json(&v["value"])
    }

    /// Return true if the WebElement is currently enabled, otherwise false.
    pub fn is_enabled(&self) -> WebDriverResult<bool> {
        let v = self.cmd(Command::IsElementEnabled(&self.element_id))?;
        convert_json(&v["value"])
    }

    /// Search for a child element of this WebElement using the specified
    /// selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']"))?;
    /// let child_elem = elem.find_element(By::Tag("button"))?;
    /// #     child_elem.click()?;
    /// #     let result_elem = elem.find_element(By::Id("button-result"))?;
    /// #     assert_eq!(result_elem.text()?, "Button 1 clicked");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn find_element(&self, by: By) -> WebDriverResult<WebElement> {
        let v = self.cmd(Command::FindElementFromElement(&self.element_id, by))?;
        convert_element_sync(self.session, &v["value"])
    }

    /// Search for all child elements of this WebElement that match the
    /// specified selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']"))?;
    /// let child_elems = elem.find_elements(By::Tag("button"))?;
    /// #     assert_eq!(child_elems.len(), 2);
    /// for child_elem in child_elems {
    ///     assert_eq!(child_elem.tag_name()?, "button");
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn find_elements(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        let v = self.cmd(Command::FindElementsFromElement(&self.element_id, by))?;
        convert_elements_sync(self.session, &v["value"])
    }

    /// Send the specified input.
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
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input1"))?;
    /// elem.send_keys("selenium")?;
    /// #     assert_eq!(elem.value()?, "selenium");
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
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// #     let elem = driver.find_element(By::Name("input1"))?;
    /// elem.send_keys("selenium")?;
    /// elem.send_keys(Keys::Control + "a")?;
    /// elem.send_keys(TypingData::from("thirtyfour") + Keys::Enter)?;
    /// #     assert_eq!(elem.value()?, "thirtyfour");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn send_keys<S>(&self, keys: S) -> WebDriverResult<()>
    where
        S: Into<TypingData>,
    {
        self.cmd(Command::ElementSendKeys(&self.element_id, keys.into()))?;
        Ok(())
    }

    /// Take a screenshot of this WebElement and return it as a base64-encoded
    /// String.
    pub fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::TakeElementScreenshot(&self.element_id))?;
        convert_json(&v["value"])
    }

    /// Take a screenshot of this WebElement and return it as PNG bytes.
    pub fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64()?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of this WebElement and write it to the specified
    /// filename.
    pub fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png()?;
        let mut file = File::create(path)?;
        file.write_all(&png)?;
        Ok(())
    }

    /// Focus this WebElement using JavaScript.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// #     driver.find_element(By::Id("pagetextinput"))?.click()?;
    /// let elem = driver.find_element(By::Name("input1"))?;
    /// elem.focus()?;
    /// #     driver.action_chain().send_keys("selenium").perform()?;
    /// #     assert_eq!(elem.value()?, "selenium");
    /// #     Ok(())
    /// # }
    /// ```
    pub fn focus(&self) -> WebDriverResult<()> {
        let mut args = ScriptArgs::new();
        args.push(&self)?;
        self.session.execute_script_with_args(r#"arguments[0].focus();"#, &args)?;
        Ok(())
    }

    /// Scroll this element into view using JavaScript.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     let caps = DesiredCapabilities::chrome();
    /// #     let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #     driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::Id("button1"))?;
    /// elem.scroll_into_view()?;
    /// #     Ok(())
    /// # }
    /// ```
    pub fn scroll_into_view(&self) -> WebDriverResult<()> {
        let mut args = ScriptArgs::new();
        args.push(&self)?;
        self.session.execute_script_with_args(r#"arguments[0].scrollIntoView();"#, &args)?;
        Ok(())
    }

    /// Get the innerHtml property of this element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #         driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::XPath(r##"//*[@id="button1"]/.."##))?;
    /// let html = elem.inner_html()?;
    /// #         assert_eq!(html, r##"<button class="pure-button pure-button-primary" id="button1">BUTTON 1</button>"##);
    /// #         Ok(())
    /// # }
    /// ```
    pub fn inner_html(&self) -> WebDriverResult<String> {
        self.get_property("innerHTML")
    }

    /// Get the outerHtml property of this element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::sync::prelude::*;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps)?;
    /// #         driver.get("http://webappdemo")?;
    /// let elem = driver.find_element(By::XPath(r##"//*[@id="button1"]/.."##))?;
    /// let html = elem.outer_html()?;
    /// #         assert_eq!(html, r##"<div class="pure-u-1-6"><button class="pure-button pure-button-primary" id="button1">BUTTON 1</button></div>"##);
    /// #         Ok(())
    /// # }
    /// ```
    pub fn outer_html(&self) -> WebDriverResult<String> {
        self.get_property("outerHTML")
    }
}

impl<'a> fmt::Display for WebElement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"(session="{}", element="{}")"#, self.session.session_id(), self.element_id)
    }
}

impl<'a> Serialize for WebElement<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(MAGIC_ELEMENTID, &self.element_id.to_string())?;
        map.end()
    }
}
