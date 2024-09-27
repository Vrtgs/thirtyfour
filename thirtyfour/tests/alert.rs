//! Alert tests

use assert_matches::assert_matches;
use rstest::rstest;
use thirtyfour::{prelude::*, support::block_on};

use crate::common::*;

mod common;

#[rstest]
fn alert_accept(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
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
    })
}

#[rstest]
fn alert_dismiss(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
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
    })
}

#[rstest]
fn alert_text(test_harness: TestHarness) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
        let sample_url = sample_page_url();
        c.goto(&sample_url).await?;
        c.find(By::Id("button-prompt")).await?.click().await?;
        assert_eq!(c.get_alert_text().await?, "What is your name?");
        c.send_alert_text("Thirtyfour").await?;
        c.accept_alert().await?;
        assert_matches!(c.get_alert_text().await, Err(WebDriverError::NoSuchAlert(..)));
        assert_eq!(c.find(By::Id("alert-answer")).await?.text().await?, "Thirtyfour");

        Ok(())
    })
}
