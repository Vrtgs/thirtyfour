use crate::components::Component;
use crate::error::WebDriverResult;
use crate::extensions::query::ElementQueryOptions;
use crate::prelude::ElementQueryable;
use crate::{By, ElementQueryFn, WebElement};
use parking_lot::Mutex;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// Type alias for `ElementResolver<WebElement>`, for convenience.
pub type ElementResolverSingle = ElementResolver<WebElement>;
/// Type alias for `ElementResolver<Vec<WebElement>>` for convenience.
pub type ElementResolverMulti = ElementResolver<Vec<WebElement>>;

/// `resolve!(x)` expands to `x.resolve().await?`
#[macro_export]
macro_rules! resolve {
    ($a:expr) => {
        $a.resolve().await?
    };
}

/// `resolve_present!(x)` expands to `x.resolve_present().await?`
#[macro_export]
macro_rules! resolve_present {
    ($a:expr) => {
        $a.resolve_present().await?
    };
}

/// Element resolver that can resolve a particular element or list of elements on demand.
///
/// Once resolved, the result will be cached for later retrieval until manually invalidated.
pub struct ElementResolver<T: Clone> {
    base_element: WebElement,
    query_fn: Arc<ElementQueryFn<T>>,
    element: Arc<Mutex<Option<T>>>,
}

impl<T: Debug + Clone> Debug for ElementResolver<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let element = self.element.lock();
        f.debug_struct("ElementResolver")
            .field("base_element", &self.base_element)
            .field("element", &element)
            .finish()
    }
}

impl<T: Clone> Clone for ElementResolver<T> {
    fn clone(&self) -> Self {
        Self {
            base_element: self.base_element.clone(),
            query_fn: self.query_fn.clone(),
            element: self.element.clone(),
        }
    }
}

impl<T: Clone> ElementResolver<T> {
    fn peek(&self) -> Option<T> {
        self.element.lock().clone()
    }

    fn replace(&self, new: T) {
        let mut element = self.element.lock();
        element.replace(new);
    }

    /// Return the cached element(s) if any, otherwise run the query and return the result.
    pub async fn resolve(&self) -> WebDriverResult<T> {
        {
            let element = self.element.lock();
            if let Some(elem) = element.as_ref() {
                return Ok(elem.clone());
            }
        }

        let elem_fut = (self.query_fn)(&self.base_element);
        let elem = elem_fut.await?;
        self.replace(elem.clone());
        Ok(elem)
    }

    /// Invalidate any cached element(s).
    pub fn invalidate(&self) {
        self.element.lock().take();
    }

