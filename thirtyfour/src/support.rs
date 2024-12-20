use crate::error::WebDriverResult;
use base64::{prelude::BASE64_STANDARD, Engine};
use std::convert::Infallible;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;
use std::{io, thread};

// used in drop code so its really bad to have a stack overflow then
const BOX_FUTURE_THRESHOLD: usize = 512;

// a global runtime that is being driven al the time
static GLOBAL_RT: LazyLock<tokio::runtime::Handle> = LazyLock::new(|| {
    fn no_unwind<T>(f: impl FnOnce() -> T) -> T {
        let res = std::panic::catch_unwind(AssertUnwindSafe(f));

        res.unwrap_or_else(|_| {
            struct Abort;
            impl Drop for Abort {
                fn drop(&mut self) {
                    eprintln!("unrecoverable error reached aborting...");
                    std::process::abort()
                }
            }

            let _abort_on_unwind = Abort;
            unreachable!("thirtyfour global runtime panicked")
        })
    }

    no_unwind(|| {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let handle = rt.handle().clone();

        // drive the runtime
        // we do this so that all calls to GLOBAL_RT.block_on() work
        thread::spawn(move || -> ! {
            async fn forever() -> ! {
                match std::future::pending::<Infallible>().await {}
            }

            no_unwind(move || rt.block_on(forever()))
        });
        handle
    })
});

/// Helper to run the specified future and block the current thread waiting for the result.
/// works even while in a tokio runtime
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    // https://github.com/tokio-rs/tokio/pull/6826
    // cfg!(debug_assertions) omitted
    if size_of::<F>() > BOX_FUTURE_THRESHOLD {
        block_on_inner(Box::pin(future))
    } else {
        block_on_inner(future)
    }
}

fn block_on_inner<F>(future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    macro_rules! block_global {
        ($future:expr) => {
            thread::scope(|scope| match scope.spawn(|| GLOBAL_RT.block_on($future)).join() {
                Ok(res) => res,
                Err(panic) => std::panic::resume_unwind(panic),
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

/// Helper to run the specified future and bind it to run before runtime shutdown
/// this is not guaranteed to not block on the future, just that it wont block on a
/// current threaded runtime true is passed in if it is placed in a newly created runtime
pub fn spawn_blocked_future<Fn, F>(future: Fn)
where
    Fn: FnOnce(bool) -> F,
    F: Future + Send + 'static,
{
    if cfg!(debug_assertions) && size_of::<F>() > BOX_FUTURE_THRESHOLD {
        spawn_blocked_future_inner(Box::new(future))
    } else {
        spawn_blocked_future_inner(future)
    }
}

fn spawn_blocked_future_inner<Fn, F>(future: Fn)
where
    Fn: FnOnce(bool) -> F,
    F: Future + Send + 'static,
{
    macro_rules! spawn_off {
        ($future: expr) => {{
            let future = $future;
            let func = move || {
                GLOBAL_RT.block_on(future);
            };
            match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    let (tx, rx) = std::sync::mpsc::sync_channel(0);
                    handle.spawn_blocking(move || {
                        if tx.send(()).is_ok() {
                            func();
                        }
                    });
                    rx.recv().expect("could not spawn task");
                }
                Err(_) => func(),
            }
        }};
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "tokio-multi-threaded")] {
            use tokio::runtime::RuntimeFlavor;

            match tokio::runtime::Handle::try_current() {
                Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
                    tokio::task::block_in_place(|| {
                        handle.block_on(future(false))
                    });
                }
                _ => spawn_off!(future(true)),
            }
        } else {
            spawn_off!(future(true))
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
