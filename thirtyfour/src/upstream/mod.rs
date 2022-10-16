//! Export upstream types.

// Re-export entire fantoccini crate to allow users to access
// anything we missed without needing to import it directly themselves.
pub extern crate fantoccini;

// Re-export fantoccini types
pub use fantoccini::{
    actions,
    cookies::Cookie,
    elements::{ElementRef, Form},
    error::NewSessionError,
    key::Key,
    wd::{Capabilities, TimeoutConfiguration, WebDriverStatus, WindowHandle},
};

// Imports needed internally to thirtyfour.
pub(crate) use fantoccini::{
    elements::Element,
    error::{CmdError, ErrorStatus},
    wd::WebDriverCompatibleCommand,
    ClientBuilder, Locator,
};
