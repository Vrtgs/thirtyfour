//! CDP-specific error type.
//!
//! Chrome DevTools Protocol commands that fail return a JSON-RPC-shaped error
//! envelope: `{ "code": -32601, "message": "...", "data": ... }`. This module
//! exposes that as a typed [`CdpError`] so callers can match on the numeric
//! `code` instead of grepping error strings.

use serde::Deserialize;
use serde_json::Value;
use std::fmt;

/// A CDP command error.
///
/// Returned by the WebSocket transport ([`crate::cdp::CdpSession`]) when a
/// command fails. The HTTP transport ([`crate::cdp::Cdp`]) routes failures
/// through [`crate::error::WebDriverError`] because chromedriver wraps CDP
/// errors in a W3C error envelope; the raw CDP code/message live in the
/// wrapped [`crate::error::WebDriverErrorInfo`].
#[derive(Debug, Clone, thiserror::Error)]
pub struct CdpError {
    /// The CDP method that failed (e.g. `"Page.navigate"`).
    pub command: String,
    /// JSON-RPC error code. Standard values: `-32700` parse, `-32600` invalid
    /// request, `-32601` method not found, `-32602` invalid params, `-32603`
    /// internal. Server-defined CDP errors use `-32000…-32099`.
    pub code: i32,
    /// Human-readable error message from the browser.
    pub message: String,
    /// Optional structured error data.
    ///
    /// Boxed to keep `CdpError` small: with serde_json's `preserve_order`
    /// feature, an inline `Value` pushes it past clippy's `result_large_err`
    /// threshold.
    pub data: Option<Box<Value>>,
}

impl CdpError {
    /// True if `code` is in the JSON-RPC server-defined range
    /// (`-32000…-32099`) — typically "Target closed", "No target with given
    /// id", "Cannot find context with specified id", etc.
    pub fn is_server_error(&self) -> bool {
        (-32099..=-32000).contains(&self.code)
    }

    /// True if the failure was caused by the target session being closed.
    /// Detected by message because CDP doesn't reserve a code for it.
    pub fn is_target_closed(&self) -> bool {
        let m = self.message.to_ascii_lowercase();
        m.contains("target closed") || m.contains("session closed")
    }
}

impl fmt::Display for CdpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CDP command {} failed: {} {}", self.command, self.code, self.message)
    }
}

impl From<CdpError> for crate::error::WebDriverError {
    fn from(e: CdpError) -> Self {
        // Render CdpError as a `FatalError` string for parity with how
        // BidiError is converted — these protocol errors don't map cleanly
        // onto the W3C HTTP error shape. Callers who want the typed CDP
        // code/message should keep `Result<_, CdpError>` instead of `?`-ing
        // through `WebDriverResult`.
        crate::error::WebDriverError::FatalError(e.to_string())
    }
}

/// Wire shape of a JSON-RPC error envelope, used by the WebSocket transport
/// when parsing responses, and by the unit tests.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(not(any(feature = "cdp-events", test)), allow(dead_code))]
pub(crate) struct CdpErrorEnvelope {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl CdpErrorEnvelope {
    #[cfg_attr(not(any(feature = "cdp-events", test)), allow(dead_code))]
    pub(crate) fn into_error(self, command: impl Into<String>) -> CdpError {
        CdpError {
            command: command.into(),
            code: self.code,
            message: self.message,
            data: self.data.map(Box::new),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // CdpErrorEnvelope wire-shape coverage: when the WebSocket transport
    // dispatches a CDP error, the integration tests in `cdp_events.rs`
    // exercise the parsing path. The tests below cover `into_error()` and
    // the predicate methods, which are our own logic — not serde derive.
    #[test]
    fn into_error_preserves_command_name() {
        let env = CdpErrorEnvelope {
            code: -32601,
            message: "x".to_string(),
            data: None,
        };
        let err = env.into_error("Page.foo");
        assert_eq!(err.command, "Page.foo");
        assert_eq!(err.code, -32601);
        assert!(!err.is_server_error());
    }

    #[test]
    fn is_server_error_inclusive_range() {
        for code in [-32000, -32050, -32099] {
            let err = CdpError {
                command: "x".to_string(),
                code,
                message: String::new(),
                data: None,
            };
            assert!(err.is_server_error(), "{code} should be server error");
        }
        for code in [-31999, -32100, -32700, 0] {
            let err = CdpError {
                command: "x".to_string(),
                code,
                message: String::new(),
                data: None,
            };
            assert!(!err.is_server_error(), "{code} should NOT be server error");
        }
    }

    #[test]
    fn is_target_closed_matches_message_substring() {
        let mk = |msg: &str| CdpError {
            command: "x".to_string(),
            code: -32000,
            message: msg.to_string(),
            data: None,
        };
        assert!(mk("Target closed.").is_target_closed());
        assert!(mk("target closed").is_target_closed());
        assert!(mk("Session closed unexpectedly").is_target_closed());
        assert!(!mk("Some other error").is_target_closed());
    }

    #[test]
    fn display_format_is_useful() {
        let err = CdpError {
            command: "Page.foo".to_string(),
            code: -32601,
            message: "no such method".to_string(),
            data: None,
        };
        let s = err.to_string();
        assert!(s.contains("Page.foo"));
        assert!(s.contains("-32601"));
        assert!(s.contains("no such method"));
    }
}
