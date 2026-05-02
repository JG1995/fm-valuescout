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

            // Seed default archetypes if table is empty
            storage::seed_default_archetypes(&conn)
                .map_err(|e| format!("Unable to seed default archetypes: {}", e))?;

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
            commands::archetypes::create_archetype_cmd,
            commands::archetypes::list_archetypes_by_role,
            commands::archetypes::list_all_archetypes_cmd,
            commands::archetypes::get_archetype_cmd,
            commands::archetypes::update_archetype_cmd,
            commands::archetypes::delete_archetype_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
