//! Element tests
use crate::common::sample_page_url;
use serial_test::serial;
use thirtyfour::prelude::*;

mod common;

async fn element_is(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    let elem = c.find_element(By::Id("checkbox-option-1")).await?;
    assert!(elem.is_enabled().await?);
    assert!(elem.is_displayed().await?);
    assert!(!elem.is_selected().await?);
    elem.click().await?;
    let elem = c.find_element(By::Id("checkbox-option-1")).await?;
    assert!(elem.is_selected().await?);

    assert!(!c.find_element(By::Id("checkbox-disabled")).await?.is_enabled().await?);
    assert!(!c.find_element(By::Id("checkbox-hidden")).await?.is_displayed().await?);
    Ok(())
}

async fn element_attr(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    let elem = c.find_element(By::Id("checkbox-option-1")).await?;
    assert_eq!(elem.get_attribute("id").await?.unwrap(), "checkbox-option-1");
    assert!(elem.get_attribute("invalid-attribute").await?.is_none());
    Ok(())
}

async fn element_prop(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    let elem = c.find_element(By::Id("checkbox-option-1")).await?;
    assert_eq!(elem.get_property("id").await?.unwrap(), "checkbox-option-1");
    assert_eq!(elem.get_property("checked").await?.unwrap(), "false");
    assert!(elem.get_attribute("invalid-property").await?.is_none());
    Ok(())
}

async fn element_css_value(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    let elem = c.find_element(By::Id("checkbox-hidden")).await?;
    assert_eq!(elem.get_css_property("display").await?, "none");
    assert_eq!(elem.get_css_property("invalid-css-value").await?, "");
    Ok(())
}

async fn element_tag_name(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    let elem = c.find_element(By::Id("checkbox-option-1")).await?;
    let tag_name = elem.tag_name().await?;
    assert!(tag_name.eq_ignore_ascii_case("input"), "{} != input", tag_name);
    Ok(())
}

async fn element_rect(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    let elem = c.find_element(By::Id("button-alert")).await?;
    let rect = elem.rect().await?;
    // Rather than try to verify the exact position and size of the element,
    // let's just verify that the returned values deserialized ok and
    // are within the expected range.
    assert!(rect.x > 0.0);
    assert!(rect.x < 100.0);
    assert!(rect.y > 0.0);
    assert!(rect.y < 1000.0);
    assert!(rect.width > 0.0);
    assert!(rect.width < 200.0);
    assert!(rect.height > 0.0);
    assert!(rect.height < 200.0);
    Ok(())
}

async fn element_send_keys(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    let elem = c.find_element(By::Id("text-input")).await?;
    assert_eq!(elem.get_property("value").await?.unwrap(), "");
    elem.send_keys("fantoccini").await?;
    assert_eq!(elem.get_property("value").await?.unwrap(), "fantoccini");
    let select_all = if cfg!(target_os = "macos") {
        Key::Command + "a"
    } else {
        Key::Control + "a"
    };
    let backspace = Key::Backspace.to_string();
    elem.send_keys(&select_all).await?;
    elem.send_keys(&backspace).await?;
    assert_eq!(elem.get_property("value").await?.unwrap(), "");

    Ok(())
}

mod firefox {
    use super::*;

    #[test]
    #[serial]
    fn element_is_test() {
        local_tester!(element_is, "firefox");
    }

    #[test]
    #[serial]
    fn element_attr_test() {
        local_tester!(element_attr, "firefox");
    }

    #[test]
    #[serial]
    fn element_prop_test() {
        local_tester!(element_prop, "firefox");
    }

    #[test]
    #[serial]
    fn element_css_value_test() {
        local_tester!(element_css_value, "firefox");
    }

    #[test]
    #[serial]
    fn element_tag_name_test() {
        local_tester!(element_tag_name, "firefox");
    }

    #[test]
    #[serial]
    fn element_rect_test() {
        local_tester!(element_rect, "firefox");
    }

    #[test]
    #[serial]
    fn element_send_keys_test() {
        local_tester!(element_send_keys, "firefox");
    }
}

mod chrome {
    use super::*;

    #[test]
    fn element_is_test() {
        local_tester!(element_is, "chrome");
    }

    #[test]
    fn element_attr_test() {
        local_tester!(element_attr, "chrome");
    }

    #[test]
    fn element_prop_test() {
        local_tester!(element_prop, "chrome");
    }

    #[test]
    fn element_css_value_test() {
        local_tester!(element_css_value, "chrome");
    }

    #[test]
    fn element_tag_name_test() {
        local_tester!(element_tag_name, "chrome");
    }

    #[test]
    fn element_rect_test() {
        local_tester!(element_rect, "chrome");
    }

    #[test]
    fn element_send_keys_test() {
        local_tester!(element_send_keys, "chrome");
    }
}
