# Migration guide

## 0.37 → 0.38

### `CdpError::data` is now boxed

`CdpError::data` changed from `Option<serde_json::Value>` to
`Option<Box<serde_json::Value>>`. Code that reads the field through a
reference (`err.data.as_ref()`, `if let Some(data) = &err.data`) mostly
keeps working via auto-deref; code that moves the value out needs to
unbox it:

```rust
// Before:
let data: Option<Value> = err.data;

// After:
let data: Option<Value> = err.data.map(|d| *d);
```

## 0.36 → 0.37

- [`WebDriver` construction: builder API](#webdriver-construction-builder-api)
- [`WebDriverConfig::reqwest_timeout` → `request_timeout`](#webdriverconfigreqwest_timeout--request_timeout)
- [`Capabilities` is now a newtype](#capabilities-is-now-a-newtype)
- [`CapabilitiesHelper` overhaul](#capabilitieshelper-overhaul)
- [`insert_browser_option` → `set_browser_option`](#insert_browser_option--set_browser_option)
- [`Alert` and `SwitchTo` types removed](#alert-and-switchto-types-removed)
- [Removed deprecated method names](#removed-deprecated-method-names)
- [Privatised public fields](#privatised-public-fields)
- [Newly deprecated: `extensions::cdp`](#newly-deprecated-extensionscdp)
- [Examples and doctests now use `anyhow`](#examples-and-doctests-now-use-anyhow)

The Chrome DevTools Protocol layer was rewritten in this release,
but the legacy [`thirtyfour::extensions::cdp::ChromeDevTools`] path is
kept (deprecated) so existing CDP code keeps compiling. See the
deprecation notes at the end of this guide for what's been marked for
removal.

### `WebDriver` construction: builder API

`WebDriver::new_with_config` and `WebDriver::new_with_config_and_client`
have been replaced by [`WebDriver::builder`], which returns a
[`WebDriverBuilder`] that implements `IntoFuture` (so `.await` opens the
session). [`WebDriver::new`] still works as the simple two-arg shortcut.

```rust
// Before:
use thirtyfour::common::config::WebDriverConfig;
let config = WebDriverConfig::builder()
    .reqwest_timeout(Duration::from_secs(30))
    .user_agent("my-app/1.0")
    .build()?;
let driver = WebDriver::new_with_config("http://localhost:4444", caps, config).await?;

// After:
let driver = WebDriver::builder("http://localhost:4444", caps)
    .request_timeout(Duration::from_secs(30))
    .user_agent("my-app/1.0")
    .await?;
```

A custom [`HttpClient`] is now supplied via `.client(...)` on the
builder rather than a separate `new_with_config_and_client` constructor.

### `WebDriverConfig::reqwest_timeout` → `request_timeout`

The field, the [`WebDriverConfig`] builder method, and the
[`WebDriverBuilder`] method are all renamed: the timeout applies to any
[`HttpClient`] implementation, not just the default reqwest-based one.

```rust
// Before:
let config = WebDriverConfig::builder()
    .reqwest_timeout(Duration::from_secs(30))
    .build()?;
let timeout = config.reqwest_timeout;

// After:
let config = WebDriverConfig::builder()
    .request_timeout(Duration::from_secs(30))
    .build()?;
let timeout = config.request_timeout;
```

### `Capabilities` is now a newtype

[`Capabilities`] used to be a type alias for `serde_json::Map<String, Value>`.
It's now a `#[serde(transparent)]` newtype with its own inherent
`get` / `get_mut` / `set` / `remove` / `contains_key` / `len` / `is_empty` /
`iter` methods. The wire format is unchanged.

The most likely breakage is `caps.insert(key, value)` (which came from
the underlying `Map`). The replacement is `caps.set(key, value)?`,
which serialises any `T: Serialize` and returns `WebDriverResult<()>`.

```rust
// Before:
use serde_json::json;
let mut caps = Capabilities::new();
caps.insert("browserName".to_string(), json!("chrome"));

// After:
let mut caps = Capabilities::new();
caps.set("browserName", "chrome")?;
```

If you genuinely need a `serde_json::Map`, `Capabilities` is `From<Map>`
and `Into<Value>`, and `iter()` returns the underlying map's iterator.

### `CapabilitiesHelper` overhaul

The [`CapabilitiesHelper`] trait used to require three private-looking
hook methods (`_get`, `_get_mut`, `insert_base_capability`) on every
implementor. It now has a blanket impl on
`AsRef<Capabilities> + AsMut<Capabilities>`, and its accessors are
renamed to match the inherent API on [`Capabilities`]:

| Before                        | After     |
|-------------------------------|-----------|
| `_get(key)`                   | `get(key)` |
| `_get_mut(key)`               | `get_mut(key)` |
| `set_base_capability(k, v)`   | `set(k, v)` |
| `insert_base_capability(...)` | (removed — use `as_mut().set(...)`) |

If you have a custom capability wrapper, replace the manual
`CapabilitiesHelper for MyCaps` impl with `AsRef`/`AsMut` impls — the
trait then comes for free.

```rust
// Before:
impl CapabilitiesHelper for MyCaps {
    fn _get(&self, key: &str) -> Option<&Value> { self.inner._get(key) }
    fn _get_mut(&mut self, key: &str) -> Option<&mut Value> { self.inner._get_mut(key) }
    fn insert_base_capability(&mut self, key: String, value: Value) {
        self.inner.insert_base_capability(key, value);
    }
}

// After:
impl AsRef<Capabilities> for MyCaps {
    fn as_ref(&self) -> &Capabilities { &self.inner }
}
impl AsMut<Capabilities> for MyCaps {
    fn as_mut(&mut self) -> &mut Capabilities { &mut self.inner }
}
```

### `insert_browser_option` → `set_browser_option`

[`BrowserCapabilitiesHelper::set_browser_option`] is the new name for
the old `insert_browser_option`, for consistency with the `set` naming
used elsewhere. Behaviour is unchanged.

```rust
// Before:
caps.insert_browser_option("binary", "/path/to/chrome")?;

// After:
caps.set_browser_option("binary", "/path/to/chrome")?;
```

### `Alert` and `SwitchTo` types removed

Every method on `Alert` and `SwitchTo` had been a deprecated forwarder
since 0.30.0. Both types are now gone, along with the `prelude`
re-exports. Use the equivalent methods on [`WebDriver`] directly:

| Before                                       | After                                     |
|----------------------------------------------|-------------------------------------------|
| `driver.switch_to().active_element().await` | `driver.active_element().await`           |
| `driver.switch_to().alert().text().await`   | `driver.get_alert_text().await`           |
| `driver.switch_to().alert().accept().await` | `driver.accept_alert().await`             |
| `driver.switch_to().alert().dismiss().await`| `driver.dismiss_alert().await`            |
| `.alert().send_keys(...).await`              | `driver.send_alert_text(...).await`       |
| `.default_content().await`                   | `driver.enter_default_frame().await`      |
| `.frame_number(n).await`                     | `driver.enter_frame(n).await`             |
| `.frame_element(&el).await`                  | `el.clone().enter_frame().await`          |
| `.parent_frame().await`                      | `driver.enter_parent_frame().await`       |
| `.new_window().await` / `.new_tab().await`   | `driver.new_window().await` / `new_tab()` |
| `.window(handle).await`                      | `driver.switch_to_window(handle).await`   |
| `.window_name(name).await`                   | `driver.switch_to_named_window(name).await` |

### Removed deprecated method names

The following method names — all marked deprecated in 0.30.x or 0.32.x —
have been removed. Use the new names that are already in the codebase:

| Removed                                  | Replacement              |
|------------------------------------------|--------------------------|
| `WebElement::rectangle`                  | `rect`                   |
| `WebElement::get_property`               | `prop`                   |
| `WebElement::get_attribute`              | `attr`                   |
| `WebElement::get_css_property`           | `css_value`              |
| `WebElement::find_element`               | `find`                   |
| `WebElement::find_elements`              | `find_all`               |
| `SessionHandle::close`                   | `close_window`           |
| `SessionHandle::page_source`             | `source`                 |
| `SessionHandle::find_element`            | `find`                   |
| `SessionHandle::find_elements`           | `find_all`               |
| `SessionHandle::execute_script`          | `execute`                |
| `SessionHandle::execute_script_async`    | `execute_async`          |
| `SessionHandle::current_window_handle`   | `window`                 |
| `SessionHandle::window_handles`          | `windows`                |
| `SessionHandle::set_timeouts`            | `update_timeouts`        |
| `SessionHandle::get_cookies`             | `get_all_cookies`        |
| `SessionHandle::get_cookie`              | `get_named_cookie`       |
| `SessionHandle::switch_to`               | `switch_to_*` methods    |
| `ScriptRet::value`                       | `json`                   |
| `ScriptRet::get_element`                 | `element`                |
| `ScriptRet::get_elements`                | `elements`               |
| `ElementQuery::all`                      | `all_from_selector`      |
| `ElementQuery::all_required`             | `all_from_selector_required` |
| `DesiredCapabilities::accept_ssl_certs`  | `accept_insecure_certs`  |
| `WebDriverConfig::default_user_agent()`  | `WebDriverConfig::DEFAULT_USER_AGENT` constant |

### Privatised public fields

The redundant `pub` fields on [`WebDriver`] and [`WebElement`] are now
`pub(crate)`. Use the accessor methods instead:

| Before                | After                |
|-----------------------|----------------------|
| `driver.handle`       | `driver.handle()`    |
| `element.handle`      | `element.handle()`   |
| `element.element_id`  | `element.element_id()` (already existed) |

`SessionHandle::new()` and the unused intermediate constructors
(`new_with_config`, `new_with_config_and_guard`) are also private now —
the only path to a `SessionHandle` is via [`WebDriver::new`],
[`WebDriver::builder`], or [`WebDriver::managed`].

### Newly deprecated: `extensions::cdp`

The legacy `thirtyfour::extensions::cdp` module is now deprecated in
favour of the new [`thirtyfour::cdp`] module. Existing code keeps
compiling; new code should migrate.

| Deprecated                                              | Replacement                                  |
|---------------------------------------------------------|----------------------------------------------|
| `extensions::cdp::ChromeDevTools`                       | [`Cdp`] via [`WebDriver::cdp`]               |
| `ChromeDevTools::execute_cdp` / `execute_cdp_with_params` | [`Cdp::send`] / [`Cdp::send_raw`]          |
| `extensions::cdp::NetworkConditions` / `ConnectionType` | equivalents in [`cdp::domains::network`]     |
| `extensions::cdp::ChromeCommand`                        | (internal — no replacement needed)           |

```rust
// Before:
use thirtyfour::extensions::cdp::ChromeDevTools;
let dev = ChromeDevTools::new(driver.handle.clone());
let v = dev.execute_cdp("Browser.getVersion").await?;
let ua = v["userAgent"].as_str().unwrap();

// After:
let info = driver.cdp().browser().get_version().await?;
let ua = info.user_agent;
```

The `extensions::cdp::NetworkConditions` wrapper specifically targets
chromedriver's `/chromium/network_conditions` vendor endpoint
(snake_case fields). The new [`cdp::domains::network::NetworkConditions`]
goes through the standard CDP `Network.emulateNetworkConditions`
command and is portable across Chrome, Edge, Brave and Opera.

### Examples and doctests now use `anyhow`

The `color-eyre` dev-dependency has been replaced with `anyhow` in the
examples and the two doctest snippets in the README and `lib.rs`. This
is purely a docs/example change and doesn't affect the public API, but
if you copied a snippet verbatim you'll see `anyhow::Result<()>` where
it used to say `color_eyre::Result<()>`.

[`WebDriver`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebDriver.html
[`WebDriver::new`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebDriver.html#method.new
[`WebDriver::builder`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebDriver.html#method.builder
[`WebDriver::managed`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebDriver.html#method.managed
[`WebDriverBuilder`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebDriverBuilder.html
[`WebDriverConfig`]: https://docs.rs/thirtyfour/latest/thirtyfour/common/config/struct.WebDriverConfig.html
[`WebElement`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebElement.html
[`Capabilities`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.Capabilities.html
[`CapabilitiesHelper`]: https://docs.rs/thirtyfour/latest/thirtyfour/trait.CapabilitiesHelper.html
[`BrowserCapabilitiesHelper::set_browser_option`]: https://docs.rs/thirtyfour/latest/thirtyfour/trait.BrowserCapabilitiesHelper.html#method.set_browser_option
[`HttpClient`]: https://docs.rs/thirtyfour/latest/thirtyfour/session/http/trait.HttpClient.html

[`thirtyfour::cdp`]: https://docs.rs/thirtyfour/latest/thirtyfour/cdp/
[`thirtyfour::extensions::cdp::ChromeDevTools`]: https://docs.rs/thirtyfour/latest/thirtyfour/extensions/cdp/struct.ChromeDevTools.html
[`Cdp`]: https://docs.rs/thirtyfour/latest/thirtyfour/cdp/struct.Cdp.html
[`Cdp::send`]: https://docs.rs/thirtyfour/latest/thirtyfour/cdp/struct.Cdp.html#method.send
[`Cdp::send_raw`]: https://docs.rs/thirtyfour/latest/thirtyfour/cdp/struct.Cdp.html#method.send_raw
[`WebDriver::cdp`]: https://docs.rs/thirtyfour/latest/thirtyfour/struct.WebDriver.html#method.cdp
[`cdp::domains::network`]: https://docs.rs/thirtyfour/latest/thirtyfour/cdp/domains/network/index.html
[`cdp::domains::network::NetworkConditions`]: https://docs.rs/thirtyfour/latest/thirtyfour/cdp/domains/network/struct.NetworkConditions.html
