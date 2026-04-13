use serde::{Deserialize, Serialize};
use specta::Type;

use super::Device;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GroupWithDevices {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub devices: Vec<Device>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateGroup {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateGroup {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GroupDeviceInput {
    pub group_id: i32,
    pub device_id: i32,
}
