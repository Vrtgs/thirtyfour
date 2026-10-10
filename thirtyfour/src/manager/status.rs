//! Status events emitted by [`WebDriverManager`] and the per-driver log
//! subscription API.
//!
//! See [`Status`] for the event vocabulary and
//! [`WebDriverManagerBuilder::on_status`] / [`WebDriverManager::subscribe`] for
//! registering subscribers. Status events are also forwarded to the `tracing`
//! ecosystem under the `thirtyfour::manager` target — installing a
//! `tracing-subscriber` is enough to get human-readable logs without writing
//! any subscriber code.
//!
//! [`WebDriverManager`]: super::WebDriverManager
//! [`WebDriverManagerBuilder::on_status`]: super::WebDriverManagerBuilder::on_status
//! [`WebDriverManager::subscribe`]: super::WebDriverManager::subscribe

use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use arc_swap::ArcSwap;
use tracing::{debug, info, warn};

use super::browser::BrowserKind;
use super::version::DriverVersion;

/// Where a resolved driver version came from.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSource {
    /// Resolved by probing the locally-installed browser binary.
    LocalBrowser,
    /// Resolved as the latest stable from upstream metadata.
    Latest,
    /// User pinned the version explicitly.
    Exact,
    /// Read from the `browserVersion` capability.
    Capabilities,
}

impl Display for VersionSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            VersionSource::LocalBrowser => "local-browser",
            VersionSource::Latest => "latest",
            VersionSource::Exact => "exact",
            VersionSource::Capabilities => "capabilities",
        };
        f.write_str(s)
    }
}

impl VersionSource {
    pub(crate) fn from_spec(spec: &DriverVersion) -> Self {
        match spec {
            DriverVersion::MatchLocalBrowser => VersionSource::LocalBrowser,
            DriverVersion::Latest => VersionSource::Latest,
            DriverVersion::Exact(_) => VersionSource::Exact,
            DriverVersion::FromCapabilities => VersionSource::Capabilities,
        }
    }
}

