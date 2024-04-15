# Waiting For Element Changes

Sometimes you already have a reference to an element, but you want to perform an action that
might change the element in some way. One way to do this would be to call `WebDriver::query()`
and poll for the element with its new attributes. But it might be challenging to get the right
query, and the query might return a different element that already has the attribute you specified.

If you already have a reference to the element, why not use that to poll for the changes you expect?

The `thirtyfour` crate provides a way to wait for virtually any desired change on an existing element.
It's called `ElementWaiter`.

## ElementWaiter

The `WebElement::wait_until()` method returns an `ElementWaiter` struct.

Using `ElementWaiter` you can do things like this:

```rust
elem.wait_until().displayed().await?;
// You can optionally provide a nicer error message like this.
elem.wait_until().error("Timed out waiting for element to disappear").not_displayed().await?;

elem.wait_until().enabled().await?;
elem.wait_until().clickable().await?;
```

And so on. See the [ElementWaiter](https://docs.rs/thirtyfour/latest/thirtyfour/extensions/query/struct.ElementWaiter.html) docs for more details.
