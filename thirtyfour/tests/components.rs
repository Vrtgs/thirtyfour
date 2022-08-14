mod common;

#[cfg(feature = "component")]
mod feature_component {
    use super::*;
    use crate::common::sample_page_url;
    use assert_matches::assert_matches;
    use serial_test::serial;
    use std::time::Instant;
    use thirtyfour::components::{Component, ElementResolver};
    use thirtyfour::extensions::query::ElementQueryOptions;
    use thirtyfour::{prelude::*, resolve, resolve_present};

    /// This component shows how to nest components inside others.
    #[derive(Debug, Clone, Component)]
    pub struct CheckboxSectionComponent {
        base: WebElement,
        #[by(tag = "label", not_empty)]
        boxes: ElementResolver<Vec<CheckboxComponent>>,
        // Other fields will be initialised with Default::default().
        my_field: bool,
    }

    /// This component shows how to wrap a simple web component.
    #[derive(Debug, Clone, Component)]
    pub struct CheckboxComponent {
        #[base]
        source: WebElement,
        #[by(css = "input[type='checkbox']", single)]
        input: ElementResolver<WebElement>,
    }

    impl CheckboxComponent {
        /// Return true if the checkbox is ticked.
        pub async fn is_ticked(&self) -> WebDriverResult<bool> {
            let prop = resolve!(self.input).prop("checked").await?;
            Ok(prop.unwrap_or_default() == "true")
        }

        /// Tick the checkbox if it is clickable and isn't already ticked.
        pub async fn tick(&self) -> WebDriverResult<()> {
            let elem = resolve_present!(self.input);
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
        assert!(!section.my_field);
        assert_eq!(section.base.id().await?.unwrap(), "checkbox-section");

        // Tick all the checkboxes, ignoring any that are disabled.
        for checkbox in resolve!(section.boxes) {
            assert_eq!(checkbox.base_element().tag_name().await?, "label");
            checkbox.tick().await?;
        }

        Ok(())
    }

    #[derive(Debug, Component, Clone)]
    pub struct TestComponent {
        base: WebElement,
        #[by(tag = "label")]
        elem_single: ElementResolver<WebElement>,
        #[by(tag = "label", single)]
        elem_single_explicit: ElementResolver<WebElement>,
        #[by(tag = "label", first)]
        elem_first: ElementResolver<WebElement>,
        #[by(tag = "label", description = "my_test_description")]
        elem_desc: ElementResolver<WebElement>,
        #[by(tag = "notfound", ignore_errors, wait(timeout_ms = 1500, interval_ms = 100))]
        elem_ignore: ElementResolver<WebElement>,
        #[by(tag = "notfound", nowait)]
        elem_nowait: ElementResolver<WebElement>,
        #[by(tag = "notfound", allow_empty, nowait)]
        elems_allow_empty: ElementResolver<Vec<WebElement>>,
        #[by(tag = "notfound", nowait)]
        elems_not_empty: ElementResolver<Vec<WebElement>>,
        #[by(tag = "notfound", nowait, not_empty)]
        elems_not_empty_explicit: ElementResolver<Vec<WebElement>>,
    }

    async fn component_attributes(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
        let url = sample_page_url(port);
        c.goto(&url).await?;

        ElementQueryOptions::default().set_description(Some("hello"));

        let elem = c.query(By::Id("checkbox-section")).single().await?;
        let tc = TestComponent::new(elem);

        let result = tc.elem_single.resolve().await;
        assert_matches!(result, Err(WebDriverError::NoSuchElement(_)));

        let result = tc.elem_single_explicit.resolve().await;
        assert_matches!(result, Err(WebDriverError::NoSuchElement(_)));

        let elem = tc.elem_first.resolve().await?;
        assert_eq!(elem.tag_name().await?, "label");

        let result = tc.elem_desc.resolve().await;
        assert_matches!(result, Err(WebDriverError::NoSuchElement(x)) if x.contains("my_test_description"));

        let start = Instant::now();
        let result = tc.elem_ignore.resolve().await;
        assert_matches!(result, Err(WebDriverError::NoSuchElement(_)));
        assert!(start.elapsed().as_secs() > 0);

        let start = Instant::now();
        let result = tc.elem_nowait.resolve().await;
        assert_matches!(result, Err(WebDriverError::NoSuchElement(_)));
        assert!(start.elapsed().as_secs() < 10);

        let elems = tc.elems_allow_empty.resolve().await?;
        assert!(elems.is_empty());

        let result = tc.elems_not_empty.resolve().await;
        assert_matches!(result, Err(WebDriverError::NoSuchElement(_)));

        let result = tc.elems_not_empty_explicit.resolve().await;
        assert_matches!(result, Err(WebDriverError::NoSuchElement(_)));

        Ok(())
    }

    #[derive(Debug, Component, Clone)]
    pub struct TestComponentCustomFn {
        base: WebElement,
        #[by(custom = "custom_resolve_fn")]
        elem_custom: ElementResolver<WebElement>,
        #[by(custom = "custom_resolve_fn_multi")]
        elems_custom: ElementResolver<Vec<WebElement>>,
        #[by(custom = "custom_resolve_fn_component")]
        component_custom: ElementResolver<CheckboxComponent>,
        #[by(custom = "custom_resolve_fn_components")]
        components_custom: ElementResolver<Vec<CheckboxComponent>>,
    }

    async fn custom_resolve_fn(elem: &WebElement) -> WebDriverResult<WebElement> {
        elem.query(By::Tag("label")).first().await
    }

    async fn custom_resolve_fn_multi(elem: &WebElement) -> WebDriverResult<Vec<WebElement>> {
        elem.query(By::Tag("label")).all().await
    }

    async fn custom_resolve_fn_component(elem: &WebElement) -> WebDriverResult<CheckboxComponent> {
        let cb = elem.query(By::Tag("label")).first().await?;
        Ok(CheckboxComponent::new(cb))
    }

    async fn custom_resolve_fn_components(
        elem: &WebElement,
    ) -> WebDriverResult<Vec<CheckboxComponent>> {
        let cbs = elem.query(By::Tag("label")).all().await?;
        Ok(cbs.into_iter().map(CheckboxComponent::new).collect())
    }

    async fn component_attributes_custom_fn(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
        let url = sample_page_url(port);
        c.goto(&url).await?;

        ElementQueryOptions::default().set_description(Some("hello"));

        let elem = c.query(By::Id("checkbox-section")).single().await?;
        let tc = TestComponentCustomFn::new(elem);

        tc.elem_custom.resolve().await.unwrap();
        tc.elems_custom.resolve().await.unwrap();
        tc.component_custom.resolve().await.unwrap();
        tc.components_custom.resolve().await.unwrap();

        Ok(())
    }

    mod firefox {
        use super::*;
        use crate::local_tester;

        #[test]
        #[serial]
        fn basic_component_test() {
            local_tester!(basic_component, "firefox");
        }

        #[test]
        #[serial]
        fn component_attributes_test() {
            local_tester!(component_attributes, "firefox");
        }

        #[test]
        #[serial]
        fn component_attributes_custom_fn_test() {
            local_tester!(component_attributes_custom_fn, "firefox");
        }
    }

    mod chrome {
        use super::*;
        use crate::local_tester;

        #[test]
        fn basic_component_test() {
            local_tester!(basic_component, "chrome");
        }

        #[test]
        fn component_attributes_test() {
            local_tester!(component_attributes, "chrome");
        }

        #[test]
        fn component_attributes_custom_fn_test() {
            local_tester!(component_attributes_custom_fn, "chrome");
        }
    }
}
