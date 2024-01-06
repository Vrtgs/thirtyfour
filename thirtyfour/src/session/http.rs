use std::{sync::Arc, time::Duration};

use base64::Engine;
use http::{
    header::{ACCEPT, AUTHORIZATION, CONNECTION, CONTENT_TYPE, USER_AGENT},
    Request, Response,
};
use serde_json::Value;
use url::Url;

use crate::{
    common::config::WebDriverConfig,
    prelude::{WebDriverError, WebDriverResult},
    ElementId, ElementRef, RequestData, WebElement,
};

use super::handle::SessionHandle;

/// Enum representing the body of an HTTP request.
#[derive(Debug, Clone)]
pub enum Body {
    /// Empty body.
    Empty,
    /// JSON body.
    Json(Value),
}

impl From<Option<Value>> for Body {
    fn from(value: Option<Value>) -> Self {
        match value {
            None => Body::Empty,
            Some(value) => Body::Json(value),
        }
    }
}

/// Trait used to implement a HTTP client.
#[async_trait::async_trait]
pub trait HttpClient {
    /// Send an HTTP request and return the response.
    async fn send(&self, request: Request<Body>) -> WebDriverResult<Response<Vec<u8>>>;
}

#[cfg(feature = "reqwest")]
#[async_trait::async_trait]
impl HttpClient for reqwest::Client {
    async fn send(&self, request: Request<Body>) -> WebDriverResult<Response<Vec<u8>>> {
        let (parts, body) = request.into_parts();
        let method = match parts.method {
            http::Method::GET => reqwest::Method::GET,
            http::Method::POST => reqwest::Method::POST,
            http::Method::DELETE => reqwest::Method::DELETE,
            http::Method::PUT => reqwest::Method::PUT,
            http::Method::PATCH => reqwest::Method::PATCH,
            http::Method::HEAD => reqwest::Method::HEAD,
            http::Method::OPTIONS => reqwest::Method::OPTIONS,
            http::Method::CONNECT => reqwest::Method::CONNECT,
            http::Method::TRACE => reqwest::Method::TRACE,
            x => return Err(WebDriverError::HttpError(format!("Invalid HTTP method: {x}"))),
        };
        let mut req = self.request(method, parts.uri.to_string());
        for (key, value) in parts.headers.into_iter() {
            let key = match key {
                Some(x) => x,
                None => continue,
            };
            req = req.header(key.to_string(), value.as_bytes().to_vec());
        }
        match body {
            Body::Empty => {}
            Body::Json(json) => {
                req = req.json(&json);
            }
        }

        let resp = req.send().await?;
        let status_u16 = resp.status().as_u16();
        let mut builder = Response::builder();

        builder = builder.status(
            http::StatusCode::from_u16(status_u16)
                .map_err(|e| WebDriverError::HttpError(format!("invalid status code: {e}")))?,
        );
        for (key, value) in resp.headers().iter() {
            builder = builder.header(key.to_string(), value.as_bytes().to_vec());
        }

        let body = resp.bytes().await?.to_vec();
        let body_str = String::from_utf8_lossy(&body).to_string();
        let resp = builder
            .body(body)
            .map_err(|_| WebDriverError::UnknownResponse(status_u16, body_str.to_string()))?;
        Ok(resp)
    }
}

pub(crate) fn create_reqwest_client(timeout: Duration) -> reqwest::Client {
    reqwest::Client::builder().timeout(timeout).build().expect("Failed to create reqwest client")
}

pub(crate) async fn run_webdriver_cmd(
    client: &(dyn HttpClient + Send + Sync),
    request_data: RequestData,
    server_url: &Url,
    config: &WebDriverConfig,
) -> WebDriverResult<CmdResponse> {
    let uri: String = server_url
        .join(&request_data.uri)
        .map_err(|e| WebDriverError::ParseError(format!("invalid url: {e}")))?
        .to_string();
    let mut builder = http::Request::builder()
        .method(request_data.method)
        .uri(uri)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json;charset=UTF-8")
        .header(USER_AGENT, &config.user_agent);

    // Authentication.
    let url_username = server_url.username();
    let url_password = server_url.password();
    if !url_username.is_empty() || url_password.is_some() {
        let base64_string = base64::prelude::BASE64_STANDARD.encode(&format!(
            "{}:{}",
            url_username,
            url_password.unwrap_or_default()
        ));
        builder = builder.header(AUTHORIZATION, format!("Basic {}", base64_string));
    }

    // Optional headers.
    if config.keep_alive {
        builder = builder.header(CONNECTION, "keep-alive");
    }

    let body = match request_data.body {
        Some(body) => Body::Json(body),
        None => Body::Empty,
    };

    let request = builder
        .body(body)
        .map_err(|e| WebDriverError::RequestFailed(format!("invalid request body: {e}")))?;
    let response = client.send(request).await?;
    let status = response.status().as_u16();
    match status {
        200..=399 => {
            match serde_json::from_slice(&response.body()) {
                Ok(v) => Ok(CmdResponse {
                    body: v,
                    status,
                }),
                Err(_) => {
                    // Try to parse the response as a string.
                    let body = String::from_utf8_lossy(&response.body()).to_string();
                    Err(WebDriverError::parse(status, body))
                }
            }
        }
        _ => {
            let body = String::from_utf8_lossy(&response.body()).to_string();
            Err(WebDriverError::parse(status, body))
        }
    }
}

/// Struct representing a WebDriver command response.
#[derive(Debug, Clone)]
pub struct CmdResponse {
    /// The body of the response.
    pub body: serde_json::Value,
    /// The HTTP status code of the response.
    pub status: u16,
}

impl CmdResponse {
    /// Get the `value` field of the response as a JSON value.
    pub fn value_json(self) -> WebDriverResult<Value> {
        match self.body {
            Value::Object(mut x) => x
                .remove("value")
                .ok_or_else(|| WebDriverError::Json("Unexpected response body".to_string())),
            _ => Err(WebDriverError::Json("Unexpected response body".to_string())),
        }
    }

    /// Deserialize the value of the response.
    pub fn value<T: serde::de::DeserializeOwned>(self) -> WebDriverResult<T> {
        serde_json::from_value(self.value_json()?)
            .map_err(|e| WebDriverError::Json(format!("Failed to decode response body: {:?}", e)))
    }

    /// Deserialize the element from the response.
    pub fn element(self, handle: Arc<SessionHandle>) -> WebDriverResult<WebElement> {
        let elem_id: ElementRef = serde_json::from_value(self.value_json()?)?;
        Ok(WebElement::new(ElementId::from(elem_id.id()), handle))
    }

    /// Deserialize a list of elements from the response.
    pub fn elements(self, handle: Arc<SessionHandle>) -> WebDriverResult<Vec<WebElement>> {
        let values: Vec<ElementRef> = serde_json::from_value(self.value_json()?)?;
        Ok(values
            .into_iter()
            .map(|x| WebElement::new(ElementId::from(x.id()), handle.clone()))
            .collect())
    }
}
