//! Tests for validating functionality based on executing crate maintained JavaScript

use rstest::rstest;
use thirtyfour::{prelude::*, support::block_on};

use crate::common::*;

mod common;

#[rstest]
fn drag_to(test_harness: TestHarness<'_>) -> WebDriverResult<()> {
    let c = test_harness.driver();
    block_on(async {
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
    })
}
