use std::path::Path;

use tauri::State;

use crate::db::Db;

use super::service::{
    capture_import_context, persist_csv_import, prepare_csv_import, CsvImportSummary,
    CsvPersistenceError,
};

#[tauri::command]
pub fn import_csv(path: String, db: State<'_, Db>) -> Result<CsvImportSummary, String> {
    import_csv_for_path(Path::new(&path), &db).map_err(|error| error.to_string())
}

pub(crate) fn import_csv_for_path(
    path: &Path,
    db: &Db,
) -> Result<CsvImportSummary, CsvPersistenceError> {
    let context = {
        let conn = db.0.lock().map_err(|_| CsvPersistenceError::Database)?;
        capture_import_context(&conn)?
    };
    let import = prepare_csv_import(path)?;
    let mut conn = db.0.lock().map_err(|_| CsvPersistenceError::Database)?;
    persist_csv_import(&mut conn, &context, import)
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use serde_json::json;

    use crate::db;
    use crate::features::csv_import::service::CsvImportFormat;
    use crate::features::snapshot::ingest::ingest_dump_file;

    use super::*;

    #[test]
    fn command_helper_imports_matched_players_after_reading_outside_the_database_lock() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = db::open(&temp_dir.path().join("import-command.db")).expect("open database");
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v6.json");
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

        let summary = import_csv_for_path(&csv_path, &db).expect("import CSV");

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
}
