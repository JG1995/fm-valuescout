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
pub struct SnapshotGameDateUpdateResultDto {
    pub snapshot: SnapshotMetadataDto,
    pub previous_current_snapshot_id: Option<i64>,
    pub current_snapshot_id: Option<i64>,
}

impl From<service::SnapshotGameDateUpdateResult> for SnapshotGameDateUpdateResultDto {
    fn from(result: service::SnapshotGameDateUpdateResult) -> Self {
        Self {
            snapshot: SnapshotMetadataDto::from(result.snapshot),
            previous_current_snapshot_id: result.previous_current_snapshot_id,
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
    let _context_guard = boost_gate::acquire_context_gate()?;
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
pub fn update_snapshot_game_date(
    snapshot_id: i64,
    context_token: String,
    game_date: String,
    db: State<'_, Db>,
) -> Result<SnapshotGameDateUpdateResultDto, String> {
    update_snapshot_game_date_for_command(db.inner(), snapshot_id, &context_token, &game_date)
}

pub(crate) fn update_snapshot_game_date_for_command(
    db: &Db,
    snapshot_id: i64,
    context_token: &str,
    game_date: &str,
) -> Result<SnapshotGameDateUpdateResultDto, String> {
    let _boost_guard = boost_gate::acquire_boost_gate()?;
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let result =
        service::update_snapshot_game_date(&mut conn, snapshot_id, context_token, game_date)?;
    Ok(SnapshotGameDateUpdateResultDto::from(result))
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

#[derive(Debug, Serialize)]
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

/// Closed Load Data phase. Wire values are camelCase literals forming a
/// monotonic ordered sequence: scan (indeterminate) -> preparing
/// (validation + raw normalization, determinate counts) -> scoring
/// (projection + compact scores, determinate) -> saving (raw DB inserts,
/// determinate) -> finalizing (selection, derived persistence,
/// Club DNA, academy, determinate). No overall percent is reported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LoadDataPhase {
    Scan,
    Preparing,
    Scoring,
    Saving,
    Finalizing,
}

/// Progress event for command-scoped best-effort delivery. Carries the
/// captured `save_id`/`context_token` so the frontend can ignore stale
/// events after a save switch. Optional `completed`/`total` are only present
/// when both values are truthful and `0 <= completed <= total`; scan
/// and preparing-start omit them. Events are monotonic in `phase` order and bounded
/// (at most one per phase transition, never per-row).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDataProgressDto {
    pub save_id: i64,
    pub context_token: String,
    pub phase: LoadDataPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

impl LoadDataProgressDto {
    fn indeterminate(save_id: i64, context_token: String, phase: LoadDataPhase) -> Self {
        Self {
            save_id,
            context_token,
            phase,
            completed: None,
            total: None,
        }
    }

    fn determinate(
        save_id: i64,
        context_token: String,
        phase: LoadDataPhase,
        completed: u64,
        total: u64,
    ) -> Self {
        debug_assert!(completed <= total);
        Self {
            save_id,
            context_token,
            phase,
            completed: Some(completed),
            total: Some(total),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDataTimingsDto {
    pub scan_ms: u64,
    pub prepare_ms: u64,
    pub scoring_ms: u64,
    pub save_ms: u64,
    pub finalize_ms: u64,
    pub total_ms: u64,
    pub ingest_ms: u64,
}

#[derive(Debug, Serialize)]
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
            prepare_ms: timings.prepare_ms,
            scoring_ms: timings.scoring_ms,
            save_ms: timings.save_ms,
            finalize_ms: timings.finalize_ms,
            total_ms: timings.total_ms,
            ingest_ms: timings.ingest_ms,
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

fn ch_send_best_effort(
    channel: &tauri::ipc::Channel<LoadDataProgressDto>,
    dto: LoadDataProgressDto,
) -> Result<(), ()> {
    channel.send(dto).map(|_| ()).map_err(|_| ())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_load_data_with<S, R, Sc, P, Pr>(
    db: &Db,
    requested: service::SaveContext,
    max_accepted: Option<i32>,
    scan: S,
    prepare_raw: R,
    score: Sc,
    publish: P,
    now_ms: &mut dyn FnMut() -> u64,
    mut on_progress: Pr,
) -> Result<LoadDataResultDto, LoadDataError>
where
    S: FnOnce(Option<i32>) -> Result<(TempPath, DumpRequestResult), LoadDataError>,
    R: FnOnce(&Path) -> Result<ingest::RawPreparedSnapshot, LoadDataError>,
    Sc: FnOnce(ingest::RawPreparedSnapshot) -> Result<ingest::PreparedSnapshot, LoadDataError>,
    P: FnOnce(
        &mut rusqlite::Connection,
        &service::SaveContext,
        ingest::PreparedSnapshot,
        &DumpRequestResult,
        &mut dyn FnMut() -> u64,
        &mut dyn FnMut(),
    ) -> Result<(load_data::LoadDataResult, u64, u64), LoadDataError>,
    Pr: FnMut(LoadDataProgressDto) -> Result<(), ()>,
{
    let _load_guard = boost_gate::acquire_load_gate().map_err(|message| LoadDataError::Scan {
        kind: "inProgress".to_string(),
        message,
    })?;
    let total_started = now_ms();
    let save_context = requested;
    // Verify requested context still exists with token and is still active, before scan.
    {
        let conn = db.0.lock().map_err(|_| LoadDataError::Scan {
            kind: "internal".to_string(),
            message: "database lock poisoned".to_string(),
        })?;
        let is_active: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND context_token = ?2 AND is_active = 1)",
                rusqlite::params![save_context.id, save_context.context_token],
                |row| row.get(0),
            )
            .map_err(|error| LoadDataError::Scan {
                kind: "internal".to_string(),
                message: error.to_string(),
            })?;
        if !is_active {
            return Err(LoadDataError::Scan {
                kind: "saveChanged".to_string(),
                message: "Save changed or no longer exists".to_string(),
            });
        }
    }
    // Scan start indeterminate
    let _ = on_progress(LoadDataProgressDto::indeterminate(
        save_context.id,
        save_context.context_token.clone(),
        LoadDataPhase::Scan,
    ));
    let scan_started = now_ms();
    let scan_result = scan(max_accepted);
    let scan_ms = now_ms().saturating_sub(scan_started);
    let (captured_dump_path, dump_result) = scan_result?;
    // Preparing start indeterminate (total unknown)
    let _ = on_progress(LoadDataProgressDto::indeterminate(
        save_context.id,
        save_context.context_token.clone(),
        LoadDataPhase::Preparing,
    ));
    let prepare_started = now_ms();
    let raw = prepare_raw(captured_dump_path.as_ref())?;
    let prepare_ms = now_ms().saturating_sub(prepare_started);
    let total_raw = (raw.players.len() + raw.staff.len()) as u64;
    let _ = on_progress(LoadDataProgressDto::determinate(
        save_context.id,
        save_context.context_token.clone(),
        LoadDataPhase::Preparing,
        total_raw,
        total_raw,
    ));
    // Scoring start 0/total
    let _ = on_progress(LoadDataProgressDto::determinate(
        save_context.id,
        save_context.context_token.clone(),
        LoadDataPhase::Scoring,
        0,
        total_raw,
    ));
    let scoring_started = now_ms();
    let prepared = score(raw)?;
    let scoring_ms = now_ms().saturating_sub(scoring_started);
    let total_entities = (prepared.players.len() + prepared.staff.len()) as u64;
    let _ = on_progress(LoadDataProgressDto::determinate(
        save_context.id,
        save_context.context_token.clone(),
        LoadDataPhase::Scoring,
        total_entities,
        total_entities,
    ));
    // Saving start 0/total before Db lock
    let _ = on_progress(LoadDataProgressDto::determinate(
        save_context.id,
        save_context.context_token.clone(),
        LoadDataPhase::Saving,
        0,
        total_entities,
    ));
    // Re-verify still active immediately before publishing, under the publishing lock.
    // This captures a context switch that succeeded while the load held its lease.
    let mut conn = db.0.lock().map_err(|_| LoadDataError::Scan {
        kind: "internal".to_string(),
        message: "database lock poisoned".to_string(),
    })?;
    {
        let is_active: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND context_token = ?2 AND is_active = 1)",
                rusqlite::params![save_context.id, save_context.context_token],
                |row| row.get(0),
            )
            .map_err(|error| LoadDataError::Ingest {
                message: error.to_string(),
            })?;
        if !is_active {
            return Err(LoadDataError::Ingest {
                message: "Save changed or no longer exists".to_string(),
            });
        }
    }
    let save_id = save_context.id;
    let context_token = save_context.context_token.clone();
    let mut boundary = || {
        let _ = on_progress(LoadDataProgressDto::determinate(
            save_id,
            context_token.clone(),
            LoadDataPhase::Saving,
            total_entities,
            total_entities,
        ));
        let _ = on_progress(LoadDataProgressDto::determinate(
            save_id,
            context_token.clone(),
            LoadDataPhase::Finalizing,
            0,
            1,
        ));
    };
    let (mut result, save_ms, finalize_ms) = publish(
        &mut conn,
        &save_context,
        prepared,
        &dump_result,
        now_ms,
        &mut boundary as &mut dyn FnMut(),
    )?;
    let _ = on_progress(LoadDataProgressDto::determinate(
        save_context.id,
        save_context.context_token.clone(),
        LoadDataPhase::Finalizing,
        1,
        1,
    ));
    let total_ms = now_ms().saturating_sub(total_started);
    let ingest_ms = save_ms.saturating_add(finalize_ms);
    result.timings = load_data::LoadDataTimings {
        scan_ms,
        prepare_ms,
        scoring_ms,
        save_ms,
        finalize_ms,
        total_ms,
        ingest_ms,
    };
    Ok(LoadDataResultDto::from(result))
}

#[tauri::command]
pub async fn load_data(
    save_id: i64,
    context_token: String,
    max_accepted: Option<i32>,
    db: State<'_, Db>,
    on_progress: tauri::ipc::Channel<LoadDataProgressDto>,
) -> Result<LoadDataResultDto, LoadDataError> {
    let start = Instant::now();
    let mut now_ms = move || start.elapsed().as_millis() as u64;
    let mut on_progress_cb = move |dto: LoadDataProgressDto| -> Result<(), ()> {
        ch_send_best_effort(&on_progress, dto)
    };
    let requested = service::SaveContext {
        id: save_id,
        context_token,
    };
    execute_load_data_with(
        db.inner(),
        requested,
        max_accepted,
        |limit| load_data::scan_dump_from_local_app_data(DumpWaitConfig::default(), limit),
        load_data::prepare_raw_for_publish,
        load_data::score_raw_for_publish,
        |conn, save_context, prepared, dump_result, now_ms, boundary| {
            load_data::publish_prepared_with_progress(
                conn,
                save_context,
                prepared,
                dump_result,
                now_ms,
                boundary,
            )
        },
        &mut now_ms,
        &mut on_progress_cb,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::io::Write;
    use std::rc::Rc;
    use std::sync::Mutex;

    use rusqlite::Connection;
    use tempfile::TempPath;

    use crate::db::{migrations, Db};
    use crate::features::memory_read::service::DumpRequestResult;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v8.json");

    fn migrated_db(path: &std::path::Path) -> Db {
        let conn = Connection::open(path).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        Db(Mutex::new(conn))
    }

    fn active_save_context(db: &Db) -> service::SaveContext {
        let guard = db.0.lock().expect("lock");
        let save = service::list_saves(&guard)
            .expect("list saves")
            .into_iter()
            .find(|s| s.is_active)
            .expect("active save");
        service::SaveContext {
            id: save.id,
            context_token: save.context_token,
        }
    }

    fn assert_load_rejected_before_scan(db: &Db, requested: service::SaveContext) {
        let scan_called = Cell::new(false);
        let mut events = Vec::new();
        let mut now_ms = || 0u64;
        let error = execute_load_data_with(
            db,
            requested,
            None,
            |_| -> Result<(TempPath, DumpRequestResult), LoadDataError> {
                scan_called.set(true);
                unreachable!("stale context must fail before scan")
            },
            |_: &Path| -> Result<ingest::RawPreparedSnapshot, LoadDataError> {
                unreachable!("stale context must fail before preparation")
            },
            |_: ingest::RawPreparedSnapshot| -> Result<ingest::PreparedSnapshot, LoadDataError> {
                unreachable!("stale context must fail before scoring")
            },
            |_: &mut rusqlite::Connection,
             _: &service::SaveContext,
             _: ingest::PreparedSnapshot,
             _: &DumpRequestResult,
             _: &mut dyn FnMut() -> u64,
             _: &mut dyn FnMut()| {
                unreachable!("stale context must fail before publication")
            },
            &mut now_ms,
            |event| {
                events.push(event);
                Ok(())
            },
        )
        .expect_err("stale context must fail");

        assert!(matches!(error, LoadDataError::Scan { kind, .. } if kind == "saveChanged"));
        assert!(!scan_called.get());
        assert!(events.is_empty());
    }

    #[test]
    fn load_data_progress_dto_serializes_camel_case_closed_phases_and_omits_counts() {
        let dto =
            LoadDataProgressDto::indeterminate(42, "ctx-123".to_string(), LoadDataPhase::Scan);
        let v = serde_json::to_value(&dto).expect("serialize scan");
        assert_eq!(v["saveId"], 42);
        assert_eq!(v["contextToken"], "ctx-123");
        assert_eq!(v["phase"], "scan");
        assert!(v.get("completed").is_none());
        assert!(v.get("total").is_none());
        for (phase, wire) in [
            (LoadDataPhase::Preparing, "preparing"),
            (LoadDataPhase::Scoring, "scoring"),
            (LoadDataPhase::Saving, "saving"),
            (LoadDataPhase::Finalizing, "finalizing"),
        ] {
            let dto = LoadDataProgressDto::determinate(1, "t".to_string(), phase, 2, 5);
            let v = serde_json::to_value(&dto).expect("serialize phase");
            assert_eq!(v["phase"], wire);
            assert_eq!(v["completed"], 2);
            assert_eq!(v["total"], 5);
        }
        // Both completed/total must be present together when determinate, None together when indeterminate.
        let indeterminate =
            LoadDataProgressDto::indeterminate(1, "t".to_string(), LoadDataPhase::Preparing);
        let v = serde_json::to_value(&indeterminate).expect("serialize preparing start");
        assert!(v.get("completed").is_none() && v.get("total").is_none());
    }

    #[test]
    fn load_data_timings_serialization_and_disjointness() {
        let timings = load_data::LoadDataTimings {
            scan_ms: 10,
            prepare_ms: 20,
            scoring_ms: 5,
            save_ms: 7,
            finalize_ms: 3,
            total_ms: 50,
            ingest_ms: 10,
        };
        assert!(
            timings.total_ms
                >= timings.scan_ms
                    + timings.prepare_ms
                    + timings.scoring_ms
                    + timings.save_ms
                    + timings.finalize_ms
        );
        assert_eq!(timings.ingest_ms, timings.save_ms + timings.finalize_ms);
        let dto = LoadDataTimingsDto::from(timings);
        let v = serde_json::to_value(dto).expect("serialize timings");
        assert_eq!(v["scanMs"], 10);
        assert_eq!(v["prepareMs"], 20);
        assert_eq!(v["scoringMs"], 5);
        assert_eq!(v["saveMs"], 7);
        assert_eq!(v["finalizeMs"], 3);
        assert_eq!(v["totalMs"], 50);
        assert_eq!(v["ingestMs"], 10);
    }

    #[test]
    fn production_orchestrator_emits_ordered_start_completion_events_with_captured_context_and_disjoint_timings(
    ) {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("orchestrator-order.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let (save_id, context_token) = {
            let guard = db.0.lock().expect("lock");
            let save = crate::features::snapshot::service::list_saves(&guard)
                .unwrap()
                .into_iter()
                .find(|s| s.is_active)
                .unwrap();
            (save.id, save.context_token)
        };
        // distinct nonuniform boundaries: total 1000, scan 1010-1055 (45), prepare 1060-1090 (30), scoring 1095-1155 (60), save 1160-1230 (70), finalize 1240-1275 (35), total 1350 (350)
        let ticks = Rc::new(RefCell::new(VecDeque::from([
            1000u64, 1010, 1055, 1060, 1090, 1095, 1155, 1160, 1230, 1240, 1275, 1350,
        ])));
        let mut now_ms = {
            let ticks = Rc::clone(&ticks);
            move || {
                ticks
                    .borrow_mut()
                    .pop_front()
                    .expect("clock tick exhausted - missing expected timing call")
            }
        };
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-progress".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |path: &Path| load_data::prepare_raw_for_publish(path);
        let score = |raw| load_data::score_raw_for_publish(raw);
        let publish = |conn: &mut rusqlite::Connection,
                       save_context: &service::SaveContext,
                       prepared: ingest::PreparedSnapshot,
                       dump_result: &DumpRequestResult,
                       now_ms: &mut dyn FnMut() -> u64,
                       boundary: &mut dyn FnMut()| {
            load_data::publish_prepared_with_progress(
                conn,
                save_context,
                prepared,
                dump_result,
                now_ms,
                boundary,
            )
        };
        let mut events = Vec::new();
        let on_progress = |dto: LoadDataProgressDto| {
            events.push(dto);
            Ok(())
        };
        let requested = active_save_context(&db);

        let result = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            on_progress,
        )
        .expect("load");
        // Exact canonical timings with distinct buckets - moving scoring start to prepare start would mismatch prepare/scoring values.
        assert_eq!(
            result.timings.scan_ms, 45,
            "scan_ms must be exact nonuniform bucket"
        );
        assert_eq!(result.timings.prepare_ms, 30, "prepare_ms must be exact");
        assert_eq!(result.timings.scoring_ms, 60, "scoring_ms must be exact");
        assert_eq!(result.timings.save_ms, 70, "save_ms must be exact");
        assert_eq!(result.timings.finalize_ms, 35, "finalize_ms must be exact");
        assert_eq!(
            result.timings.total_ms, 350,
            "total_ms must be exact wall-clock"
        );
        assert_eq!(
            result.timings.ingest_ms, 105,
            "ingest_ms must be exact save+finalize"
        );
        assert_eq!(
            result.timings.ingest_ms,
            result.timings.save_ms + result.timings.finalize_ms
        );
        assert!(
            result.timings.total_ms
                >= result.timings.scan_ms
                    + result.timings.prepare_ms
                    + result.timings.scoring_ms
                    + result.timings.save_ms
                    + result.timings.finalize_ms
        );
        // Bounded max 9 events, ordered, truthful counts
        assert!(events.len() <= 9, "max 9 events, got {}", events.len());
        let phases: Vec<_> = events.iter().map(|e| format!("{:?}", e.phase)).collect();
        assert_eq!(
            phases,
            vec![
                "Scan",
                "Preparing",
                "Preparing",
                "Scoring",
                "Scoring",
                "Saving",
                "Saving",
                "Finalizing",
                "Finalizing"
            ]
        );
        // Scan and Preparing start indeterminate
        assert!(events[0].completed.is_none() && events[0].total.is_none());
        assert!(events[1].completed.is_none() && events[1].total.is_none());
        // Preparing complete exact total
        assert_eq!(events[2].completed, Some(2));
        assert_eq!(events[2].total, Some(2));
        // Scoring start 0/total
        assert_eq!(events[3].completed, Some(0));
        assert_eq!(events[3].total, Some(2));
        // Scoring complete total/total
        assert_eq!(events[4].completed, Some(2));
        assert_eq!(events[4].total, Some(2));
        // Saving start 0/total before lock
        assert_eq!(events[5].completed, Some(0));
        assert_eq!(events[5].total, Some(2));
        // Saving complete total/total
        assert_eq!(events[6].completed, Some(2));
        assert_eq!(events[6].total, Some(2));
        // Finalizing start 0/1
        assert_eq!(events[7].completed, Some(0));
        assert_eq!(events[7].total, Some(1));
        // Finalizing complete 1/1 only after commit
        assert_eq!(events[8].completed, Some(1));
        assert_eq!(events[8].total, Some(1));
        // Captured context on all events
        for ev in &events {
            assert_eq!(ev.save_id, save_id);
            assert_eq!(ev.context_token, context_token);
        }
        // Every clock tick must be consumed - no fallback/unused tick masking ordering regressions.
        assert!(
            ticks.borrow().is_empty(),
            "all clock ticks must be consumed, leftover masks ordering regressions: {:?}",
            ticks.borrow()
        );
    }

    #[test]
    fn orchestrator_does_not_hold_db_lock_during_raw_prepare_and_scoring() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("lock-free.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-lock-free".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            // Db mutex must be free during scan
            assert!(db.0.try_lock().is_ok(), "Db mutex must be free during scan");
            Ok((path, dump_result))
        };
        let db_ref = &db;
        let prepare_raw = move |path: &Path| {
            assert!(
                db_ref.0.try_lock().is_ok(),
                "Db mutex must be free during raw-prepare"
            );
            load_data::prepare_raw_for_publish(path)
        };
        let db_ref2 = &db;
        let score = move |raw: ingest::RawPreparedSnapshot| {
            assert!(
                db_ref2.0.try_lock().is_ok(),
                "Db mutex must be free during scoring"
            );
            load_data::score_raw_for_publish(raw)
        };
        let publish = |conn: &mut Connection,
                       ctx: &service::SaveContext,
                       prepared: ingest::PreparedSnapshot,
                       dump_result: &DumpRequestResult,
                       now_ms: &mut dyn FnMut() -> u64,
                       boundary: &mut dyn FnMut()| {
            load_data::publish_prepared_with_progress(
                conn,
                ctx,
                prepared,
                dump_result,
                now_ms,
                boundary,
            )
        };
        let on_progress = |_: LoadDataProgressDto| Ok(());
        let requested = active_save_context(&db);

        let result = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            on_progress,
        )
        .expect("lock-free success");
        assert_eq!(result.request_id, "req-lock-free");
    }

    #[test]
    fn always_failed_sender_does_not_alter_successful_result_and_attempts_all_events() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("failed-sender.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-fail-sender".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |p: &Path| load_data::prepare_raw_for_publish(p);
        let score = |r| load_data::score_raw_for_publish(r);
        let publish = |c: &mut Connection,
                       ctx: &service::SaveContext,
                       p: ingest::PreparedSnapshot,
                       d: &DumpRequestResult,
                       n: &mut dyn FnMut() -> u64,
                       b: &mut dyn FnMut()| {
            load_data::publish_prepared_with_progress(c, ctx, p, d, n, b)
        };
        let attempts = Rc::new(Cell::new(0usize));
        let attempts_clone = Rc::clone(&attempts);
        let on_progress = move |_: LoadDataProgressDto| {
            attempts_clone.set(attempts_clone.get() + 1);
            Err(())
        };
        let requested = active_save_context(&db);

        let result = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            on_progress,
        )
        .expect("must succeed despite always-failing sender");
        assert_eq!(result.request_id, "req-fail-sender");
        assert_eq!(
            attempts.get(),
            9,
            "always-failing sender must be attempted for every bounded event but success unchanged"
        );
    }

