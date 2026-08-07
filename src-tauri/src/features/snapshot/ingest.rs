//! Validates and ingests `dump.json` into the active save's current snapshot.

use std::fs;
use std::path::Path;
use std::time::Instant;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::Value;

use crate::features::academy::service as academy_service;
use crate::features::memory_read::dump_validation::parse_and_validate_dump;
use crate::features::scoring::catalog::all_roles;
use crate::features::scoring::score::score_role;

use super::service;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSummary {
    pub id: i64,
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
    let save_id = service::active_save_id(conn)?;
    ingest_dump_file_for_save(conn, save_id, dump_path)
}

pub fn ingest_dump_file_for_save(
    conn: &mut Connection,
    save_id: i64,
    dump_path: &Path,
) -> Result<SnapshotSummary, String> {
    let json = fs::read_to_string(dump_path).map_err(|error| error.to_string())?;
    ingest_dump_json_for_save(conn, save_id, &json).map(|(summary, _)| summary)
}

/// Ingests a dump file and returns phase timings for measurement harnesses.
#[cfg_attr(not(test), allow(dead_code))]
pub fn ingest_dump_file_for_save_timed(
    conn: &mut Connection,
    save_id: i64,
    dump_path: &Path,
) -> Result<(SnapshotSummary, IngestTimings), String> {
    let json = fs::read_to_string(dump_path).map_err(|error| error.to_string())?;
    ingest_dump_json_for_save(conn, save_id, &json)
}

fn ingest_dump_json_for_save(
    conn: &mut Connection,
    save_id: i64,
    json: &str,
) -> Result<(SnapshotSummary, IngestTimings), String> {
    let total_started = Instant::now();

    let validation_started = Instant::now();
    let root = parse_and_validate_dump(json).map_err(|error| error.to_string())?;
    let validation_ms = validation_started.elapsed().as_millis();

    let object = root
        .as_object()
        .ok_or_else(|| "dump root must be a JSON object".to_string())?;

    let insert_started = Instant::now();
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    let snapshot_id = insert_snapshot(&tx, save_id, object)?;
    insert_players(&tx, snapshot_id, object)?;
    // ponytail: score every catalog role synchronously during ingest (one INSERT per role × player)
    // Upgrade to lazy/on-demand or batched scoring if ingest scoring time dominates Load Data
    insert_role_scores(&tx, snapshot_id, object)?;
    replace_current_snapshot(&tx, save_id, snapshot_id)?;
    academy_service::ensure_class_for_game_date(
        &tx,
        save_id,
        optional_string(object.get("gameDate"))?.as_deref(),
        &require_string(object, "gameDateSource")?,
    )?;
    tx.commit().map_err(|error| error.to_string())?;
    let insert_ms = insert_started.elapsed().as_millis();

    let summary = get_snapshot_by_id(conn, snapshot_id)?;
    let total_ms = total_started.elapsed().as_millis();

    Ok((
        summary,
        IngestTimings {
            validation_ms,
            insert_ms,
            total_ms,
        },
    ))
}

fn insert_snapshot(
    tx: &Transaction<'_>,
    save_id: i64,
    object: &serde_json::Map<String, Value>,
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
            scan_truncated,
            max_accepted,
            player_count
        ) VALUES (?1, 0, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            save_id,
            require_i64(object, "schemaVersion")?,
            require_string(object, "generatedAtUtc")?,
            require_string(object, "gameVersion")?,
            require_string(object, "supportedGameVersion")?,
            require_string(object, "bridgeVersion")?,
            require_i64(object, "protocolVersion")?,
            optional_string(object.get("gameDate"))?,
            require_string(object, "gameDateSource")?,
            i32::from(require_bool(object, "scanTruncated")?),
            optional_i64(object.get("maxAccepted"))?,
            require_i64(object, "playerCount")?,
        ],
    )
    .map_err(|error| error.to_string())?;

    Ok(tx.last_insert_rowid())
}

