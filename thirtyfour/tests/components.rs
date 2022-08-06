use crate::common::sample_page_url;
use serial_test::serial;
use thirtyfour::components::{Component, ElementResolver};
use thirtyfour::{prelude::*, resolve, resolve_present};

mod common;

/// This component shows how you can nest components inside others.
#[derive(Debug, Component, Clone)]
pub struct CheckboxSectionComponent {
    base: WebElement,
    #[by(tag = "label")]
    boxes: ElementResolver<Vec<CheckboxComponent>>,
}

/// This component shows how you can wrap a simple web component.
#[derive(Debug, Component, Clone)]
pub struct CheckboxComponent {
    #[base]
    base: WebElement,
    #[by(css = "input[type='checkbox']", first)]
    input: ElementResolver<WebElement>,
}

impl CheckboxComponent {
    pub async fn is_ticked(&self) -> WebDriverResult<bool> {
        let prop = resolve_present!(self.input).prop("checked").await?;
        Ok(prop.unwrap_or_default() == "true")
    }

    pub async fn tick(&self) -> WebDriverResult<()> {
        let elem = resolve!(self.input);
        if elem.is_clickable().await? && !self.is_ticked().await? {
            elem.click().await?;
            assert!(self.is_ticked().await?);
        }

        Ok(())
    }
}

async fn basic_component(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;

    // Get the checkbox div.
    // NOTE: components using the `Component` derive automatically implement `From<WebElement>`.
    let section: CheckboxSectionComponent =
        c.query(By::Id("checkbox-section")).single().await?.into();

    // Tick all the checkboxes, ignoring any that are disabled.
    for checkbox in resolve!(section.boxes) {
        checkbox.tick().await?;
    }

    Ok(())
}

mod firefox {
    use super::*;

    #[test]
    #[serial]
    fn basic_component_test() {
        local_tester!(basic_component, "firefox");
    }
}

mod chrome {
    use super::*;

    #[test]
    fn basic_component_test() {
        local_tester!(basic_component, "chrome");
    }
}
