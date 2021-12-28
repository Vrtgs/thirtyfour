#[cfg(not(any(feature = "tokio-runtime", feature = "async-std-runtime")))]
/// Null imports.
pub mod imports {
    pub use crate::http::nulldriver_async::NullDriverAsync as HttpClientAsync;
    pub use futures::executor::block_on;
    pub use futures::io::AsyncWriteExt;
    pub use futures::lock::Mutex;
    pub use futures::Future;
    pub use std::fs::File;

    // No-op spawn method to avoid compilation errors.
    pub fn spawn<T>(_future: T)
    where
        T: Future + Send + 'static,
    {
    }
    pub async fn sleep(_duration: std::time::Duration) {}
}

#[cfg(all(feature = "tokio-runtime", not(feature = "async-std-runtime")))]
/// Tokio runtime imports.
pub mod imports {
    use crate::error::WebDriverResult;
    use futures::Future;

    pub use crate::http::reqwest_async::ReqwestDriverAsync as HttpClientAsync;
    pub use tokio::sync::Mutex;
    pub use tokio::task::spawn;
    pub use tokio::{fs::File, io::AsyncWriteExt};

    pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
    where
        F: Future<Output = WebDriverResult<T>>,
    {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(future)
    }

    pub async fn sleep(duration: std::time::Duration) {
        tokio::time::sleep(duration).await;
    }
}

#[cfg(feature = "async-std-runtime")]
/// Async-std runtime imports.
pub mod imports {
    use crate::error::WebDriverResult;
    use futures::Future;

    pub use crate::http::surf_async::SurfDriverAsync as HttpClientAsync;
    pub use async_std::fs::File;

    pub use async_std::sync::Mutex;
    pub use async_std::task::spawn;
    pub use futures::io::AsyncWriteExt;

    pub fn block_on<F, T>(future: F) -> WebDriverResult<T>
    where
        F: Future<Output = WebDriverResult<T>>,
    {
        async_std::task::block_on(future)
    }

    pub async fn sleep(duration: std::time::Duration) {
        async_std::task::sleep(duration).await;
    }
}
