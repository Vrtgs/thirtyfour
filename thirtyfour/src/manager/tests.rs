//! Tests that exercise the manager beyond the per-module unit tests.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::net::{IpAddr, Ipv4Addr};

use super::browser::BrowserKind;
use super::download::{DownloadConfig, Mirror, resolve_version};
use super::manager::DriverKey;
use super::status::{DriverLogLine, Emitter, LogSubscribers, Status};
use super::version::DriverVersion;
use super::{StdioMode, WebDriverManager};
use crate::Capabilities;
use crate::common::config::WebDriverConfig;
use crate::extensions::query::{ElementPollerNoWait, ElementPollerWithTimeout, IntoElementPoller};

#[test]
fn builder_defaults_match_match_local() {
    let b = WebDriverManager::builder();
    let mgr = b.build();
    assert_eq!(mgr.cfg.version, DriverVersion::MatchLocalBrowser);
    assert_eq!(mgr.cfg.host.to_string(), "127.0.0.1");
    assert!(!mgr.cfg.offline);
}

#[test]
fn builder_chain_latest() {
    let mgr = WebDriverManager::builder().latest().build();
    assert_eq!(mgr.cfg.version, DriverVersion::Latest);
}

#[test]
fn builder_chain_exact() {
    let mgr = WebDriverManager::builder().exact("126").build();
    assert_eq!(mgr.cfg.version, DriverVersion::Exact("126".to_string()));
}

#[test]
fn builder_chain_from_caps() {
    let mgr = WebDriverManager::builder().from_caps().build();
    assert_eq!(mgr.cfg.version, DriverVersion::FromCapabilities);
}

#[test]
fn builder_offline_toggle() {
    let mgr = WebDriverManager::builder().offline().build();
    assert!(mgr.cfg.offline);
    let mgr = WebDriverManager::builder().offline().online().build();
    assert!(!mgr.cfg.offline);
}

#[test]
fn builder_stdio() {
    let mgr = WebDriverManager::builder().stdio(StdioMode::Null).build();
    assert!(matches!(mgr.cfg.stdio, StdioMode::Null));
}

#[test]
fn builder_web_driver_config() {
    let config = WebDriverConfig::builder()
        .keep_alive(false)
        .request_timeout(Duration::from_secs(42))
        .build()
        .unwrap();
    let manager = WebDriverManager::builder().config(config).build();

    assert!(!manager.web_driver_config.keep_alive);
    assert_eq!(manager.web_driver_config.request_timeout, Duration::from_secs(42));
}

#[test]
fn builder_poller_replaces_config_poller() {
    let initial_poller: Arc<dyn IntoElementPoller + Send + Sync> =
        Arc::new(ElementPollerWithTimeout::default());
    let replacement_poller: Arc<dyn IntoElementPoller + Send + Sync> =
        Arc::new(ElementPollerNoWait);
    let config = WebDriverConfig::builder().poller(initial_poller).build().unwrap();

    let manager =
        WebDriverManager::builder().config(config).poller(Arc::clone(&replacement_poller)).build();

    assert!(Arc::ptr_eq(&manager.web_driver_config.poller, &replacement_poller));
}

#[test]
fn builder_driver_binary_records_per_browser_paths() {
    let mgr = WebDriverManager::builder()
        .driver_binary(BrowserKind::Chrome, "/usr/local/bin/chromedriver")
        .driver_binary(BrowserKind::Firefox, "/usr/local/bin/geckodriver")
        .build();
    assert_eq!(
        mgr.cfg.driver_paths.get(&BrowserKind::Chrome).map(|p| p.to_str().unwrap()),
        Some("/usr/local/bin/chromedriver")
    );
    assert_eq!(
        mgr.cfg.driver_paths.get(&BrowserKind::Firefox).map(|p| p.to_str().unwrap()),
        Some("/usr/local/bin/geckodriver")
    );
    assert!(!mgr.cfg.driver_paths.contains_key(&BrowserKind::Edge));
}

