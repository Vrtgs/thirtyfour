use super::conditions::{handle_errors, negate};
use super::{conditions, ElementPollerNoWait, ElementPollerWithTimeout, IntoElementPoller};
use crate::error::WebDriverError;
use crate::prelude::WebDriverResult;
use crate::session::handle::SessionHandle;
use crate::{By, ElementPredicate, WebElement};
use std::time::Duration;
use stringmatch::Needle;

/// Get String containing comma-separated list of selectors used.
fn get_selector_summary(selectors: &[ElementSelector]) -> String {
    let criteria: Vec<String> = selectors.iter().map(|s| s.by.to_string()).collect();
    format!("[{}]", criteria.join(","))
}

/// Helper function to return the NoSuchElement error struct.
fn no_such_element(selectors: &[ElementSelector], description: &str) -> WebDriverError {
    let element_description = if description.is_empty() {
        String::from("Element(s)")
    } else {
        format!("'{}' element(s)", description)
    };

    WebDriverError::NoSuchElement(format!(
        "{} not found using selectors: {}",
        element_description,
        get_selector_summary(selectors)
    ))
}

pub async fn filter_elements(
    mut elements: Vec<WebElement>,
    filters: &[ElementPredicate],
) -> WebDriverResult<Vec<WebElement>> {
    for func in filters {
        let tmp_elements = std::mem::take(&mut elements);
        for element in tmp_elements {
            if func(&element).await? {
                elements.push(element);
            }
        }

        if elements.is_empty() {
            break;
        }
    }

    Ok(elements)
}

/// An ElementSelector contains a selector method (By) as well as zero or more filters.
/// The filters will be applied to any elements matched by the selector.
/// Selectors and filters all run in full on every poll iteration.
pub struct ElementSelector {
    pub by: By,
    pub filters: Vec<ElementPredicate>,
}

impl ElementSelector {
    pub fn new(by: By) -> Self {
        Self {
            by,
            filters: Vec::new(),
        }
    }

    /// Add the specified filter to the list of filters for this selector.
    pub fn add_filter(&mut self, f: ElementPredicate) {
        self.filters.push(f);
    }
}

/// Elements can be queried from either a WebDriver or from a WebElement.
/// The command issued to the webdriver will differ depending on the source,
/// i.e. FindElement vs FindElementFromElement etc. but the ElementQuery
/// interface is the same for both.
pub enum ElementQuerySource {
    Driver(SessionHandle),
    Element(WebElement),
}

/// Options for wait characteristics for an element query.
#[derive(Debug, Clone)]
pub enum ElementQueryWaitOptions {
    /// Use the default poller.
    WaitDefault,
    /// Use a poller with the specified timeout and interval.
    Wait {
        timeout: Duration,
        interval: Duration,
    },
    /// Do not wait. This uses a poller that quits immediately.
    NoWait,
}

impl Default for ElementQueryWaitOptions {
    fn default() -> Self {
        Self::WaitDefault
    }
}

/// All options applicable to an ElementQuery.
///
/// These are stored in a separate struct so that they can be constructed
/// separately and applied to an ElementQuery in bulk if required.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct ElementQueryOptions {
    ignore_errors: Option<bool>,
    description: Option<String>,
    wait: Option<ElementQueryWaitOptions>,
}

impl ElementQueryOptions {
    /// Set whether to ignore errors when querying elements.
    pub fn ignore_errors(mut self, ignore_errors: bool) -> Self {
        self.ignore_errors = Some(ignore_errors);
        self
    }

    /// Set whether to ignore errors when querying elements.
    pub fn set_ignore_errors(mut self, ignore_errors: Option<bool>) -> Self {
        self.ignore_errors = ignore_errors;
        self
    }

    /// Set the description to be used in error messages for this element query.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the description to be used in error messages for this element query.
    pub fn set_description<T: Into<String>>(mut self, description: Option<T>) -> Self {
        self.description = description.map(|x| x.into());
        self
    }

    /// Set the wait options for this element query.
    pub fn wait(mut self, wait_option: ElementQueryWaitOptions) -> Self {
        self.wait = Some(wait_option);
        self
    }

    /// Set the wait options for this element query.
    pub fn set_wait(mut self, wait_option: Option<ElementQueryWaitOptions>) -> Self {
        self.wait = wait_option;
        self
    }
}

/// High-level interface for performing powerful element queries using a
/// builder pattern.
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
/// #         driver.goto("http://localhost:8000").await?;
/// // WebDriver::query() example.
/// let elem = driver.query(By::Css("div[data-section='section-buttons']")).first().await?;
/// // WebElement::query() example.
/// let elem_button = elem.query(By::Id("button1")).first().await?;
/// #         assert_eq!(elem_button.tag_name().await?, "button");
/// #         driver.quit().await?;
/// #         Ok(())
/// #     })
/// # }
/// ```
pub struct ElementQuery {
    source: ElementQuerySource,
    poller: Option<Box<dyn IntoElementPoller + Send + Sync>>,
    selectors: Vec<ElementSelector>,
    options: ElementQueryOptions,
}

