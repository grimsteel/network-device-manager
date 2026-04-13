use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub network: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateDevice {
    pub name: String,
    pub description: String,
    pub network: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateDevice {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub network: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
}
