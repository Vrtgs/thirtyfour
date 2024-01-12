use common::*;
use rstest::rstest;
use thirtyfour::{prelude::*, support::block_on};

mod common;

#[rstest]
fn iframe_switch(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;
        // Go to the page that holds the iframe
        c.find(By::Css("#iframe_page_id")).await?.click().await?;

        c.find(By::Id("iframe_button"))
            .await
            .expect_err("should not find the button in the iframe");
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
        Ok(())
    })
}

#[rstest]
fn new_window(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        c.new_window().await?;
        let windows = c.windows().await?;
        assert_eq!(windows.len(), 2);
        c.close_window().await
    })
}

#[rstest]
fn new_window_switch(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
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
    })
}

#[rstest]
fn new_tab_switch(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
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
    })
}

#[rstest]
fn close_window(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    block_on(async {
        let c = test_harness.driver();
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
        c.window().await.expect_err(
            "After closing a window, the client can't find its currently selected window.",
        );

        let other_window = handles
            .into_iter()
            .find(|handle| handle != &window_2)
            .expect("Should find a differing handle");
        c.switch_to_window(other_window).await?;

        // Close the session by closing the remaining window
        c.close_window().await?;
        c.windows().await.expect_err("Session should be closed.");
        test_harness.disable_auto_close();
        Ok(())
    })
}

#[rstest]
fn close_window_twice_errors(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    block_on(async {
        let c = test_harness.driver();
        c.close_window().await?;
        c.close_window().await.expect_err("Should get a no such window error");
        test_harness.disable_auto_close();
        Ok(())
    })
}

#[rstest]
fn windwow_name(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        let main_title = c.title().await?;
        let handle = c.window().await?;
        c.set_window_name("main").await?;

        // Open a new tab.
        let new_handle = c.new_tab().await?;
        c.switch_to_window(new_handle).await?;

        // We are now controlling the new tab.
        let other_page_url = other_page_url();
        c.goto(&other_page_url).await?;
        assert_ne!(c.window().await?, handle);

        let other_title = c.title().await?;
        assert_ne!(other_title, main_title);

        // Switch back to original tab using window name.
        c.switch_to_named_window("main").await?;
        assert_eq!(c.window().await?, handle);

        Ok(())
    })
}

#[rstest]
fn in_new_tab(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        let main_title = c.title().await?;
        assert_eq!(main_title, "Sample Page");

        let other_page_url = other_page_url();
        let other_title = c
            .in_new_tab(|| async {
                c.goto(&other_page_url).await?;
                c.title().await
            })
            .await?;
        assert_eq!(other_title, "Other Page");
        assert_eq!(c.title().await?, main_title);

        Ok(())
    })
}

#[rstest]
fn window_rect(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    block_on(async {
        let c = test_harness.driver();
        c.set_window_rect(20, 20, 1900, 1000).await?;
        let r = c.get_window_rect().await?;

        // On Mac OS, the window position doesn't seem to be returned correctly.
        if !cfg!(target_os = "macos") {
            if test_harness.browser() == "firefox" {
                // Firefox driver seems to have a bug where it doesn't get the window size correctly.
                // The x coordinate can be completely wrong.
                assert_eq!(r.y, 20);
            } else {
                assert_eq!(r.x, 20);
                assert_eq!(r.y, 20);
            }
        }
        assert_eq!(r.width, 1900);
        assert_eq!(r.height, 1000);
        Ok(())
    })
}

#[rstest]
fn screenshot(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let url = sample_page_url();
        c.goto(&url).await?;

        let screenshot_data = c.screenshot_as_png().await?;
        assert!(!screenshot_data.is_empty(), "screenshot data is empty");
        Ok(())
    })
}
