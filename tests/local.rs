//! Tests that don't make use of external websites.
use crate::common::{other_page_url, sample_page_url};
use serial_test::serial;
use std::time::Duration;
use thirtyfour::{components::select::SelectElement, prelude::*};

mod common;

async fn goto(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;
    let current_url = c.current_url().await?;
    assert_eq!(url.as_str(), current_url.as_str());
    c.close().await
}

async fn find_and_click_link(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;
    c.find_element(By::Css("#other_page_id")).await?.click().await?;

    let new_url = c.current_url().await?;
    let expected_url = other_page_url(port);
    assert_eq!(new_url.as_str(), expected_url.as_str());

    c.close().await
}

async fn get_active_element(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;
    c.find_element(By::Css("#select1")).await?.click().await?;

    let active = c.switch_to().active_element().await?;
    assert_eq!(active.attr("id").await?, Some(String::from("select1")));

    c.close().await
}

async fn serialize_element(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;
    let elem = c.find_element(By::Css("#other_page_id")).await?;

    // Check that webdriver understands it
    c.execute_script("arguments[0].scrollIntoView(true);", vec![serde_json::to_value(elem)?])
        .await?;

    // Check that it fails with an invalid serialization (from a previous run of the test)
    let json = r#"{"element-6066-11e4-a52e-4f735466cecf":"fbe5004d-ec8b-4c7b-ad08-642c55d84505"}"#;
    c.execute_script("arguments[0].scrollIntoView(true);", vec![serde_json::from_str(json)?])
        .await
        .expect_err("Failure expected with an invalid ID");

    c.close().await
}

async fn iframe_switch(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;
    // Go to the page that holds the iframe
    c.find_element(By::Css("#iframe_page_id")).await?.click().await?;

    c.find_element(By::Id("iframe_button"))
        .await
        .expect_err("should not find the button in the iframe");
    c.find_element(By::Id("root_button")).await?; // Can find the button in the root context though.

    // find and switch into the iframe
    let iframe_element = c.find_element(By::Id("iframe")).await?;
    iframe_element.enter_frame().await?;

    // search for something in the iframe
    let button_in_iframe = c.find_element(By::Id("iframe_button")).await?;
    button_in_iframe.click().await?;
    c.find_element(By::Id("root_button"))
        .await
        .expect_err("Should not be able to access content in the root context");

    // switch back to the root context and access content there.
    c.switch_to().parent_frame().await?;
    c.find_element(By::Id("root_button")).await?;

    c.close().await
}

async fn new_window(c: WebDriver) -> Result<(), WebDriverError> {
    c.switch_to().new_window().await?;
    let windows = c.window_handles().await?;
    assert_eq!(windows.len(), 2);
    c.close().await
}

async fn new_window_switch(c: WebDriver) -> Result<(), WebDriverError> {
    let window_1 = c.current_window_handle().await?;
    c.switch_to().new_window().await?;
    let window_2 = c.current_window_handle().await?;
    assert_eq!(
        window_1, window_2,
        "After creating a new window, the session should not have switched to it"
    );

    let all_windows = c.window_handles().await?;
    assert_eq!(all_windows.len(), 2);
    let new_window = all_windows
        .into_iter()
        .find(|handle| handle != &window_1)
        .expect("Should find a differing window handle");

    c.switch_to().window(new_window).await?;

    let window_3 = c.current_window_handle().await?;
    assert_ne!(
        window_3, window_2,
        "After switching to a new window, the window handle returned from window() should differ now."
    );

    c.close().await
}

async fn new_tab_switch(c: WebDriver) -> Result<(), WebDriverError> {
    let window_1 = c.current_window_handle().await?;
    c.switch_to().new_tab().await?;
    let window_2 = c.current_window_handle().await?;
    assert_eq!(
        window_1, window_2,
        "After creating a new window, the session should not have switched to it"
    );

    let all_windows = c.window_handles().await?;
    assert_eq!(all_windows.len(), 2);
    let new_window = all_windows
        .into_iter()
        .find(|handle| handle != &window_1)
        .expect("Should find a differing window handle");

    c.switch_to().window(new_window).await?;

    let window_3 = c.current_window_handle().await?;
    assert_ne!(
        window_3, window_2,
        "After switching to a new window, the window handle returned from window() should differ now."
    );

    c.close().await
}

