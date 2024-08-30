use crate::error::WebDriverResult;
use base64::{prelude::BASE64_STANDARD, Engine};
use futures::Future;
use std::convert::Infallible;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;
use std::{io, thread};

/// Helper to run the specified future and block the current thread waiting for the result.
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    static GLOBAL_RT: OnceLock<tokio::runtime::Handle> = OnceLock::new();

    #[cold]
    fn init_global() -> tokio::runtime::Handle {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let handle = rt.handle().clone();
        // drive the runtime
        thread::spawn(move || rt.block_on(std::future::pending::<Infallible>()));
        handle
    }

    macro_rules! block_global {
        ($future:expr) => {
            thread::scope(|scope| {
                scope.spawn(|| GLOBAL_RT.get_or_init(init_global).block_on($future)).join().unwrap()
            })
        };
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "tokio-multi-threaded")] {
            use tokio::runtime::RuntimeFlavor;

            match tokio::runtime::Handle::try_current() {
                Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
                    tokio::task::block_in_place(|| handle.block_on(future))
                }
                _ => block_global!(future),
            }
        } else {
            block_global!(future)
        }
    }
}

pub(crate) async fn write_file(
    path: impl AsRef<Path>,
    bytes: impl Into<Vec<u8>>,
) -> io::Result<()> {
    async fn inner(path: &Path, bytes: Vec<u8>) -> io::Result<()> {
        let path = path.to_owned();
        tokio::task::spawn_blocking(move || std::fs::write(path, bytes)).await?
    }

    inner(path.as_ref(), bytes.into()).await
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
