use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AccessPoint {
    pub id: i32,
    pub name: String,
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Interface {
    pub id: i32,
    pub ap_id: i32,
    pub iface_name: String,
    pub group_id: Option<i32>,
    pub last_synced_at: Option<i32>,
    pub needs_sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AccessPointWithInterfaces {
    pub id: i32,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub interfaces: Vec<Interface>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateAccessPoint {
    pub name: String,
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateAccessPoint {
    pub id: i32,
    pub name: String,
    pub host: String,
    pub port: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CreateInterface {
    pub ap_id: i32,
    pub iface_name: String,
    pub group_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateInterface {
    pub id: i32,
    pub iface_name: String,
    pub group_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SyncResult {
    pub interface_id: i32,
    pub success: bool,
    pub message: String,
    pub mac_count: i32,
}