/// Progress event emitted while a [`WebDriverManager`] launches a session.
///
/// Variants carry the data the operation already has at the emission point so
/// subscribers don't need to recompute anything. Marked `#[non_exhaustive]` so
/// new variants can be added without a major version bump.
///
/// [`WebDriverManager`]: super::WebDriverManager
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Status {
    /// `browserName` was parsed from the supplied capabilities.
    BrowserKindResolved {
        /// The browser identified from the capabilities.
        browser: BrowserKind,
    },
    /// A locally-installed copy of the browser was probed for its version.
    LocalBrowserDetected {
        /// The browser whose binary was probed.
        browser: BrowserKind,
        /// Detected browser version (`--version` output / PE VersionInfo).
        version: String,
        /// Filesystem path of the probed binary.
        binary: PathBuf,
    },
    /// Starting upstream version resolution.
    ///
    /// A manager resolves each version once; later launches needing the same
    /// version reuse the result and emit neither this nor
    /// [`Status::DriverVersionResolved`].
    DriverVersionResolving {
        /// Browser whose driver is being resolved.
        browser: BrowserKind,
        /// The user-requested resolution strategy.
        requested: DriverVersion,
    },
    /// Resolved to a concrete driver version.
    DriverVersionResolved {
        /// Browser the resolved driver targets.
        browser: BrowserKind,
        /// Concrete resolved driver version (e.g. `"126.0.6478.126"`).
        version: String,
        /// Where the resolved version came from.
        source: VersionSource,
    },
    /// A cached driver binary was found and the download was skipped.
    DriverCacheHit {
        /// Browser whose driver was cache-hit.
        browser: BrowserKind,
        /// Cached driver version.
        version: String,
        /// Filesystem path of the cached binary.
        path: PathBuf,
    },
    /// A driver-binary download has begun.
    DriverDownloadStarted {
        /// Browser whose driver is being downloaded.
        browser: BrowserKind,
        /// Driver version being fetched.
        version: String,
        /// Source URL for the download.
        url: String,
    },
    /// A transient download error triggered a retry.
    DriverDownloadRetry {
        /// 1-based attempt number that failed.
        attempt: u32,
        /// Stringified error from the failed attempt.
        error: String,
    },
    /// A driver-binary download finished.
    DriverDownloadComplete {
        /// Browser whose driver was downloaded.
        browser: BrowserKind,
        /// Driver version that was downloaded.
        version: String,
        /// Total bytes received.
        bytes: u64,
        /// Wall-clock duration of the download.
        duration: Duration,
    },
    /// A driver archive was extracted into the cache directory.
    DriverArchiveExtracted {
        /// Browser whose driver archive was extracted.
        browser: BrowserKind,
        /// Filesystem path of the extracted binary.
        path: PathBuf,
    },
    /// A driver process was spawned.
    DriverProcessSpawned {
        /// Browser the driver is for.
        browser: BrowserKind,
        /// Driver version that was spawned.
        version: String,
        /// PID of the spawned driver process.
        pid: u32,
        /// Port the driver is listening on.
        port: u16,
        /// Path of the binary that was spawned.
        binary: PathBuf,
    },
    /// The driver's `/status` endpoint reported ready.
    DriverReady {
        /// Browser the driver is for.
        browser: BrowserKind,
        /// Driver version.
        version: String,
        /// Base URL of the now-ready driver.
        url: String,
        /// Time spent polling `/status` until it returned `ready`.
        elapsed: Duration,
    },
    /// An already-running driver matching the request was reused.
    DriverReused {
        /// Browser the reused driver is for.
        browser: BrowserKind,
        /// Driver version.
        version: String,
        /// Base URL of the reused driver.
        url: String,
    },
    /// A WebDriver session creation request is about to be sent.
    SessionStarting {
        /// Browser the session is being created for.
        browser: BrowserKind,
        /// Base URL of the driver receiving the request.
        url: String,
    },
    /// A WebDriver session was created.
    SessionStarted {
        /// Browser the session is for.
        browser: BrowserKind,
        /// W3C session id returned by the driver.
        session_id: String,
        /// Base URL of the driver hosting the session.
        url: String,
    },
    /// A WebDriver session was closed (either explicitly or by drop).
    SessionEnded {
        /// Browser the session was for.
        browser: BrowserKind,
        /// W3C session id of the closed session.
        session_id: String,
    },
    /// A managed driver process was shut down (last reference dropped).
    DriverShutdown {
        /// Browser the driver was for.
        browser: BrowserKind,
        /// Driver version.
        version: String,
        /// Port the driver had been listening on.
        port: u16,
    },
}

impl Status {
    fn driver_label(browser: BrowserKind) -> &'static str {
        match browser {
            BrowserKind::Chrome => "chromedriver",
            BrowserKind::Firefox => "geckodriver",
            BrowserKind::Edge => "msedgedriver",
            BrowserKind::Safari => "safaridriver",
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Status::BrowserKindResolved {
                browser,
            } => {
                write!(f, "{}: resolved browser kind", Self::driver_label(*browser))
            }
            Status::LocalBrowserDetected {
                browser,
                version,
                binary,
            } => write!(
                f,
                "{}: detected local browser {version} at {}",
                Self::driver_label(*browser),
                binary.display()
            ),
            Status::DriverVersionResolving {
                browser,
                requested,
            } => write!(
                f,
                "{}: resolving driver version (requested: {})",
                Self::driver_label(*browser),
                describe_spec(requested),
            ),
            Status::DriverVersionResolved {
                browser,
                version,
                source,
            } => {
                write!(f, "{} {version}: resolved (source: {source})", Self::driver_label(*browser),)
            }
            Status::DriverCacheHit {
                browser,
                version,
                path,
            } => write!(
                f,
                "{} {version}: cache hit at {}",
                Self::driver_label(*browser),
                path.display()
            ),
            Status::DriverDownloadStarted {
                browser,
                version,
                url,
            } => write!(f, "{} {version}: download started ({url})", Self::driver_label(*browser)),
            Status::DriverDownloadRetry {
                attempt,
                error,
            } => {
                write!(f, "driver download retry (attempt {attempt}): {error}")
            }
            Status::DriverDownloadComplete {
                browser,
                version,
                bytes,
                duration,
            } => write!(
                f,
                "{} {version}: download complete ({} in {})",
                Self::driver_label(*browser),
                human_bytes(*bytes),
                human_duration(*duration),
            ),
            Status::DriverArchiveExtracted {
                browser,
                path,
            } => write!(
                f,
                "{}: archive extracted to {}",
                Self::driver_label(*browser),
                path.display()
            ),
            Status::DriverProcessSpawned {
                browser,
                version,
                pid,
                port,
                binary,
            } => write!(
                f,
                "{} {version}: process spawned pid={pid} port={port} binary={}",
                Self::driver_label(*browser),
                binary.display()
            ),
            Status::DriverReady {
                browser,
                version,
                url,
                elapsed,
            } => write!(
                f,
                "{} {version}: ready at {url} in {}",
                Self::driver_label(*browser),
                human_duration(*elapsed)
            ),
            Status::DriverReused {
                browser,
                version,
                url,
            } => {
                write!(f, "{} {version}: reused live driver at {url}", Self::driver_label(*browser))
            }
            Status::SessionStarting {
                browser,
                url,
            } => {
                write!(f, "{}: starting session at {url}", Self::driver_label(*browser))
            }
            Status::SessionStarted {
                browser,
                session_id,
                url,
            } => write!(
                f,
                "{}: session {} started at {url}",
                Self::driver_label(*browser),
                short_id(session_id)
            ),
            Status::SessionEnded {
                browser,
                session_id,
            } => write!(
                f,
                "{}: session {} ended",
                Self::driver_label(*browser),
                short_id(session_id)
            ),
            Status::DriverShutdown {
                browser,
                version,
                port,
            } => write!(f, "{} {version}: shutdown (port {port})", Self::driver_label(*browser)),
        }
    }
}

