use std::path::Path;

use tauri::State;

use crate::db::Db;

use super::service::{
    capture_preview_context, preview_csv_file, revalidate_preview_context, CsvMatchPreview,
    CsvPreviewError,
};

#[tauri::command]
pub fn preview_csv_matches(path: String, db: State<'_, Db>) -> Result<CsvMatchPreview, String> {
    preview_csv_matches_for_path(Path::new(&path), &db).map_err(|error| error.to_string())
}

pub(crate) fn preview_csv_matches_for_path(
    path: &Path,
    db: &Db,
) -> Result<CsvMatchPreview, CsvPreviewError> {
    let context = {
        let conn = db.0.lock().map_err(|_| CsvPreviewError::Database)?;
        capture_preview_context(&conn)?
    };
    let preview = preview_csv_file(path, &context)?;
    let conn = db.0.lock().map_err(|_| CsvPreviewError::Database)?;
    revalidate_preview_context(&conn, &context)?;
    Ok(preview)
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::features::snapshot::ingest::ingest_dump_file;

    use super::*;
    use crate::features::csv_import::service::{CsvMatchPreview, CsvPreviewFormat};

    #[test]
    fn command_helper_reads_a_file_outside_the_database_lock() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = db::open(&temp_dir.path().join("preview-command.db")).expect("open database");
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v6.json");
        {
            let mut conn = db.0.lock().expect("lock database");
            ingest_dump_file(&mut conn, &dump_path).expect("ingest fixture snapshot");
        }
        let csv_path = temp_dir.path().join("player.csv");
        std::fs::write(&csv_path, "Unique ID;Player\n77;CSV name does not matter\n")
            .expect("write CSV");

        let preview = preview_csv_matches_for_path(&csv_path, &db).expect("preview CSV");

        assert_eq!(
            preview,
            CsvMatchPreview {
                format: CsvPreviewFormat::YouthTracker,
                total_players: 1,
                matched_players: 1,
                unmatched_players: 0,
            }
        );
        assert_eq!(
            serde_json::to_value(preview).expect("serialize preview"),
            serde_json::json!({
                "format": "youthTracker",
                "totalPlayers": 1,
                "matchedPlayers": 1,
                "unmatchedPlayers": 0,
            })
        );
    }
}
