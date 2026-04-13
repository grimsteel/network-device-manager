use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub network: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDevice {
    pub name: String,
    pub description: String,
    pub network: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDevice {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub network: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
}
