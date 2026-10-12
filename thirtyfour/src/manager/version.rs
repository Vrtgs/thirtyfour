/// How the manager picks which driver version to download.
///
/// Defaults to [`DriverVersion::MatchLocalBrowser`] — the lowest-friction
/// option for development.
///
/// **Note for Firefox**: Firefox version numbers and geckodriver version
/// numbers are not in 1:1 correspondence — geckodriver is on `0.36.0` while
/// Firefox is on `150.x`. The manager picks a compatible geckodriver from an
/// embedded table modelled after [SeleniumHQ's `geckodriver-support.json`].
/// `DriverVersion::Latest` for Firefox is instead the newest geckodriver
/// release on GitHub. For [`DriverVersion::Exact`], pass a literal
/// geckodriver tag like `"0.36.0"`.
///
/// [SeleniumHQ's `geckodriver-support.json`]: https://github.com/SeleniumHQ/selenium/blob/trunk/common/geckodriver/geckodriver-support.json
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DriverVersion {
    /// Probe the locally-installed browser binary for its version, then pick a
    /// matching driver. This is the default.
    ///
    /// For Firefox: see the type-level note — applies the upstream
    /// geckodriver compatibility table.
    #[default]
    MatchLocalBrowser,

    /// Read `browserVersion` from the capabilities passed to `launch()` /
    /// `WebDriver::managed()`. Errors with [`super::ManagerError::MissingCapabilityVersion`]
    /// if the field is absent.
    ///
    /// For Firefox: see the type-level note — `browserVersion` is interpreted
    /// as a Firefox version, mapped to a compatible geckodriver release.
    FromCapabilities,

    /// Latest stable available from the upstream metadata source.
    ///
    /// Checked on every launch, so a long-lived manager picks up new
    /// releases. A running driver or cached binary for the resolved version
    /// is still reused.
    ///
    /// <div class="warning">
    ///
    /// **Warning:** In offline mode, `.latest()` selects the newest cached
    /// driver, which may be older than the latest upstream release.
    ///
    /// </div>
    Latest,

    /// An exact version string.
    ///
    /// - Chrome / Edge: a full version (`"126.0.6478.126"`) or major-only
    ///   (`"126"`); the manager resolves a major-only spec to the latest
    ///   matching upstream entry.
    /// - Firefox: a geckodriver tag (`"0.36.0"` — not a Firefox version).
    Exact(String),
}

impl DriverVersion {
    /// Returns `true` if the variant requires capabilities to resolve.
    pub fn needs_capabilities(&self) -> bool {
        matches!(self, DriverVersion::FromCapabilities)
    }
}
