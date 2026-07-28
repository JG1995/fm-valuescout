use serde::Serialize;
use tauri::State;

use crate::db::Db;

use super::service;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
}

#[derive(Serialize)]
pub struct DemoValue {
    pub value: String,
}

#[tauri::command]
pub fn get_status() -> HealthStatus {
    HealthStatus {
        status: "ok".to_string(),
    }
}

#[tauri::command]
pub fn get_demo_value(db: State<'_, Db>) -> Result<DemoValue, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let value = service::get_demo_value(&conn)?;
    Ok(DemoValue { value })
}

#[tauri::command]
pub fn set_demo_value(value: String, db: State<'_, Db>) -> Result<DemoValue, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let value = service::set_demo_value(&conn, &value)?;
    Ok(DemoValue { value })
}
