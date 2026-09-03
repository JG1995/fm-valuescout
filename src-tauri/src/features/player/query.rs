use std::collections::{BTreeMap, HashMap};

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;

use crate::features::{
    moneyball::role_catalog::builtin_catalog,
    player_metrics::compact::{
        player_current_column, player_metrics_join, player_potential_column, PLAYER_METRICS_ALIAS,
        PROJECTION_MODEL_VERSION,
    },
    scoring::catalog::{all_roles, DUMP_ATTRIBUTE_KEYS},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRoleScore {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_tags: Vec<String>,
    pub score: Option<i64>,
    pub potential_score: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDetail {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub birth_year: i64,
    pub birth_day_of_year: i64,
    pub nationalities: Vec<String>,
    pub height_cm: Option<i64>,
    pub preferred_foot: String,
    pub positions: BTreeMap<String, Option<i64>>,
    pub attributes: BTreeMap<String, Option<i64>>,
    pub potential_attributes: BTreeMap<String, Option<i64>>,
    pub hidden_attributes: BTreeMap<String, Option<i64>>,
    pub personality: BTreeMap<String, Option<i64>>,
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
    pub role_scores: Vec<PlayerRoleScore>,
}

/// Load one player by `uid` from the active save's current snapshot.
/// Returns `Ok(None)` when there is no current snapshot or the uid is absent.
pub fn get_player(conn: &Connection, uid: i64) -> Result<Option<PlayerDetail>, String> {
    let snapshot: Option<(i64, i64)> = conn
        .query_row(
            "SELECT s.id, sv.reveal_hidden_information
             FROM snapshots s
             INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
             WHERE s.is_current = 1
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let Some((snapshot_id, hidden_information_revealed)) = snapshot else {
        return Ok(None);
    };

    let player = conn
        .query_row(
            "SELECT
                uid,
                name,
                age,
                birth_year,
                birth_day_of_year,
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
                ca,
                pa
             FROM players
             WHERE snapshot_id = ?1 AND uid = ?2",
            params![snapshot_id, uid],
            map_player_row,
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let Some(mut player) = player else {
        return Ok(None);
    };
    player.hidden_information_revealed = hidden_information_revealed == 1;

    player.potential_attributes = load_potential_attributes(conn, snapshot_id, uid)?;
    let role_scores = load_role_scores(conn, snapshot_id, uid)?;
    player.role_scores = role_scores;
    Ok(Some(player))
}

fn map_player_row(row: &Row<'_>) -> rusqlite::Result<PlayerDetail> {
    let nationalities_json: String = row.get(5)?;
    let positions_json: String = row.get(8)?;
    let attributes_json: String = row.get(9)?;
    let hidden_attributes_json: String = row.get(10)?;
    let personality_json: String = row.get(11)?;

    Ok(PlayerDetail {
        uid: row.get(0)?,
        name: row.get(1)?,
        age: row.get(2)?,
        birth_year: row.get(3)?,
        birth_day_of_year: row.get(4)?,
        nationalities: parse_string_array(&nationalities_json).map_err(|message| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                message.into(),
            )
        })?,
        height_cm: row.get(6)?,
        preferred_foot: row.get(7)?,
        positions: parse_positions(&positions_json).map_err(|message| {
            rusqlite::Error::FromSqlConversionFailure(
                8,
                rusqlite::types::Type::Text,
                message.into(),
            )
        })?,
        attributes: parse_nullable_int_map(&attributes_json).map_err(|message| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                message.into(),
            )
        })?,
        potential_attributes: BTreeMap::new(),
        hidden_attributes: parse_nullable_int_map(&hidden_attributes_json).map_err(|message| {
            rusqlite::Error::FromSqlConversionFailure(
                10,
                rusqlite::types::Type::Text,
                message.into(),
            )
        })?,
        personality: parse_nullable_int_map(&personality_json).map_err(|message| {
            rusqlite::Error::FromSqlConversionFailure(
                11,
                rusqlite::types::Type::Text,
                message.into(),
            )
        })?,
        weekly_wage_gbp: row.get(12)?,
        contract_expiry_year: row.get(13)?,
        contract_expiry_day_of_year: row.get(14)?,
        transfer_listed: optional_bool(row.get(15)?)?,
        loan_listed: optional_bool(row.get(16)?)?,
        not_for_sale: optional_bool(row.get(17)?)?,
        set_for_release: optional_bool(row.get(18)?)?,
        market_value_gbp: row.get(19)?,
        reputation_current: row.get(20)?,
        reputation_world: row.get(21)?,
        club: row.get(22)?,
        parent_club: row.get(23)?,
        on_loan: optional_bool(row.get(24)?)?,
        division: row.get(25)?,
        team_level: row.get(26)?,
        ca: row.get(27)?,
        pa: row.get(28)?,
        hidden_information_revealed: false,
        role_scores: Vec::new(),
    })
}

