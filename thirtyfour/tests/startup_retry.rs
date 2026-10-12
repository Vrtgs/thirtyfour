mod common;

use std::{cell::Cell, future::ready, time::Duration};

use thirtyfour::{error::WebDriverError, manager::ManagerError};

fn readiness_timeout() -> WebDriverError {
    ManagerError::DriverNotReady(Duration::from_secs(30)).into()
}

#[tokio::test]
async fn retries_readiness_timeout_then_recovers() {
    let attempts = Cell::new(0);
    let result = common::retry_chrome_startup(
        || {
            let attempt = attempts.get();
            attempts.set(attempt + 1);
            ready(if attempt == 0 {
                Err(readiness_timeout())
            } else {
                Ok(())
            })
        },
        true,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(attempts.get(), 2);
}

#[tokio::test]
async fn repeated_readiness_timeout_fails_after_one_retry() {
    let attempts = Cell::new(0);
    let result = common::retry_chrome_startup(
        || {
            attempts.set(attempts.get() + 1);
            ready(Err::<(), _>(readiness_timeout()))
        },
        true,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(attempts.get(), 2);
}

#[tokio::test]
async fn other_launch_errors_are_not_retried() {
    for error in [
        WebDriverError::SessionCreateError("browser and driver versions do not match".into()),
        ManagerError::Spawn("binary not found".into()).into(),
        ManagerError::Download("HTTP 404".into()).into(),
    ] {
        let attempts = Cell::new(0);
        let mut error = Some(error);
        let result = common::retry_chrome_startup(
            || {
                attempts.set(attempts.get() + 1);
                ready(Err::<(), _>(error.take().expect("non-readiness error was retried")))
            },
            true,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.get(), 1);
    }
}

#[tokio::test]
async fn disabled_retry_returns_first_readiness_timeout() {
    let attempts = Cell::new(0);
    let result = common::retry_chrome_startup(
        || {
            attempts.set(attempts.get() + 1);
            ready(Err::<(), _>(readiness_timeout()))
        },
        false,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(attempts.get(), 1);
}
