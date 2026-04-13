use rusqlite::OptionalExtension;
use rspc::Router;

use crate::models::*;

use super::{Ctx, R};

fn db_err(e: impl std::fmt::Display) -> rspc::Error {
    rspc::Error::new(rspc::ErrorCode::InternalServerError, e.to_string())
}

fn fetch_group_with_devices(
    conn: &rusqlite::Connection,
    id: i32,
) -> Result<Option<GroupWithDevices>, rusqlite::Error> {
    let group = conn
        .query_row(
            "SELECT id, name, description FROM groups WHERE id = ?1",
            [id],
            |row| {
                Ok(Group {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                })
            },
        )
        .optional()?;

    let group = match group {
        Some(g) => g,
        None => return Ok(None),
    };

    let mut stmt = conn.prepare(
        "SELECT d.id, d.name, d.description, d.network, d.mac_address, d.ip_address \
         FROM devices d \
         JOIN group_devices gd ON gd.device_id = d.id \
         WHERE gd.group_id = ?1 \
         ORDER BY d.name",
    )?;
    let devices = stmt
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

    Ok(Some(GroupWithDevices {
        id: group.id,
        name: group.name,
        description: group.description,
        devices,
    }))
}

pub fn router() -> Router<Ctx> {
    R.router()
        .procedure(
            "list",
            R.query(|ctx, _: ()| async move {
                ctx.db
                    .call(|conn| -> Result<Vec<Group>, rusqlite::Error> {
                        let mut stmt = conn
                            .prepare("SELECT id, name, description FROM groups ORDER BY name")?;
                        let groups = stmt
                            .query_map([], |row| {
                                Ok(Group {
                                    id: row.get(0)?,
                                    name: row.get(1)?,
                                    description: row.get(2)?,
                                })
                            })?
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(groups)
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "get",
            R.query(|ctx, id: i32| async move {
                ctx.db
                    .call(move |conn| fetch_group_with_devices(conn, id))
                    .await
                    .map_err(db_err)
                    .and_then(|opt: Option<GroupWithDevices>| {
                        opt.ok_or_else(|| {
                            rspc::Error::new(rspc::ErrorCode::NotFound, "Group not found".into())
                        })
                    })
            }),
        )
        .procedure(
            "create",
            R.mutation(|ctx, input: CreateGroup| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "INSERT INTO groups (name, description) VALUES (?1, ?2)",
                            rusqlite::params![&input.name, &input.description],
                        )?;
                        let id = conn.last_insert_rowid() as i32;
                        Ok(Group {
                            id,
                            name: input.name,
                            description: input.description,
                        })
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "update",
            R.mutation(|ctx, input: UpdateGroup| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "UPDATE groups SET name=?1, description=?2 WHERE id=?3",
                            rusqlite::params![&input.name, &input.description, input.id],
                        )?;
                        Ok(Group {
                            id: input.id,
                            name: input.name,
                            description: input.description,
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
                        conn.execute("DELETE FROM groups WHERE id=?1", [id])?;
                        Ok(())
                    })
                    .await
                    .map_err(db_err)
            }),
        )
        .procedure(
            "addDevice",
            R.mutation(|ctx, input: GroupDeviceInput| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "INSERT OR IGNORE INTO group_devices (group_id, device_id) \
                             VALUES (?1, ?2)",
                            [input.group_id, input.device_id],
                        )?;
                        conn.execute(
                            "UPDATE ap_interfaces SET needs_sync=1 WHERE group_id=?1",
                            [input.group_id],
                        )?;
                        fetch_group_with_devices(conn, input.group_id)
                    })
                    .await
                    .map_err(db_err)
                    .and_then(|opt: Option<GroupWithDevices>| {
                        opt.ok_or_else(|| {
                            rspc::Error::new(rspc::ErrorCode::NotFound, "Group not found".into())
                        })
                    })
            }),
        )
        .procedure(
            "removeDevice",
            R.mutation(|ctx, input: GroupDeviceInput| async move {
                ctx.db
                    .call(move |conn| {
                        conn.execute(
                            "DELETE FROM group_devices WHERE group_id=?1 AND device_id=?2",
                            [input.group_id, input.device_id],
                        )?;
                        conn.execute(
                            "UPDATE ap_interfaces SET needs_sync=1 WHERE group_id=?1",
                            [input.group_id],
                        )?;
                        fetch_group_with_devices(conn, input.group_id)
                    })
                    .await
                    .map_err(db_err)
                    .and_then(|opt: Option<GroupWithDevices>| {
                        opt.ok_or_else(|| {
                            rspc::Error::new(rspc::ErrorCode::NotFound, "Group not found".into())
                        })
                    })
            }),
        )
}
