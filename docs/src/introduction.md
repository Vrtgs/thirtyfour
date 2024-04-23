# Introduction

Welcome to The Book for `thirtyfour`.

`thirtyfour` is a crate for automating Web Browsers in Rust using the `WebDriver` / `Selenium` ecosystem.
It also provides some support for the `Chrome DevTools Protocol`, which is used by popular frameworks
such as Cypress and Playwright.

## Why is it called "thirtyfour" ?

Thirty-four (34) is the atomic number for the Selenium chemical element (Se) ⚛️.

## Features

- All W3C WebDriver V1 and WebElement methods are supported
- Create new browser session directly via WebDriver (e.g. chromedriver)
- Create new browser session via Selenium Standalone or Grid
- Find elements (via all common selectors e.g. Id, Class, CSS, Tag, XPath)
- Send keys to elements, including key-combinations
- Execute Javascript
- Action Chains
- Get and set cookies
- Switch to frame/window/element/alert
- Shadow DOM support
- Alert support
- Capture / Save screenshot of browser or individual element as PNG
- Chrome DevTools Protocol (CDP) support (limited)
- [Advanced query interface](./features/queries.md) including explicit waits and various predicates
- [Component](./features/components.md) Wrappers (similar to `Page Object Model`)