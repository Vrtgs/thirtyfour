use super::conditions::handle_errors;
use super::{conditions, ElementPoller, ElementPollerTicker, ElementPredicate};
use crate::error::WebDriverError;
use crate::prelude::WebDriverResult;
use crate::WebElement;
use std::time::Duration;
use stringmatch::Needle;

/// High-level interface for performing explicit waits using the builder pattern.
///
/// # Example:
/// ```no_run
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     block_on(async {
/// #         let caps = DesiredCapabilities::chrome();
/// #         let mut driver = WebDriver::new("http://localhost:4444", caps).await?;
/// #         driver.goto("http://webappdemo").await?;
/// #         let elem = driver.query(By::Id("button1")).first().await?;
/// // Wait until the element is displayed.
/// elem.wait_until().displayed().await?;
/// #         assert!(elem.is_displayed().await?);
/// #         driver.quit().await?;
/// #         Ok(())
/// #     })
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ElementWaiter {
    element: WebElement,
    poller: ElementPoller,
    message: String,
    ignore_errors: bool,
}

impl ElementWaiter {
    fn new(element: WebElement, poller: ElementPoller) -> Self {
        Self {
            element,
            poller,
            message: String::new(),
            ignore_errors: true,
        }
    }

    /// Use the specified ElementPoller for this ElementWaiter.
    /// This will not affect the default ElementPoller used for other waits.
    pub fn with_poller(mut self, poller: ElementPoller) -> Self {
        self.poller = poller;
        self
    }

    /// Provide a human-readable error message to be returned in the case of timeout.
    pub fn error(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// By default a waiter will ignore any errors that occur while polling for the desired
    /// condition(s). However, this behaviour can be modified so that the waiter will return
    /// early if an error is returned from thirtyfour.
    pub fn ignore_errors(mut self, ignore: bool) -> Self {
        self.ignore_errors = ignore;
        self
    }

    /// Force this ElementWaiter to wait for the specified timeout, polling once
    /// after each interval. This will override the poller for this
    /// ElementWaiter only.
    pub fn wait(self, timeout: Duration, interval: Duration) -> Self {
        self.with_poller(ElementPoller::TimeoutWithInterval(timeout, interval))
    }

    async fn run_poller(&self, conditions: Vec<ElementPredicate>) -> WebDriverResult<bool> {
        let mut ticker = ElementPollerTicker::new(self.poller.clone());
        loop {
            let mut conditions_met = true;
            for f in &conditions {
                if !f(&self.element).await? {
                    conditions_met = false;
                    break;
                }
            }

            if conditions_met {
                return Ok(true);
            }

            if !ticker.tick().await {
                return Ok(false);
            }
        }
    }

    fn timeout(self) -> WebDriverResult<()> {
        Err(WebDriverError::Timeout(self.message))
    }

    pub async fn condition(self, f: ElementPredicate) -> WebDriverResult<()> {
        match self.run_poller(vec![f]).await? {
            true => Ok(()),
            false => self.timeout(),
        }
    }

    pub async fn conditions(self, conditions: Vec<ElementPredicate>) -> WebDriverResult<()> {
        match self.run_poller(conditions).await? {
            true => Ok(()),
            false => self.timeout(),
        }
    }

    pub async fn stale(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(Box::new(move |elem| {
            Box::pin(
                async move { handle_errors(elem.is_present().await.map(|x| !x), ignore_errors) },
            )
        }))
        .await
    }

    pub async fn displayed(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_displayed(ignore_errors)).await
    }

    pub async fn not_displayed(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_displayed(ignore_errors)).await
    }

    pub async fn selected(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_selected(ignore_errors)).await
    }

    pub async fn not_selected(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_selected(ignore_errors)).await
    }

    pub async fn enabled(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_enabled(ignore_errors)).await
    }

    pub async fn not_enabled(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_enabled(ignore_errors)).await
    }

    pub async fn clickable(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_clickable(ignore_errors)).await
    }

    pub async fn not_clickable(self) -> WebDriverResult<()> {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_is_not_clickable(ignore_errors)).await
    }

    pub async fn has_class<N>(self, class_name: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_class(class_name, ignore_errors)).await
    }

    pub async fn lacks_class<N>(self, class_name: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_class(class_name, ignore_errors)).await
    }

    pub async fn has_text<N>(self, text: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_text(text, ignore_errors)).await
    }

    pub async fn lacks_text<N>(self, text: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_text(text, ignore_errors)).await
    }

    pub async fn has_value<N>(self, value: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_value(value, ignore_errors)).await
    }

    pub async fn lacks_value<N>(self, value: N) -> WebDriverResult<()>
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_value(value, ignore_errors)).await
    }

    pub async fn has_attribute<S, N>(self, attribute_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_attribute(attribute_name, value, ignore_errors))
            .await
    }

    pub async fn lacks_attribute<S, N>(self, attribute_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_attribute(attribute_name, value, ignore_errors))
            .await
    }

    pub async fn has_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_attributes(desired_attributes, ignore_errors)).await
    }

    pub async fn lacks_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_attributes(desired_attributes, ignore_errors))
            .await
    }

    pub async fn has_property<S, N>(self, property_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_property(property_name, value, ignore_errors)).await
    }

    pub async fn lacks_property<S, N>(self, property_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_property(property_name, value, ignore_errors))
            .await
    }

    pub async fn has_properties<S, N>(self, desired_properties: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_properties(desired_properties, ignore_errors)).await
    }

    pub async fn lacks_properties<S, N>(self, desired_properties: &[(S, N)]) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_properties(desired_properties, ignore_errors))
            .await
    }

    pub async fn has_css_property<S, N>(self, css_property_name: S, value: N) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_css_property(
            css_property_name,
            value,
            ignore_errors,
        ))
        .await
    }

    pub async fn lacks_css_property<S, N>(
        self,
        css_property_name: S,
        value: N,
    ) -> WebDriverResult<()>
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_css_property(
            css_property_name,
            value,
            ignore_errors,
        ))
        .await
    }

    pub async fn has_css_properties<S, N>(
        self,
        desired_css_properties: &[(S, N)],
    ) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_has_css_properties(
            desired_css_properties,
            ignore_errors,
        ))
        .await
    }

    pub async fn lacks_css_properties<S, N>(
        self,
        desired_css_properties: &[(S, N)],
    ) -> WebDriverResult<()>
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.condition(conditions::element_lacks_css_properties(
            desired_css_properties,
            ignore_errors,
        ))
        .await
    }
}

