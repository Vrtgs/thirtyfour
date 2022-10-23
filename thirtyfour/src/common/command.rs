use crate::Locator;
use std::fmt;
use std::fmt::Debug;

/// The webdriver selector to use when querying elements.
#[derive(Debug, Clone)]
pub enum BySelector {
    /// Query by element id.
    Id(String),
    /// Query by link text.
    LinkText(String),
    /// Query by CSS.
    Css(String),
    /// Query by XPath.
    XPath(String),
}

// NOTE: This needs to own its data so that we allow the user to specify custom
//       CSS selectors such as tag/name/class etc and send fantoccini a reference
//       to the formatted Css selector.

/// The webdriver selector to use when querying elements.
#[derive(Debug, Clone)]
pub struct By {
    selector: BySelector,
}

impl fmt::Display for By {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.selector {
            BySelector::Id(id) => write!(f, "Id({})", id),
            BySelector::XPath(xpath) => write!(f, "XPath({})", xpath),
            BySelector::LinkText(text) => write!(f, "Link Text({})", text),
            BySelector::Css(css) => write!(f, "CSS({})", css),
        }
    }
}

#[allow(non_snake_case)]
impl By {
    /// Select element by id.
    pub fn Id(id: &str) -> Self {
        Self {
            selector: BySelector::Id(id.to_string()),
        }
    }

    /// Select element by link text.
    pub fn LinkText(text: &str) -> Self {
        Self {
            selector: BySelector::LinkText(text.to_string()),
        }
    }

    /// Select element by CSS.
    pub fn Css(css: &str) -> Self {
        Self {
            selector: BySelector::Css(css.to_string()),
        }
    }

    /// Select element by XPath.
    pub fn XPath(x: &str) -> Self {
        Self {
            selector: BySelector::XPath(x.to_string()),
        }
    }

    /// Select element by name.
    pub fn Name(name: &str) -> Self {
        Self {
            selector: BySelector::Css(format!(r#"[name="{}"]"#, name)),
        }
    }

    /// Select element by tag.
    pub fn Tag(tag: &str) -> Self {
        Self {
            selector: BySelector::Css(tag.to_string()),
        }
    }

    /// Select element by class.
    pub fn ClassName(name: &str) -> Self {
        Self {
            selector: BySelector::Css(format!(".{}", name)),
        }
    }

    /// Get the [`Locator`] for this selector.
    pub fn locator(&self) -> Locator {
        match &self.selector {
            BySelector::Id(id) => Locator::Id(id),
            BySelector::LinkText(text) => Locator::LinkText(text),
            BySelector::Css(css) => Locator::Css(css),
            BySelector::XPath(x) => Locator::XPath(x),
        }
    }
}

impl<'a> From<&'a By> for Locator<'a> {
    fn from(by: &'a By) -> Self {
        by.locator()
    }
}

impl<'a> From<Locator<'a>> for By {
    fn from(locator: Locator<'a>) -> Self {
        match locator {
            Locator::Css(s) => By::Css(s),
            Locator::Id(s) => By::Id(s),
            Locator::LinkText(s) => By::LinkText(s),
            Locator::XPath(s) => By::XPath(s),
        }
    }
}

/// Convert the specified locator to a string, used for debugging.
pub fn locator_to_string(locator: Locator<'_>) -> String {
    match locator {
        Locator::Css(s) => format!("Css({})", s),
        Locator::Id(s) => format!("Id({}", s),
        Locator::LinkText(s) => format!("LinkText({})", s),
        Locator::XPath(s) => format!("XPath({})", s),
    }
}
