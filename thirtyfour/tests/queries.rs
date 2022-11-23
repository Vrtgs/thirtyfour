use crate::common::sample_page_url;
use assert_matches::assert_matches;
use serial_test::serial;
use std::time::Duration;
use thirtyfour::components::{ElementResolverMulti, ElementResolverSingle};
use thirtyfour::{components::SelectElement, prelude::*};

mod common;

async fn get_active_element(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    c.find(By::Css("#select1")).await?.click().await?;

    let active = c.active_element().await?;
    assert_eq!(active.attr("id").await?, Some(String::from("select1")));

    c.close_window().await
}

async fn find_all(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let elems = c.find_all(By::Css("nav a")).await?;
    assert_eq!(elems.len(), 2);
    Ok(())
}

async fn query(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let elem = c.query(By::Css("nav a")).first().await?;
    assert_eq!(elem.id().await?.unwrap(), "other_page_id");
    let elem_result = c.query(By::Css("nav a")).single().await;
    assert_matches!(elem_result, Err(WebDriverError::NoSuchElement(_)));
    Ok(())
}

async fn query_all(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
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
    let elem_result = c.query(By::Id("doesnotexist")).nowait().all_from_selector_required().await;
    assert_matches!(elem_result, Err(WebDriverError::NoSuchElement(_)));
    Ok(())
}

async fn query_any(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
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
    assert_matches!(elem_result, Err(WebDriverError::NoSuchElement(_)));
    Ok(())
}

async fn query_exists(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
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
}

async fn resolve(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
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
    assert_matches!(elem_result, Err(WebDriverError::NoSuchElement(_)));

    Ok(())
}

async fn resolve_all(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let base_element = c.find(By::ClassName("vertical")).await?;
    let resolver = ElementResolverMulti::new_not_empty(base_element, By::Css("nav a"));
    let elems = resolver.resolve().await?;
    assert_eq!(elems.len(), 2);
    let elems2 = resolver.resolve_present().await?;
    assert_eq!(elems.len(), 2);
    assert_eq!(elems, elems2);
    Ok(())
}

async fn stale_element(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let elem = c.find(By::Css("#other_page_id")).await?;

    // Remove the element from the DOM
    c.execute(
        "var elem = document.getElementById('other_page_id');
         elem.parentNode.removeChild(elem);",
        vec![],
    )
    .await?;

    match elem.click().await {
        Err(WebDriverError::StaleElementReference(_)) => Ok(()),
        _ => panic!("Expected a stale element reference error"),
    }
}

async fn select_by_index(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
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
}

async fn select_by_label(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
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
}

async fn find_element_from_element(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
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
}

mod firefox {
    use super::*;

    #[test]
    #[serial]
    fn get_active_element_test() {
        local_tester!(get_active_element, "firefox");
    }

    #[test]
    #[serial]
    fn find_all_test() {
        local_tester!(find_all, "firefox");
    }

    #[test]
    #[serial]
    fn query_test() {
        local_tester!(query, "firefox");
    }

    #[test]
    #[serial]
    fn query_all_test() {
        local_tester!(query_all, "firefox");
    }

    #[test]
    #[serial]
    fn query_any_test() {
        local_tester!(query_any, "firefox");
    }

    #[test]
    #[serial]
    fn query_exists_test() {
        local_tester!(query_exists, "firefox");
    }

    #[test]
    #[serial]
    fn resolve_test() {
        local_tester!(resolve, "firefox");
    }

    #[test]
    #[serial]
    fn resolve_all_test() {
        local_tester!(resolve_all, "firefox");
    }

    #[test]
    #[serial]
    fn stale_element_test() {
        local_tester!(stale_element, "firefox");
    }

    #[test]
    #[serial]
    fn select_by_index_test() {
        local_tester!(select_by_index, "firefox");
    }

    #[test]
    #[serial]
    fn select_by_label_test() {
        local_tester!(select_by_label, "firefox");
    }

    #[test]
    #[serial]
    fn find_element_from_element_test() {
        local_tester!(find_element_from_element, "firefox");
    }
}

mod chrome {

    use super::*;

    #[test]
    fn get_active_element_test() {
        local_tester!(get_active_element, "chrome");
    }

    #[test]
    fn find_all_test() {
        local_tester!(find_all, "chrome");
    }

    #[test]
    fn query_test() {
        local_tester!(query, "chrome");
    }

    #[test]
    fn query_all_test() {
        local_tester!(query_all, "chrome");
    }

    #[test]
    fn query_any_test() {
        local_tester!(query_any, "chrome");
    }

    #[test]
    fn query_exists_test() {
        local_tester!(query_exists, "chrome");
    }

    #[test]
    fn resolve_test() {
        local_tester!(resolve, "chrome");
    }

    #[test]
    fn resolve_all_test() {
        local_tester!(resolve_all, "chrome");
    }

    #[test]
    fn select_by_label_test() {
        local_tester!(select_by_label, "chrome");
    }

    #[test]
    fn select_by_index_test() {
        local_tester!(select_by_index, "chrome");
    }

    #[test]
    fn find_element_from_element_test() {
        local_tester!(find_element_from_element, "chrome");
    }
}
