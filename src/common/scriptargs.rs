use serde::Serialize;
use serde_json::Value;

use crate::error::WebDriverResult;

/// Helper struct for constructing arguments for `WebDriver::execute_script()`
/// and `WebDriver::execute_async_script()`.
///
/// See the examples for those methods for more info.
#[derive(Debug, Default, Clone)]
pub struct ScriptArgs {
    values: Vec<Value>,
}

impl ScriptArgs {
    /// Create a new, empty ScriptArgs struct.
    pub fn new() -> Self {
        ScriptArgs::default()
    }

    /// Push a JSON value onto the vec.
    pub fn push_value(&mut self, value: Value) -> &mut Self {
        self.values.push(value);
        self
    }

    /// Push any Serialize-able object onto the vec.
    /// This includes WebElement.
    pub fn push<T>(&mut self, value: T) -> WebDriverResult<&mut Self>
    where
        T: Serialize,
    {
        Ok(self.push_value(serde_json::to_value(value)?))
    }

    /// Get the args vec. This is used internally.
    pub fn get_args(&self) -> Vec<Value> {
        self.values.clone()
    }
}
