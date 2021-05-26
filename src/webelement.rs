#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use crate::runtime::imports::{AsyncWriteExt, File};
use base64::decode;
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::fmt;
#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use std::path::Path;

use crate::common::command::MAGIC_ELEMENTID;
use crate::error::WebDriverError;
use crate::webdrivercommands::WebDriverCommands;
use crate::{
    common::{
        command::Command,
        connection_common::convert_json,
        keys::TypingData,
        types::{ElementRect, ElementRef},
    },
    error::WebDriverResult,
    session::WebDriverSession,
    By, ElementId, ScriptArgs,
};

/// Convert the raw JSON into a WebElement struct.
pub fn convert_element_async<'a>(
    session: &'a WebDriverSession,
    value: &serde_json::Value,
) -> WebDriverResult<WebElement<'a>> {
    let elem_id: ElementRef = serde_json::from_value(value.clone())?;
    Ok(WebElement::new(session, ElementId::from(elem_id.id)))
}

/// Convert the raw JSON into a Vec of WebElement structs.
pub fn convert_elements_async<'a>(
    session: &'a WebDriverSession,
    value: &serde_json::Value,
) -> WebDriverResult<Vec<WebElement<'a>>> {
    let values: Vec<ElementRef> = serde_json::from_value(value.clone())?;
    Ok(values.into_iter().map(|x| WebElement::new(session, ElementId::from(x.id))).collect())
}