    /// Run the query, ignoring any cached element(s).
    pub async fn resolve_force(&self) -> Option<T> {
        self.invalidate();
        self.resolve().await.ok()
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
            query_fn: Arc::new(custom_resolver_fn),
            element: Arc::new(Mutex::new(None)),
        }
    }

    /// Validate that the cached element is present, and if so, return it.
    pub async fn validate(&self) -> WebDriverResult<Option<WebElement>> {
        match self.peek() {
            Some(elem) => match elem.is_present().await? {
                true => Ok(Some(elem)),
                false => {
                    self.invalidate();
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }

    /// Validate the element and repeat the query if it is not present, returning the result.
    ///
    /// If the element is already present, the cached element will be returned without
    /// performing an additional query.
    pub async fn resolve_present(&self) -> WebDriverResult<WebElement> {
        match self.validate().await? {
            Some(elem) => Ok(elem),
            None => {
                let elem_fut = (self.query_fn)(&self.base_element);
                let elem = elem_fut.await?;
                self.replace(elem.clone());
                Ok(elem)
            }
        }
    }
}

impl ElementResolver<Vec<WebElement>> {
    /// Create new element resolver that returns all elements, if any.
    ///
    /// If no elements were found, this will resolve to an empty Vec.
    pub fn new_allow_empty(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<Vec<WebElement>> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move { elem.query(by).all_from_selector().await })
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
            Box::pin(async move { elem.query(by).options(options).all_from_selector().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns at least one element.
    ///
    /// If no elements were found, a NoSuchElement error will be returned by the resolver's
    /// `resolve()` method.
    pub fn new_not_empty(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<Vec<WebElement>> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move { elem.query(by).all_from_selector_required().await })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns at least one element, with extra options.
    ///
    /// If no elements were found, a NoSuchElement error will be returned by the resolver's
    /// `resolve()` method.
    pub fn new_not_empty_opts(
        base_element: WebElement,
        by: By,
        options: ElementQueryOptions,
    ) -> Self {
        let resolver: ElementQueryFn<Vec<WebElement>> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(
                async move { elem.query(by).options(options).all_from_selector_required().await },
            )
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
            query_fn: Arc::new(custom_resolver_fn),
            element: Arc::new(Mutex::new(None)),
        }
    }

    /// Validate that all cached elements are present, if any.
    pub async fn validate(&self) -> WebDriverResult<Option<Vec<WebElement>>> {
        match self.peek() {
            Some(elems) => {
                for elem in &elems {
                    if !elem.is_present().await? {
                        self.invalidate();
                        return Ok(None);
                    }
                }
                Ok(Some(elems))
            }
            None => Ok(None),
        }
    }

    /// Validate all elements and repeat the query if any are not present, returning the results.
    ///
    /// If all elements are already present, the cached elements will be returned without
    /// performing an additional query.
    pub async fn resolve_present(&self) -> WebDriverResult<Vec<WebElement>> {
        match self.validate().await? {
            Some(elem) => Ok(elem),
            None => {
                let elem_fut = (self.query_fn)(&self.base_element);
                let elem = elem_fut.await?;
                self.replace(elem.clone());
                Ok(elem)
            }
        }
    }
}

impl<T: Component + Clone> ElementResolver<T> {
    /// Create new element resolver that must return a single component.
    pub fn new_single(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<T> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move {
                let elem = elem.query(by).single().await?;
                Ok(elem.into())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that must return a single component, with extra options.
    pub fn new_single_opts(base_element: WebElement, by: By, options: ElementQueryOptions) -> Self {
        let resolver: ElementQueryFn<T> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move {
                let elem = elem.query(by).options(options).single().await?;
                Ok(elem.into())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns the first component.
    pub fn new_first(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<T> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move {
                let elem = elem.query(by).first().await?;
                Ok(elem.into())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns the first component, with extra options.
    pub fn new_first_opts(base_element: WebElement, by: By, options: ElementQueryOptions) -> Self {
        let resolver: ElementQueryFn<T> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move {
                let elem = elem.query(by).options(options).first().await?;
                Ok(elem.into())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new component resolver using custom resolver function.
    pub fn new_custom(base_element: WebElement, custom_resolver_fn: ElementQueryFn<T>) -> Self {
        Self {
            base_element,
            query_fn: Arc::new(custom_resolver_fn),
            element: Arc::new(Mutex::new(None)),
        }
    }

    /// Validate that the cached component is present, and if so, return it.
    pub async fn validate(&self) -> WebDriverResult<Option<T>> {
        match self.peek() {
            Some(component) => match component.base_element().is_present().await? {
                true => Ok(Some(component)),
                false => {
                    self.invalidate();
                    Ok(None)
                }
            },
            None => Ok(None),
        }
    }

    /// Validate the component and repeat the query if it is not present, returning the result.
    ///
    /// If the component is already present, the cached component will be returned without
    /// performing an additional query.
    pub async fn resolve_present(&self) -> WebDriverResult<T> {
        match self.validate().await? {
            Some(component) => Ok(component),
            None => {
                let comp_fut = (self.query_fn)(&self.base_element);
                let comp = comp_fut.await?;
                self.replace(comp.clone());
                Ok(comp)
            }
        }
    }
}

impl<T: Component + Clone> ElementResolver<Vec<T>> {
    /// Create new element resolver that returns all components, if any.
    ///
    /// If no components were found, this will resolve to an empty Vec.
    pub fn new_allow_empty(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<Vec<T>> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move {
                let elems = elem.query(by).all_from_selector().await?;
                Ok(elems.into_iter().map(T::from).collect())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns all components (if any), with extra options.
    pub fn new_allow_empty_opts(
        base_element: WebElement,
        by: By,
        options: ElementQueryOptions,
    ) -> Self {
        let resolver: ElementQueryFn<Vec<T>> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move {
                let elems = elem.query(by).options(options).all_from_selector().await?;
                Ok(elems.into_iter().map(T::from).collect())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns at least one component.
    ///
    /// If no components were found, a NoSuchElement error will be returned by the resolver's
    /// `resolve()` method.
    pub fn new_not_empty(base_element: WebElement, by: By) -> Self {
        let resolver: ElementQueryFn<Vec<T>> = Box::new(move |elem| {
            let by = by.clone();
            Box::pin(async move {
                let elems = elem.query(by).all_from_selector_required().await?;
                Ok(elems.into_iter().map(T::from).collect())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new element resolver that returns at least one component, with extra options.
    ///
    /// If no components were found, a NoSuchElement error will be returned by the resolver's
    /// `resolve()` method.
    pub fn new_not_empty_opts(
        base_element: WebElement,
        by: By,
        options: ElementQueryOptions,
    ) -> Self {
        let resolver: ElementQueryFn<Vec<T>> = Box::new(move |elem| {
            let by = by.clone();
            let options = options.clone();
            Box::pin(async move {
                let elems = elem.query(by).options(options).all_from_selector_required().await?;
                Ok(elems.into_iter().map(T::from).collect())
            })
        });
        Self::new_custom(base_element, resolver)
    }

    /// Create new multi component resolver using a custom resolver function.
    pub fn new_custom(
        base_element: WebElement,
        custom_resolver_fn: ElementQueryFn<Vec<T>>,
    ) -> Self {
        Self {
            base_element,
            query_fn: Arc::new(custom_resolver_fn),
            element: Arc::new(Mutex::new(None)),
        }
    }

    /// Validate that all cached components are present, if any.
    pub async fn validate(&self) -> WebDriverResult<Option<Vec<T>>> {
        match self.peek() {
            Some(comps) => {
                for comp in &comps {
                    if !comp.base_element().is_present().await? {
                        self.invalidate();
                        return Ok(None);
                    }
                }
                Ok(Some(comps))
            }
            None => Ok(None),
        }
    }

    /// Validate all components and repeat the query if any are not present, returning the results.
    ///
    /// If all components are already present, the cached components will be returned without
    /// performing an additional query.
    pub async fn resolve_present(&self) -> WebDriverResult<Vec<T>> {
        match self.validate().await? {
            Some(comp) => Ok(comp),
            None => {
                let comp_fut = (self.query_fn)(&self.base_element);
                let comp = comp_fut.await?;
                self.replace(comp.clone());
                Ok(comp)
            }
        }
    }
}
