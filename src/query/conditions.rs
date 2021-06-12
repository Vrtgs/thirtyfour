use crate::error::WebDriverResult;
use crate::query::ElementPredicate;
use stringmatch::Needle;

pub(crate) fn handle_errors(
    result: WebDriverResult<bool>,
    ignore_errors: bool,
) -> WebDriverResult<bool> {
    match result {
        Ok(x) => Ok(x),
        Err(e) => {
            if ignore_errors {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

pub(crate) fn negate(result: WebDriverResult<bool>, ignore_errors: bool) -> WebDriverResult<bool> {
    handle_errors(result.map(|x| !x), ignore_errors)
}

/// Predicate that returns true for elements that are enabled.
pub fn element_is_enabled(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| {
        Box::pin(async move { handle_errors(elem.is_enabled().await, ignore_errors) })
    })
}

/// Predicate that returns true for elements that are not enabled.
pub fn element_is_not_enabled(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| Box::pin(async move { negate(elem.is_enabled().await, ignore_errors) }))
}

/// Predicate that returns true for elements that are selected.
pub fn element_is_selected(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| {
        Box::pin(async move { handle_errors(elem.is_selected().await, ignore_errors) })
    })
}

/// Predicate that returns true for elements that are not selected.
pub fn element_is_not_selected(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| Box::pin(async move { negate(elem.is_selected().await, ignore_errors) }))
}

/// Predicate that returns true for elements that are displayed.
pub fn element_is_displayed(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| {
        Box::pin(async move { handle_errors(elem.is_displayed().await, ignore_errors) })
    })
}

/// Predicate that returns true for elements that are not displayed.
pub fn element_is_not_displayed(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| Box::pin(async move { negate(elem.is_displayed().await, ignore_errors) }))
}

/// Predicate that returns true for elements that are clickable.
pub fn element_is_clickable(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| {
        Box::pin(async move { handle_errors(elem.is_clickable().await, ignore_errors) })
    })
}

/// Predicate that returns true for elements that are not clickable.
pub fn element_is_not_clickable(ignore_errors: bool) -> ElementPredicate {
    Box::new(move |elem| Box::pin(async move { negate(elem.is_clickable().await, ignore_errors) }))
}

