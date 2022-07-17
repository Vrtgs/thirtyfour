use crate::common::{other_page_url, sample_page_url};
use serial_test::serial;
use thirtyfour::prelude::*;

mod common;

async fn goto(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    let current_url = c.current_url().await?;
    assert_eq!(url.as_str(), current_url.as_str());
    let source = c.source().await?;
    println!("source = {source}");
    assert!(source.starts_with("<html"));
    c.close_window().await
}

async fn back_and_forward(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;

    assert_eq!(c.current_url().await?.as_str(), sample_url);

    let other_url = other_page_url(port);
    c.goto(&other_url).await?;
    assert_eq!(c.current_url().await?.as_str(), other_url);

    c.back().await?;
    assert_eq!(c.current_url().await?.as_str(), sample_url);

    c.forward().await?;
    assert_eq!(c.current_url().await?.as_str(), other_url);

    Ok(())
}

async fn find_and_click_link(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;
    c.find(By::Css("#other_page_id")).await?.click().await?;

    let new_url = c.current_url().await?;
    let expected_url = other_page_url(port);
    assert_eq!(new_url.as_str(), expected_url.as_str());

    c.close_window().await
}

async fn page_title(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    assert_eq!(c.title().await?, "Sample Page");
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
    fn back_and_forward_test() {
        local_tester!(back_and_forward, "firefox");
    }

    #[test]
    #[serial]
    fn find_and_click_link_test() {
        local_tester!(find_and_click_link, "firefox");
    }

    #[test]
    #[serial]
    fn title_test() {
        local_tester!(page_title, "firefox");
    }
}

mod chrome {
    use super::*;

    #[test]
    fn navigate_to_other_page() {
        local_tester!(goto, "chrome");
    }

    #[test]
    fn back_and_forward_test() {
        local_tester!(back_and_forward, "chrome");
    }

    #[test]
    fn find_and_click_link_test() {
        local_tester!(find_and_click_link, "chrome");
    }

    #[test]
    fn title_test() {
        local_tester!(page_title, "chrome");
    }
}
