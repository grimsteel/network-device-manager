use std::{borrow::Cow, collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

use rspc_procedure::{Procedure, ProcedureError, ProcedureStream, ResolverError};
use rusqlite::OptionalExtension;
use tokio::io::AsyncWriteExt;

use crate::models::*;

use super::{internal_err, Ctx};

pub fn register(map: &mut HashMap<Cow<'static, str>, Procedure<Ctx>>) {
    map.insert("accessPoints.list".into(), list());
    map.insert("accessPoints.get".into(), get());
    map.insert("accessPoints.create".into(), create());
    map.insert("accessPoints.update".into(), update());
    map.insert("accessPoints.delete".into(), delete());
    map.insert("accessPoints.addInterface".into(), add_interface());
    map.insert("accessPoints.updateInterface".into(), update_interface());
    map.insert("accessPoints.removeInterface".into(), remove_interface());
    map.insert("accessPoints.syncInterface".into(), sync_interface());
}

/// Fetches an access point together with all its interfaces.
fn fetch_ap(
    conn: &rusqlite::Connection,
    id: i32,
) -> Result<Option<AccessPoint>, rusqlite::Error> {
    let row = conn
        .query_row(
            "SELECT id, name, host, port FROM access_points WHERE id = ?1",
            [id],
            |row| -> Result<(i32, String, String, i32), _> {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            },
        )
        .optional()?;

    let (apid, name, host, port) = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let interfaces = conn.prepare(
        "SELECT id, ap_id, iface_name, group_id, last_synced_at, needs_sync \
         FROM ap_interfaces WHERE ap_id = ?1 ORDER BY iface_name",
    )?
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

    Ok(Some(AccessPoint { id: apid, name, host, port, interfaces }))
}

/// Parse "AA:BB:CC:DD:EE:FF" into 6 raw bytes.
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
///   magic        2 bytes  0x4E 0x44  ("ND" – Network Device)
///   version      1 byte   0x01
///   msg_type     1 byte   0x01  (SetMacFilter)
///   payload_len  4 bytes  u32, length of everything that follows
///   iface_len    1 byte   length of iface_name UTF-8 string
///   iface_name   N bytes  UTF-8
///   mac_count    2 bytes  u16
///   mac[0..N]    6 bytes each
fn build_set_filter_message(iface_name: &str, macs: &[[u8; 6]]) -> Vec<u8> {
    let iface_bytes = iface_name.as_bytes();
    let payload_len = 1 + iface_bytes.len() + 2 + macs.len() * 6;
    let mut buf = Vec::with_capacity(8 + payload_len);
    buf.extend_from_slice(&[0x4E, 0x44, 0x01, 0x01]);
    buf.extend_from_slice(&(payload_len as u32).to_be_bytes());
    buf.push(iface_bytes.len() as u8);
    buf.extend_from_slice(iface_bytes);
    buf.extend_from_slice(&(macs.len() as u16).to_be_bytes());
    for mac in macs {
        buf.extend_from_slice(mac);
    }
    buf
}

fn not_found() -> ProcedureError {
    ProcedureError::Resolver(ResolverError::new(
        serde_json::json!({"code": 404, "message": "Access point not found"}),
        None::<std::io::Error>,
    ))
}

fn list() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, _input| {
        ProcedureStream::from_future(async move {
            ctx.call(|conn| -> Result<Vec<AccessPoint>, rusqlite::Error> {
                conn.prepare("SELECT id, name, host, port FROM access_points ORDER BY name")?
                .query_map([], |row| {
                    Ok(AccessPoint {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        host: row.get(2)?,
                        port: row.get(3)?,
                        interfaces: vec![],
                    })
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .await
            .map_err(internal_err)
        })
    })
}

fn get() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let id = match input.deserialize::<i32>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| fetch_ap(conn, id))
                .await
                .map_err(internal_err)?
                .ok_or_else(not_found)
        })
    })
}

/// Create: deserializes an AccessPoint; `id` and `interfaces` are ignored.
fn create() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<AccessPoint>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<AccessPoint, rusqlite::Error> {
                conn.execute(
                    "INSERT INTO access_points (name, host, port) VALUES (?1, ?2, ?3)",
                    rusqlite::params![&args.name, &args.host, args.port],
                )?;
                let id = conn.last_insert_rowid() as i32;
                Ok(AccessPoint { id, name: args.name, host: args.host, port: args.port, interfaces: vec![] })
            })
            .await
            .map_err(internal_err)
        })
    })
}

