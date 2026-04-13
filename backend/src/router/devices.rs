use std::{borrow::Cow, collections::HashMap};

use rspc_procedure::{Procedure, ProcedureError, ProcedureStream};
use rusqlite::OptionalExtension;

use crate::models::*;

use super::{internal_err, Ctx};

pub fn register(map: &mut HashMap<Cow<'static, str>, Procedure<Ctx>>) {
    map.insert("devices.list".into(), list());
    map.insert("devices.get".into(), get());
    map.insert("devices.create".into(), create());
    map.insert("devices.update".into(), update());
    map.insert("devices.delete".into(), delete());
}

fn list() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, _input| {
        ProcedureStream::from_future(async move {
            ctx.call(|conn| -> Result<Vec<Device>, rusqlite::Error> {
                conn.prepare(
                    "SELECT id, name, description, network, mac_address, ip_address \
                     FROM devices ORDER BY name",
                )?
                .query_map([], |row| {
                    Ok(Device {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        network: row.get(3)?,
                        mac_address: row.get(4)?,
                        ip_address: row.get(5)?,
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
            ctx.call(move |conn| -> Result<Option<Device>, rusqlite::Error> {
                conn.query_row(
                    "SELECT id, name, description, network, mac_address, ip_address \
                     FROM devices WHERE id = ?1",
                    [id],
                    |row| {
                        Ok(Device {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            description: row.get(2)?,
                            network: row.get(3)?,
                            mac_address: row.get(4)?,
                            ip_address: row.get(5)?,
                        })
                    },
                )
                .optional()
            })
            .await
            .map_err(internal_err)?
            .ok_or_else(|| {
                ProcedureError::Resolver(rspc_procedure::ResolverError::new(
                    serde_json::json!({"code": 404, "message": "Device not found"}),
                    None::<std::io::Error>,
                ))
            })
        })
    })
}

fn create() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<CreateDevice>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<Device, rusqlite::Error> {
                conn.execute(
                    "INSERT INTO devices (name, description, network, mac_address, ip_address) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        &args.name,
                        &args.description,
                        &args.network,
                        &args.mac_address,
                        &args.ip_address
                    ],
                )?;
                let id = conn.last_insert_rowid() as i32;
                Ok(Device {
                    id,
                    name: args.name,
                    description: args.description,
                    network: args.network,
                    mac_address: args.mac_address,
                    ip_address: args.ip_address,
                })
            })
            .await
            .map_err(internal_err)
        })
    })
}

fn update() -> Procedure<Ctx> {
    Procedure::new(|ctx: Ctx, input| {
        let args = match input.deserialize::<UpdateDevice>() {
            Ok(v) => v,
            Err(e) => return ProcedureStream::from(e),
        };
        ProcedureStream::from_future(async move {
            ctx.call(move |conn| -> Result<Device, rusqlite::Error> {
                conn.execute(
                    "UPDATE devices SET name=?1, description=?2, network=?3, \
                     mac_address=?4, ip_address=?5 WHERE id=?6",
                    rusqlite::params![
                        &args.name,
                        &args.description,
                        &args.network,
                        &args.mac_address,
                        &args.ip_address,
                        args.id
                    ],
                )?;
                Ok(Device {
                    id: args.id,
                    name: args.name,
                    description: args.description,
                    network: args.network,
                    mac_address: args.mac_address,
                    ip_address: args.ip_address,
                })
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
                conn.execute("DELETE FROM devices WHERE id=?1", [id])?;
                Ok(())
            })
            .await
            .map_err(internal_err)
        })
    })
}
