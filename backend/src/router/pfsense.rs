use serde::{Deserialize, Serialize};
use specta::Type;

use rspc::Router;

use super::{Ctx, R};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PfsenseConfig {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DhcpLease {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
    pub description: String,
}

pub fn router() -> Router<Ctx> {
    R.router().procedure(
        "listDhcpLeases",
        R.query(|_ctx, cfg: PfsenseConfig| async move {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true) // pfSense often uses self-signed certs
                .build()
                .map_err(|e| {
                    rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string())
                })?;

            let url = format!("{}/api/v1/services/dhcpd/lease", cfg.base_url.trim_end_matches('/'));

            let resp = client
                .get(&url)
                .header("Authorization", &cfg.api_key)
                .send()
                .await
                .map_err(|e| {
                    rspc::Error::new(
                        rspc::ErrorCode::InternalServerError,
                        format!("pfSense API error: {}", e),
                    )
                })?;

            if !resp.status().is_success() {
                return Err(rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("pfSense returned HTTP {}", resp.status()),
                ));
            }

            let body: serde_json::Value = resp.json().await.map_err(|e| {
                rspc::Error::new(
                    rspc::ErrorCode::InternalServerError,
                    format!("Failed to parse pfSense response: {}", e),
                )
            })?;

            let leases = body["data"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|entry| {
                    let mac = entry["mac"].as_str()?.to_string();
                    let ip = entry["ip"].as_str().unwrap_or("").to_string();
                    let hostname = entry["hostname"].as_str().unwrap_or("").to_string();
                    let description = entry["descr"].as_str().unwrap_or("").to_string();
                    Some(DhcpLease { mac, ip, hostname, description })
                })
                .collect::<Vec<_>>();

            Ok(leases)
        }),
    )
}
