use std::path::{Path, PathBuf};

use tauri::{plugin::TauriPlugin, Runtime};

use crate::{
    db::Db,
    features::snapshot::ingest::{self, SnapshotSummary},
};

const DUMP_ENV: &str = "FM_VALUESCOUT_UI_AGENT_DUMP";
const BIND_ADDRESS: &str = "127.0.0.1";

pub fn prepare_from_environment(db: &mut Db) -> Result<(), String> {
    let Some(dump_path) = std::env::var_os(DUMP_ENV) else {
        return Ok(());
    };

    let summary = seed_dump(db, &PathBuf::from(dump_path))?;
    log::info!(
        "UI-agent seed loaded snapshot {} with {} player(s)",
        summary.id,
        summary.player_count
    );
    Ok(())
}

fn seed_dump(db: &mut Db, dump_path: &Path) -> Result<SnapshotSummary, String> {
    if !dump_path.is_absolute() {
        return Err(format!("{DUMP_ENV} must be an absolute path"));
    }
    if !dump_path.is_file() {
        return Err(format!(
            "{DUMP_ENV} must reference a readable regular file: {}",
            dump_path.display()
        ));
    }

    let conn = db.0.get_mut().map_err(|error| error.to_string())?;
    ingest::ingest_dump_file(conn, dump_path)
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_mcp_bridge::Builder::new()
        .bind_address(BIND_ADDRESS)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_dump_uses_the_product_snapshot_ingest_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut db = crate::db::open(&temp_dir.path().join("app.db")).expect("open database");
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v5.json");

        let summary = seed_dump(&mut db, &dump_path).expect("seed dump");

        assert_eq!(summary.player_count, 1);
        let conn = db.0.get_mut().expect("unlock database");
        let player_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM players", [], |row| row.get(0))
            .expect("count players");
        assert_eq!(player_count, 1);
    }

    #[test]
    fn invalid_dump_leaves_no_snapshot_to_control() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let dump_path = temp_dir.path().join("invalid.json");
        std::fs::write(&dump_path, "{}").expect("write invalid dump");
        let mut db = crate::db::open(&temp_dir.path().join("app.db")).expect("open database");

        let error = seed_dump(&mut db, &dump_path).expect_err("reject invalid dump");

        assert!(error.contains("schemaVersion"));
        let conn = db.0.get_mut().expect("unlock database");
        let snapshot_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .expect("count snapshots");
        assert_eq!(snapshot_count, 0);
    }

    #[test]
    fn dump_path_must_be_absolute() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut db = crate::db::open(&temp_dir.path().join("app.db")).expect("open database");

        let error = seed_dump(&mut db, Path::new("dump.json")).expect_err("reject relative path");

        assert!(error.contains("absolute path"));
    }
}
