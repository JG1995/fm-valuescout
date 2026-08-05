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
            features::memory_read::commands::get_bridge_install_status,
            features::memory_read::commands::install_bridge_plugin,
            features::memory_read::commands::remove_bridge_plugin,
            features::snapshot::commands::list_saves,
            features::snapshot::commands::create_save,
            features::snapshot::commands::rename_save,
            features::snapshot::commands::set_active_save,
            features::snapshot::commands::get_current_snapshot,
            features::snapshot::commands::list_sanity_players,
            features::snapshot::commands::load_data,
            features::search::commands::search_players,
            features::search::commands::suggest_players,
            features::player::commands::get_player,
            features::planner::commands::get_planner_club_family,
            features::planner::commands::list_planner_clubs,
            features::planner::commands::save_planner_club_family,
            features::planner::commands::get_planner_tactic,
            features::planner::commands::get_planner_tactic_options,
            features::planner::commands::save_planner_tactic,
            features::planner::commands::get_planner_depth,
            features::planner::commands::optimize_planner_depth,
            features::planner::commands::get_planner_slot_candidates,
            features::planner::commands::add_planner_string,
            features::planner::commands::remove_planner_string,
            features::planner::commands::clear_planner_depth,
            features::planner::commands::clear_planner_assignment,
            features::planner::commands::assign_planner_player,
            features::planner::commands::move_planner_player,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
