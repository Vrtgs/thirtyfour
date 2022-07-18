use std::path::Path;

use crate::common::sample_page_url;
use common::other_page_url;
use serial_test::serial;
use thirtyfour::prelude::*;

mod common;

async fn iframe_switch(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    // Go to the page that holds the iframe
    c.find(By::Css("#iframe_page_id")).await?.click().await?;

    c.find(By::Id("iframe_button")).await.expect_err("should not find the button in the iframe");
    c.find(By::Id("root_button")).await?; // Can find the button in the root context though.

    // find and switch into the iframe
    let iframe_element = c.find(By::Id("iframe")).await?;
    iframe_element.enter_frame().await?;

    // search for something in the iframe
    let button_in_iframe = c.find(By::Id("iframe_button")).await?;
    button_in_iframe.click().await?;
    c.find(By::Id("root_button"))
        .await
        .expect_err("Should not be able to access content in the root context");

    // switch back to the root context and access content there.
    c.enter_parent_frame().await?;
    c.find(By::Id("root_button")).await?;

    c.close_window().await
}

async fn new_window(c: WebDriver) -> Result<(), WebDriverError> {
    c.new_window().await?;
    let windows = c.windows().await?;
    assert_eq!(windows.len(), 2);
    c.close_window().await
}

async fn new_window_switch(c: WebDriver) -> Result<(), WebDriverError> {
    let window_1 = c.window().await?;
    c.new_window().await?;
    let window_2 = c.window().await?;
    assert_eq!(
        window_1, window_2,
        "After creating a new window, the session should not have switched to it"
    );

    let all_windows = c.windows().await?;
    assert_eq!(all_windows.len(), 2);
    let new_window = all_windows
        .into_iter()
        .find(|handle| handle != &window_1)
        .expect("Should find a differing window handle");

    c.switch_to_window(new_window).await?;

    let window_3 = c.window().await?;
    assert_ne!(
        window_3, window_2,
        "After switching to a new window, the window handle returned from window() should differ now."
    );

    c.close_window().await
}

async fn new_tab_switch(c: WebDriver) -> Result<(), WebDriverError> {
    let window_1 = c.window().await?;
    c.new_tab().await?;
    let window_2 = c.window().await?;
    assert_eq!(
        window_1, window_2,
        "After creating a new window, the session should not have switched to it"
    );

    let all_windows = c.windows().await?;
    assert_eq!(all_windows.len(), 2);
    let new_window = all_windows
        .into_iter()
        .find(|handle| handle != &window_1)
        .expect("Should find a differing window handle");

    c.switch_to_window(new_window).await?;

    let window_3 = c.window().await?;
    assert_ne!(
        window_3, window_2,
        "After switching to a new window, the window handle returned from window() should differ now."
    );

    c.close_window().await
}

async fn close_window(c: WebDriver) -> Result<(), WebDriverError> {
    let window_1 = c.window().await?;
    c.new_tab().await?;
    let window_2 = c.window().await?;
    assert_eq!(
        window_1, window_2,
        "Creating a new window should not cause the client to switch to it."
    );

    let handles = c.windows().await?;
    assert_eq!(handles.len(), 2);

    c.close_window().await?;
    c.window()
        .await
        .expect_err("After closing a window, the client can't find its currently selected window.");

    let other_window = handles
        .into_iter()
        .find(|handle| handle != &window_2)
        .expect("Should find a differing handle");
    c.switch_to_window(other_window).await?;

    // Close the session by closing the remaining window
    c.close_window().await?;

    c.windows().await.expect_err("Session should be closed.");
    Ok(())
}

async fn close_window_twice_errors(c: WebDriver) -> Result<(), WebDriverError> {
    c.close_window().await?;
    c.close_window().await.expect_err("Should get a no such window error");
    Ok(())
}

async fn window_name(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;

    let main_title = c.title().await?;
    let handle = c.window().await?;
    c.set_window_name("main").await?;

    // Open a new tab.
    let new_handle = c.new_tab().await?;
    c.switch_to_window(new_handle).await?;

    // We are now controlling the new tab.
    let other_page_url = other_page_url(port);
    c.goto(&other_page_url).await?;
    assert_ne!(c.window().await?, handle);

    let other_title = c.title().await?;
    assert_ne!(other_title, main_title);

    // Switch back to original tab using window name.
    c.switch_to_named_window("main").await?;
    assert_eq!(c.window().await?, handle);

    Ok(())
}

async fn in_new_tab(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;

    let main_title = c.title().await?;
    assert_eq!(main_title, "Sample Page");

    let other_page_url = other_page_url(port);
    let other_title = c
        .in_new_tab(|| async {
            c.goto(&other_page_url).await?;
            c.title().await
        })
        .await?;
    assert_eq!(other_title, "Other Page");
    assert_eq!(c.title().await?, main_title);

    Ok(())
}

async fn window_rect(c: WebDriver) -> Result<(), WebDriverError> {
    c.set_window_rect(10, 10, 1900, 1000).await?;
    let r = c.get_window_rect().await?;
    assert_eq!(r.x, 10);
    assert_eq!(r.y, 10);
    assert_eq!(r.width, 1900);
    assert_eq!(r.height, 1000);
    Ok(())
}

async fn screenshot(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;

    let screenshot_data = c.screenshot_as_png().await?;
    assert!(!screenshot_data.is_empty(), "screenshot data is empty");

    let path = Path::new("screenshot.png");
    c.screenshot(&path).await?;
    assert!(path.exists(), "screenshot file doesn't exist");
    let contents = std::fs::read(path)?;
    assert!(!contents.is_empty(), "screenshot file is empty");
    assert_eq!(contents, screenshot_data);

    Ok(())
}

mod firefox {
    use super::*;

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
    fn window_rect_test() {
        tester!(window_rect, "firefox");
    }

    #[test]
    #[serial]
    fn window_name_test() {
        local_tester!(window_name, "firefox");
    }

    #[test]
    #[serial]
    fn in_new_tab_test() {
        local_tester!(in_new_tab, "firefox");
    }

    #[test]
    #[serial]
    fn screenshot_test() {
        local_tester!(screenshot, "firefox");
    }
}

mod chrome {
    use super::*;

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
    fn window_rect_test() {
        tester!(window_rect, "chrome");
    }

    #[test]
    fn window_name_test() {
        local_tester!(window_name, "chrome");
    }

    #[test]
    fn in_new_tab_test() {
        local_tester!(in_new_tab, "chrome");
    }

    #[test]
    fn screenshot_test() {
        local_tester!(screenshot, "chrome");
    }
}
