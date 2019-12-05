use std::sync::Arc;

use crate::common::command::ElementId;
use crate::sync::RemoteConnectionSync;

#[derive(Debug, Clone)]
pub struct WebElement {
    element_id: ElementId,
    conn: Arc<RemoteConnectionSync>,
}

impl WebElement {
    pub fn new(conn: Arc<RemoteConnectionSync>, element_id: ElementId) -> Self {
        WebElement { conn, element_id }
    }
}
