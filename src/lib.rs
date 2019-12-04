//! Selenium client for working with W3C-compatible WebDriver implmentations.
//!
//! This project is still WIP.

pub mod common {
    pub mod capabilities;
    pub mod constant;
    pub mod keys;
}
pub mod remote {
    pub mod command;
    pub mod connection_async;
    mod connection_common;
    pub mod connection_sync;
}
pub mod error;
pub mod webdriver;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
