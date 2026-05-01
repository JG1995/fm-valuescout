mod commands;
pub mod parser;
pub mod storage;
pub use parser::parse_csv;

use std::sync::Mutex;
use tauri::Manager;
use storage::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Unable to access app data directory: {}", e))?;

            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Unable to create app data directory: {}", e))?;

            let db_path = app_data_dir.join("fm_valuescout.db");
            let db_path_str = db_path.to_string_lossy().to_string();

            let conn = storage::init_db(&db_path_str)
                .map_err(|e| format!("Unable to initialize database: {}", e))?;

            app.manage(DbState {
                conn: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::csv_parser::parse_csv,
            commands::storage::create_save,
            commands::storage::list_saves,
            commands::storage::rename_save,
            commands::storage::delete_save,
            commands::storage::list_seasons,
            commands::storage::rename_season,
            commands::storage::delete_season,
            commands::storage::import_season,
            commands::storage::get_players_for_season,
            commands::storage::get_player_career,
            commands::storage::get_latest_season,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
