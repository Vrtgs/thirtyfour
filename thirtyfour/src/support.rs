use crate::error::WebDriverResult;
use futures::Future;
use std::time::Duration;

/// Helper to run the specified future and block the current thread waiting for the result.
///
/// This is mostly used within tests.
///
/// NOTE: This cannot be used within an active tokio runtime.
pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
where
    F: Future<Output = WebDriverResult<T>>,
{
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(future)
}

/// Helper to sleep asynchronously for the specified duration.
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await
}
