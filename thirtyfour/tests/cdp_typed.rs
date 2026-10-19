//! End-to-end tests for the typed CDP API.
//!
//! These run against a real chromedriver downloaded by the manager, so they
//! gate behind the `manager-tests` feature and live in their own file (the
//! `cdp-test.yml` CI job runs `cargo test ... --test cdp_typed` without
//! pre-starting a driver).
//!
//! ```text
//! cargo test -p thirtyfour --features manager-tests --test cdp_typed -- --test-threads=1
//! ```
//!
//! Every typed command in [`thirtyfour::cdp::domains`] should have at least
//! one test here. The point is to verify the wire format against real CDP
//! — unit tests of the JSON shape are guess-against-guess and don't tell
//! you anything chromedriver doesn't.

#![cfg(all(feature = "manager-tests", feature = "cdp"))]

use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use thirtyfour::ChromeCapabilities;
use thirtyfour::cdp::domains::{
    browser, dom, emulation, fetch, input, log, network, page, performance, runtime, storage,
    target,
};
use thirtyfour::prelude::*;

use crate::common::launch_managed_chrome;

mod common;

const TEST_TIMEOUT: Duration = Duration::from_secs(180);

async fn with_timeout<F, T>(f: F) -> WebDriverResult<T>
where
    F: Future<Output = WebDriverResult<T>>,
{
    tokio::time::timeout(TEST_TIMEOUT, f).await.unwrap_or_else(|_| {
        Err(WebDriverError::FatalError(format!("test exceeded {}s budget", TEST_TIMEOUT.as_secs())))
    })
}

fn chrome_caps() -> ChromeCapabilities {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    caps.set_no_sandbox().unwrap();
    caps.set_disable_gpu().unwrap();
    caps.set_disable_dev_shm_usage().unwrap();
    caps.add_arg("--no-sandbox").unwrap();
    caps
}

const BLANK_HTML: &str = "data:text/html,<html><body>x</body></html>";

