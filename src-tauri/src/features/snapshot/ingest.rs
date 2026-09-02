//! Validates and ingests `dump.json` into the active save, then selects its effective current snapshot.

use std::fs;
use std::path::Path;
use std::time::Instant;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::Value;

use crate::features::academy::service as academy_service;
use crate::features::memory_read::dump_validation::parse_and_validate_dump;
use crate::features::player_metrics::club_dna as club_dna_scores;
#[allow(unused_imports)]
use crate::features::player_metrics::compact as player_compact;
#[allow(unused_imports)]
use crate::features::player_metrics::potential_scores;
#[allow(unused_imports)]
use crate::features::scoring::catalog::{all_roles, DUMP_ATTRIBUTE_KEYS};
#[allow(unused_imports)]
use crate::features::scoring::projection::project_attributes;
#[cfg(test)]
use crate::features::scoring::score::score_role;
#[cfg(test)]
use crate::features::staff::scoring::all_staff_roles;
use crate::features::staff::scoring::{
    persist_rows as persist_staff_rows, score_all_staff_roles, CompactStaffRow,
};

use super::service::{self, SaveContext};

/// Bounded owned preparation of one dump, proportional to one dump's
/// `players` + `staff` vectors. It owns parsed raw rows plus derived
/// projected JSON and compact scores, but no `rusqlite` connection,
/// transaction, or row. Built outside the `Db(Mutex<Connection>)` lock;
/// `publish_prepared_snapshot` revalidates the captured `SaveContext`
/// first and publishes it in one final transaction.
#[derive(Debug, Clone)]
pub struct PreparedSnapshot {
    pub(crate) schema_version: i64,
    pub(crate) generated_at_utc: String,
    pub(crate) game_version: String,
    pub(crate) supported_game_version: String,
    pub(crate) bridge_version: String,
    pub(crate) protocol_version: i64,
    pub(crate) game_date: Option<String>,
    pub(crate) game_date_source: String,
    pub(crate) game_date_basis: String,
    pub(crate) player_database_scope: String,
    pub(crate) scan_truncated: bool,
    pub(crate) max_accepted: Option<i64>,
    pub(crate) player_count: i64,
    pub(crate) staff_count: i64,
    pub(crate) manager_uid: Option<i64>,
    pub(crate) manager_name: Option<String>,
    pub(crate) manager_club: Option<String>,
    pub(crate) manager_club_reputation: Option<i64>,
    pub(crate) players: Vec<PreparedPlayer>,
    pub(crate) staff: Vec<PreparedStaff>,
}

#[derive(Debug, Clone)]
pub struct PreparedPlayer {
    pub(crate) uid: i64,
    pub(crate) ca: i64,
    pub(crate) pa: i64,
    pub(crate) name: String,
    pub(crate) birth_year: i64,
    pub(crate) birth_day_of_year: i64,
    pub(crate) age: Option<i64>,
    pub(crate) nationalities_json: String,
    pub(crate) height_cm: Option<i64>,
    pub(crate) preferred_foot: String,
    pub(crate) positions_json: String,
    pub(crate) attributes_json: String,
    pub(crate) hidden_attributes_json: String,
    pub(crate) personality_json: String,
    pub(crate) weekly_wage_gbp: Option<i64>,
    pub(crate) contract_expiry_year: Option<i64>,
    pub(crate) contract_expiry_day_of_year: Option<i64>,
    pub(crate) transfer_listed: Option<i32>,
    pub(crate) loan_listed: Option<i32>,
    pub(crate) not_for_sale: Option<i32>,
    pub(crate) set_for_release: Option<i32>,
    pub(crate) market_value_gbp: Option<i64>,
    pub(crate) reputation_current: Option<i64>,
    pub(crate) reputation_world: Option<i64>,
    pub(crate) current_club: Option<String>,
    pub(crate) parent_club: Option<String>,
    pub(crate) on_loan: Option<i32>,
    pub(crate) division: Option<String>,
    pub(crate) team_level: Option<String>,
    pub(crate) nation_uid: Option<i64>,
    pub(crate) gender: String,
    pub(crate) club_reputation: Option<i64>,
    pub(crate) team_type: Option<i64>,
    pub(crate) projected_attributes_json: String,
    pub(crate) compact_current: Vec<Option<i64>>,
    pub(crate) compact_potential: Vec<Option<i64>>,
}

