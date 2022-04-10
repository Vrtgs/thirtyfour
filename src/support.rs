use crate::error::WebDriverResult;
use futures::Future;
use std::time::Duration;

pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
where
    F: Future<Output = WebDriverResult<T>>,
{
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(future)
}

pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await
}
