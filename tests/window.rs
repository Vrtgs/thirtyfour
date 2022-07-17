use crate::common::sample_page_url;
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

async fn window_rect(c: WebDriver) -> Result<(), WebDriverError> {
    c.set_window_rect(0, 0, 1920, 1080).await?;
    let r = c.get_window_rect().await?;
    assert_eq!(r.x, 0);
    assert_eq!(r.y, 0);
    assert_eq!(r.width, 1920);
    assert_eq!(r.height, 1080);
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
}
