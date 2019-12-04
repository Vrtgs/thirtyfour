# thirtyfour

Selenium webdriver client for Rust, inspired by the python selenium library.

Named after the atomic number for the Selenium chemical element (Se).

**Status**: Very early stages, but I hope to get this up and running by early 2020.

### Example

Here's what it looks like right now:

```rust 
let caps = serde_json::json!({
    "browserName": "chrome",
    "version": "",
    "platform": "any"
});

let driver = WebDriverSync::new("http://localhost:4444/wd/hub", caps)?;
driver.get("https://mozilla.org");
println!("Title: {}", driver.title()?);
```

Both sync and async versions will be supported.
