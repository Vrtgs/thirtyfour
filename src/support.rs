#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use futures::Future;

#[cfg(any(feature = "tokio-runtime", feature = "async-std-runtime"))]
use crate::error::WebDriverResult;

#[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
where
    F: Future<Output = WebDriverResult<T>>,
{
    let mut rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(future)
}

#[cfg(feature = "async-std-runtime")]
pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
where
    F: Future<Output = WebDriverResult<T>>,
{
    async_std::task::block_on(future)
}

#[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
pub async fn sleep(duration: std::time::Duration) {
    tokio::time::delay_for(duration).await;
}

#[cfg(feature = "async-std-runtime")]
pub async fn sleep(duration: std::time::Duration) {
    async_std::task::sleep(duration).await;
}
