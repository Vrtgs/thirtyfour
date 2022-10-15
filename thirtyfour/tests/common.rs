#![allow(dead_code)]
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use thirtyfour::prelude::*;
use tokio::fs::read_to_string;

const ASSETS_DIR: &str = "tests/test_html";

pub fn make_capabilities(s: &str) -> Capabilities {
    match s {
        "firefox" => {
            let mut caps = DesiredCapabilities::firefox();
            caps.set_headless().unwrap();
            caps.into()
        }
        "chrome" => {
            let mut caps = DesiredCapabilities::chrome();
            caps.set_headless().unwrap();
            caps.set_no_sandbox().unwrap();
            caps.set_disable_gpu().unwrap();
            caps.set_disable_dev_shm_usage().unwrap();
            caps.into()
        }
        browser => unimplemented!("unsupported browser backend {}", browser),
    }
}

pub fn make_url(s: &str) -> &'static str {
    match s {
        "firefox" => "http://localhost:4444",
        "chrome" => "http://localhost:9515",
        browser => unimplemented!("unsupported browser backend {}", browser),
    }
}

pub fn handle_test_error(
    res: Result<Result<(), WebDriverError>, Box<dyn std::any::Any + Send>>,
) -> bool {
    match res {
        Ok(Ok(_)) => true,
        Ok(Err(e)) => {
            eprintln!("test future failed to resolve: {:?}", e);
            false
        }
        Err(e) => {
            if let Some(e) = e.downcast_ref::<WebDriverError>() {
                eprintln!("test future panicked: {:?}", e);
            } else {
                eprintln!("test future panicked; an assertion probably failed");
            }
            false
        }
    }
}

#[macro_export]
macro_rules! tester {
    ($f:ident, $endpoint:expr) => {{
        use common::{make_capabilities, make_url};
        let url = make_url($endpoint);
        let caps = make_capabilities($endpoint);
        tester_inner!($f, WebDriver::new(url, caps));
    }};
}

#[macro_export]
macro_rules! tester_inner {
    ($f:ident, $connector:expr) => {{
        use std::sync::{Arc, Mutex};
        use std::thread;

        let c = $connector;

        // we'll need the session_id from the thread
        // NOTE: even if it panics, so can't just return it
        let session_id = Arc::new(Mutex::new(None));

        // run test in its own thread to catch panics
        let sid = session_id.clone();
        let res = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let c = rt.block_on(c).expect("failed to construct test WebDriver");
            *sid.lock().unwrap() = rt.block_on(c.session_id()).ok();
            // make sure we close, even if an assertion fails
            let client = c.clone();
            let x = rt.block_on(async move {
                let r = tokio::spawn($f(c)).await;
                let _ = client.quit().await;
                r
            });
            drop(rt);
            x.expect("test panicked")
        })
        .join();
        let success = common::handle_test_error(res);
        assert!(success);
    }};
}

#[macro_export]
macro_rules! local_tester {
    ($f:ident, $endpoint:expr) => {{
        use thirtyfour::prelude::*;

        let port = common::setup_server();
        let url = common::make_url($endpoint);
        let caps = common::make_capabilities($endpoint);
        let f = move |c: WebDriver| async move { $f(c, port).await };
        tester_inner!(f, WebDriver::new(url, caps));
    }};
}

/// Sets up the server and returns the port it bound to.
pub fn setup_server() -> u16 {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let _ = rt.block_on(async {
            let (socket_addr, server) = start_server();
            tx.send(socket_addr.port()).expect("To be able to send port");
            server.await.expect("To start the server")
        });
    });

    rx.recv().expect("To get the bound port.")
}

/// Configures and starts the server
fn start_server() -> (SocketAddr, impl Future<Output = hyper::Result<()>> + 'static) {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

    let server = Server::bind(&socket_addr).serve(make_service_fn(move |_| async {
        Ok::<_, Infallible>(service_fn(handle_file_request))
    }));

    let addr = server.local_addr();
    (addr, server)
}

/// Tries to return the requested html file
async fn handle_file_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let uri_path = req.uri().path().trim_matches(&['/', '\\'][..]);

    // tests only contain html files
    // needed because the content-type: text/html is returned
    if !uri_path.ends_with(".html") {
        return Ok(file_not_found());
    }

    // this does not protect against a directory traversal attack
    // but in this case it's not a risk
    let asset_file = Path::new(ASSETS_DIR).join(uri_path);

    let ctn = match read_to_string(asset_file).await {
        Ok(ctn) => ctn,
        Err(_) => return Ok(file_not_found()),
    };

    let res = Response::builder()
        .header("content-type", "text/html")
        .header("content-length", ctn.len())
        .body(ctn.into())
        .unwrap();

    Ok(res)
}

/// Response returned when a file is not found or could not be read
fn file_not_found() -> Response<Body> {
    Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap()
}

pub fn sample_page_url(port: u16) -> String {
    format!("http://localhost:{}/sample_page.html", port)
}

pub fn other_page_url(port: u16) -> String {
    format!("http://localhost:{}/other_page.html", port)
}

pub fn drag_to_url(port: u16) -> String {
    format!("http://localhost:{}/drag_to.html", port)
}