// ---------------------------------------------------------------------------
// Browser
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn browser_get_version() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let info = driver.cdp().browser().get_version().await?;
        assert!(info.user_agent.to_ascii_lowercase().contains("chrome"));
        assert!(info.product.starts_with("Chrome/") || info.product.starts_with("HeadlessChrome/"));
        assert!(!info.protocol_version.is_empty());
        assert!(!info.js_version.is_empty());
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn browser_set_download_behavior() -> WebDriverResult<()> {
    with_timeout(async {
        let dir = tempfile::tempdir().expect("tempdir");
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .cdp()
            .browser()
            .set_download_behavior(
                browser::DownloadBehavior::Allow,
                Some(dir.path().to_string_lossy().into_owned()),
            )
            .await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn page_enable_disable() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().page().enable().await?;
        driver.cdp().page().disable().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn page_navigate() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let r = driver.cdp().page().navigate(BLANK_HTML).await?;
        assert!(!r.frame_id.as_str().is_empty());
        assert!(r.error_text.is_none());
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn page_reload() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto(BLANK_HTML).await?;
        driver.cdp().page().reload().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn page_capture_screenshot() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto(BLANK_HTML).await?;
        let b64 = driver.cdp().page().capture_screenshot_base64().await?;
        assert!(!b64.is_empty());
        // PNG-base64 starts with `iVBOR` (decoded magic `89 50 4E 47`).
        assert!(b64.starts_with("iVBOR"), "expected PNG base64, got prefix {:?}", &b64[..6]);
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn page_print_to_pdf() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto(BLANK_HTML).await?;
        let b64 = driver.cdp().page().print_to_pdf_base64().await?;
        assert!(!b64.is_empty());
        // PDF base64 starts with `JVBER` (decoded magic `25 50 44 46` = "%PDF").
        assert!(b64.starts_with("JVBER"), "expected PDF base64, got prefix {:?}", &b64[..6]);
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn page_add_remove_script_to_evaluate_on_new_document() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let script_id = driver
            .cdp()
            .page()
            .add_script_to_evaluate_on_new_document("window.__tf_marker = 'present';")
            .await?;
        // Script runs on every new document — navigate and check.
        driver.goto(BLANK_HTML).await?;
        let v = driver.cdp().runtime().evaluate_value("window.__tf_marker").await?;
        assert_eq!(v, serde_json::json!("present"));

        // Remove it; navigate again; marker should be gone.
        driver.cdp().page().remove_script_to_evaluate_on_new_document(script_id).await?;
        driver.goto(BLANK_HTML).await?;
        let v = driver.cdp().runtime().evaluate_value("window.__tf_marker ?? null").await?;
        assert_eq!(v, serde_json::json!(null));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn page_set_lifecycle_events_enabled() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().page().enable().await?;
        driver.cdp().page().set_lifecycle_events_enabled(true).await?;
        driver.cdp().page().set_lifecycle_events_enabled(false).await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn network_enable_disable() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().network().enable().await?;
        driver.cdp().network().disable().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn network_clear_browser_cache() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().network().clear_browser_cache().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn network_clear_browser_cookies() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().network().clear_browser_cookies().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn network_set_extra_http_headers() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let mut headers = HashMap::new();
        headers.insert("X-Thirtyfour-Test".to_string(), "yes".to_string());
        driver.cdp().network().set_extra_http_headers(headers).await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn network_user_agent_override_changes_navigator_user_agent() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().network().set_user_agent_override("thirtyfour-cdp-test/1.0").await?;
        driver.goto(BLANK_HTML).await?;
        let ua = driver.cdp().runtime().evaluate_value("navigator.userAgent").await?;
        assert_eq!(ua, serde_json::json!("thirtyfour-cdp-test/1.0"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn network_emulate_network_conditions() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .cdp()
            .network()
            .emulate_network_conditions(network::NetworkConditions {
                offline: false,
                latency: 50,
                download_throughput: 256 * 1024,
                upload_throughput: 64 * 1024,
                connection_type: Some(network::ConnectionType::Cellular3G),
            })
            .await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Fetch
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn fetch_enable_disable() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().fetch().enable().await?;
        driver.cdp().fetch().disable().await?;
        driver.quit().await
    })
    .await
}

// `Fetch.continueRequest` / `fulfillRequest` / `failRequest` exercise the
// outgoing wire format only; the incoming `RequestPaused` event and the
// full intercept loop are covered in `cdp_events.rs`.

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn runtime_enable_disable() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().runtime().enable().await?;
        driver.cdp().runtime().disable().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_evaluate_returns_typed_value() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let v = driver.cdp().runtime().evaluate_value("1 + 41").await?;
        assert_eq!(v, serde_json::json!(42));
        let s = driver.cdp().runtime().evaluate_value("'foo' + 'bar'").await?;
        assert_eq!(s, serde_json::json!("foobar"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_evaluate_remote_object_for_complex_value() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let result = driver.cdp().send(runtime::Evaluate::new("({a: 1, b: [2, 3]})")).await?;
        assert_eq!(result.result.r#type, "object");
        let object_id = result.result.object_id.expect("object handle");
        driver.cdp().runtime().release_object(object_id).await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_call_function_on_object() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        // Get a handle to a JS object.
        let result = driver.cdp().send(runtime::Evaluate::new("({greeting: 'hi'})")).await?;
        let object_id = result.result.object_id.expect("object handle");
        // Call a function on it.
        let v = driver
            .cdp()
            .runtime()
            .call_function_on(object_id.clone(), "function() { return this.greeting + '!'; }")
            .await?;
        assert_eq!(v, serde_json::json!("hi!"));
        driver.cdp().runtime().release_object(object_id).await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn runtime_release_object_group() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let mut params = runtime::Evaluate::new("({})");
        params.object_group = Some("tf-test-group".to_string());
        let _r = driver.cdp().send(params).await?;
        driver
            .cdp()
            .send(runtime::ReleaseObjectGroup {
                object_group: "tf-test-group".to_string(),
            })
            .await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// DOM
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn dom_enable_disable() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().dom().enable().await?;
        driver.cdp().dom().disable().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_get_document() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto(BLANK_HTML).await?;
        let root = driver.cdp().dom().get_document().await?;
        assert_eq!(root.node_name, "#document");
        // Document URL is the SCREAMING-`URL` field — proves the rename
        // override is correctly populated.
        assert!(root.document_url.as_deref().unwrap_or("").starts_with("data:text/html"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_query_selector() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto("data:text/html,<html><body><div id='x'>x</div></body></html>").await?;
        let root = driver.cdp().dom().get_document().await?;
        let node_id = driver.cdp().dom().query_selector(root.node_id, "#x").await?;
        assert!(node_id.get() != 0, "should have found a matching node");
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_query_selector_all() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto("data:text/html,<html><body><p>a</p><p>b</p><p>c</p></body></html>").await?;
        let root = driver.cdp().dom().get_document().await?;
        let nodes = driver.cdp().dom().query_selector_all(root.node_id, "p").await?;
        assert_eq!(nodes.len(), 3);
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_request_node_for_remote_object() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto("data:text/html,<html><body><span id='s'>s</span></body></html>").await?;
        // `DOM.requestNode` only returns a non-zero id for nodes already
        // tracked in the current DOM agent's snapshot — and that snapshot
        // is created by `DOM.getDocument`. Without this primer the call
        // returns `0`.
        let _root = driver.cdp().dom().get_document().await?;
        let r = driver.cdp().send(runtime::Evaluate::new("document.getElementById('s')")).await?;
        let object_id = r.result.object_id.expect("object id");
        let node_id = driver.cdp().dom().request_node(object_id).await?;
        assert!(node_id.get() != 0);
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_resolve_node_returns_remote_object() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.goto(BLANK_HTML).await?;
        let root = driver.cdp().dom().get_document().await?;
        let resolved = driver
            .cdp()
            .send(dom::ResolveNode {
                node_id: Some(root.node_id),
                ..Default::default()
            })
            .await?;
        assert_eq!(resolved.object.r#type, "object");
        if let Some(id) = resolved.object.object_id {
            driver.cdp().runtime().release_object(id).await?;
        }
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_describe_node_via_backend_node_id() -> WebDriverResult<()> {
    // Exercises the `WebElement` -> `BackendNodeId` -> `DescribeNode` chain.
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .goto("data:text/html,<html><body><button id='b' class='c'>hi</button></body></html>")
            .await?;
        let elem = driver.find(By::Id("b")).await?;
        let backend_id = elem.cdp_backend_node_id().await?;
        let described = driver
            .cdp()
            .send(dom::DescribeNode {
                backend_node_id: Some(backend_id),
                ..Default::default()
            })
            .await?;
        assert_eq!(described.node.node_name, "BUTTON");
        let attrs = described.node.attributes.unwrap_or_default();
        let pairs: Vec<_> = attrs.as_chunks::<2>().0.iter().collect();
        assert!(pairs.iter().any(|p| p[0] == "id" && p[1] == "b"), "attrs: {attrs:?}");
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_get_box_model() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .goto(
                "data:text/html,<html><body><div id='x' style='width:100px;height:50px'>x</div></body></html>",
            )
            .await?;
        let root = driver.cdp().dom().get_document().await?;
        let node_id = driver.cdp().dom().query_selector(root.node_id, "#x").await?;
        let r = driver
            .cdp()
            .send(dom::GetBoxModel {
                node_id: Some(node_id),
                ..Default::default()
            })
            .await?;
        // Just check the shape — `model.width` should be present.
        assert!(r.model.get("width").is_some());
        assert!(r.model.get("content").is_some());
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn dom_scroll_into_view_if_needed() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        // Tall page so the target element starts off-screen.
        driver
            .goto(
                "data:text/html,<html><body><div style='height:5000px'></div><span id='target'>here</span></body></html>",
            )
            .await?;
        let elem = driver.find(By::Id("target")).await?;
        let backend_id = elem.cdp_backend_node_id().await?;
        driver
            .cdp()
            .send(dom::ScrollIntoViewIfNeeded {
                backend_node_id: Some(backend_id),
                ..Default::default()
            })
            .await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Emulation
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn emulation_device_metrics_changes_inner_dimensions() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().emulation().set_device_metrics_override(444, 333, 1.0, false).await?;
        driver.goto(BLANK_HTML).await?;
        let w = driver.cdp().runtime().evaluate_value("window.innerWidth").await?;
        let h = driver.cdp().runtime().evaluate_value("window.innerHeight").await?;
        assert_eq!(w, serde_json::json!(444));
        assert_eq!(h, serde_json::json!(333));
        driver.cdp().emulation().clear_device_metrics_override().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn emulation_set_user_agent_override_changes_navigator() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().emulation().set_user_agent_override("EmuUA/1.0").await?;
        driver.goto(BLANK_HTML).await?;
        let ua = driver.cdp().runtime().evaluate_value("navigator.userAgent").await?;
        assert_eq!(ua, serde_json::json!("EmuUA/1.0"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn emulation_set_locale_override_changes_intl_resolved_locale() -> WebDriverResult<()> {
    // `Emulation.setLocaleOverride` affects Intl/ICU APIs but NOT
    // `navigator.language` (that's tied to `acceptLanguage` on the user-
    // agent override). Assert against `Intl.DateTimeFormat`.
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().emulation().set_locale_override("fr-FR").await?;
        driver.goto(BLANK_HTML).await?;
        let locale = driver
            .cdp()
            .runtime()
            .evaluate_value("Intl.DateTimeFormat().resolvedOptions().locale")
            .await?;
        assert_eq!(locale, serde_json::json!("fr-FR"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn emulation_set_timezone_override_changes_resolved_time_zone() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().emulation().set_timezone_override("Europe/Berlin").await?;
        driver.goto(BLANK_HTML).await?;
        let tz = driver
            .cdp()
            .runtime()
            .evaluate_value("Intl.DateTimeFormat().resolvedOptions().timeZone")
            .await?;
        assert_eq!(tz, serde_json::json!("Europe/Berlin"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn emulation_set_geolocation_override_and_clear() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().emulation().set_geolocation_override(48.85, 2.35, 50.0).await?;
        driver.cdp().emulation().clear_geolocation_override().await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn emulation_set_emulated_media_dark_mode() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .cdp()
            .send(emulation::SetEmulatedMedia {
                media: Some(emulation::MediaType::Screen),
                features: Some(vec![emulation::MediaFeature {
                    name: "prefers-color-scheme".to_string(),
                    value: "dark".to_string(),
                }]),
            })
            .await?;
        driver.goto(BLANK_HTML).await?;
        let v = driver
            .cdp()
            .runtime()
            .evaluate_value("matchMedia('(prefers-color-scheme: dark)').matches")
            .await?;
        assert_eq!(v, serde_json::json!(true));
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn input_insert_text_into_focused_input() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .goto("data:text/html,<html><body><input id='i'/><script>document.getElementById('i').focus();</script></body></html>")
            .await?;
        driver.cdp().input().insert_text("hello").await?;
        let v = driver
            .cdp()
            .runtime()
            .evaluate_value("document.getElementById('i').value")
            .await?;
        assert_eq!(v, serde_json::json!("hello"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn input_dispatch_key_event_into_focused_input() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .goto("data:text/html,<html><body><input id='i'/><script>document.getElementById('i').focus();</script></body></html>")
            .await?;
        // Send `a`: keyDown produces the keystroke; `text` carries the
        // typed character per CDP convention.
        driver
            .cdp()
            .send(input::DispatchKeyEvent {
                r#type: input::KeyEventType::KeyDown,
                modifiers: None,
                text: Some("a".to_string()),
                unmodified_text: Some("a".to_string()),
                key: Some("a".to_string()),
                code: Some("KeyA".to_string()),
                native_virtual_key_code: Some(65),
                windows_virtual_key_code: Some(65),
                auto_repeat: None,
                is_keypad: None,
                is_system_key: None,
            })
            .await?;
        driver
            .cdp()
            .send(input::DispatchKeyEvent {
                r#type: input::KeyEventType::KeyUp,
                modifiers: None,
                text: None,
                unmodified_text: None,
                key: Some("a".to_string()),
                code: Some("KeyA".to_string()),
                native_virtual_key_code: Some(65),
                windows_virtual_key_code: Some(65),
                auto_repeat: None,
                is_keypad: None,
                is_system_key: None,
            })
            .await?;
        let v = driver
            .cdp()
            .runtime()
            .evaluate_value("document.getElementById('i').value")
            .await?;
        assert_eq!(v, serde_json::json!("a"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn input_dispatch_mouse_event_clicks_button() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .goto(
                "data:text/html,<html><body style='margin:0'><button id='b' style='position:absolute;left:10px;top:10px;width:80px;height:30px' onclick=\"window.__clicked=true\">click</button></body></html>",
            )
            .await?;
        for ty in [input::MouseEventType::MousePressed, input::MouseEventType::MouseReleased] {
            driver
                .cdp()
                .send(input::DispatchMouseEvent {
                    r#type: ty,
                    x: 50.0,
                    y: 25.0,
                    modifiers: None,
                    button: Some(input::MouseButton::Left),
                    click_count: Some(1),
                    delta_x: None,
                    delta_y: None,
                })
                .await?;
        }
        let v = driver.cdp().runtime().evaluate_value("window.__clicked === true").await?;
        assert_eq!(v, serde_json::json!(true));
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Target
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn target_get_targets_and_create_close() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let before = driver.cdp().target().get_targets().await?;
        let new_id = driver.cdp().target().create_target("about:blank").await?;
        // After: must contain the new target id.
        let after = driver.cdp().target().get_targets().await?;
        assert!(
            after.iter().any(|t| t.target_id == new_id),
            "create_target didn't appear in get_targets"
        );
        assert!(after.len() > before.len());
        driver.cdp().target().close_target(new_id).await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn target_set_auto_attach() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver
            .cdp()
            .send(target::SetAutoAttach {
                auto_attach: true,
                wait_for_debugger_on_start: false,
                flatten: true,
            })
            .await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn storage_clear_data_for_origin() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().storage().clear_all_data_for_origin("https://example.com").await?;
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn storage_clear_cookies() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().storage().clear_cookies().await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Log
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn log_enable_clear_disable() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().log().enable().await?;
        driver.cdp().log().clear().await?;
        driver.cdp().log().disable().await?;
        // The `Log.entryAdded` event is exercised in cdp_events.rs.
        let _ = log::Enable; // keep `log` import live without warnings.
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Performance
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn performance_enable_get_metrics_disable() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        driver.cdp().performance().enable().await?;
        let metrics = driver.cdp().performance().get_metrics().await?;
        // Chrome should always return at least a few metrics.
        assert!(!metrics.is_empty(), "expected non-empty metrics");
        // `Timestamp` is one of the consistently-present metric names.
        assert!(metrics.iter().any(|m| m.name == "Timestamp"));
        let _ = performance::Disable; // keep import live
        driver.cdp().performance().disable().await?;
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Untyped + legacy escape hatches
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn cdp_send_raw_escape_hatch() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let v = driver.cdp().send_raw("Browser.getVersion", serde_json::json!({})).await?;
        assert!(v["userAgent"].as_str().unwrap().to_ascii_lowercase().contains("chrome"));
        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn cdp_browser_get_version_via_command_struct() -> WebDriverResult<()> {
    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        let info = driver.cdp().send(browser::GetVersion).await?;
        assert!(info.user_agent.to_ascii_lowercase().contains("chrome"));
        // Touch every typed enum so a missing import here would fail to build.
        let _ = (
            fetch::RequestStage::Request,
            page::TransitionType::Link,
            page::ImageFormat::Png,
            page::NavigationType::Navigation,
            input::MouseButton::Left,
            input::KeyEventType::KeyDown,
            input::MouseEventType::MouseMoved,
            performance::TimeDomain::TimeTicks,
            log::LogLevel::Error,
            log::LogSource::Javascript,
            runtime::ConsoleApiType::Log,
            target::SetAutoAttach {
                auto_attach: false,
                wait_for_debugger_on_start: false,
                flatten: true,
            },
            storage::ClearCookies::default(),
        );
        driver.quit().await
    })
    .await
}

// ---------------------------------------------------------------------------
// Selenium legacy `/log` endpoint (chromedriver-only)
//
// chromedriver accepts `POST /session/{id}/log` even in W3C mode (provided
// the session has a `goog:loggingPrefs` capability), but rejects the
// companion `GET /session/{id}/log/types` with
// `"Cannot call non W3C standard command while in W3C mode"`. The
// `get_log_types` wrapper is wired up for completeness / older drivers /
// Selenium Grid, but is not tested here against modern chromedriver.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn browser_log_via_selenium_endpoint() -> WebDriverResult<()> {
    use thirtyfour::LoggingPrefsLogLevel;

    with_timeout(async {
        let mut caps = chrome_caps();
        caps.set_browser_log_level(LoggingPrefsLogLevel::All)?;
        let driver = launch_managed_chrome(caps).await?;

        driver
            .goto(
                "data:text/html,<html><body><script>console.log('hello-thirtyfour-log');\
                console.error('hello-thirtyfour-err');</script></body></html>",
            )
            .await?;

        // chromedriver buffers log entries lazily — poll briefly so a
        // slow runner doesn't flake.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        let mut entries = Vec::new();
        while tokio::time::Instant::now() < deadline {
            entries.extend(driver.browser_log().await?);
            if entries.iter().any(|e| e.message.contains("hello-thirtyfour-log")) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(
            entries.iter().any(|e| e.message.contains("hello-thirtyfour-log")),
            "expected hello-thirtyfour-log in browser log, got: {:?}",
            entries.iter().map(|e| &e.message).collect::<Vec<_>>(),
        );

        // Errors should also surface via the same buffer.
        let mut have_err = entries.iter().any(|e| e.message.contains("hello-thirtyfour-err"));
        let deadline2 = tokio::time::Instant::now() + Duration::from_secs(2);
        while !have_err && tokio::time::Instant::now() < deadline2 {
            entries.extend(driver.browser_log().await?);
            have_err = entries.iter().any(|e| e.message.contains("hello-thirtyfour-err"));
            if have_err {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(
            have_err,
            "expected hello-thirtyfour-err in browser log, got: {:?}",
            entries.iter().map(|e| &e.message).collect::<Vec<_>>(),
        );

        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn browser_log_drains_on_each_call() -> WebDriverResult<()> {
    use thirtyfour::LoggingPrefsLogLevel;

    with_timeout(async {
        let mut caps = chrome_caps();
        caps.set_browser_log_level(LoggingPrefsLogLevel::All)?;
        let driver = launch_managed_chrome(caps).await?;

        driver
            .goto(
                "data:text/html,<html><body><script>console.log('drain-marker-1');</script></body></html>",
            )
            .await?;

        // First read collects (possibly across multiple polls); second
        // read should not see the same marker again — chromedriver
        // drains the buffer per call.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        let mut first = Vec::new();
        while tokio::time::Instant::now() < deadline {
            first.extend(driver.browser_log().await?);
            if first.iter().any(|e| e.message.contains("drain-marker-1")) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(first.iter().any(|e| e.message.contains("drain-marker-1")));

        let second = driver.browser_log().await?;
        assert!(
            !second.iter().any(|e| e.message.contains("drain-marker-1")),
            "drain-marker-1 reappeared after drain: {:?}",
            second.iter().map(|e| &e.message).collect::<Vec<_>>(),
        );

        driver.quit().await
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn legacy_chrome_devtools_still_works() -> WebDriverResult<()> {
    // Backward compat: the deprecated `ChromeDevTools` helper must still
    // work for the lifetime of the deprecation period.
    #[allow(deprecated)]
    use thirtyfour::extensions::cdp::ChromeDevTools;

    with_timeout(async {
        let driver = launch_managed_chrome(chrome_caps()).await?;
        #[allow(deprecated)]
        let dev = ChromeDevTools::new(driver.handle().clone());
        #[allow(deprecated)]
        let v = dev.execute_cdp("Browser.getVersion").await?;
        assert!(v["userAgent"].as_str().unwrap().to_ascii_lowercase().contains("chrome"));
        driver.quit().await
    })
    .await
}
