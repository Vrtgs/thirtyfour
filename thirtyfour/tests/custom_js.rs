//! Tests for validating functionality based on executing crate maintained JavaScript
use crate::common::drag_to_url;
use thirtyfour::prelude::*;

mod common;

async fn drag_to(c: WebDriver) -> Result<(), WebDriverError> {
    let drag_to_url = drag_to_url();
    c.goto(&drag_to_url).await?;

    // Validate we are starting with a div and an image that are adjacent to one another.
    c.find(By::XPath("//div[@id='target']/../img[@id='draggable']")).await?;

    // Drag the image to the target div
    let elem = c.find(By::Id("draggable")).await?;
    let target = c.find(By::Id("target")).await?;
    elem.js_drag_to(&target).await?;

    // Validate that the image was moved into the target div
    c.find(By::XPath("//div[@id='target']/img[@id='draggable']")).await?;
    Ok(())
}

mod tests {
    use super::*;

    local_tester!(drag_to);
}
