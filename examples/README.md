# Examples

The following examples assume you have a selenium server running at localhost:4444.

You can set this up using docker, as follows:

    docker run --rm -d -p 4444:4444 --name selenium-server -v /dev/shm:/dev/shm selenium/standalone-chrome:3.141.59-zinc

Alternatively you can download selenium from the web ([https://selenium.dev/downloads](https://selenium.dev/downloads)) and run it manually:

    java -jar selenium-server-standalone-3.141.59.jar


### Async

```bash
$ cargo run --bin async
```

### Sync

```bash
$ cargo run --bin sync
```