async fn close_window(c: WebDriver) -> Result<(), WebDriverError> {
    let window_1 = c.current_window_handle().await?;
    c.switch_to().new_tab().await?;
    let window_2 = c.current_window_handle().await?;
    assert_eq!(
        window_1, window_2,
        "Creating a new window should not cause the client to switch to it."
    );

    let handles = c.window_handles().await?;
    assert_eq!(handles.len(), 2);

    c.close().await?;
    c.current_window_handle()
        .await
        .expect_err("After closing a window, the client can't find its currently selected window.");

    let other_window = handles
        .into_iter()
        .find(|handle| handle != &window_2)
        .expect("Should find a differing handle");
    c.switch_to().window(other_window).await?;

    // Close the session by closing the remaining window
    c.close().await?;

    c.window_handles().await.expect_err("Session should be closed.");
    Ok(())
}

async fn close_window_twice_errors(c: WebDriver) -> Result<(), WebDriverError> {
    c.close().await?;
    c.close().await.expect_err("Should get a no such window error");
    Ok(())
}

async fn stale_element(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;
    let elem = c.find_element(By::Css("#other_page_id")).await?;

    // Remove the element from the DOM
    c.execute_script(
        "var elem = document.getElementById('other_page_id');
         elem.parentNode.removeChild(elem);",
        vec![],
    )
    .await?;

    match elem.click().await {
        Err(WebDriverError::NoSuchElement(_)) => Ok(()),
        _ => panic!("Expected a stale element reference error"),
    }
}

async fn select_by_index(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;

    let select_element = c.find_element(By::Css("#select1")).await?;

    // Get first display text
    let initial_text = select_element.prop("value").await?;
    assert_eq!(Some("Select1-Option1".into()), initial_text);

    // Select second option
    select_element.clone().select_by_index(1).await?;

    // Get display text after selection
    let text_after_selecting = select_element.prop("value").await?;
    assert_eq!(Some("Select1-Option2".into()), text_after_selecting);

    // Check that the second select is not changed
    let select2_text = c.find_element(By::Css("#select2")).await?.prop("value").await?;
    assert_eq!(Some("Select2-Option1".into()), select2_text);

    // Show off that it selects only options and skip any other elements
    let select_element = c.find_element(By::Css("#select2")).await?;
    select_element.clone().select_by_index(1).await?;
    let text = select_element.prop("value").await?;
    assert_eq!(Some("Select2-Option2".into()), text);

    Ok(())
}

async fn select_by_label(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;

    let select_element = c.find_element(By::Css("#select1")).await?;

    // Get first display text
    let initial_text = select_element.prop("value").await?;
    assert_eq!(Some("Select1-Option1".into()), initial_text);

    // Select second option
    select_element.clone().select_by_label("Select1-Option2").await?;

    // Get display text after selection
    let text_after_selecting = select_element.prop("value").await?;
    assert_eq!(Some("Select1-Option2".into()), text_after_selecting);

    // Check that the second select is not changed
    let select2_text = c.find_element(By::Css("#select2")).await?.prop("value").await?;
    assert_eq!(Some("Select2-Option1".into()), select2_text);

    Ok(())
}

async fn resolve_execute_async_value(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.get(&url).await?;

    let count: u64 = c
        .execute_script_async(
            "setTimeout(() => arguments[1](arguments[0] + 1))",
            vec![1_u32.into()],
        )
        .await?
        .convert()
        .expect("should be integer variant");

    assert_eq!(2, count);

    let count: u64 = c
        .execute_script_async("setTimeout(() => arguments[0](2))", vec![])
        .await?
        .convert()
        .expect("should be integer variant");

    assert_eq!(2, count);

    Ok(())
}

async fn back_and_forward(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;

    assert_eq!(c.current_url().await?.as_str(), sample_url);

    let other_url = other_page_url(port);
    c.get(&other_url).await?;
    assert_eq!(c.current_url().await?.as_str(), other_url);

    c.back().await?;
    assert_eq!(c.current_url().await?.as_str(), sample_url);

    c.forward().await?;
    assert_eq!(c.current_url().await?.as_str(), other_url);

    Ok(())
}

async fn status_firefox(c: WebDriver, _: u16) -> Result<(), WebDriverError> {
    // Geckodriver only supports a single session, and since we're already in a
    // session, it should return `false` here.
    assert!(!c.status().await?.ready);
    Ok(())
}

async fn status_chrome(c: WebDriver, _: u16) -> Result<(), WebDriverError> {
    // Chromedriver supports multiple sessions, so it should always return
    // `true` here.
    assert!(c.status().await?.ready);
    Ok(())
}