    #[test]
    fn scan_error_exposes_only_scan_boundary() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("scan-error.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| -> Result<(TempPath, DumpRequestResult), LoadDataError> {
            Err(LoadDataError::Scan {
                kind: "timeout".to_string(),
                message: "scan failed".to_string(),
            })
        };
        let prepare_raw = |_: &Path| unreachable!();
        let score = |_: ingest::RawPreparedSnapshot| unreachable!();
        let publish = |_: &mut Connection,
                       _: &service::SaveContext,
                       _: ingest::PreparedSnapshot,
                       _: &DumpRequestResult,
                       _: &mut dyn FnMut() -> u64,
                       _: &mut dyn FnMut()| unreachable!();
        let mut events = Vec::new();
        let requested = active_save_context(&db);

        let err = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("scan must fail");
        assert!(matches!(err, LoadDataError::Scan { .. }));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].phase, LoadDataPhase::Scan);
    }

    #[test]
    fn raw_prepare_error_stops_at_preparing_start_boundary() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("raw-error.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(b"{\"schemaVersion\": 8}").expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-raw-fail".to_string(),
                state: "ready".to_string(),
                players_found: Some(0),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |path: &Path| load_data::prepare_raw_for_publish(path);
        let score = |_: ingest::RawPreparedSnapshot| unreachable!();
        let publish = |_: &mut Connection,
                       _: &service::SaveContext,
                       _: ingest::PreparedSnapshot,
                       _: &DumpRequestResult,
                       _: &mut dyn FnMut() -> u64,
                       _: &mut dyn FnMut()| unreachable!();
        let mut events = Vec::new();
        let requested = active_save_context(&db);

        let err = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("raw prepare must fail");
        assert!(matches!(err, LoadDataError::Ingest { .. }));
        assert!(events.iter().any(|e| e.phase == LoadDataPhase::Scan));
        assert!(events
            .iter()
            .any(|e| e.phase == LoadDataPhase::Preparing && e.completed.is_none()));
        assert!(!events.iter().any(|e| e.phase == LoadDataPhase::Scoring));
        assert!(!events.iter().any(|e| e.phase == LoadDataPhase::Saving));
    }

    #[test]
    fn scoring_error_stops_at_scoring_start_boundary() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("scoring-error.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-score-fail".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |path: &Path| load_data::prepare_raw_for_publish(path);
        let score = |mut raw: ingest::RawPreparedSnapshot| {
            // Force scoring failure by corrupting an attribute to 0 (out of 1..20)
            raw.players[0].attributes_json = r#"{"Acceleration": 0}"#.to_string();
            load_data::score_raw_for_publish(raw)
        };
        let publish = |_: &mut Connection,
                       _: &service::SaveContext,
                       _: ingest::PreparedSnapshot,
                       _: &DumpRequestResult,
                       _: &mut dyn FnMut() -> u64,
                       _: &mut dyn FnMut()| unreachable!();
        let mut events = Vec::new();
        let requested = active_save_context(&db);

        let err = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("scoring must fail");
        assert!(matches!(err, LoadDataError::Ingest { .. }));
        assert!(events
            .iter()
            .any(|e| e.phase == LoadDataPhase::Scoring && e.completed == Some(0)));
        assert!(!events.iter().any(|e| e.phase == LoadDataPhase::Saving));
    }

    #[test]
    fn publish_error_exposes_only_saving_start_boundary() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("publish-error.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-publish-fail".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |p: &Path| load_data::prepare_raw_for_publish(p);
        let score = |r| load_data::score_raw_for_publish(r);
        let publish = |_: &mut Connection,
                       _: &service::SaveContext,
                       _: ingest::PreparedSnapshot,
                       _: &DumpRequestResult,
                       _: &mut dyn FnMut() -> u64,
                       _: &mut dyn FnMut()| {
            Err(LoadDataError::Ingest {
                message: "publish failed".to_string(),
            })
        };
        let mut events = Vec::new();
        let requested = active_save_context(&db);

        let err = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("publish must fail");
        assert!(matches!(err, LoadDataError::Ingest { .. }));
        assert!(events
            .iter()
            .any(|e| e.phase == LoadDataPhase::Saving && e.completed == Some(0)));
        assert!(!events.iter().any(|e| e.phase == LoadDataPhase::Saving
            && e.completed == Some(2)
            && e.phase == LoadDataPhase::Saving));
        // Saving complete and Finalizing start not emitted
        let saving_complete = events
            .iter()
            .filter(|e| e.phase == LoadDataPhase::Saving && e.completed == Some(2))
            .count();
        assert_eq!(saving_complete, 0);
        assert!(!events
            .iter()
            .any(|e| e.phase == LoadDataPhase::Finalizing && e.completed == Some(1)));
    }

    #[test]
    fn finalization_error_via_failed_derived_persistence_exposes_saving_complete_and_finalizing_start_but_not_complete(
    ) {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("finalize-error.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
            // trigger will cause derived compact insert to fail, rolling back finalize
            guard.execute_batch("CREATE TRIGGER fail_compact BEFORE INSERT ON player_role_metrics BEGIN SELECT RAISE(ABORT, 'finalize fail'); END;").expect("trigger");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-finalize-fail".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |p: &Path| load_data::prepare_raw_for_publish(p);
        let score = |r| load_data::score_raw_for_publish(r);
        let publish = |conn: &mut Connection,
                       ctx: &service::SaveContext,
                       prepared: ingest::PreparedSnapshot,
                       dump_result: &DumpRequestResult,
                       now_ms: &mut dyn FnMut() -> u64,
                       boundary: &mut dyn FnMut()| {
            load_data::publish_prepared_with_progress(
                conn,
                ctx,
                prepared,
                dump_result,
                now_ms,
                boundary,
            )
        };
        let mut events = Vec::new();
        let requested = active_save_context(&db);

        let err = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("finalize must fail");
        assert!(matches!(err, LoadDataError::Ingest { .. }));
        // Saving complete and finalizing start were emitted via boundary inside transaction before failure
        assert!(events
            .iter()
            .any(|e| e.phase == LoadDataPhase::Saving && e.completed == Some(2)));
        assert!(events
            .iter()
            .any(|e| e.phase == LoadDataPhase::Finalizing && e.completed == Some(0)));
        assert!(!events
            .iter()
            .any(|e| e.phase == LoadDataPhase::Finalizing && e.completed == Some(1)));
    }

    #[test]
    fn raw_staff_insert_failure_via_canonical_publisher_rolls_back_and_exposes_only_saving_start_boundary(
    ) {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("raw-staff-failure.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
            // Fail during staff insertion so snapshot + player inserts already occurred before failure.
            // This proves on_save_boundary() was correctly after all raw inserts.
            guard
                .execute_batch(
                    "CREATE TRIGGER abort_staff_insert BEFORE INSERT ON staff BEGIN SELECT RAISE(ABORT, 'raw staff failure'); END;"
                )
                .expect("trigger");
        }
        let mut now_ms = || 0u64;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            tmp.write_all(GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-raw-staff-fail".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |p: &Path| load_data::prepare_raw_for_publish(p);
        let score = |r| load_data::score_raw_for_publish(r);
        let publish = |conn: &mut Connection,
                       ctx: &service::SaveContext,
                       prepared: ingest::PreparedSnapshot,
                       dump_result: &DumpRequestResult,
                       now_ms: &mut dyn FnMut() -> u64,
                       boundary: &mut dyn FnMut()| {
            load_data::publish_prepared_with_progress(
                conn,
                ctx,
                prepared,
                dump_result,
                now_ms,
                boundary,
            )
        };
        let mut events = Vec::new();
        let requested = active_save_context(&db);

        let err = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("raw staff insert must fail");
        assert!(
            matches!(err, LoadDataError::Ingest { .. }),
            "must be authoritative ingest error"
        );
        assert!(
            err.to_string().contains("raw staff failure"),
            "error must surface trigger failure, got: {}",
            err
        );
        // Transaction fully rolls back: no snapshot/player/staff rows.
        {
            let guard = db.0.lock().expect("lock");
            let snapshot_count: i64 = guard
                .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
                .expect("count snapshots");
            assert_eq!(snapshot_count, 0, "transaction must roll back snapshot row");
            let player_count: i64 = guard
                .query_row("SELECT COUNT(*) FROM players", [], |row| row.get(0))
                .expect("count players");
            assert_eq!(player_count, 0, "transaction must roll back player rows");
            let staff_count: i64 = guard
                .query_row("SELECT COUNT(*) FROM staff", [], |row| row.get(0))
                .expect("count staff");
            assert_eq!(staff_count, 0, "transaction must roll back staff rows");
        }
        // Event sequence reaches Saving 0/total but has no Saving complete and no Finalizing event.
        // If on_save_boundary() moved before all raw inserts, Saving complete would be emitted before the staff failure.
        assert!(
            events.iter().any(|e| e.phase == LoadDataPhase::Saving
                && e.completed == Some(0)
                && e.total == Some(2)),
            "must reach Saving 0/2 before DB lock, events: {:?}",
            events
                .iter()
                .map(|e| format!("{:?} {:?}/{:?}", e.phase, e.completed, e.total))
                .collect::<Vec<_>>()
        );
        let saving_complete = events
            .iter()
            .filter(|e| {
                e.phase == LoadDataPhase::Saving && e.completed == Some(2) && e.total == Some(2)
            })
            .count();
        assert_eq!(
            saving_complete, 0,
            "must have no Saving complete 2/2 after raw failure - boundary must be after all raw inserts, events: {:?}",
            events.iter().map(|e| format!("{:?} {:?}/{:?}", e.phase, e.completed, e.total)).collect::<Vec<_>>()
        );
        assert!(
            !events.iter().any(|e| e.phase == LoadDataPhase::Finalizing),
            "must have no Finalizing event after raw failure, events: {:?}",
            events
                .iter()
                .map(|e| format!("{:?}", e.phase))
                .collect::<Vec<_>>()
        );
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
                prepare_ms: 1,
                scoring_ms: 1,
                save_ms: 1,
                finalize_ms: 1,
                total_ms: 6,
                ingest_ms: 2,
            },
        })
        .expect("serialize Load Data result");
        assert_eq!(load_data["storedSnapshot"]["contextToken"], "stored-token");
        assert_eq!(
            load_data["effectiveSnapshot"]["contextToken"],
            "effective-token"
        );
        // Timings are camelCase
        assert_eq!(load_data["timings"]["scanMs"], 1);
        assert_eq!(load_data["timings"]["prepareMs"], 1);
        assert_eq!(load_data["timings"]["scoringMs"], 1);
        assert_eq!(load_data["timings"]["saveMs"], 1);
        assert_eq!(load_data["timings"]["finalizeMs"], 1);
        assert_eq!(load_data["timings"]["totalMs"], 6);
        assert_eq!(load_data["timings"]["ingestMs"], 2);
    }

    #[test]
    fn stale_frontend_context_fails_before_scan_and_never_publishes() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("stale-context.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let default_save = {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard)
                .unwrap()
                .into_iter()
                .find(|s| s.name == "Default save")
                .unwrap()
        };
        let second_save = {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::create_save(&guard, "Second")
                .expect("create second")
        };
        {
            let mut conn = db.0.lock().expect("lock");
            crate::features::snapshot::service::set_active_save(&mut conn, second_save.id)
                .expect("switch");
        }
        let stale_requested = service::SaveContext {
            id: default_save.id,
            context_token: default_save.context_token.clone(),
        };
        let mut now_ms = || 0u64;
        let scan_called = std::cell::Cell::new(false);
        let scan = |_: Option<i32>| {
            scan_called.set(true);
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            std::io::Write::write_all(&mut tmp, GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-stale".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |path: &Path| load_data::prepare_raw_for_publish(path);
        let score = |raw| load_data::score_raw_for_publish(raw);
        let publish = |conn: &mut rusqlite::Connection,
                       ctx: &service::SaveContext,
                       prepared: ingest::PreparedSnapshot,
                       dump_result: &DumpRequestResult,
                       now_ms: &mut dyn FnMut() -> u64,
                       boundary: &mut dyn FnMut()| {
            load_data::publish_prepared_with_progress(
                conn,
                ctx,
                prepared,
                dump_result,
                now_ms,
                boundary,
            )
        };
        let mut events = Vec::new();
        let err = execute_load_data_with(
            &db,
            stale_requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("stale A vs active B must fail before scan");
        assert!(matches!(err, LoadDataError::Scan { kind, .. } if kind == "saveChanged"));
        assert!(!scan_called.get(), "must fail before bridge scan");
        assert!(
            events.is_empty(),
            "must emit no progress when stale before scan"
        );
        {
            let guard = db.0.lock().expect("lock");
            let count: i64 = guard
                .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
                .expect("count");
            assert_eq!(count, 0, "stale request must never publish");
        }
    }

    #[test]
    fn replaced_active_save_token_fails_before_scan() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("replaced-token.db"));
        let requested = active_save_context(&db);
        {
            let conn = db.0.lock().expect("lock");
            conn.execute("DELETE FROM saves WHERE id = ?1", [requested.id])
                .expect("delete captured save");
            conn.execute(
                "INSERT INTO saves (id, name, is_active, context_token)
                 VALUES (?1, 'Replacement save', 1, 'replacement-token')",
                [requested.id],
            )
            .expect("reuse active save id with a new token");
        }

        assert_load_rejected_before_scan(&db, requested);
    }

    #[test]
    fn mid_flight_context_switch_during_prepare_fails_at_pre_publish_without_snapshots_and_leaves_b_active(
    ) {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("mid-flight.db"));
        {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard).expect("seed");
        }
        let default_save = {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::list_saves(&guard)
                .unwrap()
                .into_iter()
                .find(|s| s.name == "Default save")
                .unwrap()
        };
        let second_save = {
            let guard = db.0.lock().expect("lock");
            crate::features::snapshot::service::create_save(&guard, "Second")
                .expect("create second")
        };
        // A is still active; request captures A with token
        let requested = service::SaveContext {
            id: default_save.id,
            context_token: default_save.context_token.clone(),
        };
        let mut now_ms = || 0u64;
        let publish_called = Rc::new(Cell::new(false));
        let publish_called_clone = Rc::clone(&publish_called);
        let second_id = second_save.id;
        let scan = |_: Option<i32>| {
            let mut tmp = tempfile::NamedTempFile::new().expect("tmp");
            std::io::Write::write_all(&mut tmp, GOLDEN_FIXTURE.as_bytes()).expect("write");
            tmp.flush().expect("flush");
            let path = tmp.into_temp_path();
            let dump_result = DumpRequestResult {
                request_id: "req-mid-flight".to_string(),
                state: "ready".to_string(),
                players_found: Some(1),
                dump_present: true,
                error: None,
                scan_truncated: Some(false),
                max_accepted: None,
            };
            Ok((path, dump_result))
        };
        let prepare_raw = |path: &Path| {
            // Real A→B switch while load lease held must succeed (context gate coexists with load gate).
            let switched =
                crate::features::snapshot::commands::set_active_save_for_command(&db, second_id)
                    .expect("context switch must succeed while load lease held");
            assert_eq!(switched.id, second_id);
            {
                let guard = db.0.lock().expect("lock");
                let active = crate::features::snapshot::service::list_saves(&guard)
                    .unwrap()
                    .into_iter()
                    .find(|s| s.is_active)
                    .unwrap();
                assert_eq!(
                    active.id, second_id,
                    "B must be active after mid-flight switch"
                );
            }
            load_data::prepare_raw_for_publish(path)
        };
        let score = |raw| load_data::score_raw_for_publish(raw);
        let publish = move |conn: &mut rusqlite::Connection,
                            ctx: &service::SaveContext,
                            prepared: ingest::PreparedSnapshot,
                            dump_result: &DumpRequestResult,
                            now_ms: &mut dyn FnMut() -> u64,
                            boundary: &mut dyn FnMut()| {
            publish_called_clone.set(true);
            load_data::publish_prepared_with_progress(
                conn,
                ctx,
                prepared,
                dump_result,
                now_ms,
                boundary,
            )
        };
        let mut events = Vec::new();
        let err = execute_load_data_with(
            &db,
            requested,
            None,
            scan,
            prepare_raw,
            score,
            publish,
            &mut now_ms,
            |dto| {
                events.push(dto);
                Ok(())
            },
        )
        .expect_err("mid-flight A→B must fail at pre-publication revalidation");
        assert!(
            matches!(err, LoadDataError::Ingest { .. }) && err.to_string().contains("Save changed"),
            "must fail at pre-publish revalidation with Save changed, got: {err:?}"
        );
        assert!(
            !publish_called.get(),
            "publish callback must never be invoked when pre-publish revalidation fails"
        );
        // No progress beyond saving start is emitted when publish never runs, but any emitted events carry A.
        for ev in &events {
            assert_eq!(ev.save_id, default_save.id);
            assert_eq!(ev.context_token, default_save.context_token);
        }
        {
            let guard = db.0.lock().expect("lock");
            let count: i64 = guard
                .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
                .expect("count");
            assert_eq!(count, 0, "mid-flight switch must never write snapshots");
            let active = crate::features::snapshot::service::list_saves(&guard)
                .unwrap()
                .into_iter()
                .find(|s| s.is_active)
                .unwrap();
            assert_eq!(
                active.id, second_id,
                "B must remain active after failed A load"
            );
        }
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

    fn insert_date_edit_snapshot(db: &Db, save_id: i64, game_date: Option<&str>) -> (i64, String) {
        let guard = db.0.lock().expect("lock db");
        guard
            .execute(
                "INSERT INTO snapshots (
                    save_id, is_current, schema_version, generated_at_utc, game_version,
                    supported_game_version, bridge_version, protocol_version, game_date,
                    game_date_source, scan_truncated, max_accepted, player_count, loaded_at_utc
                 ) VALUES (
                    ?1, 0, 6, '2026-08-11T10:00:00.000Z', '26.3.2', '26.3', '0.1.0', 1, ?2,
                    'memory', 0, NULL, 1, '2026-08-11T10:00:00.000Z'
                 )",
                rusqlite::params![save_id, game_date],
            )
            .expect("insert snapshot");
        let snapshot_id = guard.last_insert_rowid();
        let context_token: String = guard
            .query_row(
                "SELECT context_token FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("read snapshot token");
        (snapshot_id, context_token)
    }

    #[test]
    fn update_result_dto_serializes_named_metadata_and_camel_case_ids() {
        let dto = SnapshotGameDateUpdateResultDto::from(service::SnapshotGameDateUpdateResult {
            snapshot: service::SnapshotMetadata {
                id: 7,
                context_token: "snapshot-token".to_string(),
                save_id: 3,
                custom_name: Some("Pre-season".to_string()),
                game_date: Some("2024-02-29".to_string()),
                game_date_source: "memory".to_string(),
                player_count: 2,
                loaded_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
                is_current: true,
            },
            previous_current_snapshot_id: Some(9),
            current_snapshot_id: Some(7),
        });

        let value = serde_json::to_value(&dto).expect("serialize update result");
        assert_eq!(value["snapshot"]["id"], 7);
        assert_eq!(value["snapshot"]["gameDate"], "2024-02-29");
        assert_eq!(value["snapshot"]["contextToken"], "snapshot-token");
        assert_eq!(value["previousCurrentSnapshotId"], 9);
        assert_eq!(value["currentSnapshotId"], 7);
        assert!(value.get("previous_current_snapshot_id").is_none());
        assert!(value.get("current_snapshot_id").is_none());
    }

    #[test]
    fn update_command_holds_the_boost_gate_exclusion() {
        let _guard = crate::features::player::boost_gate::BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = migrated_db(&temp_dir.path().join("date-edit-gate.db"));
        let save_id = {
            let guard = db.0.lock().expect("lock db");
            service::list_saves(&guard)
                .expect("seed default save")
                .into_iter()
                .find(|save| save.is_active)
                .expect("active save")
                .id
        };
        let (snapshot_id, context_token) =
            insert_date_edit_snapshot(&db, save_id, Some("2026-01-01"));

        let held =
            crate::features::player::boost_gate::acquire_boost_gate().expect("hold boost gate");
        let error = match update_snapshot_game_date_for_command(
            &db,
            snapshot_id,
            &context_token,
            "2026-03-01",
        ) {
            Ok(_) => panic!("boost-held edit must fail"),
            Err(error) => error,
        };
        assert!(
            error.contains("already in progress"),
            "unexpected gate error: {error}"
        );
        let stored: Option<String> =
            db.0.lock()
                .expect("lock db")
                .query_row(
                    "SELECT game_date FROM snapshots WHERE id = ?1",
                    [snapshot_id],
                    |row| row.get(0),
                )
                .expect("read game date");
        assert_eq!(stored.as_deref(), Some("2026-01-01"));
        drop(held);

        let result =
            update_snapshot_game_date_for_command(&db, snapshot_id, &context_token, "2026-03-01")
                .expect("edit succeeds once the gate is free");
        assert_eq!(result.snapshot.game_date.as_deref(), Some("2026-03-01"));
        assert_eq!(result.current_snapshot_id, Some(snapshot_id));
    }
}
