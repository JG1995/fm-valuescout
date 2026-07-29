use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;

use crate::features::memory_read::service::{
    dump_path, request_player_dump, resolve_bridge_directory, BridgeStatusError, DumpRequestError,
    DumpRequestResult, DumpWaitConfig,
};

use super::ingest::{self, SnapshotSummary};
use super::service;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadDataResult {
    pub request_id: String,
    pub players_found: Option<i32>,
    pub scan_truncated: Option<bool>,
    pub max_accepted: Option<i32>,
    pub snapshot: SnapshotSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum LoadDataError {
    Scan { kind: String, message: String },
    Ingest { message: String },
}

impl std::fmt::Display for LoadDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scan { message, .. } | Self::Ingest { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LoadDataError {}

pub fn scan_dump_from_local_app_data(
    wait: DumpWaitConfig,
) -> Result<(PathBuf, DumpRequestResult), LoadDataError> {
    let bridge_directory = resolve_bridge_directory().map_err(map_bridge_status_error)?;
    scan_dump_from_bridge(&bridge_directory, wait)
}

pub fn scan_dump_from_bridge(
    bridge_directory: &Path,
    wait: DumpWaitConfig,
) -> Result<(PathBuf, DumpRequestResult), LoadDataError> {
    let dump_result =
        request_player_dump(bridge_directory, wait).map_err(map_dump_request_error)?;
    Ok((bridge_directory.to_path_buf(), dump_result))
}

/// Convenience for unit tests that simulate scan + ingest in one call.
#[cfg_attr(not(test), allow(dead_code))]
pub fn load_data_from_bridge(
    conn: &mut Connection,
    bridge_directory: &Path,
    wait: DumpWaitConfig,
) -> Result<LoadDataResult, LoadDataError> {
    let save_id = service::active_save_id(conn).map_err(|message| LoadDataError::Scan {
        kind: "internal".to_string(),
        message,
    })?;
    let dump_result =
        request_player_dump(bridge_directory, wait).map_err(map_dump_request_error)?;
    load_data_after_scan(conn, bridge_directory, dump_result, save_id)
}

pub fn load_data_after_scan(
    conn: &mut Connection,
    bridge_directory: &Path,
    dump_result: DumpRequestResult,
    save_id: i64,
) -> Result<LoadDataResult, LoadDataError> {
    ensure_scan_succeeded(&dump_result)?;

    let snapshot = ingest::ingest_dump_file_for_save(conn, save_id, &dump_path(bridge_directory))
        .map_err(|message| LoadDataError::Ingest { message })?;

    Ok(LoadDataResult {
        request_id: dump_result.request_id,
        players_found: dump_result.players_found,
        scan_truncated: dump_result.scan_truncated,
        max_accepted: dump_result.max_accepted,
        snapshot,
    })
}

fn ensure_scan_succeeded(dump_result: &DumpRequestResult) -> Result<(), LoadDataError> {
    if dump_result.state != "ready" {
        return Err(LoadDataError::Scan {
            kind: "bridgeFailed".to_string(),
            message: dump_result
                .error
                .clone()
                .unwrap_or_else(|| format!("bridge scan ended in state {}", dump_result.state)),
        });
    }

    if !dump_result.dump_present {
        return Err(LoadDataError::Scan {
            kind: "missingDump".to_string(),
            message: "bridge reported ready but dump.json is missing".to_string(),
        });
    }

    Ok(())
}

fn map_bridge_status_error(error: BridgeStatusError) -> LoadDataError {
    map_dump_request_error(DumpRequestError::from(error))
}

fn map_dump_request_error(error: DumpRequestError) -> LoadDataError {
    let kind = match &error {
        DumpRequestError::UnsupportedPlatform(_) => "unsupportedPlatform",
        DumpRequestError::Missing(_) => "missing",
        DumpRequestError::Corrupt(_) => "corrupt",
        DumpRequestError::Timeout(_) => "timeout",
        DumpRequestError::WriteFailed(_) => "writeFailed",
    }
    .to_string();

    LoadDataError::Scan {
        kind,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::memory_read::service::{
        request_path, status_path, BridgeRequest, BridgeStatus, PROTOCOL_VERSION,
    };
    use crate::features::snapshot::service::{create_save, list_saves, set_active_save};
    use rusqlite::OptionalExtension;
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v5.json");

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn current_snapshot_id(conn: &Connection, save_id: i64) -> Option<i64> {
        conn.query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            rusqlite::params![save_id],
            |row| row.get(0),
        )
        .optional()
        .expect("query current snapshot")
    }

    fn write_status_fixture(
        bridge_dir: &Path,
        state: &str,
        request_id: Option<&str>,
        players_found: Option<i32>,
        error: Option<&str>,
        scan_truncated: Option<bool>,
        max_accepted: Option<i32>,
    ) {
        let status = BridgeStatus {
            protocol_version: PROTOCOL_VERSION,
            plugin_version: "0.1.0".to_string(),
            state: state.to_string(),
            updated_at_utc: "2026-07-28T18:30:00+00:00".to_string(),
            game_plugin_module_present: true,
            game_assembly_module_present: true,
            request_id: request_id.map(str::to_string),
            players_found,
            error: error.map(str::to_string),
            scan_truncated,
            max_accepted,
        };
        let json = serde_json::to_string_pretty(&status).expect("serialize");
        let path = status_path(bridge_dir);
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, &json).expect("write status tmp");
        fs::rename(&temp, &path).expect("rename status");
    }

    enum ScanSimulation {
        Ready { dump_json: String },
        Failed { message: String },
    }

    fn spawn_scan_responder(bridge_dir: &Path, simulation: ScanSimulation) {
        let bridge_dir = bridge_dir.to_path_buf();
        thread::spawn(move || {
            let mut last_request_id = String::new();
            loop {
                if !request_path(&bridge_dir).is_file() {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }

                let json = fs::read_to_string(request_path(&bridge_dir)).expect("read request");
                let request: BridgeRequest = serde_json::from_str(&json).expect("parse request");
                if request.request_id == last_request_id {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                last_request_id = request.request_id.clone();

                match simulation {
                    ScanSimulation::Ready { ref dump_json } => {
                        write_status_fixture(
                            &bridge_dir,
                            "scanning",
                            Some(&request.request_id),
                            None,
                            None,
                            None,
                            None,
                        );
                        thread::sleep(Duration::from_millis(30));
                        fs::write(dump_path(&bridge_dir), dump_json).expect("dump");
                        write_status_fixture(
                            &bridge_dir,
                            "ready",
                            Some(&request.request_id),
                            Some(42),
                            None,
                            Some(false),
                            Some(10_000),
                        );
                    }
                    ScanSimulation::Failed { ref message } => {
                        write_status_fixture(
                            &bridge_dir,
                            "failed",
                            Some(&request.request_id),
                            None,
                            Some(message),
                            None,
                            None,
                        );
                    }
                }
            }
        });
    }

    fn short_wait() -> DumpWaitConfig {
        DumpWaitConfig {
            timeout: Duration::from_secs(2),
            poll_interval: Duration::from_millis(20),
        }
    }

    #[test]
    fn load_data_ingests_snapshot_after_successful_scan() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().join("bridge");
        fs::create_dir_all(&bridge_dir).expect("bridge dir");
        let mut conn = open_migrated(&temp_dir.path().join("load-data-success.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        spawn_scan_responder(
            &bridge_dir,
            ScanSimulation::Ready {
                dump_json: GOLDEN_FIXTURE.to_string(),
            },
        );

        let result =
            load_data_from_bridge(&mut conn, &bridge_dir, short_wait()).expect("load data");

        assert!(result.request_id.starts_with("req-"));
        assert_eq!(result.players_found, Some(42));
        assert_eq!(result.snapshot.save_id, active_save.id);
        assert_eq!(result.snapshot.player_count, 1);
        assert_eq!(
            current_snapshot_id(&conn, active_save.id),
            Some(result.snapshot.id)
        );
    }

    #[test]
    fn load_data_returns_scan_error_when_bridge_scan_fails() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().join("bridge");
        fs::create_dir_all(&bridge_dir).expect("bridge dir");
        let mut conn = open_migrated(&temp_dir.path().join("load-data-scan-fail.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        spawn_scan_responder(
            &bridge_dir,
            ScanSimulation::Failed {
                message: "scan produced zero player candidates".to_string(),
            },
        );

        let error =
            load_data_from_bridge(&mut conn, &bridge_dir, short_wait()).expect_err("scan failure");

        assert!(matches!(
            &error,
            LoadDataError::Scan {
                kind,
                message: _
            } if kind == "bridgeFailed"
        ));
        assert!(error.to_string().contains("zero player"));
        assert_eq!(current_snapshot_id(&conn, active_save.id), None);
    }

    #[test]
    fn load_data_returns_ingest_error_and_keeps_prior_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().join("bridge");
        fs::create_dir_all(&bridge_dir).expect("bridge dir");
        let mut conn = open_migrated(&temp_dir.path().join("load-data-ingest-fail.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        spawn_scan_responder(
            &bridge_dir,
            ScanSimulation::Ready {
                dump_json: GOLDEN_FIXTURE.to_string(),
            },
        );
        let first =
            load_data_from_bridge(&mut conn, &bridge_dir, short_wait()).expect("first load");
        let prior_snapshot_id = first.snapshot.id;

        fs::write(dump_path(&bridge_dir), "{\"schemaVersion\":99}").expect("bad dump");
        let dump_result = DumpRequestResult {
            request_id: "req-bad-ingest".to_string(),
            state: "ready".to_string(),
            players_found: Some(1),
            dump_present: true,
            error: None,
            scan_truncated: Some(false),
            max_accepted: Some(10_000),
        };
        let error = load_data_after_scan(&mut conn, &bridge_dir, dump_result, active_save.id)
            .expect_err("ingest failure");

        assert!(matches!(error, LoadDataError::Ingest { .. }));
        assert_eq!(
            current_snapshot_id(&conn, active_save.id),
            Some(prior_snapshot_id)
        );
    }

    #[test]
    fn load_data_after_scan_uses_captured_save_not_current_active() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let bridge_dir = temp_dir.path().join("bridge");
        fs::create_dir_all(&bridge_dir).expect("bridge dir");
        let mut conn = open_migrated(&temp_dir.path().join("captured-save.db"));
        let default_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let second_save = create_save(&conn, "Second save").expect("create second save");
        set_active_save(&mut conn, second_save.id).expect("switch active save");

        fs::write(dump_path(&bridge_dir), GOLDEN_FIXTURE).expect("dump");
        let dump_result = DumpRequestResult {
            request_id: "req-captured-save".to_string(),
            state: "ready".to_string(),
            players_found: Some(1),
            dump_present: true,
            error: None,
            scan_truncated: Some(false),
            max_accepted: Some(10_000),
        };

        let result = load_data_after_scan(&mut conn, &bridge_dir, dump_result, default_save.id)
            .expect("ingest into captured save");

        assert_eq!(result.snapshot.save_id, default_save.id);
        assert_eq!(
            current_snapshot_id(&conn, default_save.id),
            Some(result.snapshot.id)
        );
        assert_eq!(current_snapshot_id(&conn, second_save.id), None);
    }
}
