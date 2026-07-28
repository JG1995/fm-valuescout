pub mod migrations;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

/// SQLite filename under `app_data_dir`.
pub const APP_DB_FILE: &str = "app.db";

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
    migrations::apply(&conn).map_err(|error| error.to_string())?;

    Ok(Db(Mutex::new(conn)))
}
