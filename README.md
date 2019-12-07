# thirtyfour

Selenium webdriver client for Rust, inspired by the python selenium library.

Named after the atomic number for the Selenium chemical element (Se).

**Status**: Very early stages, but I hope to get this up and running by early 2020.

### Example

Here's what it looks like right now (synchronous version):

```rust 
let caps = serde_json::json!({
    "browserName": "chrome",
    "version": "",
    "platform": "any"
});

let driver = WebDriver::new("http://localhost:4444/wd/hub", caps)?;
driver.get("https://mozilla.org");
println!("Title: {}", driver.title()?);
let elem = driver.find_element(By::Tag("input"))?;
```

Both sync and async versions are supported.
