use crate::error::WebDriverResult;
use crate::session::handle::SessionHandle;
use crate::webelement::{convert_element_async, convert_elements_async};
use crate::WebElement;
use serde::de::DeserializeOwned;

/// Helper struct for getting return values from scripts.
/// See the examples for [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
/// and [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script).
pub struct ScriptRet<'a> {
    handle: &'a SessionHandle,
    value: serde_json::Value,
}

impl<'a> ScriptRet<'a> {
    /// Create a new ScriptRet. This is typically done automatically via
    /// [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
    /// or [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script)
    pub fn new(handle: &'a SessionHandle, value: serde_json::Value) -> Self {
        Self {
            handle,
            value,
        }
    }

    /// Get the raw JSON value.
    pub fn value(&self) -> &serde_json::Value {
        &self.value
    }

    pub fn convert<T>(&self) -> WebDriverResult<T>
    where
        T: DeserializeOwned,
    {
        let v: T = serde_json::from_value(self.value.clone())?;
        Ok(v)
    }

    /// Get a single WebElement return value.
    /// Your script must return only a single element for this to work.
    pub fn get_element(&self) -> WebDriverResult<WebElement> {
        convert_element_async(self.handle, &self.value)
    }

    /// Get a vec of WebElements from the return value.
    /// Your script must return an array of elements for this to work.
    pub fn get_elements(&self) -> WebDriverResult<Vec<WebElement>> {
        convert_elements_async(self.handle, &self.value)
    }
}
