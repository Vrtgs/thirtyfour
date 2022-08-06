mod resolver;

pub use resolver::*;
#[cfg(feature = "component")]
pub use thirtyfour_macros::Component;
