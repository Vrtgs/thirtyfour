use crate::error::WebDriverResult;
use base64::{prelude::BASE64_STANDARD, Engine};
use futures::Future;
use std::sync::OnceLock;
use std::time::Duration;

/// Helper to run the specified future and block the current thread waiting for the result.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    static GLOBAL_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    #[cold]
    fn init_global() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap()
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "tokio-multi-threaded")] {
            use tokio::runtime::RuntimeFlavor;

            match tokio::runtime::Handle::try_current() {
                Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
                    tokio::task::block_in_place(|| handle.block_on(future))
                }
                _ => std::thread::scope(|scope| {
                    scope.spawn(|| GLOBAL_RT.get_or_init(init_global).block_on(future)).join().unwrap()
                }),
            }
        } else {
            std::thread::scope(|scope| {
                scope.spawn(|| GLOBAL_RT.get_or_init(init_global).block_on(future)).join().unwrap()
            })
        }
    }
}

/// Helper to sleep asynchronously for the specified duration.
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await
}

/// Convenience wrapper for base64 encoding.
pub fn base64_encode(data: &[u8]) -> String {
    BASE64_STANDARD.encode(data)
}

/// Convenience wrapper for base64 decoding.
pub fn base64_decode(data: &str) -> WebDriverResult<Vec<u8>> {
    let value = BASE64_STANDARD.decode(data)?;
    Ok(value)
}