/// Update: uses the `id` field; `interfaces` is ignored.
fn update() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<AccessPoint>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<AccessPoint, rusqlite::Error> {
                conn.execute(
                    "UPDATE access_points SET name=?1, host=?2, port=?3 WHERE id=?4",
                    rusqlite::params![&args.name, &args.host, args.port, args.id],
                )?;
                Ok(AccessPoint { id: args.id, name: args.name, host: args.host, port: args.port, interfaces: vec![] })
            })
            .await
            .map_err(internal_err)
        })
    })
}

fn delete() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let id = match input.deserialize::<i32>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<(), rusqlite::Error> {
                conn.execute("DELETE FROM access_points WHERE id=?1", [id])?;
                Ok(())
            })
            .await
            .map_err(internal_err)
        })
    })
}

/// Add interface: deserializes an Interface; `id` is ignored, `ap_id` is used.
fn add_interface() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<Interface>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<Interface, rusqlite::Error> {
                let needs_sync = if args.group_id.is_some() { 1i32 } else { 0 };
                conn.execute(
                    "INSERT INTO ap_interfaces (ap_id, iface_name, group_id, needs_sync) \
                     VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![args.ap_id, &args.iface_name, args.group_id, needs_sync],
                )?;
                let id = conn.last_insert_rowid() as i32;
                Ok(Interface {
                    id,
                    ap_id: args.ap_id,
                    iface_name: args.iface_name,
                    group_id: args.group_id,
                    last_synced_at: None,
                    needs_sync: needs_sync != 0,
                })
            })
            .await
            .map_err(internal_err)
        })
    })
}

/// Update interface: uses the `id` field.
fn update_interface() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<Interface>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<Interface, rusqlite::Error> {
                conn.execute(
                    "UPDATE ap_interfaces SET iface_name=?1, group_id=?2, needs_sync=1 WHERE id=?3",
                    rusqlite::params![&args.iface_name, args.group_id, args.id],
                )?;
                conn.query_row(
                    "SELECT id, ap_id, iface_name, group_id, last_synced_at, needs_sync \
                     FROM ap_interfaces WHERE id=?1",
                    [args.id],
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
            .map_err(internal_err)
        })
    })
}

fn remove_interface() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let id = match input.deserialize::<i32>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<(), rusqlite::Error> {
                conn.execute("DELETE FROM ap_interfaces WHERE id=?1", [id])?;
                Ok(())
            })
            .await
            .map_err(internal_err)
        })
    })
}

fn sync_interface() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let interface_id = match input.deserialize::<i32>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            // Fetch interface + AP info
            let (iface_name, group_id, host, port) = ctx
                .call(move |conn| -> Result<_, rusqlite::Error> {
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
                .map_err(internal_err)?;

            // Resolve MAC addresses for the assigned group
            let macs: Vec<[u8; 6]> = if let Some(gid) = group_id {
                let raw_macs = ctx
                    .call(move |conn| -> Result<Vec<String>, rusqlite::Error> {
                        conn.prepare(
                            "SELECT d.mac_address FROM devices d \
                             JOIN group_devices gd ON gd.device_id = d.id \
                             WHERE gd.group_id = ?1",
                        )?
                        .query_map([gid], |row| row.get::<_, String>(0))?
                        .collect::<Result<Vec<_>, _>>()
                    })
                    .await
                    .map_err(internal_err)?;

                raw_macs.iter().filter_map(|m| parse_mac(m)).collect()
            } else {
                vec![]
            };

            let mac_count = macs.len() as i32;
            let message = build_set_filter_message(&iface_name, &macs);
            let addr = format!("{}:{}", host, port);

            match tokio::net::TcpStream::connect(&addr).await {
                Ok(mut stream) => match stream.write_all(&message).await {
                    Ok(_) => {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i32;
                        ctx.call(move |conn| -> Result<(), rusqlite::Error> {
                            conn.execute(
                                "UPDATE ap_interfaces SET needs_sync=0, last_synced_at=?1 \
                                 WHERE id=?2",
                                rusqlite::params![now, interface_id],
                            )?;
                            Ok(())
                        })
                        .await
                        .map_err(internal_err)?;

                        Ok::<_, ProcedureError>(SyncResult {
                            interface_id,
                            success: true,
                            message: format!("Synced {} MACs to {}", mac_count, addr),
                            mac_count,
                        })
                    }
                    Err(e) => Ok(SyncResult {
                        interface_id,
                        success: false,
                        message: format!("Failed to send to {}: {}", addr, e),
                        mac_count,
                    }),
                },
                Err(e) => Ok(SyncResult {
                    interface_id,
                    success: false,
                    message: format!("Failed to connect to {}: {}", addr, e),
                    mac_count,
                }),
            }
        })
    })
}
