//! Tests that don't make use of external websites.
use crate::common::sample_page_url;
use serial_test::serial;
use std::time::Duration;
use thirtyfour::prelude::*;

mod common;

async fn resolve_execute_async_value(c: WebDriver, port: u16) -> Result<(), WebDriverError> {
    let url = sample_page_url(port);
    c.goto(&url).await?;

    let count: u64 = c
        .execute_async("setTimeout(() => arguments[1](arguments[0] + 1))", vec![1_u32.into()])
        .await?
        .convert()
        .expect("should be integer variant");

    assert_eq!(2, count);

    let count: u64 = c
        .execute_async("setTimeout(() => arguments[0](2))", vec![])
        .await?
        .convert()
        .expect("should be integer variant");

    assert_eq!(2, count);

    Ok(())
}

async fn status_firefox(c: WebDriver, _: u16) -> Result<(), WebDriverError> {
    // Geckodriver only supports a single session, and since we're already in a
    // session, it should return `false` here.
    assert!(!c.status().await?.ready);
    Ok(())
}

async fn status_chrome(c: WebDriver, _: u16) -> Result<(), WebDriverError> {
    // Chromedriver supports multiple sessions, so it should always return
    // `true` here.
    assert!(c.status().await?.ready);
    Ok(())
}

async fn timeouts(c: WebDriver, _: u16) -> Result<(), WebDriverError> {
    let new_timeouts = TimeoutConfiguration::new(
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(30)),
    );
    c.update_timeouts(new_timeouts.clone()).await?;

    let got_timeouts = c.get_timeouts().await?;
    assert_eq!(got_timeouts, new_timeouts);

    // Ensure partial update also works.
    let update_timeouts = TimeoutConfiguration::new(None, None, Some(Duration::from_secs(0)));
    c.update_timeouts(update_timeouts.clone()).await?;

    let got_timeouts = c.get_timeouts().await?;
    assert_eq!(
        got_timeouts,
        TimeoutConfiguration::new(
            new_timeouts.script(),
            new_timeouts.page_load(),
            update_timeouts.implicit()
        )
    );

    Ok(())
}

mod firefox {
    use super::*;

    #[test]
    #[serial]
    fn resolve_execute_async_value_test() {
        local_tester!(resolve_execute_async_value, "firefox");
    }

    #[test]
    #[serial]
    fn status_test() {
        local_tester!(status_firefox, "firefox");
    }

    #[test]
    #[serial]
    fn timeouts_test() {
        local_tester!(timeouts, "firefox");
    }
}

mod chrome {
    use super::*;

    #[test]
    fn status_test() {
        local_tester!(status_chrome, "chrome");
    }

    #[test]
    fn timeouts_test() {
        local_tester!(timeouts, "chrome");
    }
}
