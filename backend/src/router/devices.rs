use rusqlite::OptionalExtension;
use rspc::Router;

use crate::models::*;

use super::{Ctx, R};

fn db_err(e: impl std::fmt::Display) -> rspc::Error {
    rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string())
}

pub fn router() -> Router<Ctx> {
    R.router()
        .procedure(
            "list",
            R.query(|ctx, _: ()| async move {
                ctx.db
                    .call(|conn| -> Result<Vec<Device>, rusqlite::Error> {
                        let mut stmt = conn.prepare(
                            "SELECT id, name, description, network, mac_address, ip_address \
                             FROM devices ORDER BY name",
                        )?;
                        let rows = stmt
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
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(rows)
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
                    .map_err(db_err)
                    .and_then(|opt: Option<Device>| {
                        opt.ok_or_else(|| {
                            rspc::Error::new(rspc::ErrorCode::NotFound, "Device not found".into())
                        })
                    })
            }),
        )
        .procedure(
            "create",
            R.mutation(|ctx, input: CreateDevice| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "INSERT INTO devices (name, description, network, mac_address, ip_address) \
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                            rusqlite::params![
                                &input.name,
                                &input.description,
                                &input.network,
                                &input.mac_address,
                                &input.ip_address
                            ],
                        )?;
                        let id = conn.last_insert_rowid() as i32;
                        Ok(Device {
                            id,
                            name: input.name,
                            description: input.description,
                            network: input.network,
                            mac_address: input.mac_address,
                            ip_address: input.ip_address,
                        })
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "update",
            R.mutation(|ctx, input: UpdateDevice| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "UPDATE devices SET name=?1, description=?2, network=?3, \
                             mac_address=?4, ip_address=?5 WHERE id=?6",
                            rusqlite::params![
                                &input.name,
                                &input.description,
                                &input.network,
                                &input.mac_address,
                                &input.ip_address,
                                input.id
                            ],
                        )?;
                        Ok(Device {
                            id: input.id,
                            name: input.name,
                            description: input.description,
                            network: input.network,
                            mac_address: input.mac_address,
                            ip_address: input.ip_address,
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
                        conn.execute("DELETE FROM devices WHERE id=?1", [id])?;
                        Ok(())
                    })
                    .await
                    .map_err(db_err)
            }),
        )
}
