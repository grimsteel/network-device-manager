use serde::{Deserialize, Serialize};

use super::Device;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub devices: Vec<Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDeviceInput {
    pub group_id: i32,
    pub device_id: i32,
}