fn describe_spec(spec: &DriverVersion) -> String {
    match spec {
        DriverVersion::MatchLocalBrowser => "match-local-browser".to_string(),
        DriverVersion::FromCapabilities => "from-capabilities".to_string(),
        DriverVersion::Latest => "latest".to_string(),
        DriverVersion::Exact(v) => format!("exact {v}"),
    }
}

fn short_id(id: &str) -> &str {
    let n = id.char_indices().nth(8).map(|(i, _)| i).unwrap_or(id.len());
    &id[..n]
}

fn human_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn human_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs >= 1.0 {
        format!("{secs:.2}s")
    } else {
        format!("{}ms", d.as_millis())
    }
}

/// Type alias for a status-subscriber closure stored behind an `Arc`.
pub(crate) type StatusCallback = Arc<dyn Fn(&Status) + Send + Sync>;

/// Type alias for a driver-log subscriber closure stored behind an `Arc`.
pub(crate) type DriverLogCallback = Arc<dyn Fn(&DriverLogLine) + Send + Sync>;

/// Internal subscriber entry — pairs a closure with an id used for unsubscribe.
pub(crate) struct StatusSubscriber {
    id: u64,
    cb: StatusCallback,
}

/// Emits status events to all registered subscribers (and to `tracing`).
///
/// Cheap to clone (one `Arc`). Reads on the hot path are lock-free (an `ArcSwap`
/// pointer load), so emissions are safe from any async context.
#[derive(Clone)]
pub(crate) struct Emitter {
    subs: Arc<ArcSwap<Vec<StatusSubscriber>>>,
    next_id: Arc<AtomicU64>,
}

