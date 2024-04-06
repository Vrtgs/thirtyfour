# Installation

To use the `thirtyfour` crate in your Rust project, you need to add it as a dependency in your `Cargo.toml` file:

    [dependencies]
    thirtyfour = "THIRTYFOUR_CRATE_VERSION"

To automate a web browser, `thirtyfour` needs to communicate with a webdriver server. You will need
to download the appropriate webdriver server for your browser.

* For Chrome, download [chromedriver](https://sites.google.com/a/chromium.org/chromedriver/downloads)
* For Firefox, download [geckodriver](https://github.com/mozilla/geckodriver/releases)

The webdriver may be zipped. Unzip it and place the webdriver binary somewhere in your `PATH`.
Make sure it is executable and that you have permissions to run it.

You will also need the corresponding web browser to be installed in your Operating System.
Make sure the webdriver version you download corresponds to the version of the browser you have installed.

> If the webdriver is not the right version for your browser, it will show an error message when
> you try to start a new session using `thirtyfour`.
