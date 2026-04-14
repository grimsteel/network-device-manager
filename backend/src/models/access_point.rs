use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPoint {
    pub id: i32,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub interfaces: Vec<Interface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub id: i32,
    pub ap_id: i32,
    pub iface_name: String,
    pub group_id: Option<i32>,
    pub last_synced_at: Option<i32>,
    pub needs_sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub interface_id: i32,
    pub success: bool,
    pub message: String,
    pub mac_count: i32,
}
