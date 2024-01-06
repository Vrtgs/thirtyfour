use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};
use thirtyfour::prelude::*;

static SERVER: OnceLock<Server> = OnceLock::new();
static LIMITER: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

const ASSETS_DIR: &str = "tests/test_html";
const PORT: u16 = 8081;

pub fn make_capabilities(s: &str) -> Capabilities {
    match s {
        "firefox" => {
            let mut caps = DesiredCapabilities::firefox();
            // caps.set_headless().unwrap();
            caps.into()
        }
        "chrome" => {
            let mut caps = DesiredCapabilities::chrome();
            // caps.set_headless().unwrap();
            caps.set_no_sandbox().unwrap();
            caps.set_disable_gpu().unwrap();
            caps.set_disable_dev_shm_usage().unwrap();
            caps.add_arg("--no-sandbox").unwrap();
            caps.into()
        }
        browser => unimplemented!("unsupported browser backend {}", browser),
    }
}

pub fn webdriver_url(s: &str) -> String {
    match s {
        "firefox" => "http://localhost:4444/wd/hub".to_string(),
        "chrome" => "http://localhost:9515/wd/hub".to_string(),
        browser => unimplemented!("unsupported browser backend {}", browser),
    }
}

pub struct Server;

/// Starts the web server.
pub fn start_server() {
    SERVER.get_or_init(|| {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(async {
                println!("SERVER STARTING");
                let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
                let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
                let app = axum::Router::new()
                    .nest_service("/", tower_http::services::ServeDir::new(ASSETS_DIR));
                axum::serve(listener, app).await.unwrap();
            });
        });
        Server
    });
}

pub fn get_limiter<'a>() -> &'a Arc<Mutex<()>> {
    LIMITER.get_or_init(|| Arc::new(Mutex::new(())))
}

/// Locks the Firefox browser for exclusive use.
pub fn lock_firefox<'a>(browser: &str) -> Option<MutexGuard<'a, ()>> {
    if browser == "firefox" {
        Some(get_limiter().lock().unwrap())
    } else {
        None
    }
}

#[macro_export]
macro_rules! test_inner {
    ($fn:ident, $browser:literal) => {
        common::start_server();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let _lock = common::lock_firefox($browser);
            let caps = common::make_capabilities($browser);
            let webdriver_url = common::webdriver_url($browser);
            let driver =
                WebDriver::new(webdriver_url, caps).await.expect("Failed to create WebDriver");

            let result = $fn(driver.clone()).await;
            driver.quit().await.expect("Failed to quit");
            result.expect("test failed");
        });
    };
}

#[macro_export]
macro_rules! local_tester {
    ($($fn:ident),+) => {
        paste::paste! {
            $(
                // #[test]
                // fn [<firefox_ $fn>]() {
                //     $crate::test_inner!($fn, "firefix");
                // }

                #[test]
                fn [<chrome_ $fn>]() {
                    $crate::test_inner!($fn, "chrome");
                }
            )+
        }
    };
}

pub fn sample_page_url() -> String {
    format!("http://localhost:{PORT}/sample_page.html")
}

pub fn other_page_url() -> String {
    format!("http://localhost:{PORT}/other_page.html")
}

pub fn drag_to_url() -> String {
    format!("http://localhost:{PORT}/drag_to.html")
}
