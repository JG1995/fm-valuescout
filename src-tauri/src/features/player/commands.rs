use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::features::memory_read::service::{
    request_player_boost_from_local_app_data, DumpWaitConfig, PlayerBoostResult,
};
use crate::features::planner::{service as planner_service, squad as planner_squad};

use super::boost_gate;
use super::query::{self, PlayerDetail, PlayerRoleScore};
use super::service::{
    self, PlayerBoostContext, PlayerBoostError, PreparedPlayerBoost, VerifiedPlayerBoost,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRoleScoreDto {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_tags: Vec<String>,
    pub score: Option<i64>,
    pub potential_score: Option<i64>,
}

impl From<PlayerRoleScore> for PlayerRoleScoreDto {
    fn from(row: PlayerRoleScore) -> Self {
        Self {
            role_id: row.role_id,
            display_name: row.display_name,
            phase: row.phase,
            position_tags: row.position_tags,
            score: row.score,
            potential_score: row.potential_score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDetailDto {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub birth_year: i64,
    pub birth_day_of_year: i64,
    pub nationalities: Vec<String>,
    pub height_cm: Option<i64>,
    pub preferred_foot: String,
    pub positions: std::collections::BTreeMap<String, Option<i64>>,
    pub attributes: std::collections::BTreeMap<String, Option<i64>>,
    pub potential_attributes: std::collections::BTreeMap<String, Option<i64>>,
    pub hidden_attributes: std::collections::BTreeMap<String, Option<i64>>,
    pub personality: std::collections::BTreeMap<String, Option<i64>>,
    pub weekly_wage_gbp: Option<i64>,
    pub contract_expiry_year: Option<i64>,
    pub contract_expiry_day_of_year: Option<i64>,
    pub transfer_listed: Option<bool>,
    pub loan_listed: Option<bool>,
    pub not_for_sale: Option<bool>,
    pub set_for_release: Option<bool>,
    pub market_value_gbp: Option<i64>,
    pub reputation_current: Option<i64>,
    pub reputation_world: Option<i64>,
    pub club: Option<String>,
    pub parent_club: Option<String>,
    pub on_loan: Option<bool>,
    pub division: Option<String>,
    pub team_level: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub hidden_information_revealed: bool,
    pub role_scores: Vec<PlayerRoleScoreDto>,
}

impl From<PlayerDetail> for PlayerDetailDto {
    fn from(player: PlayerDetail) -> Self {
        Self {
            uid: player.uid,
            name: player.name,
            age: player.age,
            birth_year: player.birth_year,
            birth_day_of_year: player.birth_day_of_year,
            nationalities: player.nationalities,
            height_cm: player.height_cm,
            preferred_foot: player.preferred_foot,
            positions: player.positions,
            attributes: player.attributes,
            potential_attributes: player.potential_attributes,
            hidden_attributes: player.hidden_attributes,
            personality: player.personality,
            weekly_wage_gbp: player.weekly_wage_gbp,
            contract_expiry_year: player.contract_expiry_year,
            contract_expiry_day_of_year: player.contract_expiry_day_of_year,
            transfer_listed: player.transfer_listed,
            loan_listed: player.loan_listed,
            not_for_sale: player.not_for_sale,
            set_for_release: player.set_for_release,
            market_value_gbp: player.market_value_gbp,
            reputation_current: player.reputation_current,
            reputation_world: player.reputation_world,
            club: player.club,
            parent_club: player.parent_club,
            on_loan: player.on_loan,
            division: player.division,
            team_level: player.team_level,
            ca: player.ca,
            pa: player.pa,
            hidden_information_revealed: player.hidden_information_revealed,
            role_scores: player
                .role_scores
                .into_iter()
                .map(PlayerRoleScoreDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerBoostResultDto {
    pub snapshot_id: i64,
    pub operation: String,
    pub previous_current_ability: Option<i64>,
    pub current_ability: Option<i64>,
    pub potential_ability: Option<i64>,
    pub previous_ambition: Option<i64>,
    pub ambition: Option<i64>,
    pub previous_professionalism: Option<i64>,
    pub professionalism: Option<i64>,
    pub previous_determination: Option<i64>,
    pub determination: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadPlayerBoostResultDto {
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
    pub recovery_required: bool,
    pub recovery_message: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadPlayerBoostProgressDto {
    pub processed: usize,
    pub total: usize,
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl From<VerifiedPlayerBoost> for PlayerBoostResultDto {
    fn from(result: VerifiedPlayerBoost) -> Self {
        Self {
            snapshot_id: result.snapshot_id,
            operation: result.operation,
            previous_current_ability: result.previous_current_ability,
            current_ability: result.current_ability,
            potential_ability: result.potential_ability,
            previous_ambition: result.previous_ambition,
            ambition: result.ambition,
            previous_professionalism: result.previous_professionalism,
            professionalism: result.professionalism,
            previous_determination: result.previous_determination,
            determination: result.determination,
        }
    }
}

/// Query key for frontend cache: `["player", uid]` — invalidate with snapshot/save keys
/// when Load Data or set_active_save runs (wired in a later commit).
#[tauri::command]
pub fn get_player(uid: i64, db: State<'_, Db>) -> Result<Option<PlayerDetailDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let player = query::get_player(&conn, uid)?;
    Ok(player.map(PlayerDetailDto::from))
}

#[tauri::command]
pub fn set_player_hidden_information_revealed(
    revealed: bool,
    db: State<'_, Db>,
) -> Result<bool, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    service::set_player_hidden_information_revealed(&conn, revealed)
}

#[tauri::command]
pub fn boost_current_ability(
    uid: i64,
    db: State<'_, Db>,
) -> Result<PlayerBoostResultDto, PlayerBoostError> {
    execute_player_boost(uid, db.inner(), service::prepare_current_ability_boost)
}

#[tauri::command]
pub fn boost_wonderkid_mentality(
    uid: i64,
    db: State<'_, Db>,
) -> Result<PlayerBoostResultDto, PlayerBoostError> {
    execute_player_boost(uid, db.inner(), service::prepare_wonderkid_mentality_boost)
}

#[tauri::command]
pub fn boost_squad_current_ability(
    db: State<'_, Db>,
    on_progress: tauri::ipc::Channel<SquadPlayerBoostProgressDto>,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError> {
    execute_squad_current_ability_boost_with_progress(
        db.inner(),
        request_local_player_boost,
        move |progress| match on_progress.send(progress) {
            Ok(()) => true,
            Err(error) => {
                log::debug!("squad boost progress delivery failed: {error}");
                false
            }
        },
    )
}

#[tauri::command]
pub fn boost_squad_wonderkid_mentality(
    db: State<'_, Db>,
    on_progress: tauri::ipc::Channel<SquadPlayerBoostProgressDto>,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError> {
    execute_squad_wonderkid_mentality_boost_with_progress(
        db.inner(),
        request_local_player_boost,
        move |progress| match on_progress.send(progress) {
            Ok(()) => true,
            Err(error) => {
                log::debug!("squad boost progress delivery failed: {error}");
                false
            }
        },
    )
}

fn execute_player_boost(
    uid: i64,
    db: &Db,
    prepare: fn(&rusqlite::Connection, i64) -> Result<PreparedPlayerBoost, PlayerBoostError>,
) -> Result<PlayerBoostResultDto, PlayerBoostError> {
    execute_player_boost_with(uid, db, prepare, request_local_player_boost)
}

fn execute_player_boost_with<F>(
    uid: i64,
    db: &Db,
    prepare: fn(&rusqlite::Connection, i64) -> Result<PreparedPlayerBoost, PlayerBoostError>,
    request_bridge_boost: F,
) -> Result<PlayerBoostResultDto, PlayerBoostError>
where
    F: FnOnce(&PreparedPlayerBoost) -> Result<PlayerBoostResult, PlayerBoostError>,
{
    let _boost_guard = acquire_player_boost_gate()?;
    let prepared = prepare_player_boost(uid, db, prepare, None)?;
    match request_and_reconcile_player_boost(&prepared, db, request_bridge_boost) {
        Ok(result) => Ok(result),
        Err(error @ PlayerBoostError::LiveValue { .. }) => Err(error),
        Err(error) => {
            mark_player_boost_recovery_required(db, &prepared.context())?;
            Err(error)
        }
    }
}

fn request_local_player_boost(
    prepared: &PreparedPlayerBoost,
) -> Result<PlayerBoostResult, PlayerBoostError> {
    request_player_boost_from_local_app_data(
        &prepared.source_request_id,
        prepared.player_uid,
        prepared.expected_current_ability as i32,
        prepared.expected_potential_ability as i32,
        prepared.bridge_operation(),
        DumpWaitConfig::default(),
    )
    .map_err(service::map_bridge_error)
}

#[cfg(test)]
fn execute_squad_current_ability_boost_with<F>(
    db: &Db,
    request_bridge_boost: F,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError>
where
    F: FnMut(&PreparedPlayerBoost) -> Result<PlayerBoostResult, PlayerBoostError>,
{
    execute_squad_current_ability_boost_with_progress(db, request_bridge_boost, |_| true)
}

fn execute_squad_current_ability_boost_with_progress<F, R>(
    db: &Db,
    request_bridge_boost: F,
    on_progress: R,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError>
where
    F: FnMut(&PreparedPlayerBoost) -> Result<PlayerBoostResult, PlayerBoostError>,
    R: FnMut(SquadPlayerBoostProgressDto) -> bool,
{
    execute_squad_player_boost_with(
        db,
        service::prepare_current_ability_boost,
        request_bridge_boost,
        on_progress,
    )
}

#[cfg(test)]
fn execute_squad_wonderkid_mentality_boost_with<F>(
    db: &Db,
    request_bridge_boost: F,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError>
where
    F: FnMut(&PreparedPlayerBoost) -> Result<PlayerBoostResult, PlayerBoostError>,
{
    execute_squad_wonderkid_mentality_boost_with_progress(db, request_bridge_boost, |_| true)
}

fn execute_squad_wonderkid_mentality_boost_with_progress<F, R>(
    db: &Db,
    request_bridge_boost: F,
    on_progress: R,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError>
where
    F: FnMut(&PreparedPlayerBoost) -> Result<PlayerBoostResult, PlayerBoostError>,
    R: FnMut(SquadPlayerBoostProgressDto) -> bool,
{
    execute_squad_player_boost_with(
        db,
        service::prepare_wonderkid_mentality_boost,
        request_bridge_boost,
        on_progress,
    )
}

fn execute_squad_player_boost_with<F, R>(
    db: &Db,
    prepare: fn(&rusqlite::Connection, i64) -> Result<PreparedPlayerBoost, PlayerBoostError>,
    mut request_bridge_boost: F,
    mut on_progress: R,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError>
where
    F: FnMut(&PreparedPlayerBoost) -> Result<PlayerBoostResult, PlayerBoostError>,
    R: FnMut(SquadPlayerBoostProgressDto) -> bool,
{
    let _boost_guard = acquire_player_boost_gate()?;
    let (context, player_uids) = {
        let conn = db.0.lock().map_err(|_| PlayerBoostError::Eligibility {
            kind: "databaseUnavailable".to_string(),
            message: "could not read the current snapshot for this squad boost".to_string(),
        })?;
        capture_squad_player_boost_cohort(&conn)?
    };
    let mut result = SquadPlayerBoostResultDto {
        updated: 0,
        skipped: 0,
        failed: 0,
        recovery_required: false,
        recovery_message: None,
    };
    let total = player_uids.len();
    report_squad_player_boost_progress(&mut on_progress, &result, total);

    for uid in player_uids {
        let prepared = match prepare_player_boost(uid, db, prepare, Some(&context)) {
            Ok(prepared) => prepared,
            Err(error) if is_skippable_squad_player_boost_eligibility(&error) => {
                result.skipped += 1;
                report_squad_player_boost_progress(&mut on_progress, &result, total);
                continue;
            }
            Err(error) => return recovery_required_result(db, &context, result, error),
        };

        if let Err(error) = ensure_squad_boost_context_is_current(db, &context) {
            return recovery_required_result(db, &context, result, error);
        }

        match request_and_reconcile_player_boost(&prepared, db, |prepared| {
            request_bridge_boost(prepared)
        }) {
            Ok(_) => {
                result.updated += 1;
                report_squad_player_boost_progress(&mut on_progress, &result, total);
            }
            Err(PlayerBoostError::LiveValue { .. }) => {
                result.failed += 1;
                report_squad_player_boost_progress(&mut on_progress, &result, total);
            }
            Err(error) => return recovery_required_result(db, &context, result, error),
        }
    }

    Ok(result)
}

fn report_squad_player_boost_progress<R>(
    on_progress: &mut R,
    result: &SquadPlayerBoostResultDto,
    total: usize,
) where
    R: FnMut(SquadPlayerBoostProgressDto) -> bool,
{
    let progress = SquadPlayerBoostProgressDto {
        processed: result.updated + result.skipped + result.failed,
        total,
        updated: result.updated,
        skipped: result.skipped,
        failed: result.failed,
    };
    let _ = on_progress(progress);
}

fn acquire_player_boost_gate() -> Result<std::sync::MutexGuard<'static, ()>, PlayerBoostError> {
    boost_gate::acquire_player_boost_gate().map_err(|message| PlayerBoostError::Bridge {
        kind: "inProgress".to_string(),
        message,
    })
}

fn capture_squad_player_boost_cohort(
    conn: &rusqlite::Connection,
) -> Result<(PlayerBoostContext, Vec<i64>), PlayerBoostError> {
    let context = service::capture_active_player_boost_context(conn)?;
    let club_family = planner_service::get_club_family(conn, context.save_id).map_err(|_| {
        PlayerBoostError::Eligibility {
            kind: "database".to_string(),
            message: "could not read the configured club family for this squad boost".to_string(),
        }
    })?;
    if club_family.primary_club.is_none() {
        return Err(PlayerBoostError::Eligibility {
            kind: "clubFamilyRequired".to_string(),
            message: "Set up your club family in Dashboard before boosting the squad.".to_string(),
        });
    }
    let player_uids =
        planner_squad::list_squad_player_uids(conn, context.save_id, context.snapshot_id).map_err(
            |_| PlayerBoostError::Eligibility {
                kind: "database".to_string(),
                message: "could not read the current squad for this boost".to_string(),
            },
        )?;
    Ok((context, player_uids))
}

fn prepare_player_boost(
    uid: i64,
    db: &Db,
    prepare: fn(&rusqlite::Connection, i64) -> Result<PreparedPlayerBoost, PlayerBoostError>,
    expected_context: Option<&PlayerBoostContext>,
) -> Result<PreparedPlayerBoost, PlayerBoostError> {
    let conn = db.0.lock().map_err(|_| PlayerBoostError::Eligibility {
        kind: "databaseUnavailable".to_string(),
        message: "could not read the current snapshot for this player boost".to_string(),
    })?;
    if let Some(expected_context) = expected_context {
        ensure_squad_boost_context(&conn, expected_context)?;
    }
    prepare(&conn, uid)
}

fn ensure_squad_boost_context_is_current(
    db: &Db,
    expected_context: &PlayerBoostContext,
) -> Result<(), PlayerBoostError> {
    let conn = db.0.lock().map_err(|_| PlayerBoostError::SnapshotSync {
        message:
            "The active save or snapshot changed. Load Data again before continuing the squad boost."
                .to_string(),
    })?;
    ensure_squad_boost_context(&conn, expected_context)
}

fn ensure_squad_boost_context(
    conn: &rusqlite::Connection,
    expected_context: &PlayerBoostContext,
) -> Result<(), PlayerBoostError> {
    let matches = service::active_player_boost_context_matches(conn, expected_context).map_err(
        |_| PlayerBoostError::SnapshotSync {
            message:
                "The active save or snapshot changed. Load Data again before continuing the squad boost."
                    .to_string(),
        },
    )?;
    if matches {
        Ok(())
    } else {
        Err(PlayerBoostError::SnapshotSync {
            message:
                "The active save or snapshot changed. Load Data again before continuing the squad boost."
                    .to_string(),
        })
    }
}

fn mark_player_boost_recovery_required(
    db: &Db,
    context: &PlayerBoostContext,
) -> Result<(), PlayerBoostError> {
    let conn = db.0.lock().map_err(|_| PlayerBoostError::SnapshotSync {
        message:
            "FM may have changed, but FM ValueScout could not preserve the recovery requirement. Load Data again."
                .to_string(),
    })?;
    service::require_load_data_for_player_boost(&conn, context)
}

fn request_and_reconcile_player_boost<F>(
    prepared: &PreparedPlayerBoost,
    db: &Db,
    request_bridge_boost: F,
) -> Result<PlayerBoostResultDto, PlayerBoostError>
where
    F: FnOnce(&PreparedPlayerBoost) -> Result<PlayerBoostResult, PlayerBoostError>,
{
    let bridge_result = request_bridge_boost(prepared)?;

    let mut conn = db.0.lock().map_err(|_| PlayerBoostError::SnapshotSync {
        message:
            "FM may have changed, but FM ValueScout could not update its snapshot. Load Data again."
                .to_string(),
    })?;
    service::reconcile_verified_boost(&mut conn, prepared, bridge_result)
        .map(PlayerBoostResultDto::from)
}

fn is_skippable_squad_player_boost_eligibility(error: &PlayerBoostError) -> bool {
    matches!(
        error,
            PlayerBoostError::Eligibility { kind, .. }
            if matches!(
                kind.as_str(),
                "unknownAge"
                    | "ageIneligible"
                    | "currentAbilityAtLimit"
                    | "noEligibleMentality"
            )
    )
}

fn recovery_required_result(
    db: &Db,
    context: &PlayerBoostContext,
    mut result: SquadPlayerBoostResultDto,
    error: PlayerBoostError,
) -> Result<SquadPlayerBoostResultDto, PlayerBoostError> {
    mark_player_boost_recovery_required(db, context)?;
    result.recovery_required = true;
    result.recovery_message = Some(error.to_string());
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::sync::Mutex;

    use rusqlite::{params, Connection};

    use super::*;
    use crate::db::migrations;
    use crate::features::memory_read::service::{
        PlayerBoostRequestError, PlayerBoostResult, OPERATION_BOOST_CURRENT_ABILITY,
    };
    use crate::features::planner::service as planner_service;
    use crate::features::snapshot::commands as snapshot_commands;
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service as snapshot_service;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v8.json");
    static PLAYER_BOOST_TEST_GATE: Mutex<()> = Mutex::new(());

    fn seeded_db() -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("player-boost-command.db");
        let mut conn = Connection::open(&db_path).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        let dump_path = temp_dir.path().join("player.json");
        std::fs::write(&dump_path, GOLDEN_FIXTURE).expect("write dump");
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest player");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = ?1 WHERE id = ?2",
            params!["scan-player-1", snapshot.id],
        )
        .expect("bind source request");

        (temp_dir, Db(Mutex::new(conn)))
    }

    fn seeded_history_db() -> (tempfile::TempDir, Db, i64, i64) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("player-boost-history-command.db");
        let mut conn = Connection::open(&db_path).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");

        let mut later_dump: serde_json::Value =
            serde_json::from_str(GOLDEN_FIXTURE).expect("parse golden fixture");
        later_dump["gameDate"] = serde_json::Value::from("2027-08-16");
        let later_path = temp_dir.path().join("later.json");
        std::fs::write(&later_path, later_dump.to_string()).expect("write later dump");
        let current = ingest_dump_file(&mut conn, &later_path).expect("ingest later dump");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = ?1 WHERE id = ?2",
            params!["R2", current.id],
        )
        .expect("bind current source request");

        let mut earlier_dump = later_dump;
        earlier_dump["gameDate"] = serde_json::Value::from("2026-08-14");
        let earlier_path = temp_dir.path().join("earlier.json");
        std::fs::write(&earlier_path, earlier_dump.to_string()).expect("write earlier dump");
        let historical = ingest_dump_file(&mut conn, &earlier_path).expect("ingest earlier dump");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = ?1 WHERE id = ?2",
            params!["R1", historical.id],
        )
        .expect("bind historical source request");

        (temp_dir, Db(Mutex::new(conn)), current.id, historical.id)
    }

    fn seeded_squad_db() -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("squad-boost-command.db");
        let mut conn = Connection::open(&db_path).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");

        let mut dump: serde_json::Value =
            serde_json::from_str(GOLDEN_FIXTURE).expect("parse golden fixture");
        let original = dump["players"][0].clone();
        let player = |uid: i64, age: i64, ca: i64, pa: i64, club: &str| {
            let mut player = original.clone();
            player["uid"] = serde_json::Value::from(uid);
            player["name"] = serde_json::Value::from(format!("Squad Player {uid}"));
            player["age"] = serde_json::Value::from(age);
            player["ca"] = serde_json::Value::from(ca);
            player["pa"] = serde_json::Value::from(pa);
            player["currentClub"] = serde_json::Value::from(club);
            player
        };
        dump["players"] = serde_json::Value::Array(vec![
            player(77, 20, 150, 170, "Loan FC"),
            player(78, 21, 150, 170, "Loan FC"),
            player(79, 28, 195, 200, "Loan FC"),
            player(80, 29, 150, 170, "Loan FC"),
            player(81, 20, 170, 170, "Loan FC"),
            player(82, 20, 150, 170, "Elsewhere FC"),
        ]);
        dump["playerCount"] = serde_json::Value::from(6);
        let dump_path = temp_dir.path().join("squad-players.json");
        std::fs::write(
            &dump_path,
            serde_json::to_string(&dump).expect("serialize squad dump"),
        )
        .expect("write squad dump");
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest squad players");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = ?1 WHERE id = ?2",
            params!["scan-squad-1", snapshot.id],
        )
        .expect("bind source request");
        let save_id = snapshot_service::active_save_id(&conn).expect("active save");
        planner_service::save_club_family(&conn, save_id, "Loan FC", &[])
            .expect("configure club family");

        (temp_dir, Db(Mutex::new(conn)))
    }

    fn verified_ca_result() -> PlayerBoostResult {
        PlayerBoostResult {
            operation: OPERATION_BOOST_CURRENT_ABILITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(150),
            current_ability: Some(160),
            potential_ability: Some(170),
            previous_ambition: None,
            ambition: None,
            previous_professionalism: None,
            professionalism: None,
            previous_determination: None,
            determination: None,
        }
    }

    fn verified_ca_result_for(prepared: &PreparedPlayerBoost) -> PlayerBoostResult {
        PlayerBoostResult {
            operation: OPERATION_BOOST_CURRENT_ABILITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(prepared.expected_current_ability as i32),
            current_ability: prepared.target_current_ability.map(|value| value as i32),
            potential_ability: Some(prepared.expected_potential_ability as i32),
            previous_ambition: None,
            ambition: None,
            previous_professionalism: None,
            professionalism: None,
            previous_determination: None,
            determination: None,
        }
    }

    fn verified_wonderkid_result_for(prepared: &PreparedPlayerBoost) -> PlayerBoostResult {
        let target =
            |value: Option<i64>| value.map(|value| if value <= 10 { 11 } else { value as i32 });

        PlayerBoostResult {
            operation: "wonderkid-mentality".to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(prepared.expected_current_ability as i32),
            current_ability: Some(prepared.expected_current_ability as i32),
            potential_ability: Some(prepared.expected_potential_ability as i32),
            previous_ambition: prepared.expected_ambition.map(|value| value as i32),
            ambition: target(prepared.expected_ambition),
            previous_professionalism: prepared.expected_professionalism.map(|value| value as i32),
            professionalism: target(prepared.expected_professionalism),
            previous_determination: prepared.expected_determination.map(|value| value as i32),
            determination: target(prepared.expected_determination),
        }
    }

    fn set_squad_player_mentality(
        db: &Db,
        uid: i64,
        ambition: Option<i64>,
        professionalism: Option<i64>,
        determination: Option<i64>,
    ) {
        db.0.lock()
            .expect("lock db")
            .execute(
                "UPDATE players
                 SET attributes_json = ?1, personality_json = ?2
                 WHERE uid = ?3",
                params![
                    serde_json::json!({ "Determination": determination }).to_string(),
                    serde_json::json!({
                        "Ambition": ambition,
                        "Professionalism": professionalism,
                    })
                    .to_string(),
                    uid,
                ],
            )
            .expect("set squad player mentality");
    }

    #[test]
    fn bridge_polling_runs_after_the_snapshot_lock_is_released() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_db();

        let result = execute_player_boost_with(
            77,
            &db,
            service::prepare_current_ability_boost,
            |prepared| {
                assert_eq!(prepared.current_ability_increment, Some(10));
                assert!(
                    db.0.try_lock().is_ok(),
                    "the database lock must not cover the bridge request"
                );
                assert!(
                    super::boost_gate::acquire_player_boost_gate().is_err(),
                    "a second player boost must not overwrite the in-flight bridge request"
                );
                Ok(verified_ca_result())
            },
        )
        .expect("reconcile verified bridge result");

        assert_eq!(result.current_ability, Some(160));
        let conn = db.0.lock().expect("lock db");
        let ca: i64 = conn
            .query_row("SELECT ca FROM players WHERE uid = 77", [], |row| {
                row.get(0)
            })
            .expect("read reconciled CA");
        assert_eq!(ca, 160);
    }

    #[test]
    fn age_twenty_nine_does_not_request_a_bridge_boost() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_db();
        {
            let conn = db.0.lock().expect("lock db");
            conn.execute("UPDATE players SET age = 29 WHERE uid = 77", [])
                .expect("set player age");
        }

        let bridge_called = Cell::new(false);
        let error =
            match execute_player_boost_with(77, &db, service::prepare_current_ability_boost, |_| {
                bridge_called.set(true);
                Err(PlayerBoostError::Bridge {
                    kind: "unexpectedBridgeCall".to_string(),
                    message: "age-ineligible player reached the bridge".to_string(),
                })
            }) {
                Err(error) => error,
                Ok(_) => panic!("age 29 must reject the boost"),
            };

        assert!(!bridge_called.get(), "age 29 must not reach the bridge");
        assert!(matches!(
            error,
            PlayerBoostError::Eligibility { kind, .. } if kind == "ageIneligible"
        ));
    }

    #[test]
    fn historical_load_keeps_the_later_snapshot_source_for_a_bridge_mismatch() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db, current_snapshot_id, historical_snapshot_id) = seeded_history_db();

        let error = execute_player_boost_with(
            77,
            &db,
            service::prepare_current_ability_boost,
            |prepared| {
                assert_eq!(prepared.snapshot_id, current_snapshot_id);
                assert_eq!(prepared.source_request_id, "R2");
                assert_eq!(prepared.expected_current_ability, 150);
                assert_eq!(prepared.expected_potential_ability, 170);

                Err(service::map_bridge_error(PlayerBoostRequestError::Failed(
                    "Load Data again before using player boosts".to_string(),
                )))
            },
        )
        .err()
        .expect("the R1 live index must reject the R2 source request");

        assert!(matches!(
            error,
            PlayerBoostError::Bridge { kind, message }
                if kind == "rejected" && message == "Load Data again before using player boosts"
        ));
        let conn = db.0.lock().expect("lock db");
        let active_snapshot_id: i64 = conn
            .query_row("SELECT id FROM snapshots WHERE is_current = 1", [], |row| {
                row.get(0)
            })
            .expect("current snapshot");
        assert_eq!(active_snapshot_id, current_snapshot_id);
        for snapshot_id in [current_snapshot_id, historical_snapshot_id] {
            let abilities: (i64, i64) = conn
                .query_row(
                    "SELECT ca, pa FROM players WHERE snapshot_id = ?1 AND uid = 77",
                    params![snapshot_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("retained player abilities");
            assert_eq!(abilities, (150, 170));
        }
    }

    #[test]
    fn squad_current_ability_boost_uses_the_distinct_frozen_club_family_cohort() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_current_ability_boost_with(&db, |prepared| {
            assert!(
                db.0.try_lock().is_ok(),
                "the database lock must not cover the bridge request"
            );
            if prepared.player_uid == 77 {
                let error = match execute_player_boost_with(
                    78,
                    &db,
                    service::prepare_current_ability_boost,
                    |_| {
                        panic!(
                            "the profile boost must not reach the bridge while the squad is active"
                        )
                    },
                ) {
                    Err(error) => error,
                    Ok(_) => panic!("the squad boost must hold the shared player boost gate"),
                };
                assert!(matches!(
                    error,
                    PlayerBoostError::Bridge { kind, .. } if kind == "inProgress"
                ));
            }
            if prepared.player_uid == 78 {
                let current_ca: i64 =
                    db.0.lock()
                        .expect("lock db")
                        .query_row("SELECT ca FROM players WHERE uid = 77", [], |row| {
                            row.get(0)
                        })
                        .expect("read prior squad result");
                assert_eq!(
                    current_ca, 155,
                    "each success must commit before the next bridge request"
                );
            }
            bridge_calls
                .borrow_mut()
                .push((prepared.player_uid, prepared.current_ability_increment));
            Ok(verified_ca_result_for(prepared))
        })
        .expect("boost the frozen squad cohort");

        assert_eq!(
            bridge_calls.into_inner(),
            vec![(77, Some(5)), (78, Some(10)), (79, Some(10))]
        );
        assert_eq!(result.updated, 3);
        assert_eq!(result.skipped, 2);
        assert_eq!(result.failed, 0);
        assert!(!result.recovery_required);
        assert_eq!(result.recovery_message, None);

        let conn = db.0.lock().expect("lock db");
        for (uid, expected_ca) in [(77, 155), (78, 160), (79, 200), (80, 150), (81, 170)] {
            let ca: i64 = conn
                .query_row(
                    "SELECT ca FROM players WHERE uid = ?1",
                    params![uid],
                    |row| row.get(0),
                )
                .expect("read squad CA");
            assert_eq!(ca, expected_ca, "unexpected CA for player {uid}");
        }
    }

    #[test]
    fn squad_wonderkid_boost_reports_terminal_progress_without_counting_recovery() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        set_squad_player_mentality(&db, 77, Some(10), Some(15), Some(8));
        set_squad_player_mentality(&db, 78, Some(10), Some(15), Some(8));
        set_squad_player_mentality(&db, 79, None, None, None);
        set_squad_player_mentality(&db, 80, Some(10), Some(15), Some(8));
        let progress = RefCell::new(Vec::new());

        let result = execute_squad_wonderkid_mentality_boost_with_progress(
            &db,
            |prepared| match prepared.player_uid {
                77 => Err(PlayerBoostError::LiveValue {
                    message: "player values changed in FM; Load Data again".to_string(),
                }),
                78 => Ok(verified_wonderkid_result_for(prepared)),
                80 => Err(PlayerBoostError::SnapshotSync {
                    message: "FM may have changed before this result was verified".to_string(),
                }),
                uid => panic!("player {uid} must not reach the bridge"),
            },
            |snapshot| {
                progress.borrow_mut().push(snapshot);
                true
            },
        )
        .expect("report the partial Wonderkid result");

        let snapshots = progress.into_inner();
        assert_eq!(
            snapshots
                .iter()
                .map(|snapshot| (
                    snapshot.processed,
                    snapshot.total,
                    snapshot.updated,
                    snapshot.skipped,
                    snapshot.failed,
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, 5, 0, 0, 0),
                (1, 5, 0, 0, 1),
                (2, 5, 1, 0, 1),
                (3, 5, 1, 1, 1)
            ]
        );
        assert_eq!(result.updated, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 1);
        assert!(result.recovery_required);
    }

    #[test]
    fn squad_boost_progress_delivery_failure_does_not_change_the_result() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();

        let result = execute_squad_current_ability_boost_with_progress(
            &db,
            |prepared| Ok(verified_ca_result_for(prepared)),
            |_| false,
        )
        .expect("progress delivery is best effort");

        assert_eq!(result.updated, 3);
        assert_eq!(result.skipped, 2);
        assert_eq!(result.failed, 0);
        assert!(!result.recovery_required);
    }

    #[test]
    fn empty_squad_boost_reports_zero_progress_without_a_bridge_call() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        db.0.lock()
            .expect("lock db")
            .execute("DELETE FROM players", [])
            .expect("remove squad players");
        let progress = RefCell::new(Vec::new());

        let result = execute_squad_current_ability_boost_with_progress(
            &db,
            |_| panic!("an empty cohort must not reach the bridge"),
            |snapshot| {
                progress.borrow_mut().push(snapshot);
                true
            },
        )
        .expect("report an empty cohort");

        let snapshots = progress.into_inner();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].processed, 0);
        assert_eq!(snapshots[0].total, 0);
        assert_eq!(snapshots[0].updated, 0);
        assert_eq!(snapshots[0].skipped, 0);
        assert_eq!(snapshots[0].failed, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn squad_wonderkid_boost_updates_only_known_low_mentality_fields() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        set_squad_player_mentality(&db, 77, Some(10), Some(14), Some(8));
        set_squad_player_mentality(&db, 78, Some(11), Some(12), Some(13));
        set_squad_player_mentality(&db, 79, None, None, None);
        set_squad_player_mentality(&db, 80, Some(9), Some(15), Some(16));
        set_squad_player_mentality(&db, 81, Some(18), Some(19), Some(20));
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_wonderkid_mentality_boost_with(&db, |prepared| {
            if prepared.player_uid == 77 {
                let error = match execute_squad_current_ability_boost_with(&db, |_| {
                    panic!("CA must not reach the bridge while Wonderkid is active")
                }) {
                    Err(error) => error,
                    Ok(_) => panic!("Wonderkid must hold the shared player boost gate"),
                };
                assert!(matches!(
                    error,
                    PlayerBoostError::Bridge { kind, .. } if kind == "inProgress"
                ));
            }
            bridge_calls.borrow_mut().push(prepared.player_uid);
            assert_eq!(prepared.current_ability_increment, None);
            Ok(verified_wonderkid_result_for(prepared))
        })
        .expect("apply Wonderkid Mentality to the frozen squad cohort");

        assert_eq!(bridge_calls.into_inner(), vec![77, 80]);
        assert_eq!(result.updated, 2);
        assert_eq!(result.skipped, 3);
        assert_eq!(result.failed, 0);
        assert!(!result.recovery_required);

        let conn = db.0.lock().expect("lock db");
        let read_mentality = |uid| {
            conn.query_row(
                "SELECT attributes_json, personality_json FROM players WHERE uid = ?1",
                params![uid],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .expect("read reconciled mentality")
        };
        let (attributes, personality) = read_mentality(77);
        let attributes: serde_json::Value =
            serde_json::from_str(&attributes).expect("parse attributes");
        let personality: serde_json::Value =
            serde_json::from_str(&personality).expect("parse personality");
        assert_eq!(personality["Ambition"], serde_json::json!(11));
        assert_eq!(personality["Professionalism"], serde_json::json!(14));
        assert_eq!(attributes["Determination"], serde_json::json!(11));

        let (attributes, personality) = read_mentality(80);
        let attributes: serde_json::Value =
            serde_json::from_str(&attributes).expect("parse partially changed attributes");
        let personality: serde_json::Value =
            serde_json::from_str(&personality).expect("parse partially changed personality");
        assert_eq!(personality["Ambition"], serde_json::json!(11));
        assert_eq!(personality["Professionalism"], serde_json::json!(15));
        assert_eq!(attributes["Determination"], serde_json::json!(16));

        let (attributes, personality) = read_mentality(78);
        let attributes: serde_json::Value =
            serde_json::from_str(&attributes).expect("parse untouched attributes");
        let personality: serde_json::Value =
            serde_json::from_str(&personality).expect("parse untouched personality");
        assert_eq!(personality["Ambition"], serde_json::json!(11));
        assert_eq!(personality["Professionalism"], serde_json::json!(12));
        assert_eq!(attributes["Determination"], serde_json::json!(13));

        let (attributes, personality) = read_mentality(79);
        let attributes: serde_json::Value =
            serde_json::from_str(&attributes).expect("parse unknown attributes");
        let personality: serde_json::Value =
            serde_json::from_str(&personality).expect("parse unknown personality");
        assert_eq!(personality["Ambition"], serde_json::Value::Null);
        assert_eq!(personality["Professionalism"], serde_json::Value::Null);
        assert_eq!(attributes["Determination"], serde_json::Value::Null);
    }

    #[test]
    fn squad_wonderkid_boost_continues_after_a_proven_no_write_failure_then_stops() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        for uid in [77, 78, 79] {
            set_squad_player_mentality(&db, uid, Some(10), Some(15), Some(16));
        }
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_wonderkid_mentality_boost_with(&db, |prepared| {
            bridge_calls.borrow_mut().push(prepared.player_uid);
            match prepared.player_uid {
                77 => Err(PlayerBoostError::LiveValue {
                    message: "player values changed in FM; Load Data again".to_string(),
                }),
                78 => Ok(verified_wonderkid_result_for(prepared)),
                79 => Err(PlayerBoostError::SnapshotSync {
                    message: "FM may have changed before this result was verified".to_string(),
                }),
                uid => panic!("player {uid} must not reach the bridge after recovery is required"),
            }
        })
        .expect("report the partial Wonderkid result");

        assert_eq!(bridge_calls.into_inner(), vec![77, 78, 79]);
        assert_eq!(result.updated, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 1);
        assert!(result.recovery_required);
    }

    #[test]
    fn squad_wonderkid_recovery_blocks_later_ca_boosts() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        set_squad_player_mentality(&db, 77, Some(10), Some(15), Some(8));
        db.0.lock()
            .expect("lock db")
            .execute_batch(
                "CREATE TRIGGER fail_squad_wonderkid_reconciliation
                 BEFORE UPDATE OF attributes_json ON players
                 BEGIN
                   SELECT RAISE(FAIL, 'test reconciliation failure');
                 END;",
            )
            .expect("make Wonderkid reconciliation fail");

        let first_result = execute_squad_wonderkid_mentality_boost_with(&db, |prepared| {
            Ok(verified_wonderkid_result_for(prepared))
        })
        .expect("report the reconciliation recovery outcome");
        assert!(first_result.recovery_required);

        let ca_error = match execute_squad_current_ability_boost_with(&db, |_| {
            panic!("a recovery-required snapshot must not reach the CA bridge")
        }) {
            Err(error) => error,
            Ok(_) => panic!("a Wonderkid recovery requirement must block CA"),
        };
        assert!(matches!(ca_error, PlayerBoostError::SnapshotSync { .. }));
    }

    #[test]
    fn squad_current_ability_boost_stops_when_snapshot_values_are_invalid() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        db.0.lock()
            .expect("lock db")
            .execute("UPDATE players SET ca = 171 WHERE uid = 77", [])
            .expect("make the first player snapshot-invalid");
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_current_ability_boost_with(&db, |prepared| {
            bridge_calls.borrow_mut().push(prepared.player_uid);
            Ok(verified_ca_result_for(prepared))
        })
        .expect("report the invalid snapshot recovery outcome");

        assert!(
            bridge_calls.borrow().is_empty(),
            "an invalid snapshot must stop the batch before an FM write"
        );
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
        assert!(result.recovery_required);
        assert_eq!(
            result.recovery_message.as_deref(),
            Some("snapshot ability values are invalid; Load Data again")
        );
    }

    #[test]
    fn squad_current_ability_boost_blocks_active_save_changes_during_the_bridge_request() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        let next_save = snapshot_service::create_save(&db.0.lock().expect("lock db"), "Next save")
            .expect("create next save");
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_current_ability_boost_with(&db, |prepared| {
            bridge_calls.borrow_mut().push(prepared.player_uid);
            let error = snapshot_commands::set_active_save_for_command(&db, next_save.id)
                .expect_err("an active squad boost must prevent a save change");

            assert_eq!(
                error,
                "a player boost is already in progress; wait for it to finish"
            );
            Err(PlayerBoostError::SnapshotSync {
                message: "FM may have changed before this result was verified".to_string(),
            })
        })
        .expect("report the recovery outcome");

        assert_eq!(bridge_calls.into_inner(), vec![77]);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
        assert!(result.recovery_required);

        let active_save_id: i64 =
            db.0.lock()
                .expect("lock database")
                .query_row("SELECT id FROM saves WHERE is_active = 1", [], |row| {
                    row.get(0)
                })
                .expect("read active save");
        assert_ne!(active_save_id, next_save.id);
    }

    #[test]
    fn squad_current_ability_boost_continues_after_proven_no_write_failure_then_stops() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_current_ability_boost_with(&db, |prepared| {
            bridge_calls.borrow_mut().push(prepared.player_uid);
            match prepared.player_uid {
                77 => Err(PlayerBoostError::LiveValue {
                    message: "current ability is already at its potential limit".to_string(),
                }),
                78 => Ok(verified_ca_result_for(prepared)),
                79 => Err(PlayerBoostError::SnapshotSync {
                    message: "FM may have changed before this result was verified".to_string(),
                }),
                uid => panic!("player {uid} must not reach the bridge after recovery is required"),
            }
        })
        .expect("report the partial squad result");

        assert_eq!(bridge_calls.into_inner(), vec![77, 78, 79]);
        assert_eq!(result.updated, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 1);
        assert!(result.recovery_required);
        assert_eq!(
            result.recovery_message.as_deref(),
            Some("FM may have changed before this result was verified")
        );
    }

    #[test]
    fn squad_current_ability_boost_stops_before_a_second_write_for_recovery_outcomes() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        for error in [
            PlayerBoostError::SnapshotSync {
                message: "timed out waiting for the bridge".to_string(),
            },
            PlayerBoostError::SnapshotSync {
                message: "the bridge rollback was unverified".to_string(),
            },
            PlayerBoostError::Bridge {
                kind: "rejected".to_string(),
                message: "Load Data again before using player boosts".to_string(),
            },
        ] {
            let (_temp_dir, db) = seeded_squad_db();
            let bridge_calls = RefCell::new(Vec::new());

            let result = execute_squad_current_ability_boost_with(&db, |prepared| {
                bridge_calls.borrow_mut().push(prepared.player_uid);
                Err(error.clone())
            })
            .expect("report a terminal recovery outcome");

            assert_eq!(bridge_calls.into_inner(), vec![77]);
            assert_eq!(result.updated, 0);
            assert_eq!(result.skipped, 0);
            assert_eq!(result.failed, 0);
            assert!(result.recovery_required);
        }
    }

    #[test]
    fn squad_current_ability_boost_stops_when_the_active_context_changes() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (temp_dir, db) = seeded_squad_db();
        let replacement_path = temp_dir.path().join("replacement-snapshot.json");
        let mut replacement: serde_json::Value =
            serde_json::from_str(GOLDEN_FIXTURE).expect("parse replacement fixture");
        replacement["gameDate"] = serde_json::Value::from("2027-08-14");
        std::fs::write(
            &replacement_path,
            serde_json::to_string(&replacement).expect("serialize replacement fixture"),
        )
        .expect("write replacement fixture");
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_current_ability_boost_with(&db, |prepared| {
            bridge_calls.borrow_mut().push(prepared.player_uid);
            ingest_dump_file(&mut db.0.lock().expect("lock db"), &replacement_path)
                .expect("replace the active snapshot");
            Ok(verified_ca_result_for(prepared))
        })
        .expect("report the active-context recovery outcome");

        assert_eq!(bridge_calls.into_inner(), vec![77]);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
        assert!(result.recovery_required);
    }

    #[test]
    fn squad_current_ability_boost_stops_after_a_verified_fm_result_cannot_be_reconciled() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        {
            let conn = db.0.lock().expect("lock db");
            conn.execute_batch(
                "CREATE TRIGGER fail_squad_ca_reconciliation
                 BEFORE UPDATE OF ca ON players
                 BEGIN
                   SELECT RAISE(FAIL, 'test reconciliation failure');
                 END;",
            )
            .expect("make reconciliation fail");
        }
        let bridge_calls = RefCell::new(Vec::new());

        let result = execute_squad_current_ability_boost_with(&db, |prepared| {
            bridge_calls.borrow_mut().push(prepared.player_uid);
            Ok(verified_ca_result_for(prepared))
        })
        .expect("report a reconciliation recovery outcome");

        assert_eq!(bridge_calls.into_inner(), vec![77]);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
        assert!(result.recovery_required);
    }

    #[test]
    fn squad_recovery_requires_load_data_before_later_squad_or_profile_boosts() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (temp_dir, db) = seeded_squad_db();
        {
            let conn = db.0.lock().expect("lock db");
            conn.execute_batch(
                "CREATE TRIGGER fail_squad_ca_reconciliation
                 BEFORE UPDATE OF ca ON players
                 BEGIN
                   SELECT RAISE(FAIL, 'test reconciliation failure');
                 END;",
            )
            .expect("make reconciliation fail");
        }

        let first_result = execute_squad_current_ability_boost_with(&db, |prepared| {
            Ok(verified_ca_result_for(prepared))
        })
        .expect("report the reconciliation recovery outcome");
        assert!(first_result.recovery_required);

        let recovery_required: i64 =
            db.0.lock()
                .expect("lock db")
                .query_row(
                    "SELECT player_boost_recovery_required FROM snapshots WHERE is_current = 1",
                    [],
                    |row| row.get(0),
                )
                .expect("read recovery requirement");
        assert_eq!(recovery_required, 1);

        let squad_error = match execute_squad_current_ability_boost_with(&db, |_| {
            panic!("a recovery-required snapshot must not reach the squad bridge")
        }) {
            Err(error) => error,
            Ok(_) => panic!("a later squad boost must require Load Data"),
        };
        assert!(matches!(squad_error, PlayerBoostError::SnapshotSync { .. }));

        let profile_error =
            match execute_player_boost_with(77, &db, service::prepare_current_ability_boost, |_| {
                panic!("a recovery-required snapshot must not reach the profile bridge")
            }) {
                Err(error) => error,
                Ok(_) => panic!("a later profile boost must require Load Data"),
            };
        assert!(matches!(
            profile_error,
            PlayerBoostError::SnapshotSync { .. }
        ));

        {
            let mut replacement: serde_json::Value =
                serde_json::from_str(GOLDEN_FIXTURE).expect("parse replacement fixture");
            let mut player = replacement["players"][0].clone();
            player["uid"] = serde_json::Value::from(77);
            player["age"] = serde_json::Value::from(20);
            player["ca"] = serde_json::Value::from(150);
            player["pa"] = serde_json::Value::from(170);
            player["currentClub"] = serde_json::Value::from("Loan FC");
            replacement["gameDate"] = serde_json::Value::from("2027-08-14");
            replacement["players"] = serde_json::Value::Array(vec![player]);
            replacement["playerCount"] = serde_json::Value::from(1);
            let replacement_path = temp_dir.path().join("fresh-squad.json");
            std::fs::write(
                &replacement_path,
                serde_json::to_string(&replacement).expect("serialize replacement fixture"),
            )
            .expect("write replacement fixture");

            let mut conn = db.0.lock().expect("lock db");
            conn.execute("DROP TRIGGER fail_squad_ca_reconciliation", [])
                .expect("allow fresh snapshot reconciliation");
            let snapshot = ingest_dump_file(&mut conn, &replacement_path)
                .expect("Load Data establishes a fresh snapshot");
            conn.execute(
                "UPDATE snapshots SET bridge_source_request_id = ?1 WHERE id = ?2",
                params!["scan-squad-2", snapshot.id],
            )
            .expect("bind fresh source request");
        }

        let fresh_result = execute_squad_current_ability_boost_with(&db, |prepared| {
            Ok(verified_ca_result_for(prepared))
        })
        .expect("a fresh snapshot must permit a squad boost");
        assert_eq!(fresh_result.updated, 1);
        assert!(!fresh_result.recovery_required);
    }

    #[test]
    fn profile_recovery_requires_load_data_before_a_later_squad_boost() {
        let _test_guard = PLAYER_BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp_dir, db) = seeded_squad_db();
        {
            let conn = db.0.lock().expect("lock db");
            conn.execute_batch(
                "CREATE TRIGGER fail_profile_ca_reconciliation
                 BEFORE UPDATE OF ca ON players
                 BEGIN
                   SELECT RAISE(FAIL, 'test reconciliation failure');
                 END;",
            )
            .expect("make reconciliation fail");
        }

        let profile_error = match execute_player_boost_with(
            77,
            &db,
            service::prepare_current_ability_boost,
            |prepared| Ok(verified_ca_result_for(prepared)),
        ) {
            Err(error) => error,
            Ok(_) => panic!("report the profile reconciliation failure"),
        };
        assert!(matches!(
            profile_error,
            PlayerBoostError::SnapshotSync { .. }
        ));

        let squad_error = match execute_squad_current_ability_boost_with(&db, |_| {
            panic!("a profile recovery requirement must block the squad bridge")
        }) {
            Err(error) => error,
            Ok(_) => panic!("a later squad boost must require Load Data"),
        };
        assert!(matches!(squad_error, PlayerBoostError::SnapshotSync { .. }));
    }
}
