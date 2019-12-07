use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize)]
pub struct ElementRef {
    #[serde(rename(deserialize = "element-6066-11e4-a52e-4f735466cecf"))]
    pub id: String,
}