impl Emitter {
    pub(crate) fn new() -> Self {
        Self {
            subs: Arc::new(ArcSwap::from_pointee(Vec::new())),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(crate) fn add<F>(&self, f: F) -> Subscription
    where
        F: Fn(&Status) + Send + Sync + 'static,
    {
        let cb: StatusCallback = Arc::new(f);
        self.add_arc(cb)
    }

    pub(crate) fn add_arc(&self, cb: StatusCallback) -> Subscription {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.subs.rcu(|current| {
            let mut next = (**current).iter().map(clone_sub).collect::<Vec<_>>();
            next.push(StatusSubscriber {
                id,
                cb: Arc::clone(&cb),
            });
            next
        });
        Subscription {
            id,
            subs: Arc::downgrade(&self.subs),
        }
    }

    pub(crate) fn emit(&self, status: Status) {
        emit_tracing(&status);
        let list = self.subs.load();
        for s in list.iter() {
            let cb = Arc::clone(&s.cb);
            // Use AssertUnwindSafe — closures may capture &mut state by Mutex
            // etc. and panicking through them is the user's problem to recover
            // from. We only catch_unwind so a panicking subscriber on the
            // shared singleton manager can't poison sibling subscribers.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (cb)(&status)));
        }
    }
}

fn clone_sub(s: &StatusSubscriber) -> StatusSubscriber {
    StatusSubscriber {
        id: s.id,
        cb: Arc::clone(&s.cb),
    }
}

fn emit_tracing(status: &Status) {
    match status {
        Status::DriverDownloadRetry {
            ..
        } => warn!(target: "thirtyfour::manager", "{status}"),
        Status::BrowserKindResolved {
            ..
        }
        | Status::LocalBrowserDetected {
            ..
        }
        | Status::DriverVersionResolving {
            ..
        }
        | Status::DriverArchiveExtracted {
            ..
        } => {
            debug!(target: "thirtyfour::manager", "{status}")
        }
        Status::DriverVersionResolved {
            ..
        }
        | Status::DriverCacheHit {
            ..
        }
        | Status::DriverDownloadStarted {
            ..
        }
        | Status::DriverDownloadComplete {
            ..
        }
        | Status::DriverProcessSpawned {
            ..
        }
        | Status::DriverReady {
            ..
        }
        | Status::DriverReused {
            ..
        }
        | Status::SessionStarting {
            ..
        }
        | Status::SessionStarted {
            ..
        }
        | Status::SessionEnded {
            ..
        }
        | Status::DriverShutdown {
            ..
        } => info!(target: "thirtyfour::manager", "{status}"),
    }
}

/// RAII handle returned by [`WebDriverManager::subscribe`]. Dropping it
/// removes the subscriber.
///
/// `mem::forget` the handle to keep the subscriber alive for the lifetime of
/// the manager. Subscribers attached via
/// [`WebDriverManagerBuilder::on_status`] are managed by the builder and don't
/// produce a `Subscription`.
///
/// [`WebDriverManager::subscribe`]: super::WebDriverManager::subscribe
/// [`WebDriverManagerBuilder::on_status`]: super::WebDriverManagerBuilder::on_status
pub struct Subscription {
    id: u64,
    subs: std::sync::Weak<ArcSwap<Vec<StatusSubscriber>>>,
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Subscription").field("id", &self.id).finish()
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(subs) = self.subs.upgrade() {
            let id = self.id;
            subs.rcu(|current| {
                (**current).iter().filter(|s| s.id != id).map(clone_sub).collect::<Vec<_>>()
            });
        }
    }
}

// ----- Driver-process log subscription (separate from status events) -----

/// Identifier of a driver process spawned by a [`WebDriverManager`]. Unique
/// within the manager (and effectively within the process — the underlying
/// counter is monotonic).
///
/// [`WebDriverManager`]: super::WebDriverManager
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct DriverId(pub(super) u64);

impl DriverId {
    pub(crate) fn from_raw(n: u64) -> Self {
        DriverId(n)
    }
}

impl Display for DriverId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "driver#{}", self.0)
    }
}

/// Which standard stream a driver-log line came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverStream {
    /// The driver process's standard output.
    Stdout,
    /// The driver process's standard error.
    Stderr,
}

impl Display for DriverStream {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            DriverStream::Stdout => "stdout",
            DriverStream::Stderr => "stderr",
        };
        f.write_str(s)
    }
}

/// One line of stdout/stderr from a managed driver process.
#[derive(Debug, Clone)]
pub struct DriverLogLine {
    /// Identifier of the driver process the line came from.
    pub driver_id: DriverId,
    /// Browser the driver process serves.
    pub browser: BrowserKind,
    /// Driver version.
    pub version: String,
    /// Port the driver is listening on.
    pub port: u16,
    /// Which stream the line was read from.
    pub stream: DriverStream,
    /// The raw log line, with the trailing newline stripped.
    pub line: String,
}

/// Internal log-subscriber entry.
pub(crate) struct LogSubscriber {
    pub(crate) id: u64,
    pub(crate) cb: DriverLogCallback,
}

