use std::{borrow::Cow, collections::HashMap};

use rspc_procedure::{Procedure, ProcedureStream};
use serde::{Deserialize, Serialize};

use super::{internal_err, Ctx};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PfsenseConfig {
    pub base_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpLease {
    pub mac: String,
    pub ip: String,
    pub hostname: String,
    pub description: String,
}

pub fn register(map: &mut HashMap<Cow<'static, str>, Procedure<Ctx>>) {
    map.insert("pfsense.listDhcpLeases".into(), list_dhcp_leases());
}

fn list_dhcp_leases() -> Procedure<Ctx> {
    Procedure::new(|_ctx, input| {
        let cfg = match input.deserialize::<PfsenseConfig>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()
                .map_err(internal_err)?;

            let url = format!(
                "{}/api/v1/services/dhcpd/lease",
                cfg.base_url.trim_end_matches('/')
            );

            let resp = client
                .get(&url)
                .header("Authorization", &cfg.api_key)
                .send()
                .await
                .map_err(|e| {
                    internal_err(format!("pfSense request failed: {}", e))
                })?;

            if !resp.status().is_success() {
                return Err(internal_err(format!(
                    "pfSense returned HTTP {}",
                    resp.status()
                )));
            }

            let body: serde_json::Value = resp.json().await.map_err(|e| {
                internal_err(format!("Failed to parse pfSense response: {}", e))
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
        })
    })
}