/// Trait for enabling the ElementWaiter interface.
pub trait ElementWaitable {
    fn wait_until(&self) -> ElementWaiter;
}

impl ElementWaitable for WebElement {
    /// Return an ElementWaiter instance for more executing powerful explicit waits.
    ///
    /// This uses the builder pattern to construct explicit waits using one of the
    /// provided predicates. Or you can provide your own custom predicate if desired.
    ///
    /// See [ElementWaiter](query/struct.ElementWaiter.html) for more documentation.
    fn wait_until(&self) -> ElementWaiter {
        let poller: ElementPoller = self.handle.config.get_query_poller();
        ElementWaiter::new(self.clone(), poller)
    }
}

#[cfg(test)]
/// This function checks if the public async methods implement Send. It is not intended to be executed.
async fn _test_is_send() -> WebDriverResult<()> {
    use crate::prelude::*;

    // Helper methods
    fn is_send<T: Send>() {}
    fn is_send_val<T: Send>(_val: &T) {}

    // Pre values
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;
    let elem = driver.find(By::Css(r#"div"#)).await?;

    // ElementWaitCondition
    is_send_val(&elem.wait_until().stale());
    is_send_val(&elem.wait_until().displayed());
    is_send_val(&elem.wait_until().selected());
    is_send_val(&elem.wait_until().enabled());
    is_send_val(&elem.wait_until().condition(Box::new(|elem| {
        Box::pin(async move { elem.is_enabled().await.or(Ok(false)) })
    })));

    Ok(())
}