impl ElementQuery {
    fn new(source: ElementQuerySource, by: By) -> Self {
        let selector = ElementSelector::new(by);
        Self {
            source,
            poller: None,
            selectors: vec![selector],
            options: ElementQueryOptions::default(),
        }
    }

    /// Provide the options to use with this query.
    pub fn options(mut self, options: ElementQueryOptions) -> Self {
        self.options = options;

        // Apply wait options.
        match self.options.wait {
            None | Some(ElementQueryWaitOptions::WaitDefault) => self,
            Some(ElementQueryWaitOptions::Wait {
                timeout,
                interval,
            }) => self.wait(timeout, interval),
            Some(ElementQueryWaitOptions::NoWait) => self.nowait(),
        }
    }

    /// Provide a name that will be included in the error message if the query was not successful.
    /// This is useful for providing more context about this particular query.
    pub fn desc(mut self, description: &str) -> Self {
        self.options = self.options.description(description);
        self
    }

    /// By default a query will ignore any errors that occur while polling for the desired
    /// element(s). However, this behaviour can be modified so that the waiter will return
    /// early if an error is returned from thirtyfour.
    pub fn ignore_errors(mut self, ignore: bool) -> Self {
        self.options = self.options.ignore_errors(ignore);
        self
    }

    //
    // Poller / Waiter
    //

    /// Use the specified ElementPoller for this ElementQuery.
    /// This will not affect the default ElementPoller used for other queries.
    pub fn with_poller(mut self, poller: Box<dyn IntoElementPoller + Send + Sync>) -> Self {
        self.poller = Some(poller);
        self
    }

    /// Force this ElementQuery to wait for the specified timeout, polling once
    /// after each interval. This will override the poller for this
    /// ElementQuery only.
    pub fn wait(self, timeout: Duration, interval: Duration) -> Self {
        self.with_poller(Box::new(ElementPollerWithTimeout::new(timeout, interval)))
    }

    /// Force this ElementQuery to not wait for the specified condition(s).
    /// This will override the poller for this ElementQuery only.
    pub fn nowait(self) -> Self {
        self.with_poller(Box::new(ElementPollerNoWait))
    }

    //
    // Selectors
    //

    /// Add the specified selector to this ElementQuery. Callers should use
    /// the `or()` method instead.
    fn add_selector(mut self, selector: ElementSelector) -> Self {
        self.selectors.push(selector);
        self
    }

    /// Add a new selector to this ElementQuery. All conditions specified after
    /// this selector (up until the next `or()` method) will apply to this
    /// selector.
    pub fn or(self, by: By) -> Self {
        self.add_selector(ElementSelector::new(by))
    }

    //
    // Retrievers
    //

    /// Return true if an element matches any selector, otherwise false.
    pub async fn exists(&self) -> WebDriverResult<bool> {
        let elements = self.run_poller(false).await?;
        Ok(!elements.is_empty())
    }

    /// Return true if no element matches any selector, otherwise false.
    pub async fn not_exists(&self) -> WebDriverResult<bool> {
        let elements = self.run_poller(true).await?;
        Ok(elements.is_empty())
    }

    /// Return the first WebElement that matches any selector (including filters).
    ///
    /// Returns None if no elements match.
    pub async fn first_opt(&self) -> WebDriverResult<Option<WebElement>> {
        let elements = self.run_poller(false).await?;
        Ok(elements.into_iter().next())
    }

    /// Return only the first WebElement that matches any selector (including filters).
    ///
    /// Returns Err(WebDriverError::NoSuchElement) if no elements match.
    pub async fn first(&self) -> WebDriverResult<WebElement> {
        let mut elements = self.run_poller(false).await?;

        if elements.is_empty() {
            let desc: &str = self.options.description.as_deref().unwrap_or("");
            let err = no_such_element(&self.selectors, desc);
            Err(err)
        } else {
            Ok(elements.remove(0))
        }
    }

    /// Return only a single WebElement that matches any selector (including filters).
    ///
    /// This method requires that only one element was found, and will return
    /// Err(WebDriverError::NoSuchElement) if the number of elements found was not
    /// equal to 1.
    pub async fn single(&self) -> WebDriverResult<WebElement> {
        let mut elements = self.run_poller(false).await?;

        if elements.len() == 1 {
            Ok(elements.remove(0))
        } else {
            let desc: &str = self.options.description.as_deref().unwrap_or("");
            let err = no_such_element(&self.selectors, desc);
            Err(err)
        }
    }

