mod db;
mod features;

use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

const RELEASE_LOG_MAX_FILE_SIZE_BYTES: u128 = 1_000_000;
const RELEASE_LOG_RETAINED_FILE_COUNT: usize = 3;
const RELEASE_LOG_FILE_NAME: &str = "fm-valuescout";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .max_file_size(RELEASE_LOG_MAX_FILE_SIZE_BYTES)
                    .rotation_strategy(RotationStrategy::KeepSome(RELEASE_LOG_RETAINED_FILE_COUNT))
                    .targets([
                        Target::new(TargetKind::Stdout),
                        Target::new(TargetKind::LogDir {
                            file_name: Some(RELEASE_LOG_FILE_NAME.into()),
                        }),
                    ])
                    .build(),
            )?;

            log::info!(
                "FM ValueScout {} starting; database schema target={}",
                env!("CARGO_PKG_VERSION"),
                db::migrations::latest_version()
            );

            let db_path = db::resolve_db_path(app.handle()).inspect_err(|_error| {
                log::error!("database path resolution failed during startup");
            })?;
            let db = db::open(&db_path).inspect_err(|_error| {
                log::error!("database initialization failed during startup");
            })?;
            app.manage(db);

            log::info!("FM ValueScout startup complete");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            features::memory_read::commands::get_bridge_status,
            features::memory_read::commands::request_player_dump,
            features::memory_read::commands::get_bridge_install_status,
            features::memory_read::commands::install_bridge_plugin,
            features::memory_read::commands::remove_bridge_plugin,
            features::snapshot::commands::list_saves,
            features::snapshot::commands::create_save,
            features::snapshot::commands::rename_save,
            features::snapshot::commands::set_active_save,
            features::snapshot::commands::list_snapshots,
            features::snapshot::commands::rename_snapshot,
            features::snapshot::commands::delete_snapshot,
            features::snapshot::commands::delete_save,
            features::snapshot::commands::get_current_snapshot,
            features::snapshot::commands::list_sanity_players,
            features::snapshot::commands::load_data,
            features::csv_import::commands::import_csv,
            features::search::commands::search_players,
            features::search::commands::suggest_players,
            features::player::commands::get_player,
            features::player::commands::boost_current_ability,
            features::player::commands::boost_wonderkid_mentality,
            features::player::commands::boost_squad_current_ability,
            features::player::commands::boost_squad_wonderkid_mentality,
            features::academy::commands::list_academy_classes,
            features::academy::commands::get_academy_class,
            features::academy::commands::create_academy_class,
            features::academy::commands::delete_academy_class,
            features::academy::commands::list_academy_candidates,
            features::academy::commands::assign_academy_member,
            features::academy::commands::remove_academy_member,
            features::academy::commands::set_academy_member_outcome,
            features::planner::commands::get_planner_club_family,
            features::planner::commands::list_planner_clubs,
            features::planner::commands::list_squad_players,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_logs_have_bounded_local_retention() {
        assert_eq!(RELEASE_LOG_MAX_FILE_SIZE_BYTES, 1_000_000);
        assert_eq!(RELEASE_LOG_RETAINED_FILE_COUNT, 3);
        assert_eq!(RELEASE_LOG_FILE_NAME, "fm-valuescout");
    }
}
