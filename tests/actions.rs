//! Actions tests
use crate::common::sample_page_url;
use serial_test::serial;
use thirtyfour::prelude::*;

mod common;

async fn actions_key(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;

    // Test key down/up.
    let elem = c.find_element(By::Id("text-input")).await?;
    elem.send_keys("a").await?;
    assert_eq!(elem.get_property("value").await?.unwrap(), "a");

    elem.click().await?;
    c.action_chain().key_down(Key::Backspace).key_up(Key::Backspace).perform().await?;
    let elem = c.find_element(By::Id("text-input")).await?;
    assert_eq!(elem.get_property("value").await?.unwrap(), "");
    Ok(())
}

async fn actions_mouse(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;

    let elem = c.find_element(By::Id("button-alert")).await?;

    // Test mouse down/up.
    c.action_chain().move_to_element_center(&elem).click().perform().await?;
    let alert = c.switch_to().alert();
    assert_eq!(alert.text().await?, "This is an alert");
    alert.dismiss().await?;
    Ok(())
}

async fn actions_mouse_move(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    // Set window size to avoid moving the cursor out-of-bounds during actions.
    c.set_window_rect(0, 0, 800, 800).await?;

    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;

    let elem = c.find_element(By::Id("button-alert")).await?;
    let rect = elem.rect().await?;
    let elem_center_x = rect.x + (rect.width / 2.0);
    let elem_center_y = rect.y + (rect.height / 2.0);

    // Test mouse MoveBy.

    // Sanity check - ensure no alerts are displayed prior to actions.
    assert!(matches!(c.switch_to().alert().text().await, Err(WebDriverError::NoSuchAlert(..))));

    c.action_chain()
        .move_to(0, elem_center_y as i64 - 100)
        .move_by_offset(elem_center_x as i64, 100)
        .click()
        .perform()
        .await?;
    let alert = c.switch_to().alert();
    assert_eq!(alert.text().await?, "This is an alert");
    alert.accept().await?;

    Ok(())
}

async fn actions_release(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;

    // Focus the input element.
    let elem = c.find_element(By::Id("text-input")).await?;
    elem.click().await?;

    // Add initial text.
    let elem = c.find_element(By::Id("text-input")).await?;
    assert_eq!(elem.get_property("value").await?.unwrap(), "");

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
    assert_eq!(elem.get_property("value").await?.unwrap(), "a");
    Ok(())
}

mod firefox {
    use super::*;

    #[test]
    #[serial]
    fn actions_key_test() {
        local_tester!(actions_key, "firefox");
    }

    #[test]
    #[serial]
    fn actions_mouse_test() {
        local_tester!(actions_mouse, "firefox");
    }

    #[test]
    #[serial]
    fn actions_mouse_move_test() {
        local_tester!(actions_mouse_move, "firefox");
    }

    #[test]
    #[serial]
    fn actions_release_test() {
        local_tester!(actions_release, "firefox");
    }
}

mod chrome {
    use super::*;

    #[test]
    fn actions_key_test() {
        local_tester!(actions_key, "chrome");
    }

    #[test]
    fn actions_mouse_test() {
        local_tester!(actions_mouse, "chrome");
    }

    #[test]
    fn actions_mouse_move_test() {
        local_tester!(actions_mouse_move, "chrome");
    }

    #[test]
    fn actions_release_test() {
        local_tester!(actions_release, "chrome");
    }
}