    /// Return all WebElements that match any one selector (including filters).
    ///
    /// Returns an empty Vec if no elements match.
    pub async fn all(&self) -> WebDriverResult<Vec<WebElement>> {
        self.run_poller(false).await
    }

    /// Return all WebElements that match any one selector (including filters).
    ///
    /// Returns Err(WebDriverError::NoSuchElement) if no elements match.
    pub async fn all_required(&self) -> WebDriverResult<Vec<WebElement>> {
        let elements = self.run_poller(false).await?;

        if elements.is_empty() {
            let desc: &str = self.options.description.as_deref().unwrap_or("");
            Err(no_such_element(&self.selectors, desc))
        } else {
            Ok(elements)
        }
    }

    //
    // Helper Retrievers
    //

    /// Run the poller for this ElementQuery and return the Vec of WebElements matched.
    /// NOTE: This function doesn't return a no_such_element error and the caller must handle it.
    async fn run_poller(&self, inverted: bool) -> WebDriverResult<Vec<WebElement>> {
        let desc: &str = self.options.description.as_deref().unwrap_or("");
        let no_such_element_error = no_such_element(&self.selectors, desc);
        if self.selectors.is_empty() {
            return Err(no_such_element_error);
        }

        let check = |value: bool| {
            if inverted {
                !value
            } else {
                value
            }
        };

        // Start the poller.
        let mut poller = match &self.poller {
            Some(p) => p.start(),
            None => Box::new(ElementPollerWithTimeout::default()).start(),
        };

        loop {
            for selector in &self.selectors {
                let mut elements = match self.fetch_elements_from_source(selector.by.clone()).await
                {
                    Ok(x) => x,
                    Err(WebDriverError::NoSuchElement(_)) => Vec::new(),
                    Err(e) => return Err(e),
                };

                if !elements.is_empty() {
                    elements = filter_elements(elements, &selector.filters).await?;
                }

                if check(!elements.is_empty()) {
                    return Ok(elements);
                }
            }

            if !poller.tick().await {
                return Ok(Vec::new());
            }
        }
    }

    /// Execute the specified selector and return any matched WebElements.
    async fn fetch_elements_from_source(&self, by: By) -> WebDriverResult<Vec<WebElement>> {
        match &self.source {
            ElementQuerySource::Driver(driver) => driver.find_all(by).await,
            ElementQuerySource::Element(element) => element.find_all(by).await,
        }
    }

    //
    // Filters
    //

    /// Add the specified ElementPredicate to the last selector.
    pub fn with_filter(mut self, f: ElementPredicate) -> Self {
        if let Some(selector) = self.selectors.last_mut() {
            selector.add_filter(f);
        }
        self
    }

    //
    // Advance selectors
    //

