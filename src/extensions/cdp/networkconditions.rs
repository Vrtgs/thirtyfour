use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "lowercase")]
pub enum ConnectionType {
    None,
    Cellular2G,
    Cellular3G,
    Cellular4G,
    Bluetooth,
    Ethernet,
    Wifi,
    Wimax,
    Other,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct NetworkConditions {
    pub offline: bool,
    pub latency: u32,
    pub download_throughput: i32,
    pub upload_throughput: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<ConnectionType>,
}

impl Default for NetworkConditions {
    fn default() -> Self {
        Self {
            offline: false,
            latency: 0,
            download_throughput: -1,
            upload_throughput: -1,
            connection_type: None,
        }
    }
}

impl NetworkConditions {
    pub fn new() -> Self {
        Self::default()
    }
}
