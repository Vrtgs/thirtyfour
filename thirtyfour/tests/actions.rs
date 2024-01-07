//! Actions tests

use assert_matches::assert_matches;
use rstest::rstest;
use thirtyfour::{prelude::*, support::block_on};

use crate::common::*;

mod common;

#[rstest]
fn actions_key(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let sample_url = sample_page_url();
        c.goto(&sample_url).await?;

        // Test key down/up.
        let elem = c.find(By::Id("text-input")).await?;
        elem.send_keys("a").await?;
        assert_eq!(elem.prop("value").await?.unwrap(), "a");

        elem.click().await?;
        c.action_chain().key_down(Key::Backspace).key_up(Key::Backspace).perform().await?;
        let elem = c.find(By::Id("text-input")).await?;
        assert_eq!(elem.prop("value").await?.unwrap(), "");

        Ok(())
    })
}

#[rstest]
fn actions_mouse(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let sample_url = sample_page_url();
        c.goto(&sample_url).await?;

        let elem = c.find(By::Id("button-alert")).await?;

        // Test mouse down/up.
        c.action_chain().move_to_element_center(&elem).click().perform().await?;
        assert_eq!(c.get_alert_text().await?, "This is an alert");
        c.dismiss_alert().await?;
        Ok(())
    })
}

#[rstest]
fn actions_mouse_move(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        // Set window size to avoid moving the cursor out-of-bounds during actions.
        c.set_window_rect(0, 0, 800, 800).await?;

        let sample_url = sample_page_url();
        c.goto(&sample_url).await?;

        let elem = c.find(By::Id("button-alert")).await?;
        let rect = elem.rect().await?;
        let elem_center_x = rect.x + (rect.width / 2.0);
        let elem_center_y = rect.y + (rect.height / 2.0);

        // Test mouse MoveBy.

        // Sanity check - ensure no alerts are displayed prior to actions.
        assert_matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..)));

        c.action_chain()
            .move_to(0, elem_center_y as i64 - 100)
            .move_by_offset(elem_center_x as i64, 100)
            .click()
            .perform()
            .await?;
        assert_eq!(c.get_alert_text().await?, "This is an alert");
        c.accept_alert().await?;

        Ok(())
    })
}

#[rstest]
fn actions_release(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let sample_url = sample_page_url();
        c.goto(&sample_url).await?;

        // Focus the input element.
        let elem = c.find(By::Id("text-input")).await?;
        elem.click().await?;

        // Add initial text.
        let elem = c.find(By::Id("text-input")).await?;
        assert_eq!(elem.prop("value").await?.unwrap(), "");

        // Press CONTROL key down and hold it.
        c.action_chain().key_down(Key::Control).perform().await?;

        // Now release all actions. This should release the control key.
        c.action_chain().reset_actions().await?;

        // Now press the 'a' key again.
        //
        // If the Control key was not released, this would do `Ctrl+a` (i.e. select all)
        // but there is no text so it would do nothing.
        //
        // However if the Control key was released (as expected)
        // then this will type 'a' into the text element.
        c.action_chain().key_down('a').perform().await?;
        assert_eq!(elem.prop("value").await?.unwrap(), "a");
        Ok(())
    })
}

#[rstest]
fn actions_drag_and_drop(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let drag_to_url = drag_to_url();
        c.goto(&drag_to_url).await?;

        // Validate we are starting with a div and an image that are adjacent to one another.
        c.find(By::XPath("//div[@id='target']/../img[@id='draggable']")).await?;

        // Drag the image to the target div
        let elem = c.find(By::Id("draggable")).await?;
        let target = c.find(By::Id("target")).await?;
        c.action_chain().drag_and_drop_element(&elem, &target).perform().await?;

        c.find(By::XPath("//div[@id='target']/img[@id='draggable']")).await?;

        Ok(())
    })
}