fn optional_bool(value: Option<i64>) -> rusqlite::Result<Option<bool>> {
    match value {
        None => Ok(None),
        Some(0) => Ok(Some(false)),
        Some(1) => Ok(Some(true)),
        Some(other) => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Integer,
            format!("expected 0, 1, or null boolean flag, got {other}").into(),
        )),
    }
}

fn load_potential_attributes(
    conn: &Connection,
    snapshot_id: i64,
    uid: i64,
) -> Result<BTreeMap<String, Option<i64>>, String> {
    let projected_attributes_json: Option<String> = conn
        .query_row(
            "SELECT potential_attributes_json
             FROM players
             WHERE snapshot_id = ?1 AND uid = ?2
               AND potential_projection_model_version = ?3",
            params![snapshot_id, uid, PROJECTION_MODEL_VERSION],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .flatten();
    let Some(projected_attributes_json) = projected_attributes_json else {
        return Err("Current potential snapshot is incomplete".to_string());
    };
    let projected_attributes = parse_nullable_int_map(&projected_attributes_json)
        .map_err(|_| "Current potential snapshot is incomplete".to_string())?;
    let complete = projected_attributes.len() == DUMP_ATTRIBUTE_KEYS.len()
        && DUMP_ATTRIBUTE_KEYS.iter().all(|key| {
            projected_attributes.get(*key).is_some_and(|value| {
                value.is_none() || value.is_some_and(|value| (1..=20).contains(&value))
            })
        });
    if complete {
        Ok(projected_attributes)
    } else {
        Err("Current potential snapshot is incomplete".to_string())
    }
}

fn load_role_scores(
    conn: &Connection,
    snapshot_id: i64,
    uid: i64,
) -> Result<Vec<PlayerRoleScore>, String> {
    let roles = all_roles();
    let current_columns = roles
        .iter()
        .map(|role| player_current_column(role.role_id))
        .collect::<Result<Vec<_>, _>>()?;
    let potential_columns = roles
        .iter()
        .map(|role| player_potential_column(role.role_id))
        .collect::<Result<Vec<_>, _>>()?;
    let metric_columns = current_columns
        .iter()
        .copied()
        .chain(potential_columns.iter().map(String::as_str))
        .map(|column| format!("{PLAYER_METRICS_ALIAS}.{column}"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {PLAYER_METRICS_ALIAS}.score_model_version, {metric_columns}
         FROM players p{}
         WHERE p.snapshot_id = ?1 AND p.uid = ?2",
        player_metrics_join("p", true, true)
    );
    let row = conn
        .query_row(&sql, params![snapshot_id, uid], |row| {
            let has_compact_row = row.get::<_, Option<i64>>(0)?.is_some();
            let current = (0..roles.len())
                .map(|index| row.get::<_, Option<i64>>(index + 1))
                .collect::<Result<Vec<_>, _>>()?;
            let potential = (0..roles.len())
                .map(|index| row.get::<_, Option<i64>>(index + 1 + roles.len()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((has_compact_row, current, potential))
        })
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((has_compact_row, current_scores, potential_scores)) = row else {
        return Err("Current potential snapshot is incomplete".to_string());
    };
    if !has_compact_row {
        return Err("Current potential snapshot is incomplete".to_string());
    }
    let scores_by_role = roles
        .iter()
        .zip(current_scores)
        .map(|(role, score)| (role.role_id.to_string(), score))
        .collect::<HashMap<_, _>>();
    let potential_by_role = roles
        .iter()
        .zip(potential_scores)
        .map(|(role, score)| (role.role_id.to_string(), score))
        .collect::<HashMap<_, _>>();
    let catalog = builtin_catalog()?;
    let mut role_scores = Vec::with_capacity(catalog.definitions.len());
    for role in &catalog.definitions {
        let mapped_scores = role.attribute_role_id.as_deref().map(|attribute_role_id| {
            (
                scores_by_role
                    .get(attribute_role_id)
                    .copied()
                    .unwrap_or(None),
                potential_by_role
                    .get(attribute_role_id)
                    .copied()
                    .unwrap_or(None),
            )
        });
        role_scores.push(PlayerRoleScore {
            role_id: role.id.clone(),
            display_name: role.display_name.clone(),
            phase: match role.phase {
                crate::features::moneyball::role_catalog::RolePhase::InPossession => {
                    "in_possession"
                }
                crate::features::moneyball::role_catalog::RolePhase::OutOfPossession => {
                    "out_of_possession"
                }
            }
            .to_owned(),
            position_tags: role.position_tags.clone(),
            score: mapped_scores.and_then(|(score, _)| score),
            potential_score: mapped_scores.and_then(|(_, potential_score)| potential_score),
        });
    }

    Ok(role_scores)
}

fn parse_string_array(json: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let array = value
        .as_array()
        .ok_or_else(|| "nationalities_json must be an array".to_string())?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| "nationality must be a string".to_string())
        })
        .collect()
}

fn parse_positions(json: &str) -> Result<BTreeMap<String, Option<i64>>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "positions_json must be an object".to_string())?;
    let mut positions = BTreeMap::new();
    for (key, raw) in object {
        let familiarity = match raw {
            Value::Null => None,
            Value::Number(number) => Some(
                number
                    .as_i64()
                    .ok_or_else(|| format!("position `{key}` must be an integer or null"))?,
            ),
            _ => return Err(format!("position `{key}` must be an integer or null")),
        };
        positions.insert(key.clone(), familiarity);
    }
    Ok(positions)
}

