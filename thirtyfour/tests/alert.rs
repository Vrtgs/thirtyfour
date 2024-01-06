//! Alert tests
use crate::common::sample_page_url;
use serial_test::serial;
use thirtyfour::prelude::*;

mod common;

async fn alert_accept(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    c.find(By::Id("button-alert")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "This is an alert");
    c.accept_alert().await?;
    assert!(matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..))));

    c.find(By::Id("button-confirm")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "Press OK or Cancel");
    c.accept_alert().await?;
    assert!(matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..))));
    assert_eq!(c.find(By::Id("alert-answer")).await?.text().await?, "OK");

    Ok(())
}

async fn alert_dismiss(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    c.find(By::Id("button-alert")).await?.click().await?;

    assert_eq!(c.get_alert_text().await?, "This is an alert");
    c.dismiss_alert().await?;
    assert!(matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..))));

    c.find(By::Id("button-confirm")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "Press OK or Cancel");
    c.dismiss_alert().await?;
    assert!(matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..))));
    assert_eq!(c.find(By::Id("alert-answer")).await?.text().await?, "Cancel");

    Ok(())
}

async fn alert_text(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url(port);
    c.goto(&sample_url).await?;
    c.find(By::Id("button-prompt")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "What is your name?");
    c.send_alert_text("Thirtyfour").await?;
    c.accept_alert().await?;
    assert!(matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..))));
    assert_eq!(c.find(By::Id("alert-answer")).await?.text().await?, "Thirtyfour");

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