#[test]
fn builder_driver_binary_overwrites_same_browser() {
    let mgr = WebDriverManager::builder()
        .driver_binary(BrowserKind::Chrome, "/old/chromedriver")
        .driver_binary(BrowserKind::Chrome, "/new/chromedriver")
        .build();
    assert_eq!(
        mgr.cfg.driver_paths.get(&BrowserKind::Chrome).map(|p| p.to_str().unwrap()),
        Some("/new/chromedriver")
    );
}

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut h = DefaultHasher::new();
    t.hash(&mut h);
    h.finish()
}

#[test]
fn driver_key_equal_when_all_fields_match() {
    let a = DriverKey {
        browser: BrowserKind::Chrome,
        version: "126.0.6478.126".to_string(),
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
    };
    let b = a.clone();
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn driver_key_differs_when_browser_differs() {
    let chrome = DriverKey {
        browser: BrowserKind::Chrome,
        version: "1".to_string(),
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
    };
    let firefox = DriverKey {
        browser: BrowserKind::Firefox,
        version: "1".to_string(),
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
    };
    assert_ne!(chrome, firefox);
}

#[test]
fn driver_key_differs_when_version_differs() {
    let v1 = DriverKey {
        browser: BrowserKind::Chrome,
        version: "126".to_string(),
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
    };
    let v2 = DriverKey {
        browser: BrowserKind::Chrome,
        version: "127".to_string(),
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
    };
    assert_ne!(v1, v2);
}

#[test]
fn driver_key_differs_when_host_differs() {
    let loopback = DriverKey {
        browser: BrowserKind::Chrome,
        version: "1".to_string(),
        host: IpAddr::V4(Ipv4Addr::LOCALHOST),
    };
    let any = DriverKey {
        browser: BrowserKind::Chrome,
        version: "1".to_string(),
        host: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
    };
    assert_ne!(loopback, any);
}

#[test]
fn webdriver_managed_returns_builder_with_caps() {
    let mut caps = Capabilities::new();
    caps.set("browserName", "chrome").unwrap();
    let builder = crate::WebDriver::managed(caps);
    assert!(builder.preloaded_caps.is_some());
    assert_eq!(builder.version, DriverVersion::MatchLocalBrowser);
}

#[tokio::test]
async fn resolve_version_latest_chrome() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/chrome-for-testing/LATEST_RELEASE_STABLE"))
        .respond_with(ResponseTemplate::new(200).set_body_string("126.0.6478.126"))
        .mount(&server)
        .await;

    let cfg = DownloadConfig {
        cache_dir: std::env::temp_dir(),
        mirror: Mirror {
            chrome_metadata: format!("{}/", server.uri()).parse().unwrap(),
            ..Mirror::default()
        },
        download_timeout: Duration::from_secs(5),
        offline: false,
    };

    let client = reqwest::Client::new();
    let emitter = Emitter::new();
    let v = resolve_version(
        &client,
        &cfg,
        BrowserKind::Chrome,
        &DriverVersion::Latest,
        None,
        None,
        &emitter,
    )
    .await
    .unwrap();
    assert_eq!(v, "126.0.6478.126");
}

/// Resolve `Latest` for Firefox against a geckodriver metadata mirror on `server`.
async fn resolve_firefox_latest(server: &MockServer) -> Result<String, super::ManagerError> {
    let cfg = DownloadConfig {
        cache_dir: std::env::temp_dir(),
        mirror: Mirror {
            geckodriver_metadata: format!("{}/releases/", server.uri()).parse().unwrap(),
            ..Mirror::default()
        },
        download_timeout: Duration::from_secs(5),
        offline: false,
    };
    let client = reqwest::Client::new();
    resolve_version(
        &client,
        &cfg,
        BrowserKind::Firefox,
        &DriverVersion::Latest,
        None,
        None,
        &Emitter::new(),
    )
    .await
}

