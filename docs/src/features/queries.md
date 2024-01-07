# Element Queries

## Basic queries
The `find()` and `find_all()` methods in both `WebDriver` and `WebElement` provide a simple
way to perform direct element queries, returning a result instantly.

However, for many types of queries these methods are inadequate. For example, there is no polling, 
and no way to wait for something to show up on a page.
If an element doesn't exist at the instant you look for it, you'll get an error.

Obviously this isn't helpful for the majority of element queries, so `thirtyfour` provides a 
more advanced query interface, called `ElementQuery`.

## ElementQuery

The `WebDriver::query()` and `WebElement::query()` methods return an `ElementQuery` struct.

Using `ElementQuery`, you can do things like:

```rust
let elem_text =
    driver.query(By::Css("match.this")).or(By::Id("orThis")).first().await?;
```

This will execute both queries once per poll iteration and return the first one that matches.

See [ElementQuery](https://docs.rs/thirtyfour/latest/thirtyfour/extensions/query/struct.ElementQuery.html) for more details.