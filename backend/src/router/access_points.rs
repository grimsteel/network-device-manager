use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::OptionalExtension;
use rspc::Router;
use tokio::io::AsyncWriteExt;

use crate::models::*;

use super::{Ctx, R};

fn db_err(e: impl std::fmt::Display) -> rspc::Error {
    rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string())
}

fn fetch_ap_with_interfaces(
    conn: &rusqlite::Connection,
    id: i32,
) -> Result<AccessPointWithInterfaces, rusqlite::Error> {
    let ap = conn.query_row(
        "SELECT id, name, host, port FROM access_points WHERE id = ?1",
        [id],
        |row| {
            Ok(AccessPoint {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get(3)?,
            })
        },
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, ap_id, iface_name, group_id, last_synced_at, needs_sync \
         FROM ap_interfaces WHERE ap_id = ?1 ORDER BY iface_name",
    )?;
    let interfaces = stmt
        .query_map([id], |row| {
            let needs_sync: i32 = row.get(5)?;
            Ok(Interface {
                id: row.get(0)?,
                ap_id: row.get(1)?,
                iface_name: row.get(2)?,
                group_id: row.get(3)?,
                last_synced_at: row.get(4)?,
                needs_sync: needs_sync != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AccessPointWithInterfaces {
        id: ap.id,
        name: ap.name,
        host: ap.host,
        port: ap.port,
        interfaces,
    })
}

/// Parse a colon-separated MAC address string into 6 bytes.
fn parse_mac(mac: &str) -> Option<[u8; 6]> {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() != 6 {
        return None;
    }
    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        bytes[i] = u8::from_str_radix(part, 16).ok()?;
    }
    Some(bytes)
}

/// Build the binary `SetMacFilter` message for the AP daemon.
///
/// Message layout (big-endian):
///   magic        2 bytes  0x4E 0x44 ("ND")
///   version      1 byte   0x01
///   msg_type     1 byte   0x01  (SetMacFilter)
///   payload_len  4 bytes  (u32) length of the payload below
///   --- payload ---
///   iface_len    1 byte   length of iface_name string
///   iface_name   N bytes  UTF-8 interface name
///   mac_count    2 bytes  (u16) number of MAC addresses
///   mac[0..N]    6 bytes each  raw MAC octets
fn build_set_filter_message(iface_name: &str, macs: &[[u8; 6]]) -> Vec<u8> {
    let iface_bytes = iface_name.as_bytes();
    let payload_len = 1 + iface_bytes.len() + 2 + macs.len() * 6;

    let mut buf = Vec::with_capacity(8 + payload_len);
    buf.extend_from_slice(&[0x4E, 0x44, 0x01, 0x01]); // magic + version + msg_type
    buf.extend_from_slice(&(payload_len as u32).to_be_bytes()); // payload_len
    buf.push(iface_bytes.len() as u8); // iface_len
    buf.extend_from_slice(iface_bytes); // iface_name
    buf.extend_from_slice(&(macs.len() as u16).to_be_bytes()); // mac_count
    for mac in macs {
        buf.extend_from_slice(mac); // raw MAC bytes
    }
    buf
}

pub fn router() -> Router<Ctx> {
    R.router()
        .procedure(
            "list",
            R.query(|ctx, _: ()| async move {
                ctx.db
                    .call(|conn| -> Result<Vec<AccessPoint>, rusqlite::Error> {
                        let mut stmt = conn.prepare(
                            "SELECT id, name, host, port FROM access_points ORDER BY name",
                        )?;
                        let aps = stmt
                            .query_map([], |row| {
                                Ok(AccessPoint {
                                    id: row.get(0)?,
                                    name: row.get(1)?,
                                    host: row.get(2)?,
                                    port: row.get(3)?,
                                })
                            })?
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(aps)
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "get",
            R.query(|ctx, id: i32| async move {
                ctx.db
                    .call(move |conn| {
                        match fetch_ap_with_interfaces(conn, id) {
                            Ok(ap) => Ok(Some(ap)),
                            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                            Err(e) => Err(e),
                        }
                    })
                    .await
                    .map_err(db_err)
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            rspc::Error::new(
                                rspc::ErrorCode::NotFound,
                                "Access point not found".into(),
                            )
                        })
                    })
            }),
        )
        .procedure(
            "create",
            R.mutation(|ctx, input: CreateAccessPoint| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "INSERT INTO access_points (name, host, port) VALUES (?1, ?2, ?3)",
                            rusqlite::params![&input.name, &input.host, input.port],
                        )?;
                        let id = conn.last_insert_rowid() as i32;
                        Ok(AccessPoint {
                            id,
                            name: input.name,
                            host: input.host,
                            port: input.port,
                        })
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "update",
            R.mutation(|ctx, input: UpdateAccessPoint| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "UPDATE access_points SET name=?1, host=?2, port=?3 WHERE id=?4",
                            rusqlite::params![&input.name, &input.host, input.port, input.id],
                        )?;
                        Ok(AccessPoint {
                            id: input.id,
                            name: input.name,
                            host: input.host,
                            port: input.port,
                        })
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "delete",
            R.mutation(|ctx, id: i32| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute("DELETE FROM access_points WHERE id=?1", [id])?;
                        Ok(())
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "addInterface",
            R.mutation(|ctx, input: CreateInterface| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "INSERT INTO ap_interfaces (ap_id, iface_name, group_id) \
                             VALUES (?1, ?2, ?3)",
                            rusqlite::params![input.ap_id, &input.iface_name, input.group_id],
                        )?;
                        let id = conn.last_insert_rowid() as i32;
                        Ok(Interface {
                            id,
                            ap_id: input.ap_id,
                            iface_name: input.iface_name,
                            group_id: input.group_id,
                            last_synced_at: None,
                            needs_sync: input.group_id.is_some(),
                        })
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "updateInterface",
            R.mutation(|ctx, input: UpdateInterface| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "UPDATE ap_interfaces SET iface_name=?1, group_id=?2, needs_sync=1 \
                             WHERE id=?3",
                            rusqlite::params![&input.iface_name, input.group_id, input.id],
                        )?;
                        conn.query_row(
                            "SELECT id, ap_id, iface_name, group_id, last_synced_at, needs_sync \
                             FROM ap_interfaces WHERE id=?1",
                            [input.id],
                            |row| {
                                let needs_sync: i32 = row.get(5)?;
                                Ok(Interface {
                                    id: row.get(0)?,
                                    ap_id: row.get(1)?,
                                    iface_name: row.get(2)?,
                                    group_id: row.get(3)?,
                                    last_synced_at: row.get(4)?,
                                    needs_sync: needs_sync != 0,
                                })
                            },
                        )
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "removeInterface",
            R.mutation(|ctx, id: i32| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute("DELETE FROM ap_interfaces WHERE id=?1", [id])?;
                        Ok(())
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "syncInterface",
            R.mutation(|ctx, interface_id: i32| async move {
                // Fetch interface info and AP host/port
                let (iface_name, group_id, host, port) = ctx
                    .db
                    .call(move |conn| {
                        conn.query_row(
                            "SELECT i.iface_name, i.group_id, a.host, a.port \
                             FROM ap_interfaces i \
                             JOIN access_points a ON a.id = i.ap_id \
                             WHERE i.id = ?1",
                            [interface_id],
                            |row| {
                                Ok((
                                    row.get::<_, String>(0)?,
                                    row.get::<_, Option<i32>>(1)?,
                                    row.get::<_, String>(2)?,
                                    row.get::<_, i32>(3)?,
                                ))
                            },
                        )
                    })
                    .await
                    .map_err(db_err)?;

                // Resolve MAC addresses for the group
                let macs: Vec<[u8; 6]> = if let Some(gid) = group_id {
                    let raw_macs = ctx
                        .db
                        .call(move |conn| {
                            let mut stmt = conn.prepare(
                                "SELECT d.mac_address FROM devices d \
                                 JOIN group_devices gd ON gd.device_id = d.id \
                                 WHERE gd.group_id = ?1",
                            )?;
                            stmt.query_map([gid], |row| row.get::<_, String>(0))?
                                .collect::<Result<Vec<_>, _>>()
                        })
                        .await
                        .map_err(db_err)?;

                    raw_macs
                        .iter()
                        .filter_map(|m| parse_mac(m))
                        .collect()
                } else {
                    vec![]
                };

                let mac_count = macs.len() as i32;
                let message = build_set_filter_message(&iface_name, &macs);

                // Connect to AP daemon and send the binary message
                let addr = format!("{}:{}", host, port);
                let sync_result = match tokio::net::TcpStream::connect(&addr).await {
                    Ok(mut stream) => {
                        match stream.write_all(&message).await {
                            Ok(_) => {
                                // Update sync status in DB
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i32;
                                ctx.db
                                    .call(move |conn| {
                                        conn.execute(
                                            "UPDATE ap_interfaces SET needs_sync=0, last_synced_at=?1 WHERE id=?2",
                                            rusqlite::params![now, interface_id],
                                        )?;
                                        Ok(())
                                    })
                                    .await
                                    .map_err(db_err)?;
                                SyncResult {
                                    interface_id,
                                    success: true,
                                    message: format!(
                                        "Synced {} MAC addresses to {}",
                                        mac_count, addr
                                    ),
                                    mac_count,
                                }
                            }
                            Err(e) => SyncResult {
                                interface_id,
                                success: false,
                                message: format!("Failed to send to daemon at {}: {}", addr, e),
                                mac_count,
                            },
                        }
                    }
                    Err(e) => SyncResult {
                        interface_id,
                        success: false,
                        message: format!("Failed to connect to daemon at {}: {}", addr, e),
                        mac_count,
                    },
                };

                Ok(sync_result)
            }),
        )
}
