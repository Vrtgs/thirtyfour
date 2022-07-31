//! Export upstream types.

// Re-export entire fantoccini crate to allow users to access
// anything we missed without needing to import it directly themselves.
pub extern crate fantoccini;

// Re-export fantoccini types
pub use fantoccini::actions;
pub use fantoccini::cookies::Cookie;
pub use fantoccini::elements::{ElementRef, Form};
pub use fantoccini::key::Key;
pub use fantoccini::wd::Capabilities;
pub use fantoccini::wd::TimeoutConfiguration;
pub use fantoccini::wd::WebDriverStatus;
pub use fantoccini::wd::WindowHandle;
pub(crate) use fantoccini::Locator;