#[tokio::test]
async fn resolve_version_latest_firefox() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", "/releases/tag/v0.37.1"))
        .mount(&server)
        .await;
    Mock::given(method("HEAD"))
        .and(path("/releases/tag/v0.37.1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    assert_eq!(resolve_firefox_latest(&server).await.unwrap(), "0.37.1");
}

#[tokio::test]
async fn resolve_version_latest_firefox_rejects_unexpected_url() {
    let server = MockServer::start().await;
    // No redirect to a release tag.
    Mock::given(method("HEAD"))
        .and(path("/releases/latest"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let err = resolve_firefox_latest(&server).await.unwrap_err();
    assert!(matches!(err, super::ManagerError::Parse(_)), "{err:?}");
}

#[tokio::test]
async fn resolve_version_exact_chrome_major_picks_latest_in_index() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/chrome-for-testing/known-good-versions-with-downloads.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "versions": [
                { "version": "126.0.6478.10",  "downloads": {"chromedriver": []} },
                { "version": "126.0.6478.126", "downloads": {"chromedriver": []} },
                { "version": "127.0.6533.50",  "downloads": {"chromedriver": []} },
            ]
        })))
        .mount(&server)
        .await;

    let cfg = DownloadConfig {
        cache_dir: std::env::temp_dir(),
        mirror: Mirror {
            chrome_metadata: format!("{}/", server.uri()).parse().unwrap(),
            ..Mirror::default()
        },
        download_timeout: Duration::from_secs(5),
        offline: false,
    };

    let client = reqwest::Client::new();
    let emitter = Emitter::new();
    let v = resolve_version(
        &client,
        &cfg,
        BrowserKind::Chrome,
        &DriverVersion::Exact("126".into()),
        None,
        None,
        &emitter,
    )
    .await
    .unwrap();
    assert_eq!(v, "126.0.6478.126");
}

// ---- Resolution caching in the manager ----

const CHROME_INDEX: &str = "/chrome-for-testing/known-good-versions-with-downloads.json";

fn chrome_index() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "versions": [
            { "version": "126.0.6478.10",  "downloads": {"chromedriver": []} },
            { "version": "126.0.6478.126", "downloads": {"chromedriver": []} },
            { "version": "127.0.6533.50",  "downloads": {"chromedriver": []} },
        ]
    }))
}

fn manager_with_chrome_mirror(server: &MockServer) -> Arc<WebDriverManager> {
    WebDriverManager::builder()
        .chrome_metadata_mirror(format!("{}/", server.uri()).parse().unwrap())
        .build()
}

async fn request_count(server: &MockServer) -> usize {
    server.received_requests().await.unwrap().len()
}

#[tokio::test]
async fn manager_resolves_each_version_once() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(CHROME_INDEX))
        .respond_with(chrome_index())
        .mount(&server)
        .await;
    let manager = manager_with_chrome_mirror(&server);

    for _ in 0..3 {
        let v = manager
            .resolve_version_cached(BrowserKind::Chrome, Some("126.0.6478.10"), None)
            .await
            .unwrap();
        assert_eq!(v, "126.0.6478.10");
    }
    assert_eq!(request_count(&server).await, 1);

    // Another local version is resolved on its own.
    let v = manager.resolve_version_cached(BrowserKind::Chrome, Some("127"), None).await.unwrap();
    assert_eq!(v, "127.0.6533.50");
    assert_eq!(request_count(&server).await, 2);
}

#[tokio::test]
async fn manager_resolves_concurrent_launches_once() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(CHROME_INDEX))
        .respond_with(chrome_index().set_delay(Duration::from_millis(100)))
        .mount(&server)
        .await;
    let manager = manager_with_chrome_mirror(&server);

    let resolve = || manager.resolve_version_cached(BrowserKind::Chrome, Some("126"), None);
    let (a, b, c) = tokio::join!(resolve(), resolve(), resolve());
    for v in [a, b, c] {
        assert_eq!(v.unwrap(), "126.0.6478.126");
    }
    assert_eq!(request_count(&server).await, 1);
}

