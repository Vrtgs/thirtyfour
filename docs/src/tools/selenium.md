# Running against selenium

> **NOTE:** This documentation assumes you are already familiar with Selenium.
> To learn more about Selenium, visit the [documentation](https://www.selenium.dev/documentation/)

**NOTE:** To run the selenium example, start selenium server and then run:

    cargo run --example selenium_example

Below, you can find my recommended development environment for running selenium tests.

Essentially, you need three main things set up as a minimum:

1. Selenium standalone running on some server, usually localhost at port 4444.

    For example, `http://localhost:4444`

2. The webdriver for your browser somewhere in your PATH, e.g., chromedriver (Chrome) or geckodriver (Firefox)
3. Your code that imports this library

If you want you can download selenium and the webdriver manually, copy the webdriver
to somewhere in your path, then run selenium manually using `java -jar selenium.jar`.

However, this is a lot of messing around, and you'll need to do it all again any
time either selenium or the webdriver gets updated. A better solution is to run
both selenium and webdriver in a docker container, following the instructions below.

## Setting up Docker and Selenium

To install docker, see [https://docs.docker.com/install/](https://docs.docker.com/install/) (follow the SERVER section if you're on Linux, then look for the Community Edition)

Once you have docker installed, you can start the selenium server, as follows:

    docker run --rm -d -p 4444:4444 -p 5900:5900 --name selenium-server -v /dev/shm:/dev/shm selenium/standalone-chrome:4.1.0-20211123

For more information on running selenium in docker, visit
[docker-selenium](https://github.com/SeleniumHQ/docker-selenium)
