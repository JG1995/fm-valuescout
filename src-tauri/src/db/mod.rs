pub mod migrations;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::AppHandle;
#[cfg(not(all(debug_assertions, feature = "ui-agent")))]
use tauri::Manager;

/// SQLite filename under `app_data_dir`.
pub const APP_DB_FILE: &str = "app.db";

#[cfg(all(debug_assertions, feature = "ui-agent"))]
const UI_AGENT_DATA_DIR_ENV: &str = "FM_VALUESCOUT_UI_AGENT_DATA_DIR";

/// Shared rusqlite connection for IPC commands.
pub struct Db(pub Mutex<Connection>);

pub fn resolve_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(all(debug_assertions, feature = "ui-agent"))]
    {
        let _ = app;
        let data_dir = std::env::var_os(UI_AGENT_DATA_DIR_ENV)
            .ok_or_else(|| format!("{UI_AGENT_DATA_DIR_ENV} is required"))?;
        ui_agent_db_path(PathBuf::from(data_dir))
    }

    #[cfg(not(all(debug_assertions, feature = "ui-agent")))]
    {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;

        Ok(app_data_dir.join(APP_DB_FILE))
    }
}

#[cfg(all(debug_assertions, feature = "ui-agent"))]
fn ui_agent_db_path(data_dir: PathBuf) -> Result<PathBuf, String> {
    if !data_dir.is_absolute() {
        return Err(format!("{UI_AGENT_DATA_DIR_ENV} must be an absolute path"));
    }

    Ok(data_dir.join(APP_DB_FILE))
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

    #[cfg(feature = "ui-agent")]
    #[test]
    fn ui_agent_database_path_stays_under_the_supplied_directory() {
        let temp_dir = tempfile::tempdir().expect("temp dir");

        let db_path = ui_agent_db_path(temp_dir.path().to_path_buf()).expect("resolve path");

        assert_eq!(db_path, temp_dir.path().join(APP_DB_FILE));
        assert!(ui_agent_db_path(PathBuf::from("relative/profile")).is_err());
    }
}
