# Frequently Asked Questions

If you have a question that's not answered here, head over to the 
[discussion page](https://github.com/stevepryde/thirtyfour/discussions) and ask your question there.

Remember to search through existing discussions and issues to check that your question hasn't already been asked/answered.

## Why Doesn't The Browser Close On Exit?

Rust does not have [async destructors](https://boats.gitlab.io/blog/post/poll-drop/),
which means there is no reliable way to execute an async HTTP request on Drop and wait for
it to complete. This means you are in charge of closing the browser at the end of your code,
via a call to `WebDriver::quit()` as in the above example.

If you do not call `WebDriver::quit()` then the browser will stay open until it is 
either explicitly closed later outside your code, or the session times out.