async fn page_title(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    assert_eq!(c.title().await?, "Sample Page");
    Ok(())
}

async fn timeouts(c: WebDriver, _: u16) -> Result<(), WebDriverError> {
    let new_timeouts = TimeoutConfiguration::new(
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(30)),
    );
    c.set_timeouts(new_timeouts.clone()).await?;

    let got_timeouts = c.get_timeouts().await?;
    assert_eq!(got_timeouts, new_timeouts);

    // Ensure partial update also works.
    let update_timeouts = TimeoutConfiguration::new(None, None, Some(Duration::from_secs(0)));
    c.set_timeouts(update_timeouts.clone()).await?;

    let got_timeouts = c.get_timeouts().await?;
    assert_eq!(
        got_timeouts,
        TimeoutConfiguration::new(
            new_timeouts.script(),
            new_timeouts.page_load(),
            update_timeouts.implicit()
        )
    );

    Ok(())
}

mod firefox {
    use super::*;
    #[test]
    #[serial]
    fn navigate_to_other_page() {
        local_tester!(goto, "firefox");
    }

    #[test]
    #[serial]
    fn find_and_click_link_test() {
        local_tester!(find_and_click_link, "firefox");
    }

    #[test]
    #[serial]
    fn get_active_element_test() {
        local_tester!(get_active_element, "firefox");
    }

    #[test]
    #[serial]
    fn serialize_element_test() {
        local_tester!(serialize_element, "firefox");
    }

    #[test]
    #[serial]
    fn iframe_test() {
        local_tester!(iframe_switch, "firefox");
    }

    #[test]
    #[serial]
    fn new_window_test() {
        tester!(new_window, "firefox");
    }

    #[test]
    #[serial]
    fn new_window_switch_test() {
        tester!(new_window_switch, "firefox");
    }

    #[test]
    #[serial]
    fn new_tab_switch_test() {
        tester!(new_tab_switch, "firefox");
    }

    #[test]
    #[serial]
    fn close_window_test() {
        tester!(close_window, "firefox");
    }

    #[test]
    #[serial]
    fn double_close_window_test() {
        tester!(close_window_twice_errors, "firefox");
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
    fn resolve_execute_async_value_test() {
        local_tester!(resolve_execute_async_value, "firefox");
    }

    #[test]
    #[serial]
    fn back_and_forward_test() {
        local_tester!(back_and_forward, "firefox");
    }

    #[test]
    #[serial]
    fn status_test() {
        local_tester!(status_firefox, "firefox");
    }

    #[test]
    #[serial]
    fn title_test() {
        local_tester!(page_title, "firefox");
    }

    #[test]
    #[serial]
    fn timeouts_test() {
        local_tester!(timeouts, "firefox");
    }
}

mod chrome {
    use super::*;
    #[test]
    fn navigate_to_other_page() {
        local_tester!(goto, "chrome");
    }

    #[test]
    fn find_and_click_link_test() {
        local_tester!(find_and_click_link, "chrome");
    }

    #[test]
    fn get_active_element_test() {
        local_tester!(get_active_element, "chrome");
    }

    #[test]
    fn serialize_element_test() {
        local_tester!(serialize_element, "chrome");
    }

    #[test]
    fn iframe_test() {
        local_tester!(iframe_switch, "chrome");
    }

    #[test]
    fn new_window_test() {
        tester!(new_window, "chrome");
    }

    #[test]
    fn new_window_switch_test() {
        tester!(new_window_switch, "chrome");
    }

    #[test]
    fn new_tab_test() {
        tester!(new_tab_switch, "chrome");
    }

    #[test]
    fn close_window_test() {
        tester!(close_window, "chrome");
    }

    #[test]
    fn double_close_window_test() {
        tester!(close_window_twice_errors, "chrome");
    }

    #[test]
    #[serial]
    fn select_by_label_test() {
        local_tester!(select_by_label, "chrome");
    }

    #[test]
    fn select_by_index_label() {
        local_tester!(select_by_index, "chrome");
    }

    #[test]
    fn back_and_forward_test() {
        local_tester!(back_and_forward, "chrome");
    }

    #[test]
    fn status_test() {
        local_tester!(status_chrome, "chrome");
    }

    #[test]
    fn title_test() {
        local_tester!(page_title, "chrome");
    }

    #[test]
    fn timeouts_test() {
        local_tester!(timeouts, "chrome");
    }
}
