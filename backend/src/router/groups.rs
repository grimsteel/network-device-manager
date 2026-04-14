use std::{borrow::Cow, collections::HashMap};

use rspc_procedure::{Procedure, ProcedureError, ProcedureStream, ResolverError};
use rusqlite::OptionalExtension;

use crate::models::*;

use super::{internal_err, Ctx};

pub fn register(map: &mut HashMap<Cow<'static, str>, Procedure<Ctx>>) {
    map.insert("groups.list".into(), list());
    map.insert("groups.get".into(), get());
    map.insert("groups.create".into(), create());
    map.insert("groups.update".into(), update());
    map.insert("groups.delete".into(), delete());
    map.insert("groups.addDevice".into(), add_device());
    map.insert("groups.removeDevice".into(), remove_device());
}

/// Fetches a group along with all its member devices.
fn fetch_group(
    conn: &rusqlite::Connection,
    id: i32,
) -> Result<Option<Group>, rusqlite::Error> {
    let row = conn
        .query_row(
            "SELECT id, name, description FROM groups WHERE id = ?1",
            [id],
            |row| -> Result<(i32, String, String), _> {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            },
        )
        .optional()?;

    let (gid, name, description) = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let devices = conn.prepare(
        "SELECT d.id, d.name, d.description, d.network, d.mac_address, d.ip_address \
         FROM devices d \
         JOIN group_devices gd ON gd.device_id = d.id \
         WHERE gd.group_id = ?1 \
         ORDER BY d.name",
    )?
    .query_map([id], |row| {
        Ok(Device {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            network: row.get(3)?,
            mac_address: row.get(4)?,
            ip_address: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(Group { id: gid, name, description, devices }))
}

fn not_found() -> ProcedureError {
    ProcedureError::Resolver(ResolverError::new(
        serde_json::json!({"code": 404, "message": "Group not found"}),
        None::<std::io::Error>,
    ))
}

fn list() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, _input| {
        ProcedureStream::from_future(async move {
            ctx.call(|conn| -> Result<Vec<Group>, rusqlite::Error> {
                conn.prepare("SELECT id, name, description FROM groups ORDER BY name")?
                .query_map([], |row| {
                    Ok(Group {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        devices: vec![],
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
            ctx.call(move |conn| fetch_group(conn, id))
                .await
                .map_err(internal_err)?
                .ok_or_else(not_found)
        })
    })
}

/// Create: deserializes a Group; `id` and `devices` are ignored.
fn create() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<Group>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<Group, rusqlite::Error> {
                conn.execute(
                    "INSERT INTO groups (name, description) VALUES (?1, ?2)",
                    rusqlite::params![&args.name, &args.description],
                )?;
                let id = conn.last_insert_rowid() as i32;
                Ok(Group { id, name: args.name, description: args.description, devices: vec![] })
            })
            .await
            .map_err(internal_err)
        })
    })
}

/// Update: uses the `id` field; `devices` is ignored.
fn update() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<Group>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<Group, rusqlite::Error> {
                conn.execute(
                    "UPDATE groups SET name=?1, description=?2 WHERE id=?3",
                    rusqlite::params![&args.name, &args.description, args.id],
                )?;
                Ok(Group { id: args.id, name: args.name, description: args.description, devices: vec![] })
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
                conn.execute("DELETE FROM groups WHERE id=?1", [id])?;
                Ok(())
            })
            .await
            .map_err(internal_err)
        })
    })
}

fn add_device() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<GroupDeviceInput>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| {
                conn.execute(
                    "INSERT OR IGNORE INTO group_devices (group_id, device_id) VALUES (?1, ?2)",
                    [args.group_id, args.device_id],
                )?;
                conn.execute(
                    "UPDATE ap_interfaces SET needs_sync=1 WHERE group_id=?1",
                    [args.group_id],
                )?;
                fetch_group(conn, args.group_id)
            })
            .await
            .map_err(internal_err)?
            .ok_or_else(not_found)
        })
    })
}

fn remove_device() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<GroupDeviceInput>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| {
                conn.execute(
                    "DELETE FROM group_devices WHERE group_id=?1 AND device_id=?2",
                    [args.group_id, args.device_id],
                )?;
                conn.execute(
                    "UPDATE ap_interfaces SET needs_sync=1 WHERE group_id=?1",
                    [args.group_id],
                )?;
                fetch_group(conn, args.group_id)
            })
            .await
            .map_err(internal_err)?
            .ok_or_else(not_found)
        })
    })
}
