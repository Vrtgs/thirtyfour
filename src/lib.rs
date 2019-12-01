//! Selenium client for working with W3C-compatible WebDriver implmentations.
//!
//! This project is still WIP.

mod command;
mod constant;
mod keys;
mod remote_connection;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
