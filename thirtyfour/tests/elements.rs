//! Element tests
use crate::common::sample_page_url;
use serial_test::serial;
use thirtyfour::prelude::*;

mod common;

async fn element_is(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("checkbox-option-1")).await?;
    assert!(elem.is_enabled().await?);
    assert!(elem.is_displayed().await?);
    assert!(!elem.is_selected().await?);
    assert!(elem.is_present().await?);
    assert!(elem.is_clickable().await?);
    elem.click().await?;
    let elem = c.find(By::Id("checkbox-option-1")).await?;
    assert!(elem.is_selected().await?);

    assert!(!c.find(By::Id("checkbox-disabled")).await?.is_enabled().await?);
    assert!(!c.find(By::Id("checkbox-hidden")).await?.is_displayed().await?);
    Ok(())
}

async fn element_attr(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("checkbox-option-1")).await?;
    assert_eq!(elem.attr("id").await?.unwrap(), "checkbox-option-1");
    assert_eq!(elem.id().await?.unwrap(), "checkbox-option-1");
    assert!(elem.attr("invalid-attribute").await?.is_none());
    Ok(())
}

async fn element_prop(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("checkbox-option-1")).await?;
    assert_eq!(elem.prop("id").await?.unwrap(), "checkbox-option-1");
    assert_eq!(elem.prop("checked").await?.unwrap(), "false");
    assert!(elem.attr("invalid-property").await?.is_none());
    Ok(())
}

async fn element_css_value(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("checkbox-hidden")).await?;
    assert_eq!(elem.css_value("display").await?, "none");
    assert_eq!(elem.css_value("invalid-css-value").await?, "");
    Ok(())
}

async fn element_tag_name(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("checkbox-option-1")).await?;
    let tag_name = elem.tag_name().await?;
    assert!(tag_name.eq_ignore_ascii_case("input"), "{} != input", tag_name);
    Ok(())
}

async fn element_class_name(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::ClassName("vertical")).await?;
    let class_name = elem.class_name().await?.unwrap();
    assert!(class_name.eq_ignore_ascii_case("vertical"), "{} != vertical", class_name);
    Ok(())
}

async fn element_text(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("button-copy")).await?;
    assert_eq!(elem.text().await?, "Copy");
    Ok(())
}

async fn element_rect(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("button-alert")).await?;
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
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("text-input")).await?;
    assert_eq!(elem.prop("value").await?.unwrap(), "");
    assert_eq!(elem.value().await?.unwrap(), "");
    elem.send_keys("thirtyfour").await?;
    assert_eq!(elem.prop("value").await?.unwrap(), "thirtyfour");
    assert_eq!(elem.value().await?.unwrap(), "thirtyfour");
    let select_all = if cfg!(target_os = "macos") {
        Key::Command + "a"
    } else {
        Key::Control + "a"
    };
    let backspace = Key::Backspace.to_string();
    elem.send_keys(&select_all).await?;
    elem.send_keys(&backspace).await?;
    assert_eq!(elem.prop("value").await?.unwrap(), "");
    assert_eq!(elem.value().await?.unwrap(), "");

    Ok(())
}

async fn element_clear(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    let elem = c.find(By::Id("text-input")).await?;
    assert_eq!(elem.value().await?.unwrap(), "");
    elem.send_keys("thirtyfour").await?;
    assert_eq!(elem.value().await?.unwrap(), "thirtyfour");
    elem.clear().await?;
    assert_eq!(elem.value().await?.unwrap(), "");
    Ok(())
}

async fn serialize_element(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let elem = c.find(By::Css("#other_page_id")).await?;

    // Check that webdriver understands it
    c.execute("arguments[0].scrollIntoView(true);", vec![elem.to_json()?]).await?;

    // This does the same thing.
    elem.scroll_into_view().await?;

    // Check that it fails with an invalid serialization (from a previous run of the test)
    let json = r#"{"element-6066-11e4-a52e-4f735466cecf":"fbe5004d-ec8b-4c7b-ad08-642c55d84505"}"#;

    c.execute("arguments[0].scrollIntoView(true);", vec![serde_json::from_str(json)?])
        .await
        .expect_err("Failure expected with an invalid ID");

    // You can easily deserialize elements too.
    let ret = c.execute(r#"return document.getElementById("select1");"#, vec![]).await?;
    let elem = ret.element()?;
    assert_eq!(elem.tag_name().await?, "select");

    c.close_window().await
}

async fn element_screenshot(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;

    let elem = c.find(By::Id("select1")).await?;

    let screenshot_data = elem.screenshot_as_png().await?;
    assert!(!screenshot_data.is_empty(), "screenshot data is empty");

    Ok(())
}

async fn element_focus(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let elem = c.find(By::Id("text-input")).await?;
    elem.focus().await?;
    let active_elem = c.active_element().await?;
    assert_eq!(active_elem.id().await?.unwrap(), "text-input");
    Ok(())
}

async fn element_html(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let elem = c.find(By::Id("button-copy")).await?;
    assert_eq!(elem.inner_html().await?, "Copy");

    let elem = c.find(By::Id("text-output")).await?;
    assert_eq!(elem.outer_html().await?, r#"<div id="text-output"></div>"#);
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
    fn element_class_name_test() {
        local_tester!(element_class_name, "firefox");
    }

    #[test]
    #[serial]
    fn element_text_test() {
        local_tester!(element_text, "firefox");
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

    #[test]
    #[serial]
    fn element_clear_test() {
        local_tester!(element_clear, "firefox");
    }

    #[test]
    #[serial]
    fn serialize_element_test() {
        local_tester!(serialize_element, "firefox");
    }

    #[test]
    #[serial]
    fn element_screenshot_test() {
        local_tester!(element_screenshot, "firefox");
    }

    #[test]
    #[serial]
    fn element_focus_test() {
        local_tester!(element_focus, "firefox");
    }

    #[test]
    #[serial]
    fn element_html_test() {
        local_tester!(element_html, "firefox");
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
    fn element_class_name_test() {
        local_tester!(element_class_name, "chrome");
    }

    #[test]
    fn element_text_test() {
        local_tester!(element_text, "chrome");
    }

    #[test]
    fn element_rect_test() {
        local_tester!(element_rect, "chrome");
    }

    #[test]
    fn element_send_keys_test() {
        local_tester!(element_send_keys, "chrome");
    }

    #[test]
    fn element_clear_test() {
        local_tester!(element_clear, "chrome");
    }

    #[test]
    fn serialize_element_test() {
        local_tester!(serialize_element, "chrome");
    }

    #[test]
    fn element_screenshot_test() {
        local_tester!(element_screenshot, "chrome");
    }

    #[test]
    fn element_focus_test() {
        local_tester!(element_focus, "chrome");
    }

    #[test]
    fn element_html_test() {
        local_tester!(element_html, "chrome");
    }
}
