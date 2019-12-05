use std::sync::Arc;

use crate::common::command::ElementId;
use crate::RemoteConnectionAsync;

#[derive(Debug, Clone)]
pub struct WebElement {
    element_id: ElementId,
    conn: Arc<RemoteConnectionAsync>,
}

impl WebElement {
    pub fn new(conn: Arc<RemoteConnectionAsync>, element_id: ElementId) -> Self {
        WebElement { conn, element_id }
    }
}
