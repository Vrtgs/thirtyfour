use crate::common::{other_page_url, sample_page_url};
use serial_test::serial;
use thirtyfour::{components::SelectElement, prelude::*};

mod common;

async fn goto(c: WebDriver) -> Result<(), WebDriverError> {
    let url = sample_page_url();
    c.goto(&url).await?;
    let current_url = c.current_url().await?;
    assert_eq!(url.as_str(), current_url.as_str());
    let source = c.source().await?;
    println!("source = {source}");
    assert!(source.starts_with("<html"));
    c.close_window().await
}

async fn back_and_forward(c: WebDriver) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url();
    c.goto(&sample_url).await?;

    assert_eq!(c.current_url().await?.as_str(), sample_url);

    let other_url = other_page_url();
    c.goto(&other_url).await?;
    assert_eq!(c.current_url().await?.as_str(), other_url);

    c.back().await?;
    assert_eq!(c.current_url().await?.as_str(), sample_url);

    c.forward().await?;
    assert_eq!(c.current_url().await?.as_str(), other_url);

    Ok(())
}

async fn refresh(c: WebDriver) -> Result<(), WebDriverError> {
    let url = sample_page_url();
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

    // Refresh the page.
    c.refresh().await?;

    let elem = c.find(By::Css("#select1")).await?;

    // Get display text after refresh.
    let text_after_refresh = elem.prop("value").await?;
    assert_eq!(Some("Select1-Option1".into()), text_after_refresh);

    Ok(())
}

async fn find_and_click_link(c: WebDriver) -> Result<(), WebDriverError> {
    let url = sample_page_url();
    c.goto(&url).await?;
    c.find(By::Css("#other_page_id")).await?.click().await?;

    let new_url = c.current_url().await?;
    let expected_url = other_page_url();
    assert_eq!(new_url.as_str(), expected_url.as_str());

    c.close_window().await
}

async fn page_title(c: WebDriver) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url();
    c.goto(&sample_url).await?;
    assert_eq!(c.title().await?, "Sample Page");
    Ok(())
}

mod tests {
    use super::*;

    #[test]
    #[serial]
    fn navigate_to_other_page() {
        local_tester!(goto);
    }

    #[test]
    #[serial]
    fn back_and_forward_test() {
        local_tester!(back_and_forward);
    }

    #[test]
    #[serial]
    fn refresh_test() {
        local_tester!(refresh);
    }

    #[test]
    #[serial]
    fn find_and_click_link_test() {
        local_tester!(find_and_click_link);
    }

    #[test]
    #[serial]
    fn title_test() {
        local_tester!(page_title);
    }
}
