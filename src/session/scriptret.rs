use crate::error::WebDriverResult;
use crate::session::handle::SessionHandle;
use crate::WebElement;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Helper struct for getting return values from scripts.
/// See the examples for [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
/// and [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script).
#[derive(Debug)]
pub struct ScriptRet {
    handle: SessionHandle,
    value: serde_json::Value,
}

impl ScriptRet {
    /// Create a new ScriptRet. This is typically done automatically via
    /// [WebDriver::execute_script()](struct.WebDriver.html#method.execute_script)
    /// or [WebDriver::execute_async_script()](struct.WebDriver.html#method.execute_async_script)
    pub fn new(handle: SessionHandle, value: serde_json::Value) -> Self {
        Self {
            handle,
            value,
        }
    }

    /// Get the raw JSON value.
    pub fn json(&self) -> &serde_json::Value {
        &self.value
    }

    #[deprecated(since = "0.30.0", note = "This method has been renamed to json()")]
    pub fn value(&self) -> &serde_json::Value {
        self.json()
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
    pub fn element(self) -> WebDriverResult<WebElement> {
        WebElement::from_json(self.value, self.handle)
    }

    #[deprecated(since = "0.30.0", note = "This method has been renamed to element()")]
    pub fn get_element(self) -> WebDriverResult<WebElement> {
        self.element()
    }

    /// Get a vec of WebElements from the return value.
    /// Your script must return an array of elements for this to work.
    pub fn elements(self) -> WebDriverResult<Vec<WebElement>> {
        let values: Vec<Value> = serde_json::from_value(self.value)?;
        let handle = self.handle;
        values.into_iter().map(|x| WebElement::from_json(x, handle.clone())).collect()
    }

    #[deprecated(since = "0.30.0", note = "This method has been renamed to elements()")]
    pub fn get_elements(self) -> WebDriverResult<Vec<WebElement>> {
        self.elements()
    }
}
