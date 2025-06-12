use crate::common::*;
use assert_matches::assert_matches;
use rstest::rstest;
use std::time::Duration;
use thirtyfour::components::{ElementResolverMulti, ElementResolverSingle};
use thirtyfour::error::WebDriverErrorInner;
use thirtyfour::support::block_on;
use thirtyfour::{components::SelectElement, prelude::*};

mod common;

#[rstest]
fn get_active_element(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;
        c.find(By::Css("#select1")).await?.click().await?;

        let active = c.active_element().await?;
        assert_eq!(active.attr("id").await?, Some(String::from("select1")));
        Ok(())
    })
}

#[rstest]
fn find_all(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;
        let elems = c.find_all(By::Css("nav a")).await?;
        assert_eq!(elems.len(), 2);
        Ok(())
    })
}

#[rstest]
fn query(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;
        let elem = c.query(By::Css("nav a")).first().await?;
        assert_eq!(elem.id().await?.unwrap(), "other_page_id");
        let elem_result = c.query(By::Css("nav a")).single().await;
        assert_matches!(
            elem_result.map_err(WebDriverError::into_inner),
            Err(WebDriverErrorInner::NoSuchElement(_))
        );
        // There should only be one element with the class 'vertical'.
        let elem_result = c.query(By::ClassName("vertical")).single().await;
        assert!(elem_result.unwrap().class_name().await?.unwrap().contains("vertical"));
        Ok(())
    })
}

#[rstest]
fn query_all(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        // Match all, single selector.
        let elems = c.query(By::Css("nav a")).all_from_selector_required().await?;
        assert_eq!(elems.len(), 2);
        let elems = c.query(By::Css("nav a")).all_from_selector().await?;
        assert_eq!(elems.len(), 2);

        // Multiple selectors, only 1 selector's elements were returned.
        let elems =
            c.query(By::Css("nav a")).or(By::Id("navigation")).all_from_selector_required().await?;
        assert_eq!(elems.len(), 2); // Should only return the 2 from 'nav a' and ignore the rest.
        let elems = c.query(By::Css("nav a")).or(By::Id("navigation")).all_from_selector().await?;
        assert_eq!(elems.len(), 2); // Should only return the 2 from 'nav a' and ignore the rest.

        // Match only second selector.
        let elems = c
            .query(By::Id("doesnotexist"))
            .or(By::Id("navigation"))
            .all_from_selector_required()
            .await?;
        assert_eq!(elems.len(), 1);
        let elems =
            c.query(By::Id("doesnotexist")).or(By::Id("navigation")).all_from_selector().await?;
        assert_eq!(elems.len(), 1);

        // Match none.
        let elems = c.query(By::Id("doesnotexist")).nowait().all_from_selector().await?;
        assert!(elems.is_empty());

        // Match none, but at least 1 was required.
        let elem_result =
            c.query(By::Id("doesnotexist")).nowait().all_from_selector_required().await;
        assert_matches!(
            elem_result.map_err(WebDriverError::into_inner),
            Err(WebDriverErrorInner::NoSuchElement(_))
        );
        Ok(())
    })
}

#[rstest]
fn query_any(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        // Match both selectors.
        let elems = c.query(By::Css("nav a")).or(By::Id("navigation")).any_required().await?;
        assert_eq!(elems.len(), 3); // Should be 2 from 'nav a' and 1 from '#navigation'.
        let elems = c.query(By::Css("nav a")).or(By::Id("navigation")).any().await?;
        assert_eq!(elems.len(), 3); // Should be 2 from 'nav a' and 1 from '#navigation'.

        // Match none.
        let elems = c.query(By::Id("doesnotexist")).or(By::Id("invalid")).nowait().any().await?;
        assert!(elems.is_empty());

        // Match only second selector.
        let elems = c.query(By::Id("doesnotexist")).or(By::Id("navigation")).any_required().await?;
        assert_eq!(elems.len(), 1);
        let elems = c.query(By::Id("doesnotexist")).or(By::Id("navigation")).any().await?;
        assert_eq!(elems.len(), 1);

        // Match none, but at least 1 was required.
        let elem_result =
            c.query(By::Id("doesnotexist")).or(By::Id("invalid")).nowait().any_required().await;
        assert_matches!(
            elem_result.map_err(WebDriverError::into_inner),
            Err(WebDriverErrorInner::NoSuchElement(_))
        );
        Ok(())
    })
}

