mod db;
mod features;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let db_path = db::resolve_db_path(app.handle())?;
            let db = db::open(&db_path)?;
            app.manage(db);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            features::health::commands::get_status,
            features::health::commands::get_demo_value,
            features::health::commands::set_demo_value,
            features::memory_read::commands::get_bridge_status,
            features::memory_read::commands::request_player_dump,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
