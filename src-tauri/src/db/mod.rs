pub mod migrations;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

/// SQLite filename under `app_data_dir`.
///
/// The compact-metric generation uses a fresh file so the legacy `app.db` is
/// never opened, migrated, or converted.
pub const APP_DB_FILE: &str = "app-v2.db";

/// Shared rusqlite connection for IPC commands.
pub struct Db(pub Mutex<Connection>);

pub fn resolve_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;

    Ok(app_data_dir.join(APP_DB_FILE))
}

pub fn open(db_path: &Path) -> Result<Db, String> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let conn = Connection::open(db_path).map_err(|error| error.to_string())?;
    conn.pragma_update(None, "foreign_keys", true)
        .map_err(|error| error.to_string())?;
    migrations::apply(&conn).map_err(|error| error.to_string())?;

    Ok(Db(Mutex::new(conn)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_enables_foreign_key_enforcement() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = open(&temp_dir.path().join("foreign-keys-test.db")).expect("open db");
        let conn = db.0.into_inner().expect("unlock db");

        let foreign_keys: i32 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("read foreign_keys");
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn app_database_filename_targets_only_the_fresh_v2_generation() {
        assert_eq!(APP_DB_FILE, "app-v2.db");
        assert_ne!(APP_DB_FILE, "app.db");
    }

    #[test]
    fn opening_the_v2_database_applies_the_compact_metrics_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = open(&temp_dir.path().join(APP_DB_FILE)).expect("open fresh v2 database");
        let conn = db.0.into_inner().expect("unlock db");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read schema version");
        assert_eq!(version, migrations::latest_version());

        let compact_tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table'
                   AND name IN ('player_role_metrics', 'staff_role_metrics')",
                [],
                |row| row.get(0),
            )
            .expect("count compact metric tables");
        assert_eq!(compact_tables, 2);
    }
}
