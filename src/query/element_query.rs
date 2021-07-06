use std::sync::Arc;
use std::time::Duration;

use crate::error::{WebDriverError, WebDriverErrorInfo};
use crate::prelude::{WebDriver, WebDriverResult};
use crate::{By, WebDriverCommands, WebDriverSession, WebElement};
use futures::Future;
use stringmatch::Needle;

use crate::query::conditions::{handle_errors, negate};
use crate::query::{conditions, ElementPoller, ElementPollerTicker, ElementPredicate};

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

    WebDriverError::NoSuchElement(WebDriverErrorInfo::new(&format!(
        "{} not found using selectors: {}",
        element_description,
        &get_selector_summary(selectors)
    )))
}

/// An ElementSelector contains a selector method (By) as well as zero or more filters.
/// The filters will be applied to any elements matched by the selector.
/// Selectors and filters all run in full on every poll iteration.
pub struct ElementSelector<'a> {
    /// If false (default), find_elements() will be used. If true, find_element() will be used
    /// instead. See notes below for `with_single_selector()` for potential pitfalls.
    pub single: bool,
    pub by: By<'a>,
    pub filters: Vec<ElementPredicate>,
}

impl<'a> ElementSelector<'a> {
    //
    // Constructor
    //

    pub fn new(by: By<'a>) -> Self {
        Self {
            single: false,
            by: by.clone(),
            filters: Vec::new(),
        }
    }

    //
    // Configurator
    //

    /// Call `set_single()` to tell this selector to use find_element() rather than
    /// find_elements(). This can be slightly faster but only really makes sense if
    /// (1) you're not using any filters and (2) you're only interested in the first
    /// element matched anyway.
    pub fn set_single(&mut self) {
        self.single = true;
    }

    /// Add the specified filter to the list of filters for this selector.
    pub fn add_filter(&mut self, f: ElementPredicate) {
        self.filters.push(f);
    }

    //
    // Runner
    //

    /// Run all filters for this selector on the specified WebElement vec.
    pub async fn run_filters<'b>(
        &self,
        mut elements: Vec<WebElement<'b>>,
    ) -> WebDriverResult<Vec<WebElement<'b>>> {
        for func in &self.filters {
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
}

/// Elements can be queried from either a WebDriver or from a WebElement.
/// The command issued to the webdriver will differ depending on the source,
/// i.e. FindElement vs FindElementFromElement etc. but the ElementQuery
/// interface is the same for both.
pub enum ElementQuerySource<'a> {
    Driver(&'a WebDriverSession),
    Element(&'a WebElement<'a>),
}

/// High-level interface for performing powerful element queries using a
/// builder pattern.
///
/// # Example:
/// ```rust
/// # use thirtyfour::prelude::*;
/// # use thirtyfour::support::block_on;
/// #
/// # fn main() -> WebDriverResult<()> {
/// #     use thirtyfour::query::ElementPoller;
/// #     block_on(async {
/// #         let caps = DesiredCapabilities::chrome();
/// #         let mut driver = WebDriver::new("http://localhost:4444/wd/hub", &caps).await?;
/// #         driver.get("http://webappdemo").await?;
/// // WebDriver::query() example.
/// let elem = driver.query(By::Css("div[data-section='section-buttons']")).first().await?;
/// // WebElement::query() example.
/// let elem_button = elem.query(By::Id("button1")).first().await?;
/// #         assert_eq!(elem_button.tag_name().await?, "button");
/// #         Ok(())
/// #     })
/// # }
/// ```
pub struct ElementQuery<'a> {
    source: Arc<ElementQuerySource<'a>>,
    poller: ElementPoller,
    selectors: Vec<ElementSelector<'a>>,
    ignore_errors: bool,
    description: String,
}

impl<'a> ElementQuery<'a> {
    //
    // Constructor
    //

    fn new(source: ElementQuerySource<'a>, poller: ElementPoller, by: By<'a>) -> Self {
        let selector = ElementSelector::new(by.clone());
        Self {
            source: Arc::new(source),
            poller,
            selectors: vec![selector],
            ignore_errors: true,
            description: String::new(),
        }
    }

