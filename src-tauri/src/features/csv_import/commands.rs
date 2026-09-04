use std::path::Path;

use tauri::State;

use crate::db::Db;

use super::player_shortlist::{
    capture_player_shortlist_import_context, persist_player_shortlist_import,
    prepare_player_shortlist_import, PlayerShortlistImportError, PlayerShortlistImportSummary,
};
use super::service::{
    capture_import_context, persist_csv_import, prepare_csv_import_for_expected_format,
    CsvImportFormat, CsvImportSummary, CsvPersistenceError,
};
use super::staff_shortlist::{
    capture_staff_shortlist_import_context, persist_staff_shortlist_import,
    prepare_staff_shortlist_import, StaffShortlistImportError, StaffShortlistImportSummary,
};

#[tauri::command]
pub fn import_csv(
    path: String,
    expected_format: Option<CsvImportFormat>,
    db: State<'_, Db>,
) -> Result<CsvImportSummary, String> {
    import_csv_for_path_with_expected_format(Path::new(&path), &db, expected_format)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_player_shortlist_csv(
    path: String,
    db: State<'_, Db>,
) -> Result<PlayerShortlistImportSummary, String> {
    import_player_shortlist_csv_for_path(Path::new(&path), &db).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_staff_shortlist_csv(
    path: String,
    db: State<'_, Db>,
) -> Result<StaffShortlistImportSummary, String> {
    import_staff_shortlist_csv_for_path(Path::new(&path), &db).map_err(|error| error.to_string())
}

pub(crate) fn import_csv_for_path_with_expected_format(
    path: &Path,
    db: &Db,
    expected_format: Option<CsvImportFormat>,
) -> Result<CsvImportSummary, CsvPersistenceError> {
    let context = {
        let conn = db.0.lock().map_err(|_| CsvPersistenceError::Database)?;
        capture_import_context(&conn)?
    };
    let import = prepare_csv_import_for_expected_format(path, &context, expected_format)?;
    let mut conn = db.0.lock().map_err(|_| CsvPersistenceError::Database)?;
    persist_csv_import(&mut conn, &context, import)
}

pub(crate) fn import_player_shortlist_csv_for_path(
    path: &Path,
    db: &Db,
) -> Result<PlayerShortlistImportSummary, PlayerShortlistImportError> {
    let context = {
        let conn =
            db.0.lock()
                .map_err(|_| PlayerShortlistImportError::Database)?;
        capture_player_shortlist_import_context(&conn)?
    };
    let parsed = prepare_player_shortlist_import(path)?;
    let mut conn =
        db.0.lock()
            .map_err(|_| PlayerShortlistImportError::Database)?;
    persist_player_shortlist_import(&mut conn, &context, parsed)
}

pub(crate) fn import_staff_shortlist_csv_for_path(
    path: &Path,
    db: &Db,
) -> Result<StaffShortlistImportSummary, StaffShortlistImportError> {
    let context = {
        let conn =
            db.0.lock()
                .map_err(|_| StaffShortlistImportError::Database)?;
        capture_staff_shortlist_import_context(&conn)?
    };
    let entries = prepare_staff_shortlist_import(path)?;
    let mut conn =
        db.0.lock()
            .map_err(|_| StaffShortlistImportError::Database)?;
    persist_staff_shortlist_import(&mut conn, &context, entries)
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use serde_json::json;

    use crate::db;
    use crate::features::csv_import::service::{CsvImportFormat, CsvImportServiceError};
    use crate::features::snapshot::ingest::ingest_dump_file;

    use super::*;

    #[test]
    fn command_helper_imports_matched_players_after_reading_outside_the_database_lock() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = db::open(&temp_dir.path().join("import-command.db")).expect("open database");
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");
        {
            let mut conn = db.0.lock().expect("lock database");
            ingest_dump_file(&mut conn, &dump_path).expect("ingest fixture snapshot");
        }
        let csv_path = temp_dir.path().join("player.csv");
        std::fs::write(
            &csv_path,
            "Unique ID;Player;AT Apps;Int Apps;AT Gls;Assists\n77;CSV name does not matter;12;3;4;5\n",
        )
        .expect("write CSV");

        let summary =
            import_csv_for_path_with_expected_format(&csv_path, &db, None).expect("import CSV");

        assert_eq!(
            summary,
            CsvImportSummary {
                format: CsvImportFormat::YouthTracker,
                total_players: 1,
                stored_players: 1,
                skipped_players: 0,
            }
        );
        assert_eq!(
            serde_json::to_value(&summary).expect("serialize summary"),
            json!({
                "format": "youthTracker",
                "totalPlayers": 1,
                "storedPlayers": 1,
                "skippedPlayers": 0,
            })
        );
        let conn = db.0.lock().expect("lock database");
        assert_eq!(
            conn.query_row(
                "SELECT career_appearances, international_caps, career_goals, career_assists
                 FROM player_youth_career_stats
                 WHERE player_uid = ?1",
                params![77],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .expect("read stored career values"),
            (12, 3, 4, 5)
        );
    }

    #[test]
    fn command_helper_rejects_a_selected_format_mismatch_before_writing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = db::open(&temp_dir.path().join("format-mismatch.db")).expect("open database");
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");
        {
            let mut conn = db.0.lock().expect("lock database");
            ingest_dump_file(&mut conn, &dump_path).expect("ingest fixture snapshot");
        }
        let csv_path = temp_dir.path().join("player.csv");
        std::fs::write(
            &csv_path,
            "Unique ID;Player;AT Apps\n77;CSV name does not matter;12\n",
        )
        .expect("write Youth Tracker CSV");

        assert_eq!(
            import_csv_for_path_with_expected_format(
                &csv_path,
                &db,
                Some(CsvImportFormat::Moneyball),
            )
            .expect_err("reject a Youth Tracker CSV selected as Moneyball"),
            CsvPersistenceError::Import(CsvImportServiceError::FormatMismatch)
        );

        let conn = db.0.lock().expect("lock database");
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_youth_career_stats",
                [],
                |row| row.get::<_, i64>(0),
            )
            .expect("count enrichment rows"),
            0
        );
    }

    #[test]
    fn command_helper_imports_player_shortlist_and_serializes_the_safe_summary() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db =
            db::open(&temp_dir.path().join("player-shortlist-command.db")).expect("open database");
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");
        {
            let mut conn = db.0.lock().expect("lock database");
            ingest_dump_file(&mut conn, &dump_path).expect("ingest fixture snapshot");
        }
        let csv_path = temp_dir.path().join("players.csv");
        std::fs::write(&csv_path, "Player UID\n77\n88\n").expect("write player CSV");

        let summary =
            import_player_shortlist_csv_for_path(&csv_path, &db).expect("import player shortlist");

        assert_eq!(
            serde_json::to_value(summary).expect("serialize summary"),
            json!({
                "totalPlayers": 2,
                "storedPlayers": 1,
                "skippedPlayers": 1,
            })
        );
    }

    #[test]
    fn command_helper_imports_staff_shortlist_and_serializes_the_safe_summary() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db =
            db::open(&temp_dir.path().join("staff-shortlist-command.db")).expect("open database");
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");
        {
            let mut conn = db.0.lock().expect("lock database");
            ingest_dump_file(&mut conn, &dump_path).expect("ingest fixture snapshot");
        }
        let csv_path = temp_dir.path().join("staff.csv");
        std::fs::write(
            &csv_path,
            "Unique ID;Preferred Job;Club Job;Coaching Qualifications\n88;Physio;-;Continental Pro\n99;Scout;;National C\n",
        )
        .expect("write staff CSV");

        let summary =
            import_staff_shortlist_csv_for_path(&csv_path, &db).expect("import staff shortlist");

        assert_eq!(
            serde_json::to_value(summary).expect("serialize summary"),
            json!({
                "totalStaff": 2,
                "storedStaff": 1,
                "skippedStaff": 1,
            })
        );
    }
}