/// Predicate that returns true for elements that have the specified class name.
/// See the `Needle` documentation for more details on text matching rules.
/// In particular, it is recommended to use StringMatch or Regex to perform a whole-word search.
pub fn element_has_class<N>(class_name: N, ignore_errors: bool) -> ElementPredicate
where
    N: Needle + Clone + Send + Sync + 'static,
{
    Box::new(move |elem| {
        let class_name = class_name.clone();
        Box::pin(async move {
            match elem.class_name().await {
                Ok(Some(x)) => Ok(class_name.is_match(&x)),
                Ok(None) => Ok(false),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that do not contain the specified class name.
/// See the `Needle` documentation for more details on text matching rules.
/// In particular, it is recommended to use StringMatch or Regex to perform a whole-word search.
pub fn element_lacks_class<N>(class_name: N, ignore_errors: bool) -> ElementPredicate
where
    N: Needle + Clone + Send + Sync + 'static,
{
    Box::new(move |elem| {
        let class_name = class_name.clone();
        Box::pin(async move {
            match elem.class_name().await {
                Ok(Some(x)) => Ok(!class_name.is_match(&x)),
                Ok(None) => Ok(true),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that have the specified text.
/// See the `Needle` documentation for more details on text matching rules.
pub fn element_has_text<N>(text: N, ignore_errors: bool) -> ElementPredicate
where
    N: Needle + Clone + Send + Sync + 'static,
{
    Box::new(move |elem| {
        let text = text.clone();
        Box::pin(async move {
            handle_errors(elem.text().await.map(|x| text.is_match(&x)), ignore_errors)
        })
    })
}

/// Predicate that returns true for elements that do not contain the specified text.
/// See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_text<N>(text: N, ignore_errors: bool) -> ElementPredicate
where
    N: Needle + Clone + Send + Sync + 'static,
{
    Box::new(move |elem| {
        let text = text.clone();
        Box::pin(async move { negate(elem.text().await.map(|x| text.is_match(&x)), ignore_errors) })
    })
}

/// Predicate that returns true for elements that have the specified value.
/// See the `Needle` documentation for more details on text matching rules.
pub fn element_has_value<N>(value: N, ignore_errors: bool) -> ElementPredicate
where
    N: Needle + Clone + Send + Sync + 'static,
{
    Box::new(move |elem| {
        let value = value.clone();
        Box::pin(async move {
            match elem.value().await {
                Ok(Some(x)) => Ok(value.is_match(&x)),
                Ok(None) => Ok(false),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that do not contain the specified value.
/// See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_value<N>(value: N, ignore_errors: bool) -> ElementPredicate
where
    N: Needle + Clone + Send + Sync + 'static,
{
    Box::new(move |elem| {
        let value = value.clone();
        Box::pin(async move {
            match elem.value().await {
                Ok(Some(x)) => Ok(!value.is_match(&x)),
                Ok(None) => Ok(true),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that have the specified attribute with the specified
/// value. See the `Needle` documentation for more details on text matching rules.
pub fn element_has_attribute<S, N>(
    attribute_name: S,
    value: N,
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String>,
    N: Needle + Clone + Send + Sync + 'static,
{
    let attribute_name: String = attribute_name.into();
    Box::new(move |elem| {
        let attribute_name: String = attribute_name.clone();
        let value = value.clone();
        Box::pin(async move {
            match elem.get_attribute(&attribute_name).await {
                Ok(Some(x)) => Ok(value.is_match(&x)),
                Ok(None) => Ok(false),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that lack the specified attribute with the
/// specified value. See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_attribute<S, N>(
    attribute_name: S,
    value: N,
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String>,
    N: Needle + Clone + Send + Sync + 'static,
{
    let attribute_name: String = attribute_name.into();
    Box::new(move |elem| {
        let attribute_name: String = attribute_name.clone();
        let value = value.clone();
        Box::pin(async move {
            match elem.get_attribute(&attribute_name).await {
                Ok(Some(x)) => Ok(!value.is_match(&x)),
                Ok(None) => Ok(true),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that have all of the specified attributes with the
/// specified values. See the `Needle` documentation for more details on text matching rules.
pub fn element_has_attributes<S, N>(
    desired_attributes: &[(S, N)],
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String> + Clone,
    N: Needle + Clone + Send + Sync + 'static,
{
    let desired_attributes: Vec<(String, N)> =
        desired_attributes.iter().cloned().map(|(a, b)| (a.into(), b)).collect();
    Box::new(move |elem| {
        let desired_attributes = desired_attributes.clone();
        Box::pin(async move {
            for (attribute_name, value) in &desired_attributes {
                match elem.get_attribute(&attribute_name).await {
                    Ok(Some(x)) => {
                        if !value.is_match(&x) {
                            return Ok(false);
                        }
                    }
                    Ok(None) => return Ok(false),
                    Err(e) => return handle_errors(Err(e), ignore_errors),
                }
            }
            Ok(true)
        })
    })
}

/// Predicate that returns true for elements that do not have any of the specified attributes with
/// the specified values. See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_attributes<S, N>(
    desired_attributes: &[(S, N)],
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String> + Clone,
    N: Needle + Clone + Send + Sync + 'static,
{
    let desired_attributes: Vec<(String, N)> =
        desired_attributes.iter().cloned().map(|(a, b)| (a.into(), b)).collect();
    Box::new(move |elem| {
        let desired_attributes = desired_attributes.clone();
        Box::pin(async move {
            for (attribute_name, value) in &desired_attributes {
                match elem.get_attribute(&attribute_name).await {
                    Ok(Some(x)) => {
                        if value.is_match(&x) {
                            return Ok(false);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => return handle_errors(Err(e), ignore_errors),
                }
            }
            Ok(true)
        })
    })
}

/// Predicate that returns true for elements that have the specified property with the specified
/// value. See the `Needle` documentation for more details on text matching rules.
pub fn element_has_property<S, N>(
    property_name: S,
    value: N,
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String>,
    N: Needle + Clone + Send + Sync + 'static,
{
    let property_name: String = property_name.into();
    Box::new(move |elem| {
        let property_name = property_name.clone();
        let value = value.clone();
        Box::pin(async move {
            match elem.get_property(&property_name).await {
                Ok(Some(x)) => Ok(value.is_match(&x)),
                Ok(None) => Ok(false),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that lack the specified property with the
/// specified value. See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_property<S, N>(
    property_name: S,
    value: N,
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String>,
    N: Needle + Clone + Send + Sync + 'static,
{
    let property_name: String = property_name.into();
    Box::new(move |elem| {
        let property_name = property_name.clone();
        let value = value.clone();
        Box::pin(async move {
            match elem.get_property(&property_name).await {
                Ok(Some(x)) => Ok(!value.is_match(&x)),
                Ok(None) => Ok(true),
                Err(e) => handle_errors(Err(e), ignore_errors),
            }
        })
    })
}

/// Predicate that returns true for elements that have all of the specified properties with the
/// specified value. See the `Needle` documentation for more details on text matching rules.
pub fn element_has_properties<S, N>(
    desired_properties: &[(S, N)],
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String> + Clone,
    N: Needle + Clone + Send + Sync + 'static,
{
    let desired_properties: Vec<(String, N)> =
        desired_properties.iter().cloned().map(|(a, b)| (a.into(), b)).collect();
    Box::new(move |elem| {
        let desired_properties = desired_properties.clone();
        Box::pin(async move {
            for (property_name, value) in &desired_properties {
                match elem.get_property(property_name).await {
                    Ok(Some(x)) => {
                        if !value.is_match(&x) {
                            return Ok(false);
                        }
                    }
                    Ok(None) => return Ok(false),
                    Err(e) => return handle_errors(Err(e), ignore_errors),
                }
            }
            Ok(true)
        })
    })
}

/// Predicate that returns true for elements that do not have any of the specified properties with
/// the specified values. See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_properties<S, N>(
    desired_properties: &[(S, N)],
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String> + Clone,
    N: Needle + Clone + Send + Sync + 'static,
{
    let desired_properties: Vec<(String, N)> =
        desired_properties.iter().cloned().map(|(a, b)| (a.into(), b)).collect();
    Box::new(move |elem| {
        let desired_properties = desired_properties.clone();
        Box::pin(async move {
            for (property_name, value) in &desired_properties {
                match elem.get_property(property_name).await {
                    Ok(Some(x)) => {
                        if value.is_match(&x) {
                            return Ok(false);
                        }
                    }
                    Ok(None) => {}
                    Err(e) => return handle_errors(Err(e), ignore_errors),
                }
            }
            Ok(true)
        })
    })
}

/// Predicate that returns true for elements that have the specified CSS property with the specified
/// value. See the `Needle` documentation for more details on text matching rules.
pub fn element_has_css_property<S, N>(
    css_property_name: S,
    value: N,
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String>,
    N: Needle + Clone + Send + Sync + 'static,
{
    let css_property_name: String = css_property_name.into();
    Box::new(move |elem| {
        let css_property_name = css_property_name.clone();
        let value = value.clone();
        Box::pin(async move {
            handle_errors(
                elem.get_css_property(&css_property_name).await.map(|x| value.is_match(&x)),
                ignore_errors,
            )
        })
    })
}

/// Predicate that returns true for elements that lack the specified CSS property with the
/// specified value. See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_css_property<S, N>(
    css_property_name: S,
    value: N,
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String>,
    N: Needle + Clone + Send + Sync + 'static,
{
    let css_property_name: String = css_property_name.into();
    Box::new(move |elem| {
        let css_property_name = css_property_name.clone();
        let value = value.clone();
        Box::pin(async move {
            handle_errors(
                elem.get_css_property(&css_property_name).await.map(|x| !value.is_match(&x)),
                ignore_errors,
            )
        })
    })
}

/// Predicate that returns true for elements that have all of the specified CSS properties with the
/// specified values.
/// See the `Needle` documentation for more details on text matching rules.
pub fn element_has_css_properties<S, N>(
    desired_css_properties: &[(S, N)],
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String> + Clone,
    N: Needle + Clone + Send + Sync + 'static,
{
    let desired_css_properties: Vec<(String, N)> =
        desired_css_properties.iter().cloned().map(|(a, b)| (a.into(), b)).collect();
    Box::new(move |elem| {
        let desired_css_properties = desired_css_properties.clone();
        Box::pin(async move {
            for (css_property_name, value) in &desired_css_properties {
                match elem.get_css_property(css_property_name).await {
                    Ok(x) => {
                        if !value.is_match(&x) {
                            return Ok(false);
                        }
                    }
                    Err(e) => return handle_errors(Err(e), ignore_errors),
                }
            }
            Ok(true)
        })
    })
}

/// Predicate that returns true for elements that do not have any of the specified CSS properties
/// with the specified values.
/// See the `Needle` documentation for more details on text matching rules.
pub fn element_lacks_css_properties<S, N>(
    desired_css_properties: &[(S, N)],
    ignore_errors: bool,
) -> ElementPredicate
where
    S: Into<String> + Clone,
    N: Needle + Clone + Send + Sync + 'static,
{
    let desired_css_properties: Vec<(String, N)> =
        desired_css_properties.iter().cloned().map(|(a, b)| (a.into(), b)).collect();
    Box::new(move |elem| {
        let desired_css_properties = desired_css_properties.clone();
        Box::pin(async move {
            for (css_property_name, value) in &desired_css_properties {
                match elem.get_css_property(css_property_name).await {
                    Ok(x) => {
                        if value.is_match(&x) {
                            return Ok(false);
                        }
                    }
                    Err(e) => return handle_errors(Err(e), ignore_errors),
                }
            }
            Ok(true)
        })
    })
}