    /// Provide a name that will be included in the error message if the query was not successful.
    /// This is useful for providing more context about this particular query.
    pub fn desc(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// By default a query will ignore any errors that occur while polling for the desired
    /// element(s). However, this behaviour can be modified so that the waiter will return
    /// early if an error is returned from thirtyfour.
    pub fn ignore_errors(mut self, ignore: bool) -> Self {
        self.ignore_errors = ignore;
        self
    }

    //
    // Poller / Waiter
    //

    /// Use the specified ElementPoller for this ElementQuery.
    /// This will not affect the default ElementPoller used for other queries.
    pub fn with_poller(mut self, poller: ElementPoller) -> Self {
        self.poller = poller;
        self
    }

    /// Force this ElementQuery to wait for the specified timeout, polling once
    /// after each interval. This will override the poller for this
    /// ElementQuery only.
    pub fn wait(self, timeout: Duration, interval: Duration) -> Self {
        self.with_poller(ElementPoller::TimeoutWithInterval(timeout, interval))
    }

    /// Force this ElementQuery to not wait for the specified condition(s).
    /// This will override the poller for this ElementQuery only.
    pub fn nowait(self) -> Self {
        self.with_poller(ElementPoller::NoWait)
    }

    //
    // Selectors
    //

    /// Add the specified selector to this ElementQuery. Callers should use
    /// the `or()` method instead.
    fn add_selector(mut self, selector: ElementSelector<'a>) -> Self {
        self.selectors.push(selector);
        self
    }

    /// Add a new selector to this ElementQuery. All conditions specified after
    /// this selector (up until the next `or()` method) will apply to this
    /// selector.
    pub fn or(self, by: By<'a>) -> Self {
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

    /// Return only the first WebElement that matches any selector (including all of
    /// the filters for that selector).
    pub async fn first(&self) -> WebDriverResult<WebElement<'a>> {
        let mut elements = self.run_poller(false).await?;

        if elements.is_empty() {
            Err(no_such_element(&self.selectors, &self.description))
        } else {
            Ok(elements.remove(0))
        }
    }

    /// Return all WebElements that match any one selector (including all of the
    /// filters for that selector).
    ///
    /// Returns an empty Vec if no elements match.
    pub async fn all(&self) -> WebDriverResult<Vec<WebElement<'a>>> {
        self.run_poller(false).await
    }

    /// Return all WebElements that match any one selector (including all of the
    /// filters for that selector).
    ///
    /// Returns Err(WebDriverError::NoSuchElement) if no elements match.
    pub async fn all_required(&self) -> WebDriverResult<Vec<WebElement<'a>>> {
        let elements = self.run_poller(false).await?;