#[rstest]
fn query_exists(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        // Nowait.
        assert!(
            !c.query(By::Id("doesnotexist")).nowait().exists().await.unwrap(),
            "nowait().exists() should return false for non-existent element"
        );
        assert!(
            c.query(By::Id("doesnotexist")).nowait().not_exists().await.unwrap(),
            "nowait().not_exists() should return true for non-existent element"
        );

        // Wait (1 sec).
        assert!(
            !c.query(By::Id("doesnotexist"))
                .wait(Duration::from_secs(1), Duration::from_millis(200))
                .exists()
                .await
                .unwrap(),
            "exists() should return false for non-existent element"
        );
        assert!(
            c.query(By::Id("doesnotexist")).not_exists().await.unwrap(),
            "not_exists() with poll should return true for non-existent element"
        );

        // Exists, wait (1 sec).
        assert!(
            c.query(By::Id("footer")).exists().await.unwrap(),
            "exists() should return true for existing element"
        );
        assert!(
            !c.query(By::Id("navigation"))
                .wait(Duration::from_secs(1), Duration::from_millis(200))
                .not_exists()
                .await
                .unwrap(),
            "not_exists() should return false for existing element"
        );

        Ok(())
    })
}

#[rstest]
fn resolve(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;
        let base_element = c.find(By::ClassName("vertical")).await?;
        let resolver = ElementResolverSingle::new_first(base_element.clone(), By::Css("nav a"));
        let elem = resolver.resolve().await?;
        assert_eq!(elem.id().await?.unwrap(), "other_page_id");
        let elem2 = resolver.resolve_present().await?;
        assert_eq!(elem2.id().await?.unwrap(), "other_page_id");
        assert_eq!(elem, elem2);
        let resolver = ElementResolverSingle::new_single(base_element, By::Css("nav a"));
        let elem_result = resolver.resolve().await;
        assert_matches!(
            elem_result.map_err(WebDriverError::into_inner),
            Err(WebDriverErrorInner::NoSuchElement(_))
        );

        Ok(())
    })
}

#[rstest]
fn resolve_all(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;
        let base_element = c.find(By::ClassName("vertical")).await?;
        let resolver = ElementResolverMulti::new_not_empty(base_element, By::Css("nav a"));
        let elems = resolver.resolve().await?;
        assert_eq!(elems.len(), 2);
        let elems2 = resolver.resolve_present().await?;
        assert_eq!(elems.len(), 2);
        assert_eq!(elems, elems2);
        Ok(())
    })
}

#[rstest]
fn stale_element(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;
        let elem = c.find(By::Css("#other_page_id")).await?;

        // Remove the element from the DOM
        c.execute(
            "var elem = document.getElementById('other_page_id');
         elem.parentNode.removeChild(elem);",
            vec![],
        )
        .await?;

        match elem.click().await.map_err(WebDriverError::into_inner) {
            Err(WebDriverErrorInner::StaleElementReference(_)) => Ok(()),
            _ => panic!("Expected a stale element reference error"),
        }
    })
}

#[rstest]
fn select_by_index(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        let elem = c.find(By::Css("#select1")).await?;
        let select_element = SelectElement::new(&elem).await?;

        // Get first display text
        let initial_text = elem.prop("value").await?;
        assert_eq!(Some("Select1-Option1".into()), initial_text);

        // Select 2nd option by index.
        select_element.select_by_index(1).await?;

        // Get display text after selection
        let text_after_selecting = elem.prop("value").await?;
        assert_eq!(Some("Select1-Option2".into()), text_after_selecting);

        // Check that the second select is not changed
        let select2_text = c.find(By::Css("#select2")).await?.prop("value").await?;
        assert_eq!(Some("Select2-Option1".into()), select2_text);

        // Show off that it selects only options and skip any other elements
        let elem = c.find(By::Css("#select2")).await?;
        let select_element = SelectElement::new(&elem).await?;
        select_element.select_by_index(1).await?;
        let text = elem.prop("value").await?;
        assert_eq!(Some("Select2-Option2".into()), text);

        Ok(())
    })
}

#[rstest]
fn select_by_label(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        let elem = c.find(By::Css("#select1")).await?;
        let select_element = SelectElement::new(&elem).await?;

        // Get first display text
        let initial_text = elem.prop("value").await?;
        assert_eq!(Some("Select1-Option1".into()), initial_text);

        // Select second option
        select_element.select_by_exact_text("Select1-Option2").await?;

        // Get display text after selection
        let text_after_selecting = elem.prop("value").await?;
        assert_eq!(Some("Select1-Option2".into()), text_after_selecting);

        // Check that the second select is not changed
        let select2_text = c.find(By::Css("#select2")).await?.prop("value").await?;
        assert_eq!(Some("Select2-Option1".into()), select2_text);

        Ok(())
    })
}

#[rstest]
fn find_element_from_element(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        // Find.
        let form = c.find(By::Id("textarea-form")).await?;
        let textarea = form.find(By::Tag("textarea")).await?;
        assert_eq!(textarea.attr("name").await?.unwrap(), "some_textarea");

        // Find all.
        let nav = c.find(By::Id("navigation")).await?;
        let links = nav.find_all(By::Tag("a")).await?;
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].text().await?, "Other Page");
        assert_eq!(links[1].text().await?, "Other Page");
        Ok(())
    })
}