pub(crate) fn clone_log_sub(s: &LogSubscriber) -> LogSubscriber {
    LogSubscriber {
        id: s.id,
        cb: Arc::clone(&s.cb),
    }
}

/// Shared log-subscriber list. Cheap to clone (one `Arc`); reads are lock-free.
#[derive(Clone)]
pub(crate) struct LogSubscribers {
    pub(crate) subs: Arc<ArcSwap<Vec<LogSubscriber>>>,
    next_id: Arc<AtomicU64>,
}

impl LogSubscribers {
    pub(crate) fn new() -> Self {
        Self {
            subs: Arc::new(ArcSwap::from_pointee(Vec::new())),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub(crate) fn add<F>(&self, f: F) -> DriverLogSubscription
    where
        F: Fn(&DriverLogLine) + Send + Sync + 'static,
    {
        let cb: DriverLogCallback = Arc::new(f);
        self.add_arc(cb)
    }

    pub(crate) fn add_arc(&self, cb: DriverLogCallback) -> DriverLogSubscription {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.subs.rcu(|current| {
            let mut next = (**current).iter().map(clone_log_sub).collect::<Vec<_>>();
            next.push(LogSubscriber {
                id,
                cb: Arc::clone(&cb),
            });
            next
        });
        DriverLogSubscription {
            id,
            subs: Arc::downgrade(&self.subs),
        }
    }

    pub(crate) fn dispatch(&self, line: &DriverLogLine) {
        let list = self.subs.load();
        for s in list.iter() {
            let cb = Arc::clone(&s.cb);
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (cb)(line)));
        }
    }
}

/// RAII handle returned by `on_driver_log`. Dropping it removes the subscriber;
/// `mem::forget` keeps it alive permanently.
pub struct DriverLogSubscription {
    id: u64,
    subs: std::sync::Weak<ArcSwap<Vec<LogSubscriber>>>,
}

impl std::fmt::Debug for DriverLogSubscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DriverLogSubscription").field("id", &self.id).finish()
    }
}