#[tokio::test]
async fn manager_does_not_cache_failed_resolution() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(CHROME_INDEX))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(CHROME_INDEX))
        .respond_with(chrome_index())
        .mount(&server)
        .await;
    let manager = manager_with_chrome_mirror(&server);

    assert!(manager.resolve_version_cached(BrowserKind::Chrome, Some("126"), None).await.is_err());
    let v = manager.resolve_version_cached(BrowserKind::Chrome, Some("126"), None).await.unwrap();
    assert_eq!(v, "126.0.6478.126");
    assert_eq!(request_count(&server).await, 2);
}

#[tokio::test]
async fn manager_resolution_does_not_block_other_keys() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(CHROME_INDEX))
        .respond_with(chrome_index().set_delay(Duration::from_millis(500)))
        .mount(&server)
        .await;
    for (major, version) in [("126", "126.0.2592.56"), ("127", "127.0.2651.74")] {
        Mock::given(method("GET"))
            .and(path(format!("/LATEST_RELEASE_{major}")))
            .respond_with(ResponseTemplate::new(200).set_body_string(version))
            .mount(&server)
            .await;
    }
    let base: url::Url = format!("{}/", server.uri()).parse().unwrap();
    let manager = WebDriverManager::builder()
        .chrome_metadata_mirror(base.clone())
        .edge_downloads_mirror(base)
        .build();
    manager.resolve_version_cached(BrowserKind::Edge, Some("126"), None).await.unwrap();

    let chrome = tokio::spawn({
        let manager = manager.clone();
        async move { manager.resolve_version_cached(BrowserKind::Chrome, Some("126"), None).await }
    });
    while !server.received_requests().await.unwrap().iter().any(|r| r.url.path() == CHROME_INDEX) {
        tokio::task::yield_now().await;
    }

    // Both a cache hit and a cache miss for Edge go through while Chrome waits.
    for (local, expected) in [("126", "126.0.2592.56"), ("127", "127.0.2651.74")] {
        let v = tokio::time::timeout(
            Duration::from_millis(100),
            manager.resolve_version_cached(BrowserKind::Edge, Some(local), None),
        )
        .await
        .expect("Edge resolution blocked by Chrome")
        .unwrap();
        assert_eq!(v, expected);
    }
    assert!(!chrome.is_finished());
    assert_eq!(chrome.await.unwrap().unwrap(), "126.0.6478.126");
}

#[tokio::test]
async fn manager_resolves_latest_on_every_launch() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/chrome-for-testing/LATEST_RELEASE_STABLE"))
        .respond_with(ResponseTemplate::new(200).set_body_string("126.0.6478.126"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/chrome-for-testing/LATEST_RELEASE_STABLE"))
        .respond_with(ResponseTemplate::new(200).set_body_string("127.0.6533.50"))
        .mount(&server)
        .await;
    let manager = WebDriverManager::builder()
        .latest()
        .chrome_metadata_mirror(format!("{}/", server.uri()).parse().unwrap())
        .build();

    for expected in ["126.0.6478.126", "127.0.6533.50"] {
        let v = manager.resolve_version_cached(BrowserKind::Chrome, None, None).await.unwrap();
        assert_eq!(v, expected);
    }
    assert_eq!(request_count(&server).await, 2);
}

#[tokio::test]
async fn from_capabilities_errors_when_caps_missing_version() {
    let cfg = DownloadConfig {
        cache_dir: std::env::temp_dir(),
        mirror: Mirror::default(),
        download_timeout: Duration::from_secs(5),
        offline: false,
    };
    let client = reqwest::Client::new();
    let emitter = Emitter::new();
    let err = resolve_version(
        &client,
        &cfg,
        BrowserKind::Chrome,
        &DriverVersion::FromCapabilities,
        None,
        None,
        &emitter,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, super::ManagerError::MissingCapabilityVersion));
}

// ---- Status / subscriber tests ----

