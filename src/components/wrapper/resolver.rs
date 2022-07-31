use crate::error::WebDriverResult;
use crate::extensions::query::ElementQueryOptions;
use crate::prelude::ElementQueryable;
use crate::{By, ElementQueryFn, WebElement};
use std::fmt::{Debug, Formatter};

pub type ElementResolverSingle = ElementResolver<WebElement>;
pub type ElementResolverMulti = ElementResolver<Vec<WebElement>>;

/// Element resolver that can resolve a particular element or list of elements on demand.
///
/// Once resolved, the result will be cached for later retrieval until manually invalidated.
pub struct ElementResolver<T> {
    base_element: WebElement,
    query_fn: ElementQueryFn<T>,
    element: Option<T>,
}

impl<T: Debug> Debug for ElementResolver<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementResolver")
            .field("base_element", &self.base_element)
            .field("element", &self.element)
            .finish()
    }
}

impl<T> ElementResolver<T> {
    /// Return the cached element(s) if any, otherwise run the query and return the result.
    pub async fn query(&mut self) -> WebDriverResult<&T> {
        if self.element.is_none() {
            let elem_fut = (self.query_fn)(&self.base_element);
            let elem = elem_fut.await?;
            self.element.replace(elem);
        }
        Ok(self.element.as_ref().expect("cached element not found"))
    }

    /// Invalidate any cached element(s).
    pub fn invalidate(&mut self) {
        self.element = None;
    }

    /// Run the query, ignoring any cached element(s).
    pub async fn requery(&mut self) -> Option<&T> {
        self.invalidate();
        self.query().await.ok()
    }
}

impl ElementResolver<WebElement> {
    /// Create new element resolver that must return a single element.
    pub fn new_single(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<WebElement> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move { elem.query(by).single().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that must return a single element, with extra options.
    pub fn new_single_opts(base_element: WebElement, by: By, options: ElementQueryOptions) -> Self {
        let resolver: ElementQueryFn<WebElement> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move { elem.query(by).options(options).single().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns the first element.
    pub fn new_first(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<WebElement> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move { elem.query(by).first().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns the first element, with extra options.
    pub fn new_first_opts(base_element: WebElement, by: By, options: ElementQueryOptions) -> Self {
        let resolver: ElementQueryFn<WebElement> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move { elem.query(by).options(options).first().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver using custom resolver function.
    pub fn new_custom(
        base_element: WebElement,
        custom_resolver_fn: ElementQueryFn<WebElement>,
    ) -> Self {
        Self {
            base_element,
            query_fn: custom_resolver_fn,
            element: None,
        }
    }

    /// Validate that the cached element is present, if any.
    pub async fn validate(&mut self) -> WebDriverResult<bool> {
        let mut set_invalid = false;
        if let Some(elem) = &self.element {
            if !elem.is_present().await? {
                set_invalid = true;
            }
        }

        if set_invalid {
            self.invalidate();
        }
        Ok(self.element.is_some())
    }

    /// Validate the element and repeat the query if it is not present, returning the result.
    ///
    /// If the element is already present, the cached element will be returned without
    /// performing an additional query.
    pub async fn query_checked(&mut self) -> WebDriverResult<&WebElement> {
        if !self.validate().await? {
            let elem_fut = (self.query_fn)(&self.base_element);
            let elem = elem_fut.await?;
            self.element.replace(elem);
        }
        Ok(self.element.as_ref().expect("cached element not found"))
    }
}

impl ElementResolver<Vec<WebElement>> {
    /// Create new element resolver that returns all elements, if any.
    ///
    /// If no elements were found, this will resolve to an empty Vec.
    pub fn new_allow_empty(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<Vec<WebElement>> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move { elem.query(by).all().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns all elements (if any), with extra options.
    pub fn new_allow_empty_opts(
        base_element: WebElement,
        by: By,
        options: ElementQueryOptions,
    ) -> Self {
        let resolver: ElementQueryFn<Vec<WebElement>> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move { elem.query(by).options(options).all().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns at least one element.
    ///
    /// If no elements were found, a NoSuchElement error will be returned by the resolver's
    /// `query()` method.
    pub fn new_not_empty(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<Vec<WebElement>> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move { elem.query(by).all_required().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns at least one element, with extra options.
    ///
    /// If no elements were found, a NoSuchElement error will be returned by the resolver's
    /// `query()` method.
    pub fn new_not_empty_opts(
        base_element: WebElement,
        by: By,
        options: ElementQueryOptions,
    ) -> Self {
        let resolver: ElementQueryFn<Vec<WebElement>> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move { elem.query(by).options(options).all_required().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new multi element resolver using a custom resolver function.
    pub fn new_custom(
        base_element: WebElement,
        custom_resolver_fn: ElementQueryFn<Vec<WebElement>>,
    ) -> Self {
        Self {
            base_element,
            query_fn: custom_resolver_fn,
            element: None,
        }
    }

    /// Validate that all cached elements are present, if any.
    pub async fn validate(&mut self) -> WebDriverResult<bool> {
        let mut set_invalid = false;
        if let Some(elems) = &self.element {
            for elem in elems {
                if !elem.is_present().await? {
                    set_invalid = true;
                    break;
                }
            }
        }

        if set_invalid {
            self.invalidate();
        }
        Ok(self.element.is_some())
    }

    /// Validate all elements and repeat the query if any are not present, returning the results.
    ///
    /// If all elements are already present, the cached elements will be returned without
    /// performing an additional query.
    pub async fn query_checked(&mut self) -> WebDriverResult<&Vec<WebElement>> {
        if !self.validate().await? {
            let elem_fut = (self.query_fn)(&self.base_element);
            let elem = elem_fut.await?;
            self.element.replace(elem);
        }
        Ok(self.element.as_ref().expect("cached elements not found"))
    }
}
