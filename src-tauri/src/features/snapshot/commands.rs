use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use tauri::State;
use tempfile::TempPath;

use crate::db::Db;
use crate::features::memory_read::service::{DumpRequestResult, DumpWaitConfig};
use crate::features::player::boost_gate;

use super::ingest::{self, SnapshotSummary};
use super::load_data::{self, LoadDataError, LoadDataResult};
use super::query;
use super::service::{self, SaveDeleteResult, SaveSummary, SnapshotDeleteResult, SnapshotMetadata};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummaryDto {
    pub id: i64,
    pub context_token: String,
    pub name: String,
    pub is_active: bool,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

impl From<SaveSummary> for SaveSummaryDto {
    fn from(save: SaveSummary) -> Self {
        Self {
            id: save.id,
            context_token: save.context_token,
            name: save.name,
            is_active: save.is_active,
            created_at_utc: save.created_at_utc,
            updated_at_utc: save.updated_at_utc,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMetadataDto {
    pub id: i64,
    pub context_token: String,
    pub save_id: i64,
    pub custom_name: Option<String>,
    pub game_date: Option<String>,
    pub game_date_source: String,
    pub player_count: i64,
    pub loaded_at_utc: String,
    pub is_current: bool,
}

impl From<SnapshotMetadata> for SnapshotMetadataDto {
    fn from(snapshot: SnapshotMetadata) -> Self {
        Self {
            id: snapshot.id,
            context_token: snapshot.context_token,
            save_id: snapshot.save_id,
            custom_name: snapshot.custom_name,
            game_date: snapshot.game_date,
            game_date_source: snapshot.game_date_source,
            player_count: snapshot.player_count,
            loaded_at_utc: snapshot.loaded_at_utc,
            is_current: snapshot.is_current,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDeleteResultDto {
    pub deleted_snapshot_id: i64,
    pub save_id: i64,
    pub current_snapshot_id: Option<i64>,
}

impl From<SnapshotDeleteResult> for SnapshotDeleteResultDto {
    fn from(result: SnapshotDeleteResult) -> Self {
        Self {
            deleted_snapshot_id: result.deleted_snapshot_id,
            save_id: result.save_id,
            current_snapshot_id: result.current_snapshot_id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDeleteResultDto {
    pub deleted_save_id: i64,
    pub deleted_was_active: bool,
    pub active_save: SaveSummaryDto,
}

impl From<SaveDeleteResult> for SaveDeleteResultDto {
    fn from(result: SaveDeleteResult) -> Self {
        Self {
            deleted_save_id: result.deleted_save_id,
            deleted_was_active: result.deleted_was_active,
            active_save: SaveSummaryDto::from(result.active_save),
        }
    }
}

#[tauri::command]
pub fn list_saves(db: State<'_, Db>) -> Result<Vec<SaveSummaryDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let saves = service::list_saves(&conn)?;
    Ok(saves.into_iter().map(SaveSummaryDto::from).collect())
}

#[tauri::command]
pub fn create_save(name: String, db: State<'_, Db>) -> Result<SaveSummaryDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save = service::create_save(&conn, &name)?;
    Ok(SaveSummaryDto::from(save))
}

#[tauri::command]
pub fn rename_save(
    save_id: i64,
    name: String,
    db: State<'_, Db>,
) -> Result<SaveSummaryDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save = service::rename_save(&conn, save_id, &name)?;
    Ok(SaveSummaryDto::from(save))
}

#[tauri::command]
pub fn set_active_save(save_id: i64, db: State<'_, Db>) -> Result<SaveSummaryDto, String> {
    set_active_save_for_command(db.inner(), save_id).map(SaveSummaryDto::from)
}

pub(crate) fn set_active_save_for_command(db: &Db, save_id: i64) -> Result<SaveSummary, String> {
    let _boost_guard = boost_gate::acquire_boost_gate()?;
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    service::set_active_save(&mut conn, save_id)
}

#[tauri::command]
pub fn list_snapshots(
    save_id: Option<i64>,
    db: State<'_, Db>,
) -> Result<Vec<SnapshotMetadataDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let snapshots = service::list_snapshot_metadata(&conn, save_id)?;
    Ok(snapshots
        .into_iter()
        .map(SnapshotMetadataDto::from)
        .collect())
}

#[tauri::command]
pub fn rename_snapshot(
    snapshot_id: i64,
    context_token: String,
    custom_name: Option<String>,
    db: State<'_, Db>,
) -> Result<SnapshotMetadataDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let snapshot =
        service::rename_snapshot(&conn, snapshot_id, &context_token, custom_name.as_deref())?;
    Ok(SnapshotMetadataDto::from(snapshot))
}

#[tauri::command]
pub fn delete_snapshot(
    snapshot_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<SnapshotDeleteResultDto, String> {
    let _boost_guard = boost_gate::acquire_boost_gate()?;
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let result = service::delete_snapshot(&mut conn, snapshot_id, &context_token)?;
    Ok(SnapshotDeleteResultDto::from(result))
}

#[tauri::command]
pub fn delete_save(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<SaveDeleteResultDto, String> {
    let _boost_guard = boost_gate::acquire_boost_gate()?;
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let result = service::delete_save(&mut conn, save_id, &context_token)?;
    Ok(SaveDeleteResultDto::from(result))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummaryDto {
    pub id: i64,
    pub context_token: String,
    pub save_id: i64,
    pub schema_version: i64,
    pub generated_at_utc: String,
    pub game_version: String,
    pub supported_game_version: String,
    pub bridge_version: String,
    pub protocol_version: i64,
    pub game_date: Option<String>,
    pub game_date_source: String,
    pub scan_truncated: bool,
    pub max_accepted: Option<i64>,
    pub player_count: i64,
    pub loaded_at_utc: String,
}

impl From<SnapshotSummary> for SnapshotSummaryDto {
    fn from(snapshot: SnapshotSummary) -> Self {
        Self {
            id: snapshot.id,
            context_token: snapshot.context_token,
            save_id: snapshot.save_id,
            schema_version: snapshot.schema_version,
            generated_at_utc: snapshot.generated_at_utc,
            game_version: snapshot.game_version,
            supported_game_version: snapshot.supported_game_version,
            bridge_version: snapshot.bridge_version,
            protocol_version: snapshot.protocol_version,
            game_date: snapshot.game_date,
            game_date_source: snapshot.game_date_source,
            scan_truncated: snapshot.scan_truncated,
            max_accepted: snapshot.max_accepted,
            player_count: snapshot.player_count,
            loaded_at_utc: snapshot.loaded_at_utc,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDataTimingsDto {
    pub scan_ms: u64,
    pub ingest_ms: u64,
    pub total_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDataResultDto {
    pub request_id: String,
    pub players_found: Option<i32>,
    pub scan_truncated: Option<bool>,
    pub max_accepted: Option<i32>,
    pub stored_snapshot: SnapshotSummaryDto,
    pub effective_snapshot: SnapshotSummaryDto,
    pub timings: LoadDataTimingsDto,
}

impl From<load_data::LoadDataTimings> for LoadDataTimingsDto {
    fn from(timings: load_data::LoadDataTimings) -> Self {
        Self {
            scan_ms: timings.scan_ms,
            ingest_ms: timings.ingest_ms,
            total_ms: timings.total_ms,
        }
    }
}

impl From<LoadDataResult> for LoadDataResultDto {
    fn from(result: LoadDataResult) -> Self {
        Self {
            request_id: result.request_id,
            players_found: result.players_found,
            scan_truncated: result.scan_truncated,
            max_accepted: result.max_accepted,
            stored_snapshot: SnapshotSummaryDto::from(result.stored_snapshot),
            effective_snapshot: SnapshotSummaryDto::from(result.effective_snapshot),
            timings: LoadDataTimingsDto::from(result.timings),
        }
    }
}

#[tauri::command]
pub fn get_current_snapshot(db: State<'_, Db>) -> Result<Option<SnapshotSummaryDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let snapshot = query::get_current_snapshot(&conn)?;
    Ok(snapshot.map(SnapshotSummaryDto::from))
}

pub(crate) fn execute_load_data_with<S, P, C>(
    db: &Db,
    max_accepted: Option<i32>,
    scan: S,
    prepare: P,
    mut now_ms: C,
) -> Result<LoadDataResultDto, LoadDataError>
where
    S: FnOnce(Option<i32>) -> Result<(TempPath, DumpRequestResult), LoadDataError>,
    P: FnOnce(&Path) -> Result<ingest::PreparedSnapshot, LoadDataError>,
    C: FnMut() -> u64,
{
    let _boost_guard = boost_gate::acquire_boost_gate().map_err(|message| LoadDataError::Scan {
        kind: "inProgress".to_string(),
        message,
    })?;
    let total_started = now_ms();
    // Capture save context under a brief Db mutex lock.
    let save_context = {
        let conn = db.0.lock().map_err(|_| LoadDataError::Scan {
            kind: "internal".to_string(),
            message: "database lock poisoned".to_string(),
        })?;
        service::capture_active_save_context(&conn).map_err(|message| LoadDataError::Scan {
            kind: "internal".to_string(),
            message,
        })?
    };
    // Scan without the Db mutex.
    let scan_started = now_ms();
    let (captured_dump_path, dump_result) = scan(max_accepted)?;
    let scan_ms = now_ms().saturating_sub(scan_started);
    // Prepare (validation + raw normalization + projection + compact scoring) without the Db mutex.
    // This is pure and performs zero database reads/writes or rusqlite ownership.
    let prepared = prepare(captured_dump_path.as_ref())?;
    // Final publication: start DB-publication timing, lock once, publish, finish timings.
    let ingest_started = now_ms();
    let mut conn = db.0.lock().map_err(|_| LoadDataError::Scan {
        kind: "internal".to_string(),
        message: "database lock poisoned".to_string(),
    })?;
    let mut result =
        load_data::publish_prepared_dump(&mut conn, dump_result, &save_context, prepared)?;
    let ingest_ms = now_ms().saturating_sub(ingest_started);
    let total_ms = now_ms().saturating_sub(total_started);
    result.timings = load_data::LoadDataTimings {
        scan_ms,
        ingest_ms,
        total_ms,
    };
    Ok(LoadDataResultDto::from(result))
}

#[tauri::command]
pub fn load_data(
    max_accepted: Option<i32>,
    db: State<'_, Db>,
) -> Result<LoadDataResultDto, LoadDataError> {
    let start = Instant::now();
    let now_ms = move || start.elapsed().as_millis() as u64;
    execute_load_data_with(
        db.inner(),
        max_accepted,
        |limit| load_data::scan_dump_from_local_app_data(DumpWaitConfig::default(), limit),
        load_data::prepare_dump_for_publish,
        now_ms,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::collections::VecDeque;
    use std::io::Write;
    use std::rc::Rc;
    use std::sync::Mutex;

    use rusqlite::Connection;
    use tempfile::TempPath;

    use crate::db::{migrations, Db};
    use crate::features::memory_read::service::DumpRequestResult;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v8.json");

    #[test]
    fn production_load_data_orchestrator_does_not_hold_db_lock_during_prepare_and_reports_deterministic_timings(
    ) {
        let _boost_test_guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("cmd-orchestrator.db");
        let conn = Connection::open(&db_path).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        let db = Db(Mutex::new(conn));
        // Seed default save so capture_active_save_context succeeds.
        {
            let guard = db.0.lock().expect("lock db");
            crate::features::snapshot::service::list_saves(&guard).expect("seed default save");
        }
        // Deterministic monotonic ticks: total_start=0, scan_start=10, scan_end=50 => scan_ms=40,
        // ingest_start=80, ingest_end=85 => ingest_ms=5 (DB-only), total_end=100 => total_ms=100.
        // The gap 50->80 (30ms) is preparation, excluded from ingest_ms but included in total.
        let ticks = Rc::new(std::cell::RefCell::new(VecDeque::from([
            0u64, 10, 50, 80, 85, 100,
        ])));
        let call_count = Rc::new(Cell::new(0usize));
        let call_count_for_clock = Rc::clone(&call_count);
        let ticks_for_clock = Rc::clone(&ticks);
        let mut now_ms = move || {
            call_count_for_clock.set(call_count_for_clock.get() + 1);
            ticks_for_clock
                .borrow_mut()
                .pop_front()
                .expect("too many clock calls")
        };
        let call_count_for_prepare = Rc::clone(&call_count);
        let db_for_scan = &db;
        let db_for_prepare = &db;
        let scan = move |max_accepted: Option<i32>| -> Result<(TempPath, DumpRequestResult), LoadDataError> {
            let db = db_for_scan;
            assert!(
                db.0.try_lock().is_ok(),
                "Db mutex must be free during scan – orchestrator must not hold lock across scan"
            );
            assert_eq!(max_accepted, None);
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp file");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write golden");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-deterministic-test".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare = move |path: &Path| -> Result<ingest::PreparedSnapshot, LoadDataError> {
            let db = db_for_prepare;
            assert!(
                db.0.try_lock().is_ok(),
                "Db mutex must be free during prepare – orchestrator must not reacquire Db before preparation or start ingest timing before preparation"
            );
            // Verify ordering: ingest_started must not have been called yet.
            // Clock calls so far: total_start (1), scan_start (2), scan_end (3). Next is ingest_start (4).
            assert_eq!(
                call_count_for_prepare.get(),
                3,
                "prepare must run after scan but before ingest_started – ingest timing started too early"
            );
            load_data::prepare_dump_for_publish(path)
        };
        let result = execute_load_data_with(&db, None, scan, prepare, &mut now_ms)
            .expect("deterministic load_data via production orchestrator");
        assert_eq!(result.request_id, "req-deterministic-test");
        assert_eq!(
            result.timings.scan_ms, 40,
            "scan_ms must be deterministic DB-free interval"
        );
        assert_eq!(
            result.timings.ingest_ms, 5,
            "ingest_ms must be DB-only, excluding preparation"
        );
        assert_eq!(
            result.timings.total_ms, 100,
            "total must include preparation"
        );
        assert!(
            result.timings.total_ms >= result.timings.scan_ms + result.timings.ingest_ms,
            "total must cover scan + ingest"
        );
        // Real publication must have succeeded with the golden dump.
        assert_eq!(result.stored_snapshot.player_count, 1);
        assert_eq!(result.effective_snapshot.player_count, 1);
        let guard = db.0.lock().expect("lock db");
        let snapshot_count: i64 = guard
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .expect("count snapshots");
        assert_eq!(snapshot_count, 1);
    }

    #[test]
    fn snapshot_summary_dto_exposes_context_tokens_for_current_and_load_data_results() {
        let summary = |context_token: &str| SnapshotSummaryDto {
            id: 7,
            context_token: context_token.to_string(),
            save_id: 3,
            schema_version: 8,
            generated_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
            game_version: "26.3".to_string(),
            supported_game_version: "26.3".to_string(),
            bridge_version: "0.4".to_string(),
            protocol_version: 1,
            game_date: Some("2026-08-01".to_string()),
            game_date_source: "memory".to_string(),
            scan_truncated: false,
            max_accepted: None,
            player_count: 25,
            loaded_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
        };
        let current = serde_json::to_value(summary("current-token")).expect("serialize current");
        assert_eq!(current["id"], 7);
        assert_eq!(current["contextToken"], "current-token");

        let load_data = serde_json::to_value(LoadDataResultDto {
            request_id: "request".to_string(),
            players_found: Some(25),
            scan_truncated: Some(false),
            max_accepted: None,
            stored_snapshot: summary("stored-token"),
            effective_snapshot: summary("effective-token"),
            timings: LoadDataTimingsDto {
                scan_ms: 1,
                ingest_ms: 2,
                total_ms: 3,
            },
        })
        .expect("serialize Load Data result");
        assert_eq!(load_data["storedSnapshot"]["contextToken"], "stored-token");
        assert_eq!(
            load_data["effectiveSnapshot"]["contextToken"],
            "effective-token"
        );
    }

    #[test]
    fn snapshot_management_dtos_expose_metadata_and_invalidation_context_only() {
        let snapshot = SnapshotMetadataDto::from(SnapshotMetadata {
            id: 7,
            context_token: "snapshot-token".to_string(),
            save_id: 3,
            custom_name: Some("Pre-season".to_string()),
            game_date: Some("2026-08-01".to_string()),
            game_date_source: "memory".to_string(),
            player_count: 25,
            loaded_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
            is_current: true,
        });
        let snapshot_value = serde_json::to_value(snapshot).expect("serialize snapshot metadata");
        assert_eq!(snapshot_value["contextToken"], "snapshot-token");
        assert_eq!(snapshot_value["isCurrent"], true);
        assert!(snapshot_value.get("players").is_none());

        let deleted = SnapshotDeleteResultDto::from(SnapshotDeleteResult {
            deleted_snapshot_id: 7,
            save_id: 3,
            current_snapshot_id: Some(6),
        });
        let deleted_value = serde_json::to_value(deleted).expect("serialize delete result");
        assert_eq!(deleted_value["deletedSnapshotId"], 7);
        assert_eq!(deleted_value["saveId"], 3);
        assert_eq!(deleted_value["currentSnapshotId"], 6);

        let deleted_save = SaveDeleteResultDto::from(SaveDeleteResult {
            deleted_save_id: 3,
            deleted_was_active: true,
            active_save: SaveSummary {
                id: 4,
                context_token: "fallback-token".to_string(),
                name: "Fallback".to_string(),
                is_active: true,
                created_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
                updated_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
            },
        });
        let deleted_save_value =
            serde_json::to_value(deleted_save).expect("serialize save delete result");
        assert_eq!(deleted_save_value["deletedSaveId"], 3);
        assert_eq!(deleted_save_value["deletedWasActive"], true);
        assert_eq!(deleted_save_value["activeSave"]["id"], 4);
    }
}
