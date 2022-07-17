//! Alert tests
use crate::common::sample_page_url;
use fantoccini::error::CmdError;
use serial_test::serial;
use thirtyfour::prelude::*;

mod common;

async fn alert_accept(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    c.find_element(By::Id("button-alert")).await?.click().await?;
    let alert = c.switch_to().alert();
    assert_eq!(alert.text().await?, "This is an alert");
    alert.accept().await?;
    assert!(matches!(alert.text().await, Err(WebDriverError::NoSuchAlert(..))));

    c.find_element(By::Id("button-confirm")).await?.click().await?;
    let alert = c.switch_to().alert();
    assert_eq!(alert.text().await?, "Press OK or Cancel");
    alert.accept().await?;
    assert!(matches!(alert.text().await, Err(WebDriverError::NoSuchAlert(..))));
    assert_eq!(c.find_element(By::Id("alert-answer")).await?.text().await?, "OK");

    Ok(())
}

async fn alert_dismiss(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    c.find_element(By::Id("button-alert")).await?.click().await?;

    let alert = c.switch_to().alert();
    assert_eq!(alert.text().await?, "This is an alert");
    alert.dismiss().await?;
    assert!(matches!(alert.text().await, Err(WebDriverError::NoSuchAlert(..))));

    c.find_element(By::Id("button-confirm")).await?.click().await?;
    let alert = c.switch_to().alert();
    assert_eq!(alert.text().await?, "Press OK or Cancel");
    alert.dismiss().await?;
    assert!(matches!(alert.text().await, Err(WebDriverError::NoSuchAlert(..))));
    assert_eq!(c.find_element(By::Id("alert-answer")).await?.text().await?, "Cancel");

    Ok(())
}

async fn alert_text(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.get(&sample_url).await?;
    c.find_element(By::Id("button-prompt")).await?.click().await?;
    let alert = c.switch_to().alert();
    assert_eq!(alert.text().await?, "What is your name?");
    alert.send_keys("Fantoccini").await?;
    alert.accept().await?;
    assert!(matches!(alert.text().await, Err(WebDriverError::NoSuchAlert(..))));
    assert_eq!(c.find_element(By::Id("alert-answer")).await?.text().await?, "Fantoccini");

    Ok(())
}

mod firefox {
    use super::*;

    #[test]
    #[serial]
    fn alert_accept_test() {
        local_tester!(alert_accept, "firefox");
    }

    #[test]
    #[serial]
    fn alert_dismiss_test() {
        local_tester!(alert_dismiss, "firefox");
    }

    #[test]
    #[serial]
    fn alert_text_test() {
        local_tester!(alert_text, "firefox");
    }
}

mod chrome {
    use super::*;

    #[test]
    fn alert_accept_test() {
        local_tester!(alert_accept, "chrome");
    }

    #[test]
    fn alert_dismiss_test() {
        local_tester!(alert_dismiss, "chrome");
    }

    #[test]
    fn alert_text_test() {
        local_tester!(alert_text, "chrome");
    }
}
