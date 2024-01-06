//! Alert tests

use assert_matches::assert_matches;
use thirtyfour::prelude::*;

use crate::common::sample_page_url;

mod common;

async fn alert_accept(c: WebDriver) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url();
    c.goto(&sample_url).await?;
    c.find(By::Id("button-alert")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "This is an alert");
    c.accept_alert().await?;
    assert_matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..)));

    c.find(By::Id("button-confirm")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "Press OK or Cancel");
    c.accept_alert().await?;
    assert_matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..)));
    assert_eq!(c.find(By::Id("alert-answer")).await?.text().await?, "OK");

    Ok(())
}

async fn alert_dismiss(c: WebDriver) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url();
    c.goto(&sample_url).await?;
    c.find(By::Id("button-alert")).await?.click().await?;

    assert_eq!(c.get_alert_text().await?, "This is an alert");
    c.dismiss_alert().await?;
    assert_matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..)));

    c.find(By::Id("button-confirm")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "Press OK or Cancel");
    c.dismiss_alert().await?;
    assert_matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..)));
    assert_eq!(c.find(By::Id("alert-answer")).await?.text().await?, "Cancel");

    Ok(())
}

async fn alert_text(c: WebDriver) -> Result<(), WebDriverError> {
    let sample_url = sample_page_url();
    c.goto(&sample_url).await?;
    c.find(By::Id("button-prompt")).await?.click().await?;
    assert_eq!(c.get_alert_text().await?, "What is your name?");
    c.send_alert_text("Thirtyfour").await?;
    c.accept_alert().await?;
    assert_matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..)));
    assert_eq!(c.find(By::Id("alert-answer")).await?.text().await?, "Thirtyfour");

    Ok(())
}

mod tests {
    use super::*;

    local_tester!(alert_accept, alert_dismiss, alert_text);
}