        if elements.is_empty() {
            Err(no_such_element(&self.selectors, &self.description))
        } else {
            Ok(elements)
        }
    }

    //
    // Helper Retrievers
    //

    /// Run the poller for this ElementQuery and return the Vec of WebElements matched.
    /// NOTE: This function doesn't return a no_such_element error and the caller must handle it.
    async fn run_poller(&self, inverted: bool) -> WebDriverResult<Vec<WebElement<'a>>> {
        let no_such_element_error = no_such_element(&self.selectors, &self.description);
        if self.selectors.is_empty() {
            return Err(no_such_element_error);
        }
        let mut ticker = ElementPollerTicker::new(self.poller.clone());

        let check = |value: bool| {
            if inverted {
                !value
            } else {
                value
            }
        };

        loop {
            for selector in &self.selectors {
                let mut elements = match self.fetch_elements_from_source(selector).await {
                    Ok(x) => x,
                    Err(WebDriverError::NoSuchElement(_)) => Vec::new(),
                    Err(e) => return Err(e),
                };

                if !elements.is_empty() {
                    elements = selector.run_filters(elements).await?;
                }

                if check(!elements.is_empty()) {
                    return Ok(elements);
                }
            }

            if !ticker.tick().await {
                return Ok(Vec::new());
            }
        }
    }

    /// Execute the specified selector and return any matched WebElements.
    fn fetch_elements_from_source(
        &self,
        selector: &ElementSelector<'a>,
    ) -> impl Future<Output = WebDriverResult<Vec<WebElement<'a>>>> + Send {
        let by = selector.by.clone();
        let single = selector.single;
        let source = self.source.clone();
        async move {
            match single {
                true => match source.as_ref() {
                    ElementQuerySource::Driver(driver) => {
                        driver.find_element(by).await.map(|x| vec![x])
                    }
                    ElementQuerySource::Element(element) => {
                        element.find_element(by).await.map(|x| vec![x])
                    }
                },
                false => match source.as_ref() {
                    ElementQuerySource::Driver(driver) => driver.find_elements(by).await,
                    ElementQuerySource::Element(element) => element.find_elements(by).await,
                },
            }
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

    /// Set the previous selector to only return the first matched element.
    /// WARNING: Use with caution! This may result in (slightly) faster lookups, but will probably
    ///          break any filters on this selector.
    ///
    /// If you are simply want to get the first element after filtering from a list,
    /// use the `first()` method instead.
    pub fn with_single_selector(mut self) -> Self {
        if let Some(selector) = self.selectors.last_mut() {
            selector.set_single();
        }
        self
    }

    //
    // Advance selectors
    //

    /// Only match elements that are enabled.
    pub fn and_enabled(self) -> Self {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_is_enabled(ignore_errors))
    }

    /// Only match elements that are NOT enabled.
    pub fn and_not_enabled(self) -> Self {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_is_not_enabled(ignore_errors))
    }

    /// Only match elements that are selected.
    pub fn and_selected(self) -> Self {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_is_selected(ignore_errors))
    }

    /// Only match elements that are NOT selected.
    pub fn and_not_selected(self) -> Self {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_is_not_selected(ignore_errors))
    }

    /// Only match elements that are displayed.
    pub fn and_displayed(self) -> Self {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_is_displayed(ignore_errors))
    }

    /// Only match elements that are NOT displayed.
    pub fn and_not_displayed(self) -> Self {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_is_not_displayed(ignore_errors))
    }

    /// Only match elements that are clickable.
    pub fn and_clickable(self) -> Self {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_is_clickable(ignore_errors))
    }

    /// Only match elements that are NOT clickable.
    pub fn and_not_clickable(self) -> Self {
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_has_text(text, ignore_errors))
    }

    /// Only match elements that do not have the specified text.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_text<N>(self, text: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_text(text, ignore_errors))
    }

    /// Only match elements that have the specified id.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_id<N>(self, id: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_has_class(class_name, ignore_errors))
    }

    /// Only match elements that do not contain the specified class name.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_class<N>(self, class_name: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_class(class_name, ignore_errors))
    }

    /// Only match elements that have the specified tag.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_tag<N>(self, tag_name: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_has_value(value, ignore_errors))
    }

    /// Only match elements that do not have the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_value<N>(self, value: N) -> Self
    where
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_value(value, ignore_errors))
    }

    /// Only match elements that have the specified attribute with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_attribute<S, N>(self, attribute_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_has_attribute(attribute_name, value, ignore_errors))
    }

    /// Only match elements that do not have the specified attribute with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_attribute<S, N>(self, attribute_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_attribute(attribute_name, value, ignore_errors))
    }

    /// Only match elements that have all of the specified attributes with the specified values.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_has_attributes(desired_attributes, ignore_errors))
    }

    /// Only match elements that do not have any of the specified attributes with the specified
    /// values. See the `Needle` documentation for more details on text matching rules.
    pub fn without_attributes<S, N>(self, desired_attributes: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_attributes(desired_attributes, ignore_errors))
    }

    /// Only match elements that have the specified property with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_property<S, N>(self, property_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_has_property(property_name, value, ignore_errors))
    }

    /// Only match elements that do not have the specified property with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn without_property<S, N>(self, property_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_property(property_name, value, ignore_errors))
    }

    /// Only match elements that have all of the specified properties with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_properties<S, N>(self, desired_properties: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_has_properties(desired_properties, ignore_errors))
    }

    /// Only match elements that do not have any of the specified properties with the specified
    /// value. See the `Needle` documentation for more details on text matching rules.
    pub fn without_properties<S, N>(self, desired_properties: &[(S, N)]) -> Self
    where
        S: Into<String> + Clone,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_properties(desired_properties, ignore_errors))
    }

    /// Only match elements that have the specified CSS property with the specified value.
    /// See the `Needle` documentation for more details on text matching rules.
    pub fn with_css_property<S, N>(self, css_property_name: S, value: N) -> Self
    where
        S: Into<String>,
        N: Needle + Clone + Send + Sync + 'static,
    {
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
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
        let ignore_errors = self.ignore_errors;
        self.with_filter(conditions::element_lacks_css_properties(
            desired_css_properties,
            ignore_errors,
        ))
    }
}

/// Trait for enabling the ElementQuery interface.
pub trait ElementQueryable {
    fn query<'a>(&'a self, by: By<'a>) -> ElementQuery<'a>;
}

impl ElementQueryable for WebElement<'_> {
    /// Return an ElementQuery instance for more executing powerful element queries.
    ///
    /// This uses the builder pattern to construct queries that will return one or
    /// more elements, depending on the method specified at the end of the chain.
    ///
    /// See [ElementQuery](query/struct.ElementQuery.html) for more documentation.
    fn query<'a>(&'a self, by: By<'a>) -> ElementQuery<'a> {
        let poller: ElementPoller = self.session.config().query_poller.clone();
        ElementQuery::new(ElementQuerySource::Element(&self), poller, by)
    }
}

impl ElementQueryable for WebDriver {
    /// Return an ElementQuery instance for more executing powerful element queries.
    ///
    /// This uses the builder pattern to construct queries that will return one or
    /// more elements, depending on the method specified at the end of the chain.
    ///
    /// See [ElementQuery](query/struct.ElementQuery.html) for more documentation.
    fn query<'a>(&'a self, by: By<'a>) -> ElementQuery<'a> {
        let poller: ElementPoller = self.session.config().query_poller.clone();
        ElementQuery::new(ElementQuerySource::Driver(&self.session), poller, by)
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
    is_send_val(&selector.run_filters(Vec::new()));

    // Pre values
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:4444", &caps).await?;

    // ElementQuery
    let query = driver.query(By::Css("div"));
    is_send_val(&query.exists());
    is_send_val(&query.not_exists());
    is_send_val(&query.first());
    is_send_val(&query.all());
    is_send_val(&query.all_required());

    Ok(())
}