#[tokio::test]
async fn resolve_version_emits_resolving_then_resolved() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/chrome-for-testing/LATEST_RELEASE_STABLE"))
        .respond_with(ResponseTemplate::new(200).set_body_string("126.0.6478.126"))
        .mount(&server)
        .await;

    let cfg = DownloadConfig {
        cache_dir: std::env::temp_dir(),
        mirror: Mirror {
            chrome_metadata: format!("{}/", server.uri()).parse().unwrap(),
            ..Mirror::default()
        },
        download_timeout: Duration::from_secs(5),
        offline: false,
    };

    let recorded: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let emitter = Emitter::new();
    let r = Arc::clone(&recorded);
    let _sub = emitter.add(move |s| match s {
        Status::DriverVersionResolving {
            ..
        } => r.lock().unwrap().push("resolving".to_string()),
        Status::DriverVersionResolved {
            version,
            ..
        } => r.lock().unwrap().push(format!("resolved:{version}")),
        _ => {}
    });

    let client = reqwest::Client::new();
    resolve_version(
        &client,
        &cfg,
        BrowserKind::Chrome,
        &DriverVersion::Latest,
        None,
        None,
        &emitter,
    )
    .await
    .unwrap();

    assert_eq!(*recorded.lock().unwrap(), vec!["resolving", "resolved:126.0.6478.126"]);
}

#[test]
fn builder_on_status_subscriber_receives_events() {
    let log = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured = Arc::clone(&log);
    let mgr = WebDriverManager::builder()
        .on_status(move |s| captured.lock().unwrap().push(s.to_string()))
        .build();

    mgr.emitter.emit(Status::BrowserKindResolved {
        browser: BrowserKind::Chrome,
    });
    assert_eq!(log.lock().unwrap().len(), 1);
}

#[test]
fn manager_subscribe_returns_raii_guard() {
    let mgr = WebDriverManager::builder().build();
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let captured = Arc::clone(&log);
    let sub = mgr.subscribe(move |s| captured.lock().unwrap().push(s.to_string()));

    mgr.emitter.emit(Status::BrowserKindResolved {
        browser: BrowserKind::Chrome,
    });
    assert_eq!(log.lock().unwrap().len(), 1);

    drop(sub);
    mgr.emitter.emit(Status::BrowserKindResolved {
        browser: BrowserKind::Firefox,
    });
    assert_eq!(
        log.lock().unwrap().len(),
        1,
        "events emitted after subscription drop must not be observed"
    );
}

#[test]
fn driver_logs_fan_out_to_per_process_and_manager_subscribers() {
    // Simulate the dispatch path that `spawn_pump` uses: a per-process
    // `LogSubscribers` and a manager-wide one. Lines pushed through both
    // should reach both subscribers, and each subscription drop is independent.
    let process_subs = LogSubscribers::new();
    let manager_subs = LogSubscribers::new();

    let process_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let manager_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let p = Arc::clone(&process_log);
    let process_sub = process_subs.add(move |line| p.lock().unwrap().push(line.line.clone()));
    let m = Arc::clone(&manager_log);
    let manager_sub = manager_subs.add(move |line| m.lock().unwrap().push(line.line.clone()));

    let line = DriverLogLine {
        driver_id: super::status::DriverId::from_raw(1),
        browser: BrowserKind::Chrome,
        version: "126".to_string(),
        port: 51234,
        stream: super::status::DriverStream::Stdout,
        line: "first".to_string(),
    };
    process_subs.dispatch(&line);
    manager_subs.dispatch(&line);
    assert_eq!(process_log.lock().unwrap().as_slice(), &["first"]);
    assert_eq!(manager_log.lock().unwrap().as_slice(), &["first"]);

    // Drop the per-process subscription — manager-wide one keeps receiving.
    drop(process_sub);
    let line2 = DriverLogLine {
        line: "second".to_string(),
        ..line.clone()
    };
    process_subs.dispatch(&line2);
    manager_subs.dispatch(&line2);
    assert_eq!(process_log.lock().unwrap().len(), 1, "dropped per-process sub must stop");
    assert_eq!(manager_log.lock().unwrap().as_slice(), &["first", "second"]);

    // Drop the manager-wide subscription — both stop.
    drop(manager_sub);
    let line3 = DriverLogLine {
        line: "third".to_string(),
        ..line.clone()
    };
    process_subs.dispatch(&line3);
    manager_subs.dispatch(&line3);
    assert_eq!(manager_log.lock().unwrap().len(), 2);
}
