use fantoccini::Locator;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum BySelector {
    Id(String),
    LinkText(String),
    Css(String),
    XPath(String),
}

// NOTE: This needs to own its data so that we allow the user to specify custom
//       CSS selectors such as tag/name/class etc and send fantoccini a reference
//       to the formatted Css selector.
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

impl By {
    #[allow(non_snake_case)]
    pub fn Id(id: &str) -> Self {
        Self {
            selector: BySelector::Id(id.to_string()),
        }
    }

    #[allow(non_snake_case)]
    pub fn LinkText(text: &str) -> Self {
        Self {
            selector: BySelector::LinkText(text.to_string()),
        }
    }

    #[allow(non_snake_case)]
    pub fn Css(css: &str) -> Self {
        Self {
            selector: BySelector::Css(css.to_string()),
        }
    }

    #[allow(non_snake_case)]
    pub fn XPath(x: &str) -> Self {
        Self {
            selector: BySelector::XPath(x.to_string()),
        }
    }

    #[allow(non_snake_case)]
    pub fn Name(name: &str) -> Self {
        Self {
            selector: BySelector::Css(format!(r#"[name="{}"]"#, name)),
        }
    }

    #[allow(non_snake_case)]
    pub fn Tag(tag: &str) -> Self {
        Self {
            selector: BySelector::Css(tag.to_string()),
        }
    }

    #[allow(non_snake_case)]
    pub fn ClassName(name: &str) -> Self {
        Self {
            selector: BySelector::Css(format!(".{}", name)),
        }
    }

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

pub fn locator_to_string(locator: Locator<'_>) -> String {
    match locator {
        Locator::Css(s) => format!("Css({s})"),
        Locator::Id(s) => format!("Id({s}"),
        Locator::LinkText(s) => format!("LinkText({s})"),
        Locator::XPath(s) => format!("XPath({s})"),
    }
}