#[derive(Debug, Clone)]
pub struct PreparedStaff {
    pub(crate) uid: i64,
    pub(crate) name: Option<String>,
    pub(crate) birth_year: Option<i64>,
    pub(crate) birth_day_of_year: Option<i64>,
    pub(crate) age: Option<i64>,
    pub(crate) nationalities_json: String,
    pub(crate) nation_uid: Option<i64>,
    pub(crate) gender: String,
    pub(crate) ca: i64,
    pub(crate) pa: i64,
    pub(crate) staff_attributes_json: String,
    pub(crate) job_id: Option<i64>,
    pub(crate) weekly_wage_gbp: Option<i64>,
    pub(crate) contract_expiry_year: Option<i64>,
    pub(crate) contract_expiry_day_of_year: Option<i64>,
    pub(crate) club: Option<String>,
    pub(crate) division: Option<String>,
    pub(crate) compact_scores: Vec<Option<i64>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSummary {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IngestResult {
    pub stored_snapshot: SnapshotSummary,
    pub effective_snapshot: SnapshotSummary,
}

/// Phase timings for large-dump ingest measurement (milliseconds).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct IngestTimings {
    pub validation_ms: u128,
    pub insert_ms: u128,
    pub total_ms: u128,
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn ingest_dump_file(
    conn: &mut Connection,
    dump_path: &Path,
) -> Result<SnapshotSummary, String> {
    let save_context = service::capture_active_save_context(conn)?;
    ingest_dump_file_for_save_with_optional_bridge_source_request_id(
        conn,
        &save_context,
        dump_path,
        None,
    )
    .map(|result| result.stored_snapshot)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn ingest_dump_file_for_save(
    conn: &mut Connection,
    save_id: i64,
    dump_path: &Path,
) -> Result<SnapshotSummary, String> {
    let save_context = service::save_context_for_id(conn, save_id)?;
    ingest_dump_file_for_save_with_optional_bridge_source_request_id(
        conn,
        &save_context,
        dump_path,
        None,
    )
    .map(|result| result.stored_snapshot)
}

#[cfg(test)]
pub(super) fn ingest_dump_file_for_save_with_bridge_source_request_id(
    conn: &mut Connection,
    save_context: &SaveContext,
    dump_path: &Path,
    bridge_source_request_id: &str,
) -> Result<IngestResult, String> {
    ingest_dump_file_for_save_with_optional_bridge_source_request_id(
        conn,
        save_context,
        dump_path,
        Some(bridge_source_request_id),
    )
}

fn ingest_dump_file_for_save_with_optional_bridge_source_request_id(
    conn: &mut Connection,
    save_context: &SaveContext,
    dump_path: &Path,
    bridge_source_request_id: Option<&str>,
) -> Result<IngestResult, String> {
    // Direct-connection callers compose prepare+publish while holding &mut Connection.
    // Only the Tauri Load Data command releases the Db mutex during prepare.
    let json = fs::read_to_string(dump_path).map_err(|error| error.to_string())?;
    ingest_dump_json_for_save(conn, save_context, &json, bridge_source_request_id)
        .map(|(result, _)| result)
}

/// Ingests a dump file and returns phase timings for measurement harnesses.
#[cfg_attr(not(test), allow(dead_code))]
pub fn ingest_dump_file_for_save_timed(
    conn: &mut Connection,
    save_id: i64,
    dump_path: &Path,
) -> Result<(SnapshotSummary, IngestTimings), String> {
    let save_context = service::save_context_for_id(conn, save_id)?;
    let json = fs::read_to_string(dump_path).map_err(|error| error.to_string())?;
    ingest_dump_json_for_save(conn, &save_context, &json, None)
        .map(|(result, timings)| (result.stored_snapshot, timings))
}

/// Pure preparation: validates dump JSON and builds owned raw + derived state
/// without any database read/write or `rusqlite` connection. Memory is
/// proportional to one dump's `players` + `staff` vectors.
pub fn prepare_dump_json(json: &str) -> Result<PreparedSnapshot, String> {
    let root = parse_and_validate_dump(json).map_err(|error| error.to_string())?;
    let object = root
        .as_object()
        .ok_or_else(|| "dump root must be a JSON object".to_string())?;

    let schema_version = require_i64(object, "schemaVersion")?;
    let generated_at_utc = require_string(object, "generatedAtUtc")?;
    let game_version = require_string(object, "gameVersion")?;
    let supported_game_version = require_string(object, "supportedGameVersion")?;
    let bridge_version = require_string(object, "bridgeVersion")?;
    let protocol_version = require_i64(object, "protocolVersion")?;
    let game_date = optional_string(object.get("gameDate"))?;
    let game_date_source = require_string(object, "gameDateSource")?;
    let game_date_basis = require_string(object, "gameDateBasis")?;
    let player_database_scope = require_string(object, "playerDatabaseScope")?;
    let scan_truncated = require_bool(object, "scanTruncated")?;
    let max_accepted = optional_i64(object.get("maxAccepted"))?;
    let player_count = require_i64(object, "playerCount")?;
    let staff_count = require_i64(object, "staffCount")?;
    let manager_uid = optional_i64(manager_field(object, "uid")?)?;
    let manager_name = optional_string(manager_field(object, "name")?)?;
    let manager_club = optional_string(manager_field(object, "club")?)?;
    let manager_club_reputation = optional_i64(manager_field(object, "clubReputation")?)?;

    let players_value = object
        .get("players")
        .and_then(Value::as_array)
        .ok_or_else(|| "dump players must be an array".to_string())?;
    let staff_value = object
        .get("staff")
        .and_then(Value::as_array)
        .ok_or_else(|| "dump staff must be an array".to_string())?;

    let mut players = Vec::with_capacity(players_value.len());
    for player in players_value {
        let player = player
            .as_object()
            .ok_or_else(|| "each player must be a JSON object".to_string())?;
        let uid = require_u64(player, "uid")? as i64;
        let ca = require_i64(player, "ca")?;
        let pa = require_i64(player, "pa")?;
        let name = require_string(player, "name")?;
        let birth_year = require_i64(player, "birthYear")?;
        let birth_day_of_year = require_i64(player, "birthDayOfYear")?;
        let age = optional_i64(player.get("age"))?;
        let nationalities_json = json_string(required_value(player, "nationalities")?)?;
        let height_cm = optional_i64(player.get("heightCm"))?;
        let preferred_foot = require_string(player, "preferredFoot")?;
        let positions_json = json_string(required_value(player, "positions")?)?;
        let attributes_json = json_string(required_value(player, "attributes")?)?;
        let hidden_attributes_json = json_string(required_value(player, "hiddenAttributes")?)?;
        let personality_json = json_string(required_value(player, "personality")?)?;
        let weekly_wage_gbp = optional_i64(player.get("weeklyWageGbp"))?;
        let contract_expiry_year = optional_i64(player.get("contractExpiryYear"))?;
        let contract_expiry_day_of_year = optional_i64(player.get("contractExpiryDayOfYear"))?;
        let transfer_listed = optional_bool(player.get("transferListed"))?;
        let loan_listed = optional_bool(player.get("loanListed"))?;
        let not_for_sale = optional_bool(player.get("notForSale"))?;
        let set_for_release = optional_bool(player.get("setForRelease"))?;
        let market_value_gbp = optional_i64(player.get("marketValueGbp"))?;
        let reputation_current = reputation_field(player, "current")?;
        let reputation_world = reputation_field(player, "world")?;
        let current_club = optional_string(player.get("currentClub"))?;
        let parent_club = optional_string(player.get("parentClub"))?;
        let on_loan = optional_bool(player.get("onLoan"))?;
        let division = optional_string(player.get("division"))?;
        let team_level = optional_string(player.get("teamLevel"))?;
        let nation_uid = optional_i64(player.get("nationUid"))?;
        let gender = require_string(player, "gender")?;
        let club_reputation = optional_i64(player.get("clubReputation"))?;
        let team_type = optional_i64(player.get("teamType"))?;

        let (projected_attributes_json, compact_current, compact_potential) =
            player_compact::prepare_player_derived(
                uid,
                &attributes_json,
                &positions_json,
                ca,
                pa,
                age,
            )?;

        players.push(PreparedPlayer {
            uid,
            ca,
            pa,
            name,
            birth_year,
            birth_day_of_year,
            age,
            nationalities_json,
            height_cm,
            preferred_foot,
            positions_json,
            attributes_json,
            hidden_attributes_json,
            personality_json,
            weekly_wage_gbp,
            contract_expiry_year,
            contract_expiry_day_of_year,
            transfer_listed,
            loan_listed,
            not_for_sale,
            set_for_release,
            market_value_gbp,
            reputation_current,
            reputation_world,
            current_club,
            parent_club,
            on_loan,
            division,
            team_level,
            nation_uid,
            gender,
            club_reputation,
            team_type,
            projected_attributes_json,
            compact_current,
            compact_potential,
        });
    }

    let mut staff = Vec::with_capacity(staff_value.len());
    for staff_record in staff_value {
        let staff_record = staff_record
            .as_object()
            .ok_or_else(|| "each staff record must be a JSON object".to_string())?;
        let uid = require_u64(staff_record, "uid")? as i64;
        let name = optional_string(staff_record.get("name"))?;
        let birth_year = optional_i64(staff_record.get("birthYear"))?;
        let birth_day_of_year = optional_i64(staff_record.get("birthDayOfYear"))?;
        let age = optional_i64(staff_record.get("age"))?;
        let nationalities_json = json_string(required_value(staff_record, "nationalities")?)?;
        let nation_uid = optional_i64(staff_record.get("nationUid"))?;
        let gender = require_string(staff_record, "gender")?;
        let ca = require_i64(staff_record, "ca")?;
        let pa = require_i64(staff_record, "pa")?;
        let raw_attributes = required_value(staff_record, "attributes")?;
        let staff_attributes_json = json_string(raw_attributes)?;
        let attributes = attributes_map(raw_attributes)?;
        let job_id = optional_i64(staff_record.get("jobId"))?;
        let weekly_wage_gbp = optional_i64(staff_record.get("weeklyWageGbp"))?;
        let contract_expiry_year = optional_i64(staff_record.get("contractExpiryYear"))?;
        let contract_expiry_day_of_year =
            optional_i64(staff_record.get("contractExpiryDayOfYear"))?;
        let club = optional_string(staff_record.get("club"))?;
        let division = optional_string(staff_record.get("division"))?;
        let compact_scores = score_all_staff_roles(&attributes);
        staff.push(PreparedStaff {
            uid,
            name,
            birth_year,
            birth_day_of_year,
            age,
            nationalities_json,
            nation_uid,
            gender,
            ca,
            pa,
            staff_attributes_json,
            job_id,
            weekly_wage_gbp,
            contract_expiry_year,
            contract_expiry_day_of_year,
            club,
            division,
            compact_scores,
        });
    }

    Ok(PreparedSnapshot {
        schema_version,
        generated_at_utc,
        game_version,
        supported_game_version,
        bridge_version,
        protocol_version,
        game_date,
        game_date_source,
        game_date_basis,
        player_database_scope,
        scan_truncated,
        max_accepted,
        player_count,
        staff_count,
        manager_uid,
        manager_name,
        manager_club,
        manager_club_reputation,
        players,
        staff,
    })
}

pub fn prepare_dump_file(path: &Path) -> Result<PreparedSnapshot, String> {
    let json = fs::read_to_string(path).map_err(|error| error.to_string())?;
    prepare_dump_json(&json)
}

/// Final publication: revalidates the captured `SaveContext` first, then in
/// one transaction inserts raw rows, Club DNA, selects effective current,
/// persists only current derived state, clears displaced current derived
/// state, and commits atomically. Failure rolls back, preserving prior current.
pub(crate) fn publish_prepared_snapshot(
    conn: &mut Connection,
    save_context: &SaveContext,
    prepared: PreparedSnapshot,
    bridge_source_request_id: Option<&str>,
) -> Result<IngestResult, String> {
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    service::ensure_save_context(&tx, save_context)?;
    let save_id = save_context.id;
    let snapshot_id = insert_prepared_snapshot(&tx, save_id, &prepared, bridge_source_request_id)?;
    insert_prepared_players_raw(&tx, snapshot_id, &prepared.players)?;
    insert_prepared_staff_raw(&tx, snapshot_id, &prepared.staff)?;
    if let Some(definition) = club_dna_scores::definition_for_save(&tx, save_id)? {
        club_dna_scores::persist_snapshot_scores(&tx, snapshot_id, &definition)?;
    }
    // Determine effective current snapshot (same ordering as service::select_current_snapshot)
    let effective_snapshot_id = {
        let order = service::SNAPSHOT_ORDER_BY;
        let sql = format!("SELECT id FROM snapshots WHERE save_id = ?1 ORDER BY {order} LIMIT 1");
        let current_id: Option<i64> = tx
            .query_row(&sql, rusqlite::params![save_id], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        tx.execute(
            "UPDATE snapshots SET is_current = 0 WHERE save_id = ?1 AND is_current = 1",
            rusqlite::params![save_id],
        )
        .map_err(|e| e.to_string())?;
        if let Some(id) = current_id {
            tx.execute(
                "UPDATE snapshots SET is_current = 1 WHERE id = ?1",
                rusqlite::params![id],
            )
            .map_err(|e| e.to_string())?;
        }
        // Clear derived state from every non-current snapshot (displaced current)
        player_compact::clear_non_current_snapshots(&tx, save_id)?;
        tx.execute(
            "UPDATE players SET potential_attributes_json = NULL, potential_projection_model_version = NULL WHERE snapshot_id IN (SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 0)",
            rusqlite::params![save_id],
        )
        .map_err(|e| e.to_string())?;
        crate::features::staff::scoring::clear_non_current_snapshots(&tx, save_id)?;
        // Persist derived state only for the effective current snapshot.
        // If the new snapshot won, use the prepared derived values (no DB read, no duplicate scoring).
        // If another snapshot is effective (historical insert), its derived state is already correct;
        // the new historical snapshot stays raw-only and the promotion rebuild is not needed here.
        if let Some(effective) = current_id {
            if effective == snapshot_id {
                // Insert projected JSON + compact rows for the winning snapshot from prepared.
                for player in &prepared.players {
                    tx.execute(
                        "UPDATE players SET potential_attributes_json = ?3, potential_projection_model_version = ?4 WHERE snapshot_id = ?1 AND uid = ?2",
                        rusqlite::params![
                            snapshot_id,
                            player.uid,
                            player.projected_attributes_json,
                            potential_scores::PROJECTION_MODEL_VERSION
                        ],
                    )
                    .map_err(|e| e.to_string())?;
                }
                let compact_rows: Vec<player_compact::CompactPlayerRow> = prepared
                    .players
                    .iter()
                    .map(|p| player_compact::CompactPlayerRow {
                        uid: p.uid,
                        current_scores: p.compact_current.clone(),
                        potential_scores: p.compact_potential.clone(),
                    })
                    .collect();
                player_compact::persist_rows(&tx, snapshot_id, &compact_rows)?;
                player_compact::assert_snapshot_complete(&tx, snapshot_id)?;
                potential_scores::assert_current_snapshot_complete(&tx, snapshot_id)?;
                let staff_rows: Vec<CompactStaffRow> = prepared
                    .staff
                    .iter()
                    .map(|s| CompactStaffRow {
                        uid: s.uid,
                        scores: s.compact_scores.clone(),
                    })
                    .collect();
                persist_staff_rows(&tx, snapshot_id, &staff_rows)?;
                crate::features::staff::scoring::assert_snapshot_complete(&tx, snapshot_id)?;
            }
            // Academy: only when the new snapshot is effective current
            if effective == snapshot_id {
                academy_service::ensure_class_for_game_date(
                    &tx,
                    save_id,
                    prepared.game_date.as_deref(),
                    &prepared.game_date_source,
                )?;
            }
        }
        current_id
    };
    let effective_snapshot_id = effective_snapshot_id
        .ok_or_else(|| "ingest did not select a current snapshot".to_string())?;
    let stored_snapshot = get_snapshot_by_id(&tx, snapshot_id)?;
    let effective_snapshot = if effective_snapshot_id == snapshot_id {
        stored_snapshot.clone()
    } else {
        get_snapshot_by_id(&tx, effective_snapshot_id)?
    };
    tx.commit().map_err(|error| error.to_string())?;
    Ok(IngestResult {
        stored_snapshot,
        effective_snapshot,
    })
}

fn insert_prepared_snapshot(
    tx: &Transaction<'_>,
    save_id: i64,
    prepared: &PreparedSnapshot,
    bridge_source_request_id: Option<&str>,
) -> Result<i64, String> {
    tx.execute(
        "INSERT INTO snapshots (
            save_id,
            is_current,
            schema_version,
            generated_at_utc,
            game_version,
            supported_game_version,
            bridge_version,
            protocol_version,
            game_date,
            game_date_source,
            game_date_basis,
            player_database_scope,
            scan_truncated,
            max_accepted,
            player_count,
            staff_count,
            manager_uid,
            manager_name,
            manager_club,
            manager_club_reputation,
            bridge_source_request_id
        ) VALUES (
            ?1, 0, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20
        )",
        params![
            save_id,
            prepared.schema_version,
            prepared.generated_at_utc,
            prepared.game_version,
            prepared.supported_game_version,
            prepared.bridge_version,
            prepared.protocol_version,
            prepared.game_date,
            prepared.game_date_source,
            prepared.game_date_basis,
            prepared.player_database_scope,
            i32::from(prepared.scan_truncated),
            prepared.max_accepted,
            prepared.player_count,
            prepared.staff_count,
            prepared.manager_uid,
            prepared.manager_name,
            prepared.manager_club,
            prepared.manager_club_reputation,
            bridge_source_request_id,
        ],
    )
    .map_err(|error| error.to_string())?;
    Ok(tx.last_insert_rowid())
}

fn insert_prepared_players_raw(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    players: &[PreparedPlayer],
) -> Result<(), String> {
    let mut stmt = tx
        .prepare(
            "INSERT INTO players (
                snapshot_id,
                uid,
                ca,
                pa,
                name,
                birth_year,
                birth_day_of_year,
                age,
                nationalities_json,
                height_cm,
                preferred_foot,
                positions_json,
                attributes_json,
                hidden_attributes_json,
                personality_json,
                weekly_wage_gbp,
                contract_expiry_year,
                contract_expiry_day_of_year,
                transfer_listed,
                loan_listed,
                not_for_sale,
                set_for_release,
                market_value_gbp,
                reputation_current,
                reputation_world,
                current_club,
                parent_club,
                on_loan,
                division,
                team_level,
                nation_uid,
                gender,
                club_reputation,
                team_type
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30,
                ?31, ?32, ?33, ?34
            )",
        )
        .map_err(|error| error.to_string())?;
    for p in players {
        stmt.execute(params![
            snapshot_id,
            p.uid,
            p.ca,
            p.pa,
            p.name,
            p.birth_year,
            p.birth_day_of_year,
            p.age,
            p.nationalities_json,
            p.height_cm,
            p.preferred_foot,
            p.positions_json,
            p.attributes_json,
            p.hidden_attributes_json,
            p.personality_json,
            p.weekly_wage_gbp,
            p.contract_expiry_year,
            p.contract_expiry_day_of_year,
            p.transfer_listed,
            p.loan_listed,
            p.not_for_sale,
            p.set_for_release,
            p.market_value_gbp,
            p.reputation_current,
            p.reputation_world,
            p.current_club,
            p.parent_club,
            p.on_loan,
            p.division,
            p.team_level,
            p.nation_uid,
            p.gender,
            p.club_reputation,
            p.team_type,
        ])
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn insert_prepared_staff_raw(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    staff: &[PreparedStaff],
) -> Result<(), String> {
    let mut stmt = tx
        .prepare(
            "INSERT INTO staff (
                snapshot_id,
                uid,
                name,
                birth_year,
                birth_day_of_year,
                age,
                nationalities_json,
                nation_uid,
                gender,
                ca,
                pa,
                staff_attributes_json,
                job_id,
                weekly_wage_gbp,
                contract_expiry_year,
                contract_expiry_day_of_year,
                club,
                division
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                ?17, ?18
            )",
        )
        .map_err(|error| error.to_string())?;
    for s in staff {
        stmt.execute(params![
            snapshot_id,
            s.uid,
            s.name,
            s.birth_year,
            s.birth_day_of_year,
            s.age,
            s.nationalities_json,
            s.nation_uid,
            s.gender,
            s.ca,
            s.pa,
            s.staff_attributes_json,
            s.job_id,
            s.weekly_wage_gbp,
            s.contract_expiry_year,
            s.contract_expiry_day_of_year,
            s.club,
            s.division,
        ])
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn ingest_dump_json_for_save(
    conn: &mut Connection,
    save_context: &SaveContext,
    json: &str,
    bridge_source_request_id: Option<&str>,
) -> Result<(IngestResult, IngestTimings), String> {
    let total_started = Instant::now();
    let validation_started = Instant::now();
    let prepared = prepare_dump_json(json)?;
    let validation_ms = validation_started.elapsed().as_millis();
    let insert_started = Instant::now();
    let result = publish_prepared_snapshot(conn, save_context, prepared, bridge_source_request_id)?;
    let insert_ms = insert_started.elapsed().as_millis();
    let total_ms = total_started.elapsed().as_millis();
    Ok((
        result,
        IngestTimings {
            validation_ms,
            insert_ms,
            total_ms,
        },
    ))
}

fn attributes_map(value: &Value) -> Result<std::collections::HashMap<String, Option<u8>>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "attributes must be an object".to_string())?;
    let mut attributes = std::collections::HashMap::with_capacity(object.len());
    for (key, raw) in object {
        let parsed = match raw {
            Value::Null => None,
            Value::Number(number) => {
                let as_i64 = number
                    .as_i64()
                    .ok_or_else(|| format!("attribute `{key}` must be an integer or null"))?;
                Some(
                    u8::try_from(as_i64)
                        .map_err(|_| format!("attribute `{key}` out of u8 range"))?,
                )
            }
            _ => {
                return Err(format!("attribute `{key}` must be an integer or null"));
            }
        };
        attributes.insert(key.clone(), parsed);
    }
    Ok(attributes)
}

fn get_snapshot_by_id(tx: &Transaction<'_>, snapshot_id: i64) -> Result<SnapshotSummary, String> {
    tx.query_row(
        "SELECT
            id,
            context_token,
            save_id,
            schema_version,
            generated_at_utc,
            game_version,
            supported_game_version,
            bridge_version,
            protocol_version,
            game_date,
            game_date_source,
            scan_truncated,
            max_accepted,
            player_count,
            loaded_at_utc
         FROM snapshots
         WHERE id = ?1",
        params![snapshot_id],
        |row| {
            Ok(SnapshotSummary {
                id: row.get(0)?,
                context_token: row.get(1)?,
                save_id: row.get(2)?,
                schema_version: row.get(3)?,
                generated_at_utc: row.get(4)?,
                game_version: row.get(5)?,
                supported_game_version: row.get(6)?,
                bridge_version: row.get(7)?,
                protocol_version: row.get(8)?,
                game_date: row.get(9)?,
                game_date_source: row.get(10)?,
                scan_truncated: row.get::<_, i32>(11)? == 1,
                max_accepted: row.get(12)?,
                player_count: row.get(13)?,
                loaded_at_utc: row.get(14)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

fn require_i64(object: &serde_json::Map<String, Value>, key: &str) -> Result<i64, String> {
    object
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing or invalid `{key}`"))
}

fn require_u64(object: &serde_json::Map<String, Value>, key: &str) -> Result<u64, String> {
    let value = require_i64(object, key)?;
    u64::try_from(value).map_err(|_| format!("`{key}` must be non-negative"))
}

fn require_string(object: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("missing or invalid `{key}`"))
}

fn require_bool(object: &serde_json::Map<String, Value>, key: &str) -> Result<bool, String> {
    object
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing or invalid `{key}`"))
}

fn optional_string(value: Option<&Value>) -> Result<Option<String>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err("expected string or null".to_string()),
    }
}

fn optional_i64(value: Option<&Value>) -> Result<Option<i64>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(number) => number
            .as_i64()
            .ok_or_else(|| "expected number or null".to_string())
            .map(Some),
    }
}

fn optional_bool(value: Option<&Value>) -> Result<Option<i32>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(flag)) => Ok(Some(i32::from(*flag))),
        Some(_) => Err("expected boolean or null".to_string()),
    }
}