fn insert_players(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    object: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    let players = object
        .get("players")
        .and_then(Value::as_array)
        .ok_or_else(|| "dump players must be an array".to_string())?;

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
                team_level
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30
            )",
        )
        .map_err(|error| error.to_string())?;

    for player in players {
        let player = player
            .as_object()
            .ok_or_else(|| "each player must be a JSON object".to_string())?;

        stmt.execute(params![
            snapshot_id,
            require_u64(player, "uid")? as i64,
            require_i64(player, "ca")?,
            require_i64(player, "pa")?,
            require_string(player, "name")?,
            require_i64(player, "birthYear")?,
            require_i64(player, "birthDayOfYear")?,
            optional_i64(player.get("age"))?,
            json_string(required_value(player, "nationalities")?)?,
            optional_i64(player.get("heightCm"))?,
            require_string(player, "preferredFoot")?,
            json_string(required_value(player, "positions")?)?,
            json_string(required_value(player, "attributes")?)?,
            json_string(required_value(player, "hiddenAttributes")?)?,
            json_string(required_value(player, "personality")?)?,
            optional_i64(player.get("weeklyWageGbp"))?,
            optional_i64(player.get("contractExpiryYear"))?,
            optional_i64(player.get("contractExpiryDayOfYear"))?,
            optional_bool(player.get("transferListed"))?,
            optional_bool(player.get("loanListed"))?,
            optional_bool(player.get("notForSale"))?,
            optional_bool(player.get("setForRelease"))?,
            optional_i64(player.get("marketValueGbp"))?,
            reputation_field(player, "current")?,
            reputation_field(player, "world")?,
            optional_string(player.get("currentClub"))?,
            optional_string(player.get("parentClub"))?,
            optional_bool(player.get("onLoan"))?,
            optional_string(player.get("division"))?,
            optional_string(player.get("teamLevel"))?,
        ])
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn insert_role_scores(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    object: &serde_json::Map<String, Value>,
) -> Result<(), String> {
    let players = object
        .get("players")
        .and_then(Value::as_array)
        .ok_or_else(|| "dump players must be an array".to_string())?;

    let mut stmt = tx
        .prepare(
            "INSERT INTO player_role_scores (snapshot_id, uid, role_id, phase, score)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|error| error.to_string())?;

    let roles = all_roles();
    for player in players {
        let player = player
            .as_object()
            .ok_or_else(|| "each player must be a JSON object".to_string())?;
        let uid = require_u64(player, "uid")? as i64;
        let attributes = attributes_map(required_value(player, "attributes")?)?;

        for role in roles {
            let score = score_role(&attributes, role).map(i64::from);
            stmt.execute(params![
                snapshot_id,
                uid,
                role.role_id,
                role.phase.as_db_str(),
                score,
            ])
            .map_err(|error| error.to_string())?;
        }
    }

    Ok(())
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

fn replace_current_snapshot(
    tx: &Transaction<'_>,
    save_id: i64,
    new_snapshot_id: i64,
) -> Result<(), String> {
    let old_snapshot_id: Option<i64> = tx
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    tx.execute(
        "UPDATE snapshots SET is_current = 0 WHERE save_id = ?1 AND is_current = 1",
        params![save_id],
    )
    .map_err(|error| error.to_string())?;

    tx.execute(
        "UPDATE snapshots SET is_current = 1 WHERE id = ?1",
        params![new_snapshot_id],
    )
    .map_err(|error| error.to_string())?;

    if let Some(old_snapshot_id) = old_snapshot_id {
        if old_snapshot_id != new_snapshot_id {
            tx.execute(
                "DELETE FROM snapshots WHERE id = ?1",
                params![old_snapshot_id],
            )
            .map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn get_snapshot_by_id(conn: &Connection, snapshot_id: i64) -> Result<SnapshotSummary, String> {
    conn.query_row(
        "SELECT
            id,
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
                save_id: row.get(1)?,
                schema_version: row.get(2)?,
                generated_at_utc: row.get(3)?,
                game_version: row.get(4)?,
                supported_game_version: row.get(5)?,
                bridge_version: row.get(6)?,
                protocol_version: row.get(7)?,
                game_date: row.get(8)?,
                game_date_source: row.get(9)?,
                scan_truncated: row.get::<_, i32>(10)? == 1,
                max_accepted: row.get(11)?,
                player_count: row.get(12)?,
                loaded_at_utc: row.get(13)?,
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
    use crate::features::snapshot::service::list_saves;
    use std::path::Path;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v5.json");

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

    fn player_count_for_snapshot(conn: &Connection, snapshot_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM players WHERE snapshot_id = ?1",
            params![snapshot_id],
            |row| row.get(0),
        )
        .expect("count players")
    }

    fn role_score_count_for_snapshot(conn: &Connection, snapshot_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM player_role_scores WHERE snapshot_id = ?1",
            params![snapshot_id],
            |row| row.get(0),
        )
        .expect("count role scores")
    }

    fn role_scores_for_player(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Vec<(String, String, Option<i64>)> {
        let mut statement = conn
            .prepare(
                "SELECT role_id, phase, score
                 FROM player_role_scores
                 WHERE snapshot_id = ?1 AND uid = ?2
                 ORDER BY role_id",
            )
            .expect("prepare role score query");
        statement
            .query_map(params![snapshot_id, uid], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("query role scores")
            .collect::<Result<Vec<_>, _>>()
            .expect("read role scores")
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

    #[test]
    fn ingest_writes_null_role_scores_when_required_attributes_are_missing() {
        use crate::features::scoring::catalog::all_roles;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-role-scores-null.db"));
        list_saves(&conn).expect("seed default save");

        let dump_path = write_dump(&temp_dir, "sparse.json", GOLDEN_FIXTURE);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest sparse dump");

        let expected_roles = all_roles().len() as i64;
        assert_eq!(
            role_score_count_for_snapshot(&conn, snapshot.id),
            expected_roles
        );

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
    fn second_ingest_replaces_prior_role_scores_with_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-role-scores-replace.db"));
        list_saves(&conn).expect("seed default save");

        let first_path = write_dump(&temp_dir, "first.json", &dump_with_uniform_attributes(10));
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");
        assert!(role_score_count_for_snapshot(&conn, first.id) > 0);

        let second_path = write_dump(&temp_dir, "second.json", &dump_with_uniform_attributes(20));
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        assert_eq!(role_score_count_for_snapshot(&conn, first.id), 0);
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

        let bad_json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 5", "\"schemaVersion\": 4");
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
        let dump_path = write_dump(&temp_dir, "dump.json", &json_with_null_attribute);
        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest golden dump");

        assert_eq!(snapshot.save_id, active_save.id);
        assert_eq!(snapshot.schema_version, 5);
        assert_eq!(snapshot.generated_at_utc, "2026-07-29T10:00:00.000Z");
        assert_eq!(snapshot.game_version, "26.3.2.2329565");
        assert_eq!(snapshot.supported_game_version, "26.3");
        assert_eq!(snapshot.bridge_version, "0.1.0");
        assert_eq!(snapshot.protocol_version, 1);
        assert_eq!(snapshot.game_date.as_deref(), Some("2026-08-14"));
        assert_eq!(snapshot.game_date_source, "memory");
        assert!(!snapshot.scan_truncated);
        assert_eq!(snapshot.max_accepted, Some(500));
        assert_eq!(snapshot.player_count, 1);

        assert_eq!(
            current_snapshot_id(&conn, active_save.id),
            Some(snapshot.id)
        );
        assert_eq!(player_count_for_snapshot(&conn, snapshot.id), 1);

        let (player_ca, current_club, attributes_json): (i64, Option<String>, String) = conn
            .query_row(
                "SELECT ca, current_club, attributes_json FROM players WHERE snapshot_id = ?1",
                params![snapshot.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("player row");
        assert_eq!(player_ca, 150);
        assert_eq!(current_club.as_deref(), Some("Loan FC"));

        let attributes: Value =
            serde_json::from_str(&attributes_json).expect("parse attributes_json");
        assert_eq!(attributes["Acceleration"], 14);
        assert_eq!(attributes["Pace"], 15);
        assert_eq!(
            attributes.get("Dribbling"),
            Some(&Value::Null),
            "null attribute must be stored as JSON null, not omitted"
        );
    }

    #[test]
    fn second_successful_ingest_replaces_current_snapshot_and_deletes_prior() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-replace.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let first_path = write_dump(&temp_dir, "first.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &first_path).expect("first ingest");

        let updated_json = GOLDEN_FIXTURE.replace("\"ca\": 150", "\"ca\": 155");
        let second_path = write_dump(&temp_dir, "second.json", &updated_json);
        let second = ingest_dump_file(&mut conn, &second_path).expect("second ingest");

        assert_ne!(second.id, first.id);
        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(second.id));
        assert!(!snapshot_row_exists(&conn, first.id));
        assert_eq!(player_count_for_snapshot(&conn, first.id), 0);
        assert_eq!(player_count_for_snapshot(&conn, second.id), 1);

        let ca: i64 = conn
            .query_row(
                "SELECT ca FROM players WHERE snapshot_id = ?1",
                params![second.id],
                |row| row.get(0),
            )
            .expect("updated player ca");
        assert_eq!(ca, 155);
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

        let bad_json = GOLDEN_FIXTURE.replace("\"schemaVersion\": 5", "\"schemaVersion\": 4");
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
            .replace("\"maxAccepted\": 500", "\"maxAccepted\": 250");
        let dump_path = write_dump(&temp_dir, "truncated.json", &truncated_json);

        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest truncated dump");

        assert!(snapshot.scan_truncated);
        assert_eq!(snapshot.max_accepted, Some(250));
    }

    #[test]
    fn failed_player_insert_leaves_prior_snapshot_current() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ingest-rollback.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");

        let good_path = write_dump(&temp_dir, "good.json", GOLDEN_FIXTURE);
        let first = ingest_dump_file(&mut conn, &good_path).expect("first ingest");

        let duplicate_uid_json = GOLDEN_FIXTURE
            .replace("\"playerCount\": 1", "\"playerCount\": 2")
            .replace(
                "    }\n  ]",
                "    },\n    {\n      \"uid\": 77,\n      \"ca\": 140,\n      \"pa\": 160,\n      \"name\": \"Duplicate UID\",\n      \"birthYear\": 2001,\n      \"birthDayOfYear\": 101,\n      \"age\": 25,\n      \"nationalities\": [\"SCO\"],\n      \"heightCm\": 180,\n      \"preferredFoot\": \"left\",\n      \"positions\": { \"ST\": 15 },\n      \"attributes\": { \"Finishing\": 13 },\n      \"hiddenAttributes\": { \"Consistency\": 10 },\n      \"personality\": { \"Ambition\": 12 },\n      \"weeklyWageGbp\": null,\n      \"contractExpiryYear\": null,\n      \"contractExpiryDayOfYear\": null,\n      \"transferListed\": null,\n      \"loanListed\": null,\n      \"notForSale\": null,\n      \"setForRelease\": null,\n      \"marketValueGbp\": null,\n      \"reputation\": { \"current\": 90, \"world\": 80 },\n      \"currentClub\": null,\n      \"parentClub\": null,\n      \"onLoan\": null,\n      \"division\": null,\n      \"teamLevel\": null\n    }\n  ]",
            );
        let broken_path = write_dump(&temp_dir, "broken.json", &duplicate_uid_json);
        let _ = ingest_dump_file(&mut conn, &broken_path).expect_err("reject duplicate uid");

        assert_eq!(current_snapshot_id(&conn, active_save.id), Some(first.id));
        assert_eq!(player_count_for_snapshot(&conn, first.id), 1);
    }

    fn write_generated_minimal_dump(path: &Path, player_count: usize) {
        use std::io::Write;

        let mut file = fs::File::create(path).expect("create generated dump");
        write!(
            file,
            concat!(
                r#"{{"schemaVersion":5,"generatedAtUtc":"2026-07-30T12:00:00.000Z","#,
                r#""gameVersion":"26.3.2","supportedGameVersion":"26.3","bridgeVersion":"0.1.0","#,
                r#""protocolVersion":1,"gameDate":null,"gameDateSource":"unknown","#,
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
                    r#""age":null,"nationalities":[],"heightCm":null,"preferredFoot":"right","#,
                    r#""positions":{{}},"attributes":{{}},"hiddenAttributes":{{}},"personality":{{}},"#,
                    r#""weeklyWageGbp":null,"contractExpiryYear":null,"contractExpiryDayOfYear":null,"#,
                    r#""transferListed":null,"loanListed":null,"notForSale":null,"setForRelease":null,"#,
                    r#""marketValueGbp":null,"reputation":{{"current":null,"world":null}},"#,
                    r#""currentClub":null,"parentClub":null,"onLoan":null,"division":null,"teamLevel":null}}"#
                ),
                uid = uid
            )
            .expect("write player");
        }

        write!(file, "]}}").expect("write dump footer");
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

    /// Scale check — role-score matrix (~68 rows/player) is too heavy for the default gate.
    /// Run: `cargo test ingest_completes_generated_184k_players_with_timings -- --ignored`
    #[test]
    #[ignore = "role-score matrix at 184k players is too heavy for the default gate"]
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
            (player_count * all_roles().len()) as i64
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
}
