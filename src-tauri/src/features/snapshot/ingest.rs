//! Validates and ingests `dump.json` into the active save's current snapshot.

use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::Value;

use crate::features::memory_read::dump_validation::validate_dump_json;

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
    ingest_dump_json_for_save(conn, save_id, &json)
}

fn ingest_dump_json_for_save(
    conn: &mut Connection,
    save_id: i64,
    json: &str,
) -> Result<SnapshotSummary, String> {
    validate_dump_json(json).map_err(|error| error.to_string())?;

    let root: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = root
        .as_object()
        .ok_or_else(|| "dump root must be a JSON object".to_string())?;

    let tx = conn.transaction().map_err(|error| error.to_string())?;
    let snapshot_id = insert_snapshot(&tx, save_id, object)?;
    insert_players(&tx, snapshot_id, object)?;
    replace_current_snapshot(&tx, save_id, snapshot_id)?;
    tx.commit().map_err(|error| error.to_string())?;

    get_snapshot_by_id(conn, snapshot_id)
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

    for player in players {
        let player = player
            .as_object()
            .ok_or_else(|| "each player must be a JSON object".to_string())?;

        tx.execute(
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
            params![
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
            ],
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
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
        assert_eq!(snapshot.max_accepted, Some(10_000));
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
            .replace("\"maxAccepted\": 10000", "\"maxAccepted\": 5000");
        let dump_path = write_dump(&temp_dir, "truncated.json", &truncated_json);

        let snapshot = ingest_dump_file(&mut conn, &dump_path).expect("ingest truncated dump");

        assert!(snapshot.scan_truncated);
        assert_eq!(snapshot.max_accepted, Some(5000));
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
}
