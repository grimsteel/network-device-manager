use tokio_rusqlite::Connection;

const MIGRATIONS: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS devices (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT    NOT NULL DEFAULT '',
    network     TEXT    NOT NULL DEFAULT '',
    mac_address TEXT    NOT NULL UNIQUE,
    ip_address  TEXT
);

CREATE TABLE IF NOT EXISTS groups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS group_devices (
    group_id  INTEGER NOT NULL REFERENCES groups(id)  ON DELETE CASCADE,
    device_id INTEGER NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, device_id)
);

CREATE TABLE IF NOT EXISTS access_points (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT    NOT NULL,
    host TEXT    NOT NULL,
    port INTEGER NOT NULL DEFAULT 8765
);

CREATE TABLE IF NOT EXISTS ap_interfaces (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    ap_id          INTEGER NOT NULL REFERENCES access_points(id) ON DELETE CASCADE,
    iface_name     TEXT    NOT NULL,
    group_id       INTEGER REFERENCES groups(id) ON DELETE SET NULL,
    last_synced_at INTEGER,
    needs_sync     INTEGER NOT NULL DEFAULT 0,
    UNIQUE (ap_id, iface_name)
);
"#;

pub async fn open(path: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path).await?;
    conn.call(|conn| -> Result<(), rusqlite::Error> {
        conn.execute_batch(MIGRATIONS)?;
        Ok(())
    })
    .await?;
    Ok(conn)
}
