//! Actions tests

use assert_matches::assert_matches;
use thirtyfour::prelude::*;

use crate::common::{drag_to_url, sample_page_url};

mod common;

async fn actions_key(c: WebDriver) -> Result<(), WebDriverError> {
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
}

async fn actions_mouse(c: WebDriver) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url();
    c.goto(&sample_url).await?;

    let elem = c.find(By::Id("button-alert")).await?;

    // Test mouse down/up.
    c.action_chain().move_to_element_center(&elem).click().perform().await?;
    assert_eq!(c.get_alert_text().await?, "This is an alert");
    c.dismiss_alert().await?;
    Ok(())
}

async fn actions_mouse_move(c: WebDriver) -> Result<(), WebDriverError> {
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
}

async fn actions_release(c: WebDriver) -> Result<(), WebDriverError> {
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
}

async fn actions_drag_and_drop(c: WebDriver) -> Result<(), WebDriverError> {
    let drag_to_url = drag_to_url();
    c.goto(&drag_to_url).await?;

    // Validate we are starting with a div and an image that are adjacent to one another.
    c.find(By::XPath("//div[@id='target']/../img[@id='draggable']")).await?;

    // Drag the image to the target div
    let elem = c.find(By::Id("draggable")).await?;
    let target = c.find(By::Id("target")).await?;
    c.action_chain().drag_and_drop_element(&elem, &target).perform().await?;

    // Currently drag and drop is known to fail due to a bug.
    // See the docs for `drag_and_drop_element()` for more info.
    // For now we just confirm that the issue persists. If this unit test fails,
    // it means the bug has been fixed and we need to update the docs (and this test).

    assert_matches!(
        c.find(By::XPath("//div[@id='target']/img[@id='draggable']")).await,
        Err(WebDriverError::NoSuchElement(_))
    );

    Ok(())
}

mod tests {
    use super::*;

    local_tester!(
        actions_key,
        actions_mouse,
        actions_mouse_move,
        actions_release,
        actions_drag_and_drop
    );
}
