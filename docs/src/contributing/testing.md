# Running the tests for `thirtyfour`

> You only need to run the tests if you plan on contributing to the development of `thirtyfour`.
> If you just want to use the crate in your own project, you can skip this section.

Make sure selenium is not still running (or anything else that might use port 4444 or port 9515).

To run the tests, you need to have an instance of `geckodriver` and an instance of `chromedriver` running in the background, perhaps in separate tabs in your terminal.

Download links for these are here:

* chromedriver: https://chromedriver.chromium.org/downloads
* geckodriver: https://github.com/mozilla/geckodriver/releases

In separate terminal tabs, run the following:

* Tab 1:

      chromedriver

* Tab 2:

      geckodriver

* Tab 3 (navigate to the root of this repository):

      cargo test

  **NOTE:** By default the tests will run in chrome only. If you want to run in firefox, do:
      
      THIRTYFOUR_BROWSER=firefox cargo test