use futures::Future;

use crate::error::WebDriverResult;

#[cfg(all(feature = "async-std-runtime", not(feature = "tokio-runtime")))]
pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
where
    F: Future<Output = WebDriverResult<T>>,
{
    async_std::task::block_on(future)
}

#[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
where
    F: Future<Output = WebDriverResult<T>>,
{
    let mut rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(future)
}