impl Drop for DriverLogSubscription {
    fn drop(&mut self) {
        if let Some(subs) = self.subs.upgrade() {
            let id = self.id;
            subs.rcu(|current| {
                (**current).iter().filter(|s| s.id != id).map(clone_log_sub).collect::<Vec<_>>()
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;

    #[test]
    fn display_browser_kind_resolved() {
        let s = Status::BrowserKindResolved {
            browser: BrowserKind::Chrome,
        };
        assert_eq!(s.to_string(), "chromedriver: resolved browser kind");
    }

    #[test]
    fn display_local_browser_detected() {
        let s = Status::LocalBrowserDetected {
            browser: BrowserKind::Firefox,
            version: "128.0.1".to_string(),
            binary: PathBuf::from("/usr/bin/firefox"),
        };
        assert_eq!(
            s.to_string(),
            "geckodriver: detected local browser 128.0.1 at /usr/bin/firefox"
        );
    }

    #[test]
    fn display_driver_version_resolved() {
        let s = Status::DriverVersionResolved {
            browser: BrowserKind::Edge,
            version: "126.0.0".to_string(),
            source: VersionSource::Latest,
        };
        assert_eq!(s.to_string(), "msedgedriver 126.0.0: resolved (source: latest)");
    }

    #[test]
    fn display_driver_cache_hit() {
        let s = Status::DriverCacheHit {
            browser: BrowserKind::Chrome,
            version: "126.0.6478.126".to_string(),
            path: PathBuf::from("/cache/chromedriver"),
        };
        assert_eq!(s.to_string(), "chromedriver 126.0.6478.126: cache hit at /cache/chromedriver");
    }

    #[test]
    fn display_driver_download_complete() {
        let s = Status::DriverDownloadComplete {
            browser: BrowserKind::Chrome,
            version: "126.0.6478.126".to_string(),
            bytes: 5 * 1024 * 1024,
            duration: Duration::from_millis(1500),
        };
        assert_eq!(
            s.to_string(),
            "chromedriver 126.0.6478.126: download complete (5.0 MiB in 1.50s)"
        );
    }

    #[test]
    fn display_driver_ready() {
        let s = Status::DriverReady {
            browser: BrowserKind::Chrome,
            version: "126.0.6478.126".to_string(),
            url: "http://127.0.0.1:51234".to_string(),
            elapsed: Duration::from_millis(412),
        };
        assert_eq!(
            s.to_string(),
            "chromedriver 126.0.6478.126: ready at http://127.0.0.1:51234 in 412ms"
        );
    }

    #[test]
    fn display_session_started_truncates_id() {
        let s = Status::SessionStarted {
            browser: BrowserKind::Chrome,
            session_id: "abc123def456".to_string(),
            url: "http://127.0.0.1:51234".to_string(),
        };
        assert_eq!(
            s.to_string(),
            "chromedriver: session abc123de started at http://127.0.0.1:51234"
        );
    }

    #[test]
    fn display_session_ended() {
        let s = Status::SessionEnded {
            browser: BrowserKind::Chrome,
            session_id: "abc123def456".to_string(),
        };
        assert_eq!(s.to_string(), "chromedriver: session abc123de ended");
    }

    #[test]
    fn display_driver_shutdown() {
        let s = Status::DriverShutdown {
            browser: BrowserKind::Firefox,
            version: "0.36.0".to_string(),
            port: 51234,
        };
        assert_eq!(s.to_string(), "geckodriver 0.36.0: shutdown (port 51234)");
    }

    #[test]
    fn display_driver_id_and_stream() {
        assert_eq!(DriverId(7).to_string(), "driver#7");
        assert_eq!(DriverStream::Stdout.to_string(), "stdout");
        assert_eq!(DriverStream::Stderr.to_string(), "stderr");
    }

    #[test]
    fn emitter_dispatches_to_all_subscribers() {
        let e = Emitter::new();
        let log_a: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let log_b: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let a = Arc::clone(&log_a);
        let _sub_a = e.add(move |s| a.lock().unwrap().push(s.to_string()));
        let b = Arc::clone(&log_b);
        let _sub_b = e.add(move |s| b.lock().unwrap().push(s.to_string()));

        e.emit(Status::BrowserKindResolved {
            browser: BrowserKind::Chrome,
        });
        assert_eq!(log_a.lock().unwrap().len(), 1);
        assert_eq!(log_b.lock().unwrap().len(), 1);
    }

    #[test]
    fn dropping_subscription_unsubscribes() {
        let e = Emitter::new();
        let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let captured = Arc::clone(&log);
        let sub = e.add(move |s| captured.lock().unwrap().push(s.to_string()));

        e.emit(Status::BrowserKindResolved {
            browser: BrowserKind::Chrome,
        });
        assert_eq!(log.lock().unwrap().len(), 1);

        drop(sub);
        e.emit(Status::BrowserKindResolved {
            browser: BrowserKind::Firefox,
        });
        assert_eq!(log.lock().unwrap().len(), 1, "second emit must not reach a dropped subscriber");
    }

    #[test]
    fn panicking_subscriber_does_not_block_others() {
        let e = Emitter::new();
        let log: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

        let _bad = e.add(|_| panic!("nope"));
        let counter = Arc::clone(&log);
        let _good = e.add(move |_| *counter.lock().unwrap() += 1);

        e.emit(Status::BrowserKindResolved {
            browser: BrowserKind::Chrome,
        });
        e.emit(Status::BrowserKindResolved {
            browser: BrowserKind::Firefox,
        });
        assert_eq!(*log.lock().unwrap(), 2);
    }

    #[test]
    fn log_subscribers_dispatch_and_drop() {
        let subs = LogSubscribers::new();
        let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

        let captured = Arc::clone(&log);
        let sub = subs.add(move |line| captured.lock().unwrap().push(line.line.clone()));

        let l1 = DriverLogLine {
            driver_id: DriverId(1),
            browser: BrowserKind::Chrome,
            version: "126".to_string(),
            port: 51234,
            stream: DriverStream::Stdout,
            line: "hello".to_string(),
        };
        subs.dispatch(&l1);
        assert_eq!(log.lock().unwrap().as_slice(), &["hello"]);

        drop(sub);
        let l2 = DriverLogLine {
            line: "world".to_string(),
            ..l1.clone()
        };
        subs.dispatch(&l2);
        assert_eq!(log.lock().unwrap().len(), 1, "dropped log subscription must stop receiving");
    }
}
