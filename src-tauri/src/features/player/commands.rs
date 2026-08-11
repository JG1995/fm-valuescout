use std::sync::Mutex;

use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::features::memory_read::service::{
    request_player_boost_from_local_app_data, DumpWaitConfig, PlayerBoostResult,
};

use super::query::{self, PlayerDetail, PlayerRoleScore};
use super::service::{self, PlayerBoostError, PreparedPlayerBoost, VerifiedPlayerBoost};

static PLAYER_BOOST_GATE: Mutex<()> = Mutex::new(());

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
    pub positions: std::collections::BTreeMap<String, i64>,
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

fn execute_player_boost(
    uid: i64,
    db: &Db,
    prepare: fn(&rusqlite::Connection, i64) -> Result<PreparedPlayerBoost, PlayerBoostError>,
) -> Result<PlayerBoostResultDto, PlayerBoostError> {
    execute_player_boost_with(uid, db, prepare, |prepared| {
        request_player_boost_from_local_app_data(
            &prepared.source_request_id,
            prepared.player_uid,
            prepared.expected_current_ability as i32,
            prepared.expected_potential_ability as i32,
            prepared.bridge_operation(),
            DumpWaitConfig::default(),
        )
        .map_err(service::map_bridge_error)
    })
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
    let prepared = {
        let conn = db.0.lock().map_err(|_| PlayerBoostError::Eligibility {
            kind: "databaseUnavailable".to_string(),
            message: "could not read the current snapshot for this player boost".to_string(),
        })?;
        prepare(&conn, uid)?
    };

    let _boost_guard = PLAYER_BOOST_GATE
        .try_lock()
        .map_err(|_| PlayerBoostError::Bridge {
            kind: "inProgress".to_string(),
            message: "a player boost is already in progress; wait for it to finish".to_string(),
        })?;
    let bridge_result = request_bridge_boost(&prepared)?;

    let mut conn = db.0.lock().map_err(|_| PlayerBoostError::SnapshotSync {
        message:
            "FM may have changed, but FM ValueScout could not update its snapshot. Load Data again."
                .to_string(),
    })?;
    service::reconcile_verified_boost(&mut conn, &prepared, bridge_result)
        .map(PlayerBoostResultDto::from)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use rusqlite::{params, Connection};

    use super::*;
    use crate::db::migrations;
    use crate::features::memory_read::service::{
        PlayerBoostRequestError, PlayerBoostResult, OPERATION_BOOST_CURRENT_ABILITY,
    };
    use crate::features::snapshot::ingest::ingest_dump_file;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v6.json");

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

    #[test]
    fn bridge_polling_runs_after_the_snapshot_lock_is_released() {
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
                    super::PLAYER_BOOST_GATE.try_lock().is_err(),
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
    fn historical_load_keeps_the_later_snapshot_source_for_a_bridge_mismatch() {
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
}