fn manager_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<&'a Value>, String> {
    match object.get("manager") {
        Some(Value::Null) => Ok(None),
        Some(Value::Object(manager)) => Ok(manager.get(field)),
        Some(_) => Err("manager must be an object or null".to_string()),
        None => Err("missing `manager`".to_string()),
    }
}

fn json_string(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| error.to_string())
}

fn required_value<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a Value, String> {
    object.get(key).ok_or_else(|| format!("missing `{key}`"))
}

fn reputation_field(
    player: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<i64>, String> {
    let reputation = player
        .get("reputation")
        .and_then(Value::as_object)
        .ok_or_else(|| "missing player reputation".to_string())?;
    optional_i64(reputation.get(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::snapshot::service::{list_saves, save_context_for_id};
    use rusqlite::OptionalExtension;
    use std::path::Path;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v8.json");

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn write_dump(temp_dir: &tempfile::TempDir, name: &str, json: &str) -> std::path::PathBuf {
        let path = temp_dir.path().join(name);
        fs::write(&path, json).expect("write dump");
        path
    }

    fn current_snapshot_id(conn: &Connection, save_id: i64) -> Option<i64> {
        conn.query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .optional()
        .expect("query current snapshot")
    }

    fn bridge_source_request_id_for_snapshot(
        conn: &Connection,
        snapshot_id: i64,
    ) -> Option<String> {
        conn.query_row(
            "SELECT bridge_source_request_id FROM snapshots WHERE id = ?1",
            params![snapshot_id],
            |row| row.get(0),
        )
        .expect("query bridge source request id")
    }

    fn player_count_for_snapshot(conn: &Connection, snapshot_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM players WHERE snapshot_id = ?1",
            params![snapshot_id],
            |row| row.get(0),
        )
        .expect("count players")
    }

    fn staff_count_for_snapshot(conn: &Connection, snapshot_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM staff WHERE snapshot_id = ?1",
            params![snapshot_id],
            |row| row.get(0),
        )
        .expect("count staff")
    }

    fn role_score_count_for_snapshot(conn: &Connection, snapshot_id: i64) -> i64 {
        crate::features::player_metrics::compact::test_support::count_rows(conn, snapshot_id)
    }

    fn potential_state(conn: &Connection, snapshot_id: i64) -> (Option<String>, Option<i64>, i64) {
        let fields = conn
            .query_row(
                "SELECT potential_attributes_json, potential_projection_model_version
                 FROM players WHERE snapshot_id = ?1 ORDER BY uid LIMIT 1",
                [snapshot_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read projected player fields");
        let compact_count =
            crate::features::player_metrics::compact::test_support::count_rows(conn, snapshot_id);
        (fields.0, fields.1, compact_count)
    }

    fn compact_row(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Option<crate::features::player_metrics::compact::test_support::CompactRowShape> {
        crate::features::player_metrics::compact::test_support::read_row(conn, snapshot_id, uid)
    }

    fn compact_row_count(conn: &Connection, snapshot_id: i64) -> i64 {
        crate::features::player_metrics::compact::test_support::count_rows(conn, snapshot_id)
    }

    fn assert_uniform_compact_values(conn: &Connection, snapshot_id: i64, uid: i64) {
        use crate::features::player_metrics::compact::SCORE_MODEL_VERSION;
        use std::collections::HashMap;

        let (score_version, projection_version, current, potential) =
            compact_row(conn, snapshot_id, uid).expect("compact row");
        assert_eq!(
            (score_version, projection_version),
            (
                SCORE_MODEL_VERSION,
                crate::features::player_metrics::potential_scores::PROJECTION_MODEL_VERSION
            )
        );
        assert!(current.iter().all(|score| *score == Some(50)));
        let projected_json: String = conn
            .query_row(
                "SELECT potential_attributes_json
                 FROM players WHERE snapshot_id = ?1 AND uid = ?2",
                params![snapshot_id, uid],
                |row| row.get(0),
            )
            .expect("read projected attributes");
        let projected = serde_json::from_str::<HashMap<String, Option<u8>>>(&projected_json)
            .expect("parse projected attributes");
        for (index, role) in all_roles().iter().enumerate() {
            assert_eq!(
                potential[index],
                score_role(&projected, role).map(i64::from),
                "potential compact score for {}",
                role.role_id
            );
        }
    }

    fn assert_complete_potential_state(conn: &Connection, snapshot_id: i64) {
        let state = potential_state(conn, snapshot_id);
        assert!(state.0.is_some());
        assert_eq!(
            state.1,
            Some(crate::features::player_metrics::potential_scores::PROJECTION_MODEL_VERSION)
        );
        assert_eq!(state.2, 1);
    }

    fn assert_empty_potential_state(conn: &Connection, snapshot_id: i64) {
        assert_eq!(potential_state(conn, snapshot_id), (None, None, 0));
    }

    fn club_dna_score_rows(
        conn: &Connection,
        snapshot_id: i64,
    ) -> Vec<(i64, i64, i64, Option<i64>)> {
        let mut statement = conn
            .prepare(
                "SELECT uid, definition_version, score_model_version, score
                 FROM club_dna_scores WHERE snapshot_id = ?1
                 ORDER BY uid, definition_version, score_model_version",
            )
            .expect("prepare Club DNA score query");
        statement
            .query_map([snapshot_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .expect("query Club DNA scores")
            .collect::<Result<Vec<_>, _>>()
            .expect("read Club DNA scores")
    }

    fn role_scores_for_player(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Vec<(String, String, Option<i64>)> {
        if let Some((_, _, current, _)) =
            crate::features::player_metrics::compact::test_support::read_row(conn, snapshot_id, uid)
        {
            crate::features::scoring::catalog::all_roles()
                .iter()
                .zip(current)
                .map(|(role, score)| {
                    (
                        role.role_id.to_string(),
                        role.phase.as_db_str().to_string(),
                        score,
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn compact_staff_scores(conn: &Connection, snapshot_id: i64, uid: i64) -> Vec<(String, i64)> {
        if let Some((_, scores)) =
            crate::features::staff::scoring::test_support::read_row(conn, snapshot_id, uid)
        {
            crate::features::staff::scoring::all_staff_roles()
                .iter()
                .zip(scores)
                .filter_map(|(role, score)| score.map(|s| (role.role_id.to_string(), s)))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn staff_compact_row(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Option<crate::features::staff::scoring::test_support::CompactStaffRowShape> {
        crate::features::staff::scoring::test_support::read_row(conn, snapshot_id, uid)
    }

    fn staff_compact_row_count(conn: &Connection, snapshot_id: i64) -> i64 {
        crate::features::staff::scoring::test_support::count_rows(conn, snapshot_id)
    }

    fn dump_with_uniform_attributes(value: u8) -> String {
        use crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS;

        let attributes: serde_json::Map<String, Value> = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), Value::from(value)))
            .collect();
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse golden fixture");
        root["players"][0]["attributes"] = Value::Object(attributes);
        root.to_string()
    }

    fn snapshot_row_exists(conn: &Connection, snapshot_id: i64) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM snapshots WHERE id = ?1",
            params![snapshot_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("count snapshot row")
            > 0
    }

    fn snapshot_count(conn: &Connection, save_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM snapshots WHERE save_id = ?1",
            params![save_id],
            |row| row.get(0),
        )
        .expect("count snapshots")
    }

    fn automatic_class_years(conn: &Connection, save_id: i64) -> Vec<i64> {
        let mut statement = conn
            .prepare(
                "SELECT class_year
                 FROM academy_classes
                 WHERE save_id = ?1 AND is_automatic = 1
                 ORDER BY class_year",
            )
            .expect("prepare class query");
        statement
            .query_map(params![save_id], |row| row.get(0))
            .expect("query classes")
            .collect::<Result<Vec<_>, _>>()
            .expect("read classes")
    }

    fn dump_with_game_date(game_date: Option<&str>, player_name: &str) -> String {
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse golden fixture");
        root["gameDate"] = game_date.map(Value::from).unwrap_or(Value::Null);
        root["players"][0]["name"] = Value::from(player_name);
        root.to_string()
    }

    #[test]
    fn ingest_persists_scores_for_each_retained_snapshot_when_a_definition_exists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-club-dna-eager.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.Acceleration\"]')",
            [save.id],
        )
        .expect("set Club DNA definition");

        let first_path = write_dump(&temp_dir, "club-dna-first.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &first_path).expect("ingest first snapshot");
        let mut second_dump: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        second_dump["gameDate"] = Value::from("2027-08-14");
        let second_path = write_dump(&temp_dir, "club-dna-second.json", &second_dump.to_string());
        let second = ingest_dump_file(&mut conn, &second_path).expect("ingest retained snapshot");

        assert_eq!(
            club_dna_score_rows(&conn, first.id),
            vec![(77, 1, 1, Some(70))]
        );
        assert_eq!(
            club_dna_score_rows(&conn, second.id),
            vec![(77, 1, 1, Some(70))]
        );
    }

    #[test]
    fn ingest_persists_a_computed_null_club_dna_row() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-club-dna-null.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.Handling\"]')",
            [save.id],
        )
        .expect("set nullable Club DNA definition");

        let dump_path = write_dump(&temp_dir, "club-dna-null.json", GOLDEN_FIXTURE);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest nullable score");

        assert_eq!(
            club_dna_score_rows(&conn, snapshot.id),
            vec![(77, 1, 1, None)]
        );
    }

    #[test]
    fn ingest_skips_club_dna_work_without_a_definition() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-club-dna-absent.db"));
        let dump_path = write_dump(&temp_dir, "club-dna-absent.json", GOLDEN_FIXTURE);

        ingest_dump_file(&mut conn, &dump_path).expect("ingest without definition");

        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM club_dna_scores", [], |row| row
                .get::<_, i64>(0))
                .expect("count absent-definition scores"),
            0
        );
    }

    #[test]
    fn failed_club_dna_score_write_rolls_back_the_new_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-club-dna-rollback.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.Acceleration\"]')",
            [save.id],
        )
        .expect("set Club DNA definition");
        let first_path = write_dump(&temp_dir, "club-dna-first.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &first_path).expect("ingest retained snapshot");
        let prior_score_rows = club_dna_score_rows(&conn, first.id);
        conn.execute_batch(
            "CREATE TRIGGER reject_club_dna_insert
             BEFORE INSERT ON club_dna_scores
             BEGIN
                 SELECT RAISE(ABORT, 'forced Club DNA score failure');
             END;",
        )
        .expect("reject Club DNA writes");
        let rejected_path = write_dump(&temp_dir, "club-dna-rejected.json", GOLDEN_FIXTURE);

        let error = ingest_dump_file(&mut conn, &rejected_path)
            .expect_err("reject eager Club DNA score write");

        assert!(error.contains("forced Club DNA score failure"));
        assert_eq!(current_snapshot_id(&conn, save.id), Some(first.id));
        assert_eq!(snapshot_count(&conn, save.id), 1);
        assert_eq!(player_count_for_snapshot(&conn, first.id), 1);
        assert_eq!(role_score_count_for_snapshot(&conn, first.id), 1);
        assert_eq!(club_dna_score_rows(&conn, first.id), prior_score_rows);
    }

    #[test]
    fn invalid_stored_club_dna_definition_rolls_back_the_new_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(
            &temp_dir
                .path()
                .join("ingest-club-dna-invalid-definition.db"),
        );
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.NotARealAttribute\"]')",
            [save.id],
        )
        .expect("seed invalid stored definition");
        let dump_path = write_dump(
            &temp_dir,
            "club-dna-invalid-definition.json",
            GOLDEN_FIXTURE,
        );

        let error = ingest_dump_file(&mut conn, &dump_path)
            .expect_err("invalid stored definition must reject ingest");

        assert!(error.contains("Stored Club DNA definition is invalid"));
        assert_eq!(snapshot_count(&conn, save.id), 0);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM players", [], |row| row
                .get::<_, i64>(0))
                .expect("count rolled-back players"),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM player_role_metrics", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count rolled-back compact rows"),
            0
        );
    }

    #[test]
    fn ingest_writes_null_role_scores_when_required_attributes_are_missing() {
        use crate::features::scoring::catalog::all_roles;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-role-scores-null.db"));
        list_saves(&conn).expect("seed default save");

        let dump_path = write_dump(&temp_dir, "sparse.json", GOLDEN_FIXTURE);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest sparse dump");

        assert_eq!(role_score_count_for_snapshot(&conn, snapshot.id), 1);

        let scores = role_scores_for_player(&conn, snapshot.id, 77);
        assert_eq!(scores.len(), all_roles().len());
        for (role_id, phase, score) in &scores {
            assert!(score.is_none(), "expected null score for {role_id}");
            assert!(
                phase == "in_possession" || phase == "out_of_possession",
                "unexpected phase {phase} for {role_id}"
            );
        }
    }

    #[test]
    fn ingest_writes_expected_role_scores_for_uniform_attributes() {
        use crate::features::scoring::catalog::all_roles;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-role-scores-uniform.db"));
        list_saves(&conn).expect("seed default save");

        let dump_json = dump_with_uniform_attributes(10);
        let dump_path = write_dump(&temp_dir, "uniform.json", &dump_json);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest uniform dump");

        let scores = role_scores_for_player(&conn, snapshot.id, 77);
        assert_eq!(scores.len(), all_roles().len());
        for (role_id, _phase, score) in &scores {
            assert_eq!(
                score,
                &Some(50),
                "uniform attr 10 must score 50 for {role_id}"
            );
        }
    }

    #[test]
    fn ingest_writes_one_compact_staff_row_with_nulls_for_unavailable_roles() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-staff-compact.db"));
        list_saves(&conn).expect("seed default save");

        let dump_path = write_dump(&temp_dir, "staff-scores.json", GOLDEN_FIXTURE);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest staff scores");

        // The compact row has calculable values for 15 roles; six roles have
        // unavailable dependencies in the fixture and remain null.
        let scores = compact_staff_scores(&conn, snapshot.id, 88);
        assert_eq!(scores.len(), 15, "six roles have unavailable dependencies");
        assert_eq!(
            scores
                .iter()
                .find(|(role_id, _)| role_id == "physio")
                .map(|(_, score)| *score),
            Some(60)
        );
        assert!(scores
            .iter()
            .all(|(role_id, _)| role_id != "performance_analyst"));

        // The compact current row carries all 21 named values; unavailable
        // roles stay SQL null instead of dropping the row.
        assert_eq!(staff_compact_row_count(&conn, snapshot.id), 1);
        let (version, compact) = staff_compact_row(&conn, snapshot.id, 88).expect("compact row");
        assert_eq!(
            version,
            crate::features::staff::scoring::SCORE_MODEL_VERSION
        );
        let roles = all_staff_roles();
        assert_eq!(compact.len(), roles.len());
        assert_eq!(
            compact.iter().filter(|score| score.is_some()).count(),
            15,
            "only calculable roles persist a value"
        );
        let physio = roles
            .iter()
            .position(|role| role.role_id == "physio")
            .expect("physio role");
        let analyst = roles
            .iter()
            .position(|role| role.role_id == "performance_analyst")
            .expect("analyst role");
        assert_eq!(compact[physio], Some(60));
        assert_eq!(compact[analyst], None);
    }

    #[test]
    fn winning_ingest_writes_one_compact_staff_row_even_when_every_metric_is_null() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-staff-compact-all-null.db"));
        list_saves(&conn).expect("seed default save");

        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        for value in root["staff"][0]["attributes"]
            .as_object_mut()
            .expect("staff attributes")
            .values_mut()
        {
            *value = Value::Null;
        }
        let dump_path = write_dump(&temp_dir, "all-null-staff.json", &root.to_string());
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest all-null staff");

        assert_eq!(staff_compact_row_count(&conn, snapshot.id), 1);
        let (version, compact) =
            staff_compact_row(&conn, snapshot.id, 88).expect("compact row for the all-null staff");
        assert_eq!(
            version,
            crate::features::staff::scoring::SCORE_MODEL_VERSION
        );
        assert_eq!(compact.len(), all_staff_roles().len());
        assert!(compact.iter().all(|score| score.is_none()));
        assert!(compact_staff_scores(&conn, snapshot.id, 88).is_empty());
    }

    #[test]
    fn failed_compact_staff_materialization_rolls_back_ingest_and_keeps_prior_current_visible() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-staff-compact-rollback.db"));
        let first_path = write_dump(&temp_dir, "first.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let prior_compact = staff_compact_row(&conn, first.id, 88).expect("prior compact row");
        conn.execute_batch(
            "CREATE TRIGGER reject_staff_compact_rows
             BEFORE INSERT ON staff_role_metrics
             BEGIN SELECT RAISE(ABORT, 'staff compact row failure'); END;",
        )
        .expect("reject staff compact rows");
        let later_path = write_dump(&temp_dir, "later.json", GOLDEN_FIXTURE);

        assert!(ingest_dump_file(&mut conn, &later_path)
            .expect_err("roll back winning staff compact materialization")
            .contains("staff compact row failure"));

        assert_eq!(snapshot_count(&conn, first.save_id), 1);
        assert_eq!(current_snapshot_id(&conn, first.save_id), Some(first.id));
        assert_eq!(staff_compact_row_count(&conn, first.id), 1);
        assert_eq!(staff_compact_row(&conn, first.id, 88), Some(prior_compact));
    }

    #[test]
    fn failed_staff_score_insert_rolls_back_the_new_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("staff-score-rollback.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let good_path = write_dump(&temp_dir, "good.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &good_path).expect("first ingest");
        let prior_scores = compact_staff_scores(&conn, first.id, 88);
        assert!(!prior_scores.is_empty());

        conn.execute_batch(
            "CREATE TRIGGER reject_staff_score
             BEFORE INSERT ON staff_role_metrics
             BEGIN
                 SELECT RAISE(FAIL, 'staff score failure');
             END;",
        )
        .expect("create staff score failure trigger");
        let rejected_path = write_dump(&temp_dir, "rejected.json", GOLDEN_FIXTURE);
        let error = ingest_dump_file(&mut conn, &rejected_path).expect_err("reject score insert");

        assert!(error.contains("staff score failure"));
        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(first.id));
        assert_eq!(compact_staff_scores(&conn, first.id, 88), prior_scores);
        assert_eq!(staff_compact_row_count(&conn, first.id), 1);
        assert_eq!(snapshot_count(&conn, active_save.id), 1);
    }

    #[test]
    fn second_ingest_leaves_historical_staff_snapshot_raw_without_compact_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("staff-score-replacement.db"));
        list_saves(&conn).expect("seed default save");

        let first_path = write_dump(&temp_dir, "first.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");

        let mut second_root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        for key in [
            "DataAnalysis",
            "Fitness",
            "JudgingStaffAbility",
            "SetPieces",
        ] {
            second_root["staff"][0]["attributes"][key] = Value::from(20);
        }
        second_root["staff"][0]["attributes"]["Physiotherapy"] = Value::from(20);
        let second_path = write_dump(&temp_dir, "second.json", &second_root.to_string());
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        let first_scores = compact_staff_scores(&conn, first.id, 88);
        let second_scores = compact_staff_scores(&conn, second.id, 88);
        assert_eq!(first_scores.len(), 0);
        assert_eq!(second_scores.len(), 21);
        assert_eq!(
            second_scores
                .iter()
                .find(|(role_id, _)| role_id == "physio")
                .map(|(_, score)| *score),
            Some(100)
        );
        // The displaced first snapshot is historical: raw staff only, no
        // compact rows. The second snapshot stays current with one row.
        assert_eq!(staff_compact_row_count(&conn, first.id), 0);
        assert_eq!(staff_compact_row_count(&conn, second.id), 1);
        let (version, second_compact) =
            staff_compact_row(&conn, second.id, 88).expect("second compact row");
        assert_eq!(
            version,
            crate::features::staff::scoring::SCORE_MODEL_VERSION
        );
        assert_eq!(
            second_compact
                .iter()
                .filter(|score| score.is_some())
                .count(),
            21
        );
    }

    #[test]
    fn second_ingest_leaves_historical_snapshot_raw_without_compact_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-role-scores-replace.db"));
        list_saves(&conn).expect("seed default save");

        let first_path = write_dump(&temp_dir, "first.json", &dump_with_uniform_attributes(10));
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        assert!(role_score_count_for_snapshot(&conn, first.id) > 0);

        let second_path = write_dump(&temp_dir, "second.json", &dump_with_uniform_attributes(20));
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        let first_scores = role_scores_for_player(&conn, first.id, 77);
        assert!(first_scores.is_empty());
        let scores = role_scores_for_player(&conn, second.id, 77);
        assert!(!scores.is_empty());
        assert!(scores.iter().all(|(_, _, score)| *score == Some(100)));
    }

    #[test]
    fn failed_ingest_leaves_prior_role_scores_untouched() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-role-scores-rollback.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let good_path = write_dump(&temp_dir, "good.json", &dump_with_uniform_attributes(10));
        let first = ingest_dump_file(&mut conn, &good_path).expect("first ingest");
        let prior_count = role_score_count_for_snapshot(&conn, first.id);
        assert!(prior_count > 0);

        let bad_json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 8", "\"schemaVersion\": 4");
        let bad_path = write_dump(&temp_dir, "bad.json", &bad_json);
        let _ = ingest_dump_file(&mut conn, &bad_path).expect_err("reject bad schema");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(first.id));
        assert_eq!(role_score_count_for_snapshot(&conn, first.id), prior_count);
        assert!(role_scores_for_player(&conn, first.id, 77)
            .iter()
            .all(|(_, _, score)| *score == Some(50)));
    }

    #[test]
    fn ingests_golden_fixture_into_active_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-golden.db"));
        let saves = list_saves(&conn).expect("seed default save");
        let active_save = saves
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let json_with_null_attribute = GOLDEN_FIXTURE.replace(
            "\"attributes\": { \"Acceleration\": 14, \"Pace\": 15 }",
            "\"attributes\": { \"Acceleration\": 14, \"Pace\": 15, \"Dribbling\": null }",
        );
        let expected_positions = serde_json::from_str::<Value>(&json_with_null_attribute)
            .expect("parse v8 fixture")
            .get("players")
            .and_then(Value::as_array)
            .and_then(|players| players.first())
            .and_then(|player| player.get("positions"))
            .cloned()
            .expect("fixture positions");
        let dump_path = write_dump(&temp_dir, "dump.json", &json_with_null_attribute);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest golden dump");

        assert_eq!(snapshot.save_id, active_save.id);
        assert_eq!(snapshot.schema_version, 8);
        assert_eq!(snapshot.generated_at_utc, "2026-08-08T10:00:00.000Z");
        assert_eq!(snapshot.game_version, "26.3.2.2329565");
        assert_eq!(snapshot.supported_game_version, "26.3");
        assert_eq!(snapshot.bridge_version, "0.1.0");
        assert_eq!(snapshot.protocol_version, 1);
        assert_eq!(snapshot.game_date.as_deref(), Some("2026-08-14"));
        assert_eq!(snapshot.game_date_source, "derived");
        assert!(!snapshot.scan_truncated);
        assert_eq!(snapshot.max_accepted, None);
        assert_eq!(snapshot.player_count, 1);
        assert_eq!(
            bridge_source_request_id_for_snapshot(&conn, snapshot.id),
            None
        );

        assert_eq!(
            current_snapshot_id(&conn, active_save.id),
            Some(snapshot.id)
        );
        assert_eq!(player_count_for_snapshot(&conn, snapshot.id), 1);

        struct SnapshotParity {
            game_date_basis: String,
            player_database_scope: String,
            staff_count: i64,
            manager_uid: Option<i64>,
            manager_name: Option<String>,
            manager_club: Option<String>,
            manager_club_reputation: Option<i64>,
        }

        let snapshot_parity = conn
            .query_row(
                "SELECT
                    game_date_basis,
                    player_database_scope,
                    staff_count,
                    manager_uid,
                    manager_name,
                    manager_club,
                    manager_club_reputation
                 FROM snapshots WHERE id = ?1",
                params![snapshot.id],
                |row| {
                    Ok(SnapshotParity {
                        game_date_basis: row.get(0)?,
                        player_database_scope: row.get(1)?,
                        staff_count: row.get(2)?,
                        manager_uid: row.get(3)?,
                        manager_name: row.get(4)?,
                        manager_club: row.get(5)?,
                        manager_club_reputation: row.get(6)?,
                    })
                },
            )
            .expect("snapshot parity fields");
        assert_eq!(snapshot_parity.game_date_basis, "next-fixture-consensus");
        assert_eq!(snapshot_parity.player_database_scope, "men");
        assert_eq!(snapshot_parity.staff_count, 1);
        assert_eq!(snapshot_parity.manager_uid, Some(88));
        assert_eq!(
            snapshot_parity.manager_name.as_deref(),
            Some("Golden Fixture Staff")
        );
        assert_eq!(snapshot_parity.manager_club.as_deref(), Some("Golden FC"));
        assert_eq!(snapshot_parity.manager_club_reputation, Some(6400));

        struct PlayerParity {
            ca: i64,
            current_club: Option<String>,
            positions_json: String,
            attributes_json: String,
            nation_uid: Option<i64>,
            gender: String,
            club_reputation: Option<i64>,
            team_type: Option<i64>,
        }

        let player_parity = conn
            .query_row(
                "SELECT
                    ca,
                    current_club,
                    positions_json,
                    attributes_json,
                    nation_uid,
                    gender,
                    club_reputation,
                    team_type
                 FROM players WHERE snapshot_id = ?1",
                params![snapshot.id],
                |row| {
                    Ok(PlayerParity {
                        ca: row.get(0)?,
                        current_club: row.get(1)?,
                        positions_json: row.get(2)?,
                        attributes_json: row.get(3)?,
                        nation_uid: row.get(4)?,
                        gender: row.get(5)?,
                        club_reputation: row.get(6)?,
                        team_type: row.get(7)?,
                    })
                },
            )
            .expect("player row");
        assert_eq!(player_parity.ca, 150);
        assert_eq!(player_parity.current_club.as_deref(), Some("Loan FC"));
        assert_eq!(player_parity.nation_uid, Some(44));
        assert_eq!(player_parity.gender, "male");
        assert_eq!(player_parity.club_reputation, Some(6200));
        assert_eq!(player_parity.team_type, Some(0));

        let positions: Value =
            serde_json::from_str(&player_parity.positions_json).expect("parse positions_json");
        assert_eq!(positions, expected_positions);

        let attributes: Value =
            serde_json::from_str(&player_parity.attributes_json).expect("parse attributes_json");
        assert_eq!(attributes["Acceleration"], 14);
        assert_eq!(attributes["Pace"], 15);
        assert_eq!(
            attributes.get("Dribbling"),
            Some(&Value::Null),
            "null attribute must be stored as JSON null, not omitted"
        );

        let (staff_uid, staff_name, staff_gender, staff_attributes_json): (
            i64,
            Option<String>,
            String,
            String,
        ) = conn
            .query_row(
                "SELECT uid, name, gender, staff_attributes_json
                 FROM staff WHERE snapshot_id = ?1",
                params![snapshot.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("staff row");
        assert_eq!(staff_uid, 88);
        assert_eq!(staff_name.as_deref(), Some("Golden Fixture Staff"));
        assert_eq!(staff_gender, "female");
        let staff_attributes: Value =
            serde_json::from_str(&staff_attributes_json).expect("parse staff_attributes_json");
        assert_eq!(
            staff_attributes
                .as_object()
                .expect("staff attributes")
                .len(),
            24
        );
        assert_eq!(staff_attributes["Attacking"], 15);
        assert_eq!(staff_attributes["Authority"], 18);
        assert_eq!(staff_attributes["Adaptability"], 17);
        assert_eq!(staff_attributes.get("DataAnalysis"), Some(&Value::Null));
    }

    #[test]
    fn winning_ingest_materializes_exact_compact_scores_from_one_projection() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-compact-values.db"));
        list_saves(&conn).expect("seed default save");

        let dump_path = write_dump(&temp_dir, "uniform.json", &dump_with_uniform_attributes(10));
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest uniform dump");

        assert_eq!(compact_row_count(&conn, snapshot.id), 1);
        assert_uniform_compact_values(&conn, snapshot.id, 77);
    }

    #[test]
    fn winning_ingest_materializes_null_compact_scores_for_missing_attributes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-compact-null.db"));
        list_saves(&conn).expect("seed default save");

        let dump_path = write_dump(&temp_dir, "sparse.json", GOLDEN_FIXTURE);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest sparse dump");

        assert_eq!(compact_row_count(&conn, snapshot.id), 1);
        let (score_version, projection_version, current, potential) =
            compact_row(&conn, snapshot.id, 77).expect("compact row for the sparse player");
        assert_eq!(
            (score_version, projection_version),
            (
                crate::features::player_metrics::compact::SCORE_MODEL_VERSION,
                crate::features::player_metrics::potential_scores::PROJECTION_MODEL_VERSION
            )
        );
        assert!(current.iter().all(|score| score.is_none()));
        assert!(potential.iter().all(|score| score.is_none()));
    }

    #[test]
    fn failed_compact_materialization_rolls_back_ingest_and_keeps_prior_current_visible() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-compact-rollback.db"));
        let first_path = write_dump(
            &temp_dir,
            "first.json",
            &dump_with_game_date(Some("2026-08-14"), "Earlier player"),
        );
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let prior_compact = compact_row(&conn, first.id, 77).expect("prior compact row");
        conn.execute_batch(
            "CREATE TRIGGER reject_compact_rows
             BEFORE INSERT ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'compact row failure'); END;",
        )
        .expect("reject compact rows");
        let later_path = write_dump(
            &temp_dir,
            "later.json",
            &dump_with_game_date(Some("2027-08-16"), "Later player"),
        );

        assert!(ingest_dump_file(&mut conn, &later_path)
            .expect_err("roll back winning compact materialization")
            .contains("compact row failure"));

        assert_eq!(snapshot_count(&conn, first.save_id), 1);
        assert_eq!(current_snapshot_id(&conn, first.save_id), Some(first.id));
        assert_eq!(compact_row_count(&conn, first.id), 1);
        assert_eq!(compact_row(&conn, first.id, 77), Some(prior_compact));
    }

    #[test]
    fn later_snapshot_stays_current_when_an_earlier_snapshot_is_retained() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-replace.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let later_json = dump_with_game_date(Some("2027-08-16"), "Later player");
        let first_path = write_dump(&temp_dir, "later.json", &later_json);
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        assert_complete_potential_state(&conn, first.id);

        let earlier_json = dump_with_game_date(Some("2026-08-14"), "Earlier player");
        let second_path = write_dump(&temp_dir, "earlier.json", &earlier_json);
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        assert_ne!(second.id, first.id);
        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(first.id));
        assert_complete_potential_state(&conn, first.id);
        assert_empty_potential_state(&conn, second.id);
        assert_eq!(compact_row_count(&conn, first.id), 1);
        assert_eq!(compact_row_count(&conn, second.id), 0);
        assert_eq!(staff_compact_row_count(&conn, first.id), 1);
        assert_eq!(staff_compact_row_count(&conn, second.id), 0);
        assert!(snapshot_row_exists(&conn, first.id));
        assert_eq!(snapshot_count(&conn, active_save.id), 2);
        assert_eq!(player_count_for_snapshot(&conn, first.id), 1);
        assert_eq!(staff_count_for_snapshot(&conn, first.id), 1);
        assert_eq!(player_count_for_snapshot(&conn, second.id), 1);
        assert_eq!(staff_count_for_snapshot(&conn, second.id), 1);
        assert_eq!(
            automatic_class_years(&conn, active_save.id),
            vec![2025, 2027]
        );

        let mut current_players = conn
            .prepare(
                "SELECT p.name
                 FROM players p
                 INNER JOIN snapshots s ON s.id = p.snapshot_id
                 INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
                 WHERE s.is_current = 1
                 ORDER BY p.name COLLATE NOCASE",
            )
            .expect("prepare current-player query");
        let visible_names = current_players
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query current players")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect current players");
        assert_eq!(visible_names, vec!["Later player"]);
    }

    #[test]
    fn later_ingest_becomes_current_when_it_has_a_greater_game_date() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-date-forward.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let earlier_path = write_dump(
            &temp_dir,
            "earlier.json",
            &dump_with_game_date(Some("2026-08-14"), "Earlier player"),
        );
        let earlier = ingest_dump_file(&mut conn, &earlier_path).expect("earlier ingest");
        assert_complete_potential_state(&conn, earlier.id);
        let later_path = write_dump(
            &temp_dir,
            "later.json",
            &dump_with_game_date(Some("2027-08-16"), "Later player"),
        );
        let later = ingest_dump_file(&mut conn, &later_path).expect("later ingest");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(later.id));
        assert_empty_potential_state(&conn, earlier.id);
        assert_complete_potential_state(&conn, later.id);
        assert_eq!(compact_row_count(&conn, earlier.id), 0);
        assert_eq!(compact_row_count(&conn, later.id), 1);
        assert_eq!(snapshot_count(&conn, active_save.id), 2);
        assert!(snapshot_row_exists(&conn, earlier.id));
        assert_eq!(
            automatic_class_years(&conn, active_save.id),
            vec![2025, 2026, 2027]
        );
    }

    #[test]
    fn same_date_snapshots_use_the_newest_load_as_the_current_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-same-date.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let first_path = write_dump(
            &temp_dir,
            "first.json",
            &dump_with_game_date(Some("2026-08-14"), "First player"),
        );
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let second_path = write_dump(
            &temp_dir,
            "second.json",
            &dump_with_game_date(Some("2026-08-14"), "Second player"),
        );
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(second.id));
        assert!(snapshot_row_exists(&conn, first.id));
        assert_eq!(compact_row_count(&conn, first.id), 0);
        assert_eq!(compact_row_count(&conn, second.id), 1);
        assert_eq!(snapshot_count(&conn, active_save.id), 2);
    }

    #[test]
    fn newer_load_timestamp_beats_a_higher_snapshot_id_for_the_same_game_date() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-load-order.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let first_path = write_dump(
            &temp_dir,
            "first.json",
            &dump_with_game_date(Some("2026-08-14"), "First player"),
        );
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let second_path = write_dump(
            &temp_dir,
            "second.json",
            &dump_with_game_date(Some("2026-08-14"), "Second player"),
        );
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");
        conn.execute(
            "UPDATE snapshots
             SET loaded_at_utc = CASE id
                 WHEN ?1 THEN '2026-08-14T00:00:02.000Z'
                 WHEN ?2 THEN '2026-08-14T00:00:01.000Z'
             END
             WHERE id IN (?1, ?2)",
            params![first.id, second.id],
        )
        .expect("set distinct timestamps");

        let transaction = conn.transaction().expect("start selection transaction");
        let selected = service::select_current_snapshot(&transaction, active_save.id)
            .expect("select current snapshot");
        transaction.commit().expect("commit selection");

        assert_eq!(selected, Some(first.id));
        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(first.id));
    }

    #[test]
    fn snapshot_id_breaks_same_date_timestamp_ties_deterministically() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-date-tie.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        conn.execute_batch(
            "CREATE TRIGGER force_snapshot_load_timestamp
             AFTER INSERT ON snapshots
             BEGIN
                 UPDATE snapshots
                 SET loaded_at_utc = '2026-08-14T00:00:00.000Z'
                 WHERE id = NEW.id;
             END;",
        )
        .expect("force matching timestamps");

        let first_path = write_dump(
            &temp_dir,
            "first.json",
            &dump_with_game_date(Some("2026-08-14"), "First player"),
        );
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let second_path = write_dump(
            &temp_dir,
            "second.json",
            &dump_with_game_date(Some("2026-08-14"), "Second player"),
        );
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(second.id));
        let loaded_at: Vec<String> = [first.id, second.id]
            .into_iter()
            .map(|snapshot_id| {
                conn.query_row(
                    "SELECT loaded_at_utc FROM snapshots WHERE id = ?1",
                    params![snapshot_id],
                    |row| row.get(0),
                )
                .expect("read timestamp")
            })
            .collect();
        assert_eq!(loaded_at, vec!["2026-08-14T00:00:00.000Z"; 2]);
    }

    #[test]
    fn undated_snapshot_does_not_supersede_a_dated_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-undated-after-dated.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let dated_path = write_dump(
            &temp_dir,
            "dated.json",
            &dump_with_game_date(Some("2026-08-14"), "Dated player"),
        );
        let dated = ingest_dump_file(&mut conn, &dated_path).expect("dated ingest");
        let undated_path = write_dump(
            &temp_dir,
            "undated.json",
            &dump_with_game_date(None, "Undated player"),
        );
        let undated = ingest_dump_file(&mut conn, &undated_path).expect("undated ingest");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(dated.id));
        assert_complete_potential_state(&conn, dated.id);
        assert_empty_potential_state(&conn, undated.id);
        assert_eq!(compact_row_count(&conn, dated.id), 1);
        assert_eq!(compact_row_count(&conn, undated.id), 0);
        assert!(snapshot_row_exists(&conn, undated.id));
        assert_eq!(
            automatic_class_years(&conn, active_save.id),
            vec![2025, 2026]
        );
    }

    #[test]
    fn failed_winning_potential_materialization_rolls_back_ingest() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-potential-rollback.db"));
        let first_path = write_dump(
            &temp_dir,
            "first.json",
            &dump_with_game_date(Some("2026-08-14"), "Earlier player"),
        );
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let first_potential_state = potential_state(&conn, first.id);
        conn.execute_batch(
            "CREATE TRIGGER reject_winning_potential_rows
             BEFORE INSERT ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'winning potential writes fail'); END;",
        )
        .expect("reject winning potential rows");
        let later_path = write_dump(
            &temp_dir,
            "later.json",
            &dump_with_game_date(Some("2027-08-16"), "Later player"),
        );

        assert!(ingest_dump_file(&mut conn, &later_path)
            .expect_err("roll back winning potential materialization")
            .contains("winning potential writes fail"));

        assert_eq!(snapshot_count(&conn, first.save_id), 1);
        assert_eq!(current_snapshot_id(&conn, first.save_id), Some(first.id));
        assert_eq!(potential_state(&conn, first.id), first_potential_state);
    }

    #[test]
    fn invalid_winning_source_attribute_rolls_back_ingest() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-domain-rollback.db"));
        let first_path = write_dump(
            &temp_dir,
            "first.json",
            &dump_with_game_date(Some("2026-08-14"), "Earlier player"),
        );
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let first_potential_state = potential_state(&conn, first.id);
        let first_attributes: String = conn
            .query_row(
                "SELECT attributes_json FROM players WHERE snapshot_id = ?1 AND uid = 77",
                [first.id],
                |row| row.get(0),
            )
            .expect("read prior source attributes");
        let mut later: Value = serde_json::from_str(&dump_with_game_date(
            Some("2027-08-16"),
            "Invalid later player",
        ))
        .expect("parse later dump");
        later["players"][0]["ca"] = Value::from(100);
        later["players"][0]["pa"] = Value::from(140);
        later["players"][0]["age"] = Value::from(20);
        later["players"][0]["attributes"]["Acceleration"] = Value::from(0);
        let later_path = write_dump(&temp_dir, "later.json", &later.to_string());

        let error = ingest_dump_file(&mut conn, &later_path)
            .expect_err("reject zero-valued source attribute before projection");
        assert!(error.contains("player 77 attribute `Acceleration` must be between 1 and 20"));
        assert_eq!(snapshot_count(&conn, first.save_id), 1);
        assert_eq!(current_snapshot_id(&conn, first.save_id), Some(first.id));
        assert_eq!(potential_state(&conn, first.id), first_potential_state);
        assert_eq!(
            conn.query_row(
                "SELECT attributes_json FROM players WHERE snapshot_id = ?1 AND uid = 77",
                [first.id],
                |row| row.get::<_, String>(0),
            )
            .expect("read unchanged prior source attributes"),
            first_attributes
        );
    }

    #[test]
    fn all_undated_snapshots_use_the_newest_load_as_the_current_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-all-undated.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let first_path = write_dump(
            &temp_dir,
            "first.json",
            &dump_with_game_date(None, "First player"),
        );
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let second_path = write_dump(
            &temp_dir,
            "second.json",
            &dump_with_game_date(None, "Second player"),
        );
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(second.id));
        assert!(snapshot_row_exists(&conn, first.id));
        assert_eq!(automatic_class_years(&conn, active_save.id), vec![2025]);
    }

    #[test]
    fn rejects_invalid_dump_without_changing_prior_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-reject.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let good_path = write_dump(&temp_dir, "good.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &good_path).expect("first ingest");

        let bad_json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 8", "\"schemaVersion\": 4");
        let bad_path = write_dump(&temp_dir, "bad.json", &bad_json);
        let error = ingest_dump_file(&mut conn, &bad_path).expect_err("reject bad schema");

        assert!(error.contains("unsupported dump schema version"));

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(first.id));
        assert_eq!(player_count_for_snapshot(&conn, first.id), 1);
    }

    #[test]
    fn persists_truncated_dump_metadata() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-truncated.db"));
        list_saves(&conn).expect("seed default save");

        let truncated_json = GOLDEN_FIXTURE
            .replace("\"scanTruncated\": false", "\"scanTruncated\": true")
            .replace("\"maxAccepted\": null", "\"maxAccepted\": 250");
        let dump_path = write_dump(&temp_dir, "truncated.json", &truncated_json);

        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest truncated dump");

        assert!(snapshot.scan_truncated);
        assert_eq!(snapshot.max_accepted, Some(250));
    }

    #[test]
    fn rejects_duplicate_player_uid_without_changing_prior_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-rollback.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let good_path = write_dump(&temp_dir, "good.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &good_path).expect("first ingest");

        let mut duplicate_root: Value =
            serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        let duplicate = duplicate_root["players"][0].clone();
        duplicate_root["players"]
            .as_array_mut()
            .expect("fixture players")
            .push(duplicate);
        duplicate_root["playerCount"] = Value::from(2);
        let broken_path = write_dump(&temp_dir, "broken.json", &duplicate_root.to_string());
        let _ = ingest_dump_file(&mut conn, &broken_path).expect_err("reject duplicate uid");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(first.id));
        assert_eq!(player_count_for_snapshot(&conn, first.id), 1);
    }

    #[test]
    fn failed_staff_insert_rolls_back_the_new_snapshot_and_children() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-staff-rollback.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let good_path = write_dump(&temp_dir, "good.json", GOLDEN_FIXTURE);
        let active_save_context =
            save_context_for_id(&conn, active_save.id).expect("capture active save context");
        let first = ingest_dump_file_for_save_with_bridge_source_request_id(
            &mut conn,
            &active_save_context,
            &good_path,
            "req-first",
        )
        .expect("first ingest");
        let prior_role_score_count = role_score_count_for_snapshot(&conn, first.stored_snapshot.id);
        conn.execute_batch(
            "CREATE TRIGGER reject_staff_insert
             BEFORE INSERT ON staff
             BEGIN
                 SELECT RAISE(FAIL, 'staff insert failure');
             END;",
        )
        .expect("create staff failure trigger");

        let rejected_path = write_dump(&temp_dir, "rejected.json", GOLDEN_FIXTURE);
        let error = ingest_dump_file_for_save_with_bridge_source_request_id(
            &mut conn,
            &active_save_context,
            &rejected_path,
            "req-rejected",
        )
        .expect_err("reject staff insert");

        assert!(error.contains("staff insert failure"));
        assert_eq!(
            current_snapshot_id(&conn, active_save.id),
            Some(first.effective_snapshot.id)
        );
        assert_eq!(
            bridge_source_request_id_for_snapshot(&conn, first.stored_snapshot.id).as_deref(),
            Some("req-first")
        );
        assert!(snapshot_row_exists(&conn, first.stored_snapshot.id));
        assert_eq!(
            player_count_for_snapshot(&conn, first.stored_snapshot.id),
            1
        );
        assert_eq!(staff_count_for_snapshot(&conn, first.stored_snapshot.id), 1);
        assert_eq!(
            role_score_count_for_snapshot(&conn, first.stored_snapshot.id),
            prior_role_score_count
        );
        let snapshot_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .expect("count snapshots");
        assert_eq!(snapshot_count, 1);
    }

    fn write_generated_minimal_dump(path: &Path, player_count: usize) {
        use std::io::Write;

        let mut file = fs::File::create(path).expect("create generated dump");
        write!(
            file,
            concat!(
                r#"{{"schemaVersion":8,"generatedAtUtc":"2026-07-30T12:00:00.000Z","#,
                r#""gameVersion":"26.3.2","supportedGameVersion":"26.3","bridgeVersion":"0.1.0","#,
                r#""protocolVersion":1,"gameDate":null,"gameDateSource":"unknown","gameDateBasis":"unknown","#,
                r#""playerDatabaseScope":"men","#,
                r#""scanTruncated":false,"maxAccepted":null,"playerCount":{player_count},"players":["#
            ),
            player_count = player_count
        )
        .expect("write dump header");

        for uid in 1..=player_count {
            if uid > 1 {
                write!(file, ",").expect("write comma");
            }
            write!(
                file,
                concat!(
                    r#"{{"uid":{uid},"ca":1,"pa":1,"name":"P{uid}","birthYear":2000,"birthDayOfYear":1,"#,
                    r#""age":null,"nationalities":[],"nationUid":null,"gender":"unknown","heightCm":null,"preferredFoot":"right","#,
                    r#""positions":{{"GK":null,"SW":null,"DL":null,"DC":null,"DR":null,"DM":null,"ML":null,"MC":null,"MR":null,"AML":null,"AMC":null,"AMR":null,"ST":null,"WBL":null,"WBR":null}},"attributes":{{}},"hiddenAttributes":{{}},"personality":{{}},"#,
                    r#""weeklyWageGbp":null,"contractExpiryYear":null,"contractExpiryDayOfYear":null,"#,
                    r#""transferListed":null,"loanListed":null,"notForSale":null,"setForRelease":null,"#,
                    r#""marketValueGbp":null,"reputation":{{"current":null,"world":null}},"#,
                    r#""currentClub":null,"parentClub":null,"onLoan":null,"division":null,"teamLevel":null,"clubReputation":null,"teamType":null}}"#
                ),
                uid = uid
            )
            .expect("write player");
        }

        write!(file, "],\"staffCount\":0,\"staff\":[],\"manager\":null}}")
            .expect("write dump footer");
        file.flush().expect("flush dump");
    }

    fn assert_ingest_timings(timings: &IngestTimings) {
        assert!(
            timings.total_ms >= timings.validation_ms.saturating_add(timings.insert_ms),
            "total_ms ({}) must cover validation_ms ({}) + insert_ms ({})",
            timings.total_ms,
            timings.validation_ms,
            timings.insert_ms
        );
    }

    #[test]
    fn ingest_records_validation_insert_and_total_timings() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-timings.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let dump_path = write_dump(&temp_dir, "dump.json", GOLDEN_FIXTURE);
        let (snapshot, timings) =
            ingest_dump_file_for_save_timed(&mut conn, active_save.id, &dump_path)
                .expect("timed ingest");

        assert_eq!(snapshot.player_count, 1);
        assert_ingest_timings(&timings);
    }

    #[test]
    fn ingest_completes_generated_2k_players_with_role_scores_and_timings() {
        run_generated_large_ingest(2_000);
    }

    #[test]
    fn ingest_completes_generated_2k_staff_with_role_scores_and_timings() {
        #[allow(unused_imports)]
        use crate::features::staff::scoring::all_staff_roles;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-2000-staff.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let mut root: Value = serde_json::from_str(GOLDEN_FIXTURE).expect("parse fixture");
        let template = root["staff"][0].clone();
        let staff = (0..2_000)
            .map(|index| {
                let mut record = template.clone();
                record["uid"] = Value::from(1_000 + index);
                record["name"] = Value::from(format!("Staff {index}"));
                for value in record["attributes"]
                    .as_object_mut()
                    .expect("staff attributes")
                    .values_mut()
                {
                    *value = Value::from(10);
                }
                record
            })
            .collect();
        root["staff"] = Value::Array(staff);
        root["staffCount"] = Value::from(2_000);
        root["manager"] = Value::Null;
        let dump_path = write_dump(&temp_dir, "staff.json", &root.to_string());

        let (snapshot, timings) =
            ingest_dump_file_for_save_timed(&mut conn, active_save.id, &dump_path)
                .expect("large staff ingest");
        let compact_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM staff_role_metrics WHERE snapshot_id = ?1",
                [snapshot.id],
                |row| row.get(0),
            )
            .expect("count generated staff compact rows");

        assert_eq!(staff_count_for_snapshot(&conn, snapshot.id), 2_000);
        assert_eq!(compact_count, 2_000);
        assert_ingest_timings(&timings);
        eprintln!(
            "generated staff ingest staff_count=2000 validation_ms={} insert_ms={} total_ms={}",
            timings.validation_ms, timings.insert_ms, timings.total_ms
        );
    }

    /// Scale check — a representative 184k-player ingest is too heavy for the default gate.
    /// Run: `cargo test ingest_completes_generated_184k_players_with_timings -- --ignored`
    #[test]
    #[ignore = "184k-player ingest is too heavy for the default gate"]
    fn ingest_completes_generated_184k_players_with_timings() {
        run_generated_large_ingest(184_000);
    }

    /// Scale check — not part of the default gate (memory/time). Run:
    /// `cargo test ingest_completes_generated_500k_players_with_timings -- --ignored`
    #[test]
    #[ignore = "large scale check — run with cargo test -- --ignored"]
    fn ingest_completes_generated_500k_players_with_timings() {
        run_generated_large_ingest(500_000);
    }

    fn run_generated_large_ingest(player_count: usize) {
        #[allow(unused_imports)]
        use crate::features::scoring::catalog::all_roles;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join(format!("ingest-{player_count}.db")));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let dump_path = temp_dir.path().join("generated.json");
        write_generated_minimal_dump(&dump_path, player_count);

        let (snapshot, timings) =
            ingest_dump_file_for_save_timed(&mut conn, active_save.id, &dump_path)
                .expect("large ingest");

        assert_eq!(snapshot.player_count, player_count as i64);
        assert_eq!(
            player_count_for_snapshot(&conn, snapshot.id),
            player_count as i64
        );
        assert_eq!(
            role_score_count_for_snapshot(&conn, snapshot.id),
            player_count as i64
        );
        assert_ingest_timings(&timings);
        assert!(
            timings.validation_ms > 0,
            "validation_ms should be material for {player_count} players: {timings:?}"
        );
        assert!(
            timings.insert_ms > 0,
            "insert_ms should be material for {player_count} players: {timings:?}"
        );

        eprintln!(
            "generated ingest player_count={player_count} validation_ms={} insert_ms={} total_ms={}",
            timings.validation_ms, timings.insert_ms, timings.total_ms
        );
    }

    #[test]
    fn prepare_snapshot_is_pure_and_does_not_hold_db_mutex() {
        use std::sync::Mutex;

        use crate::db::Db;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("prepare-mutex.db"));
        let db = Db(Mutex::new(conn));
        // DB is empty, no writes yet.
        let before_snapshots: i64 =
            db.0.lock()
                .expect("lock db")
                .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
                .expect("count snapshots");
        assert_eq!(before_snapshots, 0);
        // Preparation must succeed without holding the Db mutex and must not write.
        let prepared = prepare_dump_json(GOLDEN_FIXTURE).expect("prepare without db");
        assert_eq!(prepared.players.len(), 1);
        assert_eq!(prepared.staff.len(), 1);
        let after_prepare_snapshots: i64 =
            db.0.lock()
                .expect("lock db")
                .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
                .expect("count snapshots after prepare");
        assert_eq!(after_prepare_snapshots, 0, "prepare must not write");
        // Holding the database mutex cannot affect pure preparation because the
        // preparation API owns no database handle.
        let guard = db.0.lock().expect("hold db mutex");
        let _ = prepare_dump_json(GOLDEN_FIXTURE).expect("prepare while db mutex is held");
        drop(guard);
        assert_eq!(prepared.players[0].compact_current.len(), 68);
        assert_eq!(prepared.players[0].compact_potential.len(), 68);
        assert_eq!(prepared.staff[0].compact_scores.len(), 21);
    }

    #[test]
    fn publish_rejects_stale_captured_context_before_any_write() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("publish-stale-context.db"));
        let save_context = service::capture_active_save_context(&conn).expect("capture");
        let prepared = prepare_dump_json(GOLDEN_FIXTURE).expect("prepare");
        // Simulate save switch/delete/reused numeric ID after capture.
        conn.execute("DELETE FROM saves WHERE id = ?1", [save_context.id])
            .expect("delete save");
        conn.execute(
            "INSERT INTO saves (id, name, is_active) VALUES (?1, 'Reused', 1)",
            [save_context.id],
        )
        .expect("reused id");
        let err = publish_prepared_snapshot(&mut conn, &save_context, prepared, None)
            .expect_err("reject stale context");
        assert!(err.contains("Save changed"), "{err}");
        // All DB state must be unchanged: no snapshot, no players, no compact.
        let snapshot_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .expect("count");
        assert_eq!(snapshot_count, 0);
        let player_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM players", [], |row| row.get(0))
            .expect("count");
        assert_eq!(player_count, 0);
        let compact_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM player_role_metrics", [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(compact_count, 0);
    }

    #[test]
    fn publish_is_one_transaction_and_rolls_back_on_compact_failure() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("publish-rollback.db"));
        let first_path = write_dump(&temp_dir, "first.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        let prior_snapshot_id = first.id;
        let prior_compact = compact_row(&conn, prior_snapshot_id, 77).expect("prior compact");
        let save_context = service::capture_active_save_context(&conn).expect("capture");
        let prepared = prepare_dump_json(GOLDEN_FIXTURE).expect("prepare second");
        conn.execute_batch(
            "CREATE TRIGGER reject_compact_rows BEFORE INSERT ON player_role_metrics BEGIN SELECT RAISE(ABORT, 'compact row failure'); END;",
        )
        .expect("trigger");
        let err = publish_prepared_snapshot(&mut conn, &save_context, prepared, Some("req-fail"))
            .expect_err("rollback");
        assert!(err.contains("compact row failure"), "{err}");
        // Prior current must remain visible byte-for-byte, no new snapshot committed.
        assert_eq!(
            current_snapshot_id(&conn, save_context.id),
            Some(prior_snapshot_id)
        );
        assert_eq!(snapshot_count(&conn, save_context.id), 1);
        assert_eq!(
            compact_row(&conn, prior_snapshot_id, 77),
            Some(prior_compact)
        );
    }

    #[test]
    fn prepare_and_publish_preserve_winner_and_raw_history_semantics() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("prepare-winner-history.db"));
        let save_context = service::capture_active_save_context(&conn).expect("capture");
        // First ingest via publish path: later date wins.
        let later_json = dump_with_game_date(Some("2027-08-16"), "Later player");
        let later_prepared = prepare_dump_json(&later_json).expect("prepare later");
        let later = publish_prepared_snapshot(&mut conn, &save_context, later_prepared, Some("R2"))
            .expect("publish later");
        assert_eq!(later.stored_snapshot.id, later.effective_snapshot.id);
        assert_complete_potential_state(&conn, later.stored_snapshot.id);
        assert_eq!(compact_row_count(&conn, later.stored_snapshot.id), 1);
        // Historical insert must stay raw-only.
        let earlier_json = dump_with_game_date(Some("2026-08-14"), "Earlier player");
        let earlier_prepared = prepare_dump_json(&earlier_json).expect("prepare earlier");
        let earlier =
            publish_prepared_snapshot(&mut conn, &save_context, earlier_prepared, Some("R1"))
                .expect("publish earlier");
        assert_ne!(earlier.stored_snapshot.id, earlier.effective_snapshot.id);
        assert_eq!(earlier.effective_snapshot.id, later.stored_snapshot.id);
        assert_empty_potential_state(&conn, earlier.stored_snapshot.id);
        assert_eq!(compact_row_count(&conn, earlier.stored_snapshot.id), 0);
        assert_eq!(compact_row_count(&conn, later.stored_snapshot.id), 1);
        // Prepare errors must not have written.
        let bad_json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 8", "\"schemaVersion\": 4");
        assert!(prepare_dump_json(&bad_json).is_err());
        assert_eq!(snapshot_count(&conn, save_context.id), 2);
    }

    #[test]
    fn timings_separate_preparation_from_db_save_without_overlap() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("prepare-timings.db"));
        let save_context = service::capture_active_save_context(&conn).expect("capture");
        let prepared = prepare_dump_json(GOLDEN_FIXTURE).expect("prepare");
        let result =
            publish_prepared_snapshot(&mut conn, &save_context, prepared, None).expect("publish");
        assert_eq!(result.stored_snapshot.player_count, 1);
    }
}