/// The WebElement struct encapsulates a single element on a page.
///
/// WebElement structs are generally not constructed manually, but rather
/// they are returned from a 'find_element()' operation using a WebDriver.
///
/// # Example:
/// ```rust
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// #         let caps = DesiredCapabilities::chrome();
/// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
/// #         driver.get("http://webappdemo").await?;
/// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
/// let elem = driver.find_element(By::Id("input-result")).await?;
/// #         assert_eq!(elem.get_attribute("id").await?, Some("input-result".to_string()));
/// #         Ok(())
/// #     })
/// # }
/// ```
///
/// You can also search for a child element of another element as follows:
/// ```rust
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// #         let caps = DesiredCapabilities::chrome();
/// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
/// #         driver.get("http://webappdemo").await?;
/// let elem = driver.find_element(By::Css("div[data-section='section-buttons']")).await?;
/// let child_elem = elem.find_element(By::Tag("button")).await?;
/// #         child_elem.click().await?;
/// #         let result_elem = elem.find_element(By::Id("button-result")).await?;
/// #         assert_eq!(result_elem.text().await?, "Button 1 clicked");
/// #         Ok(())
/// #     })
/// # }
/// ```
///
/// Elements can be clicked using the `click()` method, and you can send
/// input to an element using the `send_keys()` method.
///
#[derive(Debug, Clone)]
pub struct WebElement<'a> {
    pub element_id: ElementId,
    pub session: &'a WebDriverSession,
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
    async fn cmd(&self, command: Command) -> WebDriverResult<serde_json::Value> {
        self.session.cmd(command).await
    }

    /// Get the bounding rectangle for this WebElement.
    pub async fn rect(&self) -> WebDriverResult<ElementRect> {
        let v = self.cmd(Command::GetElementRect(self.element_id.clone())).await?;
        let r: ElementRect = serde_json::from_value((&v["value"]).clone())?;
        Ok(r)
    }

    /// Get the tag name for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// assert_eq!(elem.tag_name().await?, "button");
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn tag_name(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementTagName(self.element_id.clone())).await?;
        convert_json(&v["value"])
    }

    /// Get the class name for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let class_name_option = elem.class_name().await?;  // Option<String>
    /// #         assert!(class_name_option.expect("Missing class name").contains("pure-button"));
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn class_name(&self) -> WebDriverResult<Option<String>> {
        self.get_attribute("class").await
    }

    /// Get the id for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// let id_option = elem.id().await?;  // Option<String>
    /// #         assert_eq!(id_option, Some("button1".to_string()));
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn id(&self) -> WebDriverResult<Option<String>> {
        self.get_attribute("id").await
    }

    /// Get the text contents for this WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("button1")).await?.click().await?;
    /// let elem = driver.find_element(By::Id("button-result")).await?;
    /// let text = elem.text().await?;
    /// #         assert_eq!(text, "Button 1 clicked");
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn text(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::GetElementText(self.element_id.clone())).await?;
        convert_json(&v["value"])
    }

    /// Convenience method for getting the (optional) value attribute of this element.
    pub async fn value(&self) -> WebDriverResult<Option<String>> {
        self.get_attribute("value").await
    }

    /// Click the WebElement.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.click().await?;
    /// #         let elem = driver.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(elem.text().await?, "Button 1 clicked");
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn click(&self) -> WebDriverResult<()> {
        self.cmd(Command::ElementClick(self.element_id.clone())).await?;
        Ok(())
    }

    /// Clear the WebElement contents.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input2")).await?;
    /// #         elem.clear().await?;
    /// # let cleared_text = elem.text().await?;
    /// #         assert_eq!(cleared_text, "");
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn clear(&self) -> WebDriverResult<()> {
        self.cmd(Command::ElementClear(self.element_id.clone())).await?;
        Ok(())
    }

    /// Get the specified property.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input2")).await?;
    /// let property_value_option = elem.get_property("checked").await?; // Option<String>
    /// assert_eq!(property_value_option, Some("true".to_string()));
    /// #         assert_eq!(elem.get_property("invalid-property").await?, None);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_property(&self, name: &str) -> WebDriverResult<Option<String>> {
        let v =
            self.cmd(Command::GetElementProperty(self.element_id.clone(), name.to_owned())).await?;

        if v["value"].is_null() {
            Ok(None)
        } else if !v["value"].is_string() {
            Ok(Some(v["value"].to_string()))
        } else {
            convert_json(&v["value"]).map(Some)
        }
    }

    /// Get the specified attribute.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input2")).await?;
    /// let attribute_option = elem.get_attribute("name").await?;  // Option<String>
    /// assert_eq!(attribute_option, Some("input2".to_string()));
    /// #         assert_eq!(elem.get_attribute("invalid-attribute").await?, None);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_attribute(&self, name: &str) -> WebDriverResult<Option<String>> {
        let v = self
            .cmd(Command::GetElementAttribute(self.element_id.clone(), name.to_owned()))
            .await?;
        if !v["value"].is_string() {
            Ok(None)
        } else {
            convert_json(&v["value"]).map(Some)
        }
    }

    /// Get the specified CSS property.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input2")).await?;
    /// let css_color = elem.get_css_property("color").await?;
    /// assert_eq!(css_color, "rgba(0, 0, 0, 1)");
    /// #         assert_eq!(elem.get_css_property("invalid-css-property").await?, "");
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn get_css_property(&self, name: &str) -> WebDriverResult<String> {
        let v =
            self.cmd(Command::GetElementCssValue(self.element_id.clone(), name.to_owned())).await?;
        if !v["value"].is_string() {
            Ok(String::new())
        } else {
            convert_json(&v["value"])
        }
    }

    /// Return true if the WebElement is currently selected, otherwise false.
    pub async fn is_selected(&self) -> WebDriverResult<bool> {
        let v = self.cmd(Command::IsElementSelected(self.element_id.clone())).await?;
        convert_json(&v["value"])
    }

    /// Return true if the WebElement is currently displayed, otherwise false.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let displayed = elem.is_displayed().await?;
    /// #         assert_eq!(displayed, true);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_displayed(&self) -> WebDriverResult<bool> {
        let v = self.cmd(Command::IsElementDisplayed(self.element_id.clone())).await?;
        convert_json(&v["value"])
    }

    /// Return true if the WebElement is currently enabled, otherwise false.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let enabled = elem.is_enabled().await?;
    /// #         assert_eq!(enabled, true);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_enabled(&self) -> WebDriverResult<bool> {
        let v = self.cmd(Command::IsElementEnabled(self.element_id.clone())).await?;
        convert_json(&v["value"])
    }

    /// Return true if the WebElement is currently clickable (visible and enabled),
    /// otherwise false.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let clickable = elem.is_clickable().await?;
    /// #         assert_eq!(clickable, true);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_clickable(&self) -> WebDriverResult<bool> {
        Ok(self.is_displayed().await? && self.is_enabled().await?)
    }

    /// Return true if the WebElement is currently (still) present
    /// and not stale.
    ///
    /// NOTE: This method simply queries the tag name in order to
    ///       determine whether the element is still present.
    ///
    /// IMPORTANT:
    /// If an element is re-rendered it may be considered stale even
    /// though to the user it looks like it is still there.
    ///
    /// The recommended way to check for the presence of an element is
    /// to simply search for the element again.
    ///
    /// # Example
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         let elem = driver.find_element(By::Id("button1")).await?;
    /// let present = elem.is_present().await?;
    /// #         assert_eq!(present, true);
    /// #         // Check negative case as well.
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         assert_eq!(elem.is_present().await?, false);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn is_present(&self) -> WebDriverResult<bool> {
        let present = match self.tag_name().await {
            Ok(_) => true,
            Err(WebDriverError::NoSuchElement(_))
            | Err(WebDriverError::StaleElementReference(_)) => false,
            Err(e) => return Err(e),
        };
        Ok(present)
    }

    /// Search for a child element of this WebElement using the specified
    /// selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']")).await?;
    /// let child_elem = elem.find_element(By::Tag("button")).await?;
    /// #         child_elem.click().await?;
    /// #         let result_elem = elem.find_element(By::Id("button-result")).await?;
    /// #         assert_eq!(result_elem.text().await?, "Button 1 clicked");
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find_element(&self, by: By<'a>) -> WebDriverResult<WebElement<'a>> {
        let v = self
            .cmd(Command::FindElementFromElement(self.element_id.clone(), by.get_w3c_selector()))
            .await?;
        convert_element_async(self.session.session(), &v["value"])
    }

    /// Search for all child elements of this WebElement that match the
    /// specified selector.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Css("div[data-section='section-buttons']")).await?;
    /// let child_elems = elem.find_elements(By::Tag("button")).await?;
    /// #         assert_eq!(child_elems.len(), 2);
    /// for child_elem in child_elems {
    ///     assert_eq!(child_elem.tag_name().await?, "button");
    /// }
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn find_elements(&self, by: By<'a>) -> WebDriverResult<Vec<WebElement<'a>>> {
        let v = self
            .cmd(Command::FindElementsFromElement(self.element_id.clone(), by.get_w3c_selector()))
            .await?;
        convert_elements_async(self.session.session(), &v["value"])
    }

    /// Send the specified input.
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
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input1")).await?;
    /// elem.send_keys("selenium").await?;
    /// #         assert_eq!(elem.value().await?, Some("selenium".to_string()));
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
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// #         let elem = driver.find_element(By::Name("input1")).await?;
    /// elem.send_keys("selenium").await?;
    /// elem.send_keys(Keys::Control + "a").await?;
    /// elem.send_keys(TypingData::from("thirtyfour") + Keys::Enter).await?;
    /// #         assert_eq!(elem.value().await?, Some("thirtyfour".to_string()));
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn send_keys<S>(&self, keys: S) -> WebDriverResult<()>
    where
        S: Into<TypingData>,
    {
        self.cmd(Command::ElementSendKeys(self.element_id.clone(), keys.into())).await?;
        Ok(())
    }

    /// Take a screenshot of this WebElement and return it as a base64-encoded
    /// String.
    pub async fn screenshot_as_base64(&self) -> WebDriverResult<String> {
        let v = self.cmd(Command::TakeElementScreenshot(self.element_id.clone())).await?;
        convert_json(&v["value"])
    }

    /// Take a screenshot of this WebElement and return it as PNG bytes.
    pub async fn screenshot_as_png(&self) -> WebDriverResult<Vec<u8>> {
        let s = self.screenshot_as_base64().await?;
        let bytes: Vec<u8> = decode(&s)?;
        Ok(bytes)
    }

    /// Take a screenshot of this WebElement and write it to the specified
    /// filename.
    #[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
    pub async fn screenshot(&self, path: &Path) -> WebDriverResult<()> {
        let png = self.screenshot_as_png().await?;
        let mut file = File::create(path).await?;
        file.write_all(&png).await?;
        Ok(())
    }

    /// Focus this WebElement using JavaScript.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// #         driver.find_element(By::Id("pagetextinput")).await?.click().await?;
    /// let elem = driver.find_element(By::Name("input1")).await?;
    /// elem.focus().await?;
    /// #         driver.action_chain().send_keys("selenium").perform().await?;
    /// #         assert_eq!(elem.value().await?, Some("selenium".to_string()));
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn focus(&self) -> WebDriverResult<()> {
        let mut args = ScriptArgs::new();
        args.push(&self)?;
        self.session.execute_script_with_args(r#"arguments[0].focus();"#, &args).await?;
        Ok(())
    }

    /// Scroll this element into view using JavaScript.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::Id("button1")).await?;
    /// elem.scroll_into_view().await?;
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn scroll_into_view(&self) -> WebDriverResult<()> {
        let mut args = ScriptArgs::new();
        args.push(&self)?;
        self.session.execute_script_with_args(r#"arguments[0].scrollIntoView();"#, &args).await?;
        Ok(())
    }

    /// Get the innerHtml property of this element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::XPath(r##"//*[@id="button1"]/.."##)).await?;
    /// let html = elem.inner_html().await?;
    /// #         assert_eq!(html, r##"<button class="pure-button pure-button-primary" id="button1">BUTTON 1</button>"##);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn inner_html(&self) -> WebDriverResult<String> {
        self.get_property("innerHTML").await.map(|x| x.unwrap_or_default())
    }

    /// Get the outerHtml property of this element.
    ///
    /// # Example:
    /// ```rust
    /// # use thirtyfour::prelude::*;
    /// # use thirtyfour::support::block_on;
    /// #
    /// # fn main() -> WebDriverResult<()> {
    /// #     block_on(async {
    /// #         let caps = DesiredCapabilities::chrome();
    /// #         let driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
    /// #         driver.get("http://webappdemo").await?;
    /// let elem = driver.find_element(By::XPath(r##"//*[@id="button1"]/.."##)).await?;
    /// let html = elem.outer_html().await?;
    /// #         assert_eq!(html, r##"<div class="pure-u-1-6"><button class="pure-button pure-button-primary" id="button1">BUTTON 1</button></div>"##);
    /// #         Ok(())
    /// #     })
    /// # }
    /// ```
    pub async fn outer_html(&self) -> WebDriverResult<String> {
        self.get_property("outerHTML").await.map(|x| x.unwrap_or_default())
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