fn parse_nullable_int_map(json: &str) -> Result<BTreeMap<String, Option<i64>>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let object = value
        .as_object()
        .ok_or_else(|| "attribute map must be an object".to_string())?;
    let mut map = BTreeMap::new();
    for (key, raw) in object {
        let parsed = match raw {
            Value::Null => None,
            Value::Number(number) => Some(
                number
                    .as_i64()
                    .ok_or_else(|| format!("attribute `{key}` must be an integer or null"))?,
            ),
            _ => {
                return Err(format!("attribute `{key}` must be an integer or null"));
            }
        };
        map.insert(key.clone(), parsed);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service::{create_save, set_active_save};
    use crate::features::{
        moneyball::role_catalog::builtin_catalog,
        player_metrics::potential_scores::PROJECTION_MODEL_VERSION,
        scoring::catalog::{all_roles, RolePhase, DUMP_ATTRIBUTE_KEYS},
    };
    use rusqlite::params;
    use serde_json::{json, Value};
    use std::path::Path;

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn ingest_players(conn: &mut Connection, players: Vec<Value>) {
        let mut root: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v8.json"))
                .expect("parse golden fixture");
        let mut players = players;
        for player in &mut players {
            complete_position_map(player);
        }
        root["players"] = Value::Array(players);
        root["playerCount"] = json!(root["players"].as_array().unwrap().len());

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let dump_path = temp_dir.path().join("player-dump.json");
        std::fs::write(&dump_path, root.to_string()).expect("write dump");
        ingest_dump_file(conn, &dump_path).expect("ingest dump");
    }

    fn complete_position_map(player: &mut Value) {
        const POSITION_KEYS: [&str; 15] = [
            "GK", "SW", "DL", "DC", "DR", "DM", "ML", "MC", "MR", "AML", "AMC", "AMR", "ST", "WBL",
            "WBR",
        ];
        let Some(positions) = player.get_mut("positions").and_then(Value::as_object_mut) else {
            return;
        };
        let existing = positions.clone();
        positions.clear();
        for key in POSITION_KEYS {
            positions.insert(
                key.to_string(),
                existing.get(key).cloned().unwrap_or(Value::Null),
            );
        }
    }

    fn player_template(uid: u64, name: &str, ca: i64) -> Value {
        json!({
            "uid": uid,
            "ca": ca,
            "pa": ca + 10,
            "name": name,
            "birthYear": 2000,
            "birthDayOfYear": 100,
            "age": 26,
            "nationalities": ["ENG"],
            "nationUid": null,
            "gender": "unknown",
            "heightCm": 180,
            "preferredFoot": "right",
            "positions": { "MC": 18 },
            "attributes": { "Acceleration": 10 },
            "hiddenAttributes": { "Consistency": 10 },
            "personality": { "Ambition": 10 },
            "weeklyWageGbp": 1000,
            "contractExpiryYear": 2028,
            "contractExpiryDayOfYear": 180,
            "transferListed": false,
            "loanListed": false,
            "notForSale": false,
            "setForRelease": false,
            "marketValueGbp": 1_000_000,
            "reputation": { "current": 50, "world": 40 },
            "currentClub": "Test FC",
            "parentClub": null,
            "onLoan": false,
            "division": "League One",
            "teamLevel": "senior",
            "clubReputation": null,
            "teamType": null
        })
    }

    #[test]
    fn position_parser_preserves_zero_and_unread_values() {
        assert_eq!(
            parse_positions(r#"{"AMR":20,"GK":0,"SW":null}"#).expect("positions"),
            BTreeMap::from([
                ("AMR".to_string(), Some(20)),
                ("GK".to_string(), Some(0)),
                ("SW".to_string(), None),
            ])
        );
    }

    fn set_role_score(conn: &Connection, uid: i64, role_id: &str, score: Option<i64>) {
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        let column = crate::features::player_metrics::compact::player_current_column(role_id)
            .expect("current role column");
        conn.execute(
            &format!(
                "UPDATE player_role_metrics
                 SET {column} = ?1
                 WHERE snapshot_id = ?2 AND uid = ?3"
            ),
            params![score, snapshot_id, uid],
        )
        .expect("update role score");
    }

    fn set_potential_role_score(conn: &Connection, uid: i64, role_id: &str, score: Option<i64>) {
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        let column = crate::features::player_metrics::compact::player_potential_column(role_id)
            .expect("potential role column");
        conn.execute(
            &format!(
                "UPDATE player_role_metrics
                 SET {column} = ?1
                 WHERE snapshot_id = ?2 AND uid = ?3"
            ),
            params![score, snapshot_id, uid],
        )
        .expect("update potential role score");
    }

    fn read_potential_score(conn: &Connection, uid: i64, role_id: &str) -> Option<i64> {
        let column = crate::features::player_metrics::compact::player_potential_column(role_id)
            .expect("potential role column");
        conn.query_row(
            &format!(
                "SELECT {column}
                 FROM player_role_metrics
                 WHERE snapshot_id = (SELECT id FROM snapshots WHERE is_current = 1)
                   AND uid = ?1"
            ),
            [uid],
            |row| row.get(0),
        )
        .expect("read persisted potential score")
    }

    type PotentialState = (
        Option<String>,
        Option<i64>,
        Option<crate::features::player_metrics::compact::test_support::CompactRowShape>,
    );

    fn potential_state(conn: &Connection, uid: i64) -> PotentialState {
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        let fields = conn
            .query_row(
                "SELECT potential_attributes_json, potential_projection_model_version
                 FROM players WHERE snapshot_id = ?1 AND uid = ?2",
                params![snapshot_id, uid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read projected fields");
        let compact_row = crate::features::player_metrics::compact::test_support::read_row(
            conn,
            snapshot_id,
            uid,
        );
        (fields.0, fields.1, compact_row)
    }

    fn deny_potential_writes(conn: &Connection) {
        conn.execute_batch(
            "CREATE TRIGGER deny_projected_player_updates
             BEFORE UPDATE OF potential_attributes_json, potential_projection_model_version ON players
             BEGIN SELECT RAISE(ABORT, 'potential player writes are forbidden'); END;
             CREATE TRIGGER deny_compact_role_inserts
             BEFORE INSERT ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'compact role writes are forbidden'); END;
             CREATE TRIGGER deny_compact_role_updates
             BEFORE UPDATE ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'compact role writes are forbidden'); END;
             CREATE TRIGGER deny_compact_role_deletes
             BEFORE DELETE ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'compact role writes are forbidden'); END;",
        )
        .expect("deny potential writes");
    }

    fn assert_profile_rejects_corrupt_potential_state(conn: &Connection, uid: i64) {
        let before = potential_state(conn, uid);
        deny_potential_writes(conn);

        assert_eq!(
            get_player(conn, uid),
            Err("Current potential snapshot is incomplete".to_string())
        );
        assert_eq!(potential_state(conn, uid), before);
    }

    #[test]
    fn returns_player_name_and_role_scores_for_known_uid() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("known-uid.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");
        ingest_dump_file(&mut conn, &dump_path).expect("ingest golden dump");
        set_role_score(&conn, 77, "goalkeeper_ip", Some(42));

        let player = get_player(&conn, 77)
            .expect("get_player")
            .expect("player present");

        assert_eq!(player.uid, 77);
        assert_eq!(player.name, "Golden Fixture Player");
        assert_eq!(player.height_cm, Some(182));
        assert_eq!(player.preferred_foot, "right");
        assert_eq!(player.club.as_deref(), Some("Loan FC"));
        assert!(!player.role_scores.is_empty());
        let goalkeeper = player
            .role_scores
            .iter()
            .find(|row| row.role_id == "gk_traditional_goalkeeper_ip")
            .expect("goalkeeper_ip row");
        assert_eq!(goalkeeper.display_name, "Traditional Goalkeeper");
        assert_eq!(goalkeeper.phase, RolePhase::InPossession.as_db_str());
        assert_eq!(goalkeeper.position_tags, vec!["GK".to_string()]);
        assert_eq!(
            goalkeeper.score,
            Some(42),
            "must round-trip score from the compact metrics row"
        );
        assert_eq!(goalkeeper.potential_score, None);
        assert!(player.hidden_information_revealed);
    }

    #[test]
    fn returns_the_active_save_information_visibility_for_each_player() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("visibility-by-save.db"));
        ingest_players(
            &mut conn,
            vec![player_template(1, "First save player", 150)],
        );
        let first_save = create_save(&conn, "Second save").expect("create second save");
        set_active_save(&mut conn, first_save.id).expect("switch to second save");
        ingest_players(
            &mut conn,
            vec![player_template(2, "Second save player", 150)],
        );
        conn.execute(
            "UPDATE saves SET reveal_hidden_information = 0 WHERE id = ?1",
            [first_save.id],
        )
        .expect("conceal second save");

        let second_player = get_player(&conn, 2)
            .expect("get second-save player")
            .expect("second-save player present");
        assert!(!second_player.hidden_information_revealed);

        let first_save_id: i64 = conn
            .query_row(
                "SELECT id FROM saves WHERE name = 'Default save'",
                [],
                |row| row.get(0),
            )
            .expect("find first save");
        set_active_save(&mut conn, first_save_id).expect("switch to first save");
        let first_player = get_player(&conn, 1)
            .expect("get first-save player")
            .expect("first-save player present");
        assert!(first_player.hidden_information_revealed);
    }

    #[test]
    fn returns_role_potential_from_persisted_values() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("potential-score.db"));
        let mut player = player_template(1, "Potential Role", 80);
        player["pa"] = json!(170);
        player["age"] = json!(20);
        player["positions"] = json!({
            "AMR": 20,
            "MR": 17,
            "AMC": 14,
            "GK": 0,
            "SW": null
        });
        player["attributes"] = Value::Object(
            DUMP_ATTRIBUTE_KEYS
                .iter()
                .map(|key| ((*key).to_string(), json!(10)))
                .collect(),
        );
        ingest_players(&mut conn, vec![player]);

        let stored_potential_attributes = parse_nullable_int_map(
            &potential_state(&conn, 1)
                .0
                .expect("persisted projected attributes"),
        )
        .expect("parse persisted projected attributes");
        let stored_potential_score = read_potential_score(&conn, 1, "centre_forward_ip");

        let detail = get_player(&conn, 1)
            .expect("get_player")
            .expect("player present");
        let catalog = builtin_catalog().expect("built-in catalog");
        let unmapped_role_ids = catalog
            .definitions
            .iter()
            .filter(|role| role.attribute_role_id.is_none())
            .map(|role| role.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(detail.potential_attributes, stored_potential_attributes);
        assert_eq!(detail.role_scores.len(), 88);
        assert!(detail.role_scores.iter().any(|role| {
            unmapped_role_ids.contains(role.role_id.as_str())
                && role.score.is_none()
                && role.potential_score.is_none()
        }));
        assert!(detail
            .role_scores
            .iter()
            .filter(|role| !unmapped_role_ids.contains(role.role_id.as_str()))
            .all(|role| { role.score.is_some() && role.potential_score.is_some() }));
        let centre_forward = detail
            .role_scores
            .iter()
            .find(|row| row.role_id == "st_centre_forward_ip")
            .expect("centre forward row");

        assert_eq!(centre_forward.score, Some(50));
        assert_eq!(centre_forward.potential_score, stored_potential_score);
        assert_ne!(centre_forward.potential_score, centre_forward.score);
    }

    #[test]
    fn reads_persisted_potential_values_when_source_values_diverge() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("persisted-potential-wins.db"));
        ingest_players(
            &mut conn,
            vec![player_template(1, "Persisted Potential", 80)],
        );

        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        let mut stored_attributes = parse_nullable_int_map(
            &potential_state(&conn, 1)
                .0
                .expect("persisted projected attributes"),
        )
        .expect("parse persisted projected attributes");
        stored_attributes.insert("Acceleration".to_string(), Some(1));
        conn.execute(
            "UPDATE players SET potential_attributes_json = ?3
             WHERE snapshot_id = ?1 AND uid = ?2",
            params![
                snapshot_id,
                1,
                serde_json::to_string(&stored_attributes).expect("serialize projected attributes")
            ],
        )
        .expect("change persisted projected attributes");
        set_potential_role_score(&conn, 1, "centre_forward_ip", Some(1));

        let detail = get_player(&conn, 1)
            .expect("get player")
            .expect("player present");
        let centre_forward = detail
            .role_scores
            .iter()
            .find(|row| row.role_id == "st_centre_forward_ip")
            .expect("centre-forward row");

        assert_eq!(
            detail.potential_attributes.get("Acceleration"),
            Some(&Some(1))
        );
        assert_eq!(centre_forward.potential_score, Some(1));
    }

    #[test]
    fn profile_reads_only_requested_player_potential_state() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("profile-potential-scope.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Requested Player", 150),
                player_template(2, "Unrelated Player", 150),
            ],
        );
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        conn.execute(
            "DELETE FROM player_role_metrics
             WHERE snapshot_id = ?1 AND uid = 2",
            params![snapshot_id],
        )
        .expect("delete unrelated player compact row");
        deny_potential_writes(&conn);

        let detail = get_player(&conn, 1)
            .expect("read requested player")
            .expect("requested player exists");
        assert_eq!(detail.uid, 1);
    }

    #[test]
    fn profile_rejects_missing_potential_role_without_rebuild() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("missing-potential-role.db"));
        ingest_players(&mut conn, vec![player_template(1, "Missing Role", 150)]);
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        conn.execute(
            "DELETE FROM player_role_metrics
             WHERE snapshot_id = ?1 AND uid = 1",
            params![snapshot_id],
        )
        .expect("delete compact row");

        assert_profile_rejects_corrupt_potential_state(&conn, 1);
    }

    #[test]
    fn profile_rejects_wrong_version_potential_role_without_rebuild() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("wrong-version-potential-role.db"));
        ingest_players(
            &mut conn,
            vec![player_template(1, "Wrong Role Version", 150)],
        );
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        conn.execute(
            "UPDATE player_role_metrics
             SET projection_model_version = ?2
             WHERE snapshot_id = ?1 AND uid = 1",
            params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
        )
        .expect("change compact projection version");

        assert_profile_rejects_corrupt_potential_state(&conn, 1);
    }

    #[test]
    fn profile_rejects_null_potential_attributes_without_rebuild() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("null-potential-attributes.db"));
        ingest_players(
            &mut conn,
            vec![player_template(1, "Null Potential Map", 150)],
        );
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        conn.execute(
            "UPDATE players SET potential_attributes_json = NULL
             WHERE snapshot_id = ?1 AND uid = 1",
            [snapshot_id],
        )
        .expect("clear projected attributes");

        assert_profile_rejects_corrupt_potential_state(&conn, 1);
    }

    #[test]
    fn profile_rejects_wrong_version_potential_attributes_without_rebuild() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(
            &temp_dir
                .path()
                .join("wrong-version-potential-attributes.db"),
        );
        ingest_players(
            &mut conn,
            vec![player_template(1, "Wrong Potential Map Version", 150)],
        );
        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        conn.execute(
            "UPDATE players SET potential_projection_model_version = ?2
             WHERE snapshot_id = ?1 AND uid = 1",
            params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
        )
        .expect("change projected attribute version");

        assert_profile_rejects_corrupt_potential_state(&conn, 1);
    }

    #[test]
    fn age_twenty_nine_returns_current_attributes_and_role_scores_as_potential() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("age-capped-potential.db"));
        let mut player = player_template(1, "Mature Player", 80);
        player["pa"] = json!(170);
        player["age"] = json!(29);
        player["positions"] = json!({ "ST": 20 });
        player["attributes"] = Value::Object(
            DUMP_ATTRIBUTE_KEYS
                .iter()
                .map(|key| ((*key).to_string(), json!(10)))
                .collect(),
        );
        ingest_players(&mut conn, vec![player]);

        let detail = get_player(&conn, 1)
            .expect("get_player")
            .expect("player present");

        assert_eq!(detail.potential_attributes, detail.attributes);
        assert!(detail
            .role_scores
            .iter()
            .all(|role| role.potential_score == role.score));
    }

    #[test]
    fn presents_duplicate_moneyball_roles_from_one_attribute_score_and_keeps_unmapped_null() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-inventory.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");
        ingest_dump_file(&mut conn, &dump_path).expect("ingest golden dump");
        set_role_score(&conn, 77, "wing_back_ip", Some(73));

        let detail = get_player(&conn, 77)
            .expect("get_player")
            .expect("player present");
        let catalog = builtin_catalog().expect("built-in catalog");

        assert_eq!(detail.role_scores.len(), catalog.definitions.len());
        let duplicate_rows = detail
            .role_scores
            .iter()
            .filter(|role| {
                role.role_id == "dl_dr_wing_back_ip" || role.role_id == "wbl_wbr_wing_back_ip"
            })
            .collect::<Vec<_>>();
        assert_eq!(duplicate_rows.len(), 2);
        assert!(duplicate_rows.iter().all(|role| role.score == Some(73)));
        assert_eq!(
            duplicate_rows[0].potential_score,
            duplicate_rows[1].potential_score
        );

        let unmapped = detail
            .role_scores
            .iter()
            .find(|role| role.role_id == "amc_attacking_midfielder_oop")
            .expect("unmapped presentation role");
        assert_eq!(unmapped.score, None);
        assert_eq!(unmapped.potential_score, None);
        assert_eq!(all_roles().len(), 79);
    }

    #[test]
    fn returns_none_for_unknown_uid() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("unknown-uid.db"));
        ingest_players(&mut conn, vec![player_template(1, "Known", 150)]);

        let missing = get_player(&conn, 999_999).expect("get_player");
        assert!(missing.is_none());
    }

    #[test]
    fn returns_none_when_uid_belongs_to_inactive_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("inactive-save.db"));
        ingest_players(
            &mut conn,
            vec![player_template(1, "Only On First Save", 150)],
        );

        let second_save = create_save(&conn, "Second save").expect("create save");
        set_active_save(&mut conn, second_save.id).expect("switch save");

        let missing = get_player(&conn, 1).expect("get_player after switch");
        assert!(missing.is_none());
    }

    #[test]
    fn preserves_null_attribute_and_null_role_score() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("nulls.db"));
        let mut player = player_template(1, "Null Attrs", 150);
        player["attributes"] = json!({ "Acceleration": null, "Pace": 12 });
        ingest_players(&mut conn, vec![player]);
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", None);

        let detail = get_player(&conn, 1)
            .expect("get_player")
            .expect("player present");

        assert_eq!(detail.attributes.get("Acceleration"), Some(&None));
        assert_eq!(detail.attributes.get("Pace"), Some(&Some(12)));
        assert_eq!(detail.potential_attributes.get("Acceleration"), Some(&None));
        assert_ne!(
            detail.attributes.get("Acceleration"),
            Some(&Some(0)),
            "null attribute must not coerce to 0"
        );

        let dlp = detail
            .role_scores
            .iter()
            .find(|row| row.role_id == "dm_deep_lying_playmaker_ip")
            .expect("dlp row");
        assert_eq!(dlp.score, None);
    }
}
