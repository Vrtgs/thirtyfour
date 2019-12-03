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
    pub mod connection_common;
    pub mod connection_sync;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
