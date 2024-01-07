# Using Selenium Manager

The selenium team is working on a project called "Selenium Manager", which is similar
to bonigarcia's WebDriverManager but as a CLI. It's written in Rust as a Clap CLI,
so we have the benefit of using it as a library as well. To add it to your project,
you can add the selenium project as a git dependency in your Cargo.toml. Be sure to specify
the branch is "trunk", like so.

```
[dependencies]
selenium-manager = { git = "https://github.com/SeleniumHQ/selenium", branch = "trunk" }
```

> **NOTE:** Due to the way the selenium repository is structured, it is quite large. 
> Better integration with selenium-manager in `thirtyfour` is planned.