    /// Only match elements that are enabled.
    pub fn and_enabled(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_enabled(ignore_errors))
    }

    /// Only match elements that are NOT enabled.
    pub fn and_not_enabled(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_not_enabled(ignore_errors))
    }

    /// Only match elements that are selected.
    pub fn and_selected(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_selected(ignore_errors))
    }

    /// Only match elements that are NOT selected.
    pub fn and_not_selected(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_not_selected(ignore_errors))
    }

    /// Only match elements that are displayed.
    pub fn and_displayed(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_displayed(ignore_errors))
    }

    /// Only match elements that are NOT displayed.
    pub fn and_not_displayed(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_not_displayed(ignore_errors))
    }

    /// Only match elements that are clickable.
    pub fn and_clickable(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_clickable(ignore_errors))
    }

    /// Only match elements that are NOT clickable.
    pub fn and_not_clickable(self) -> Self {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_is_not_clickable(ignore_errors))
    }

    //
    // By alternative helper selectors
    //

    /// Only match elements that have the specified text.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_text<N>(self, text: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_text(text, ignore_errors))
    }

    /// Only match elements that do not have the specified text.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_text<N>(self, text: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_text(text, ignore_errors))
    }

    /// Only match elements that have the specified id.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_id<N>(self, id: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(Box::new(move |elem| {
            let id = id.clone();
            Box::pin(async move {
                match elem.id().await {
                    Ok(Some(x)) => Ok(id.is_match(&x)),
                    Ok(None) => Ok(false),
                    Err(e) => handle_errors(Err(e), ignore_errors),
                }
            })
        }))
    }

    /// Only match elements that do not have the specified id.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_id<N>(self, id: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(Box::new(move |elem| {
            let id = id.clone();
            Box::pin(async move {
                match elem.id().await {
                    Ok(Some(x)) => Ok(!id.is_match(&x)),
                    Ok(None) => Ok(true),
                    Err(e) => handle_errors(Err(e), ignore_errors),
                }
            })
        }))
    }

    /// Only match elements that contain the specified class name.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_class<N>(self, class_name: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_class(class_name, ignore_errors))
    }

    /// Only match elements that do not contain the specified class name.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_class<N>(self, class_name: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_class(class_name, ignore_errors))
    }

    /// Only match elements that have the specified tag.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_tag<N>(self, tag_name: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(Box::new(move |elem| {
            let tag_name = tag_name.clone();
            Box::pin(async move {
                handle_errors(elem.tag_name().await.map(|x| tag_name.is_match(&x)), ignore_errors)
            })
        }))
    }

    /// Only match elements that do not have the specified tag.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_tag<N>(self, tag_name: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(Box::new(move |elem| {
            let tag_name = tag_name.clone();
            Box::pin(async move {
                negate(elem.tag_name().await.map(|x| tag_name.is_match(&x)), ignore_errors)
            })
        }))
    }

    /// Only match elements that have the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_value<N>(self, value: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_value(value, ignore_errors))
    }

    /// Only match elements that do not have the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_value<N>(self, value: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_value(value, ignore_errors))
    }

    /// Only match elements that have the specified attribute with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_attribute<S, N>(self, attribute_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_attribute(attribute_name, value, ignore_errors))
    }

    /// Only match elements that do not have the specified attribute with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_attribute<S, N>(self, attribute_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_attribute(attribute_name, value, ignore_errors))
    }

    /// Only match elements that have all of the specified attributes with the specified values.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_attributes(desired_attributes, ignore_errors))
    }

    /// Only match elements that do not have any of the specified attributes with the specified
    /// values. See the `Needle` documentation for more details on text matching rules.
    pub fn without_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_attributes(desired_attributes, ignore_errors))
    }

    /// Only match elements that have the specified property with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_property<S, N>(self, property_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_property(property_name, value, ignore_errors))
    }

    /// Only match elements that do not have the specified property with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_property<S, N>(self, property_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_property(property_name, value, ignore_errors))
    }

    /// Only match elements that have all of the specified properties with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_properties<S, N>(self, desired_properties: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_properties(desired_properties, ignore_errors))
    }

    /// Only match elements that do not have any of the specified properties with the specified
    /// value. See the `Needle` documentation for more details on text matching rules.
    pub fn without_properties<S, N>(self, desired_properties: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_properties(desired_properties, ignore_errors))
    }

    /// Only match elements that have the specified CSS property with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_css_property<S, N>(self, css_property_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_css_property(
            css_property_name,
            value,
            ignore_errors,
        ))
    }

    /// Only match elements that do not have the specified CSS property with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_css_property<S, N>(self, css_property_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_css_property(
            css_property_name,
            value,
            ignore_errors,
        ))
    }

    /// Only match elements that have all of the specified CSS properties with the
    /// specified values.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_css_properties<S, N>(self, desired_css_properties: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_has_css_properties(
            desired_css_properties,
            ignore_errors,
        ))
    }

    /// Only match elements that do not have any of the specified CSS properties with the
    /// specified values.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_css_properties<S, N>(self, desired_css_properties: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.options.ignore_errors.unwrap_or_default();
        self.with_filter(conditions::element_lacks_css_properties(
            desired_css_properties,
            ignore_errors,
        ))
    }
}

/// Trait for enabling the ElementQuery interface.
pub trait ElementQueryable {
    fn query(&self, by: By) -> ElementQuery;
}

impl ElementQueryable for WebElement {
    /// Return an ElementQuery instance for more executing powerful element queries.
    ///
    /// This uses the builder pattern to construct queries that will return one or
    /// more elements, depending on the method specified at the end of the chain.
    ///
    /// See [`ElementQuery`] for more documentation.
    fn query(&self, by: By) -> ElementQuery {
        ElementQuery::new(ElementQuerySource::Element(self.clone()), by)
    }
}

impl ElementQueryable for SessionHandle {
    /// Return an ElementQuery instance for more executing powerful element queries.
    ///
    /// This uses the builder pattern to construct queries that will return one or
    /// more elements, depending on the method specified at the end of the chain.
    ///
    /// See [`ElementQuery`] for more documentation.
    fn query(&self, by: By) -> ElementQuery {
        ElementQuery::new(ElementQuerySource::Driver(self.clone()), by)
    }
}

#[cfg(test)]
/// This function checks if the public async methods implement Send. It is not intended to be executed.
async fn _test_is_send() -> WebDriverResult<()> {
    use crate::prelude::*;

    // Helper methods
    fn is_send<T: Send>() {}
    fn is_send_val<T: Send>(_val: &T) {}

    // ElementSelector
    let selector = ElementSelector::new(By::Css("div"));
    is_send_val(&filter_elements(Vec::new(), &selector.filters));

    // Pre values
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // ElementQuery
    let query = driver.query(By::Css("div"));
    is_send_val(&query.exists());
    is_send_val(&query.not_exists());
    is_send_val(&query.first());
    is_send_val(&query.all());
    is_send_val(&query.all_required());

    Ok(())
}
