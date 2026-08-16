use std::collections::{BTreeMap, HashMap};

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;

use crate::features::scoring::{
    catalog::all_roles, projection::project_attributes, score::score_role,
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

    let attributes = scoring_attributes(&player.attributes)?;
    let projected_attributes = project_attributes(
        &attributes,
        player.ca,
        player.pa,
        player.age,
        player
            .positions
            .iter()
            .map(|(position, familiarity)| (position.as_str(), *familiarity)),
    );
    let potential_attributes = projected_attributes
        .iter()
        .map(|(key, value)| (key.clone(), value.map(i64::from)))
        .collect();
    let role_scores = load_role_scores(conn, snapshot_id, uid, &projected_attributes)?;
    player.potential_attributes = potential_attributes;
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

fn load_role_scores(
    conn: &Connection,
    snapshot_id: i64,
    uid: i64,
    projected_attributes: &HashMap<String, Option<u8>>,
) -> Result<Vec<PlayerRoleScore>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT role_id, score
             FROM player_role_scores
             WHERE snapshot_id = ?1 AND uid = ?2",
        )
        .map_err(|error| error.to_string())?;

    let rows = stmt
        .query_map(params![snapshot_id, uid], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
        })
        .map_err(|error| error.to_string())?;

    let mut scores_by_role = HashMap::new();
    for row in rows {
        let (role_id, score) = row.map_err(|error| error.to_string())?;
        scores_by_role.insert(role_id, score);
    }

    let mut role_scores = Vec::with_capacity(all_roles().len());
    for role in all_roles() {
        role_scores.push(PlayerRoleScore {
            role_id: role.role_id.to_string(),
            display_name: role.display_name.to_string(),
            phase: role.phase.as_db_str().to_string(),
            position_tags: role
                .position_tags
                .iter()
                .map(|tag| (*tag).to_string())
                .collect(),
            score: scores_by_role.get(role.role_id).copied().unwrap_or(None),
            potential_score: score_role(projected_attributes, role).map(i64::from),
        });
    }

    Ok(role_scores)
}

fn scoring_attributes(
    attributes: &BTreeMap<String, Option<i64>>,
) -> Result<HashMap<String, Option<u8>>, String> {
    attributes
        .iter()
        .map(|(key, value)| {
            let value = value
                .map(|value| {
                    u8::try_from(value)
                        .map_err(|_| format!("attribute `{key}` is outside the u8 range"))
                })
                .transpose()?;
            Ok((key.clone(), value))
        })
        .collect()
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
    use crate::features::scoring::{
        catalog::{all_roles, RolePhase, DUMP_ATTRIBUTE_KEYS},
        projection::project_attributes,
        score::score_role,
    };
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service::{create_save, set_active_save};
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
        conn.execute(
            "UPDATE player_role_scores
             SET score = ?1
             WHERE snapshot_id = ?2 AND uid = ?3 AND role_id = ?4",
            params![score, snapshot_id, uid, role_id],
        )
        .expect("update role score");
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
            .find(|row| row.role_id == "goalkeeper_ip")
            .expect("goalkeeper_ip row");
        assert_eq!(goalkeeper.display_name, "Goalkeeper");
        assert_eq!(goalkeeper.phase, RolePhase::InPossession.as_db_str());
        assert_eq!(goalkeeper.position_tags, vec!["GK".to_string()]);
        assert_eq!(
            goalkeeper.score,
            Some(42),
            "must round-trip score from player_role_scores"
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
    fn returns_role_potential_from_projected_visible_attributes() {
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

        let attributes = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), Some(10)))
            .collect();
        let projected_attributes =
            project_attributes(&attributes, 80, 170, Some(20), [("AMR", Some(20))]);
        let expected_potential_attributes = projected_attributes
            .iter()
            .map(|(key, value)| (key.clone(), value.map(i64::from)))
            .collect::<BTreeMap<_, _>>();
        let centre_forward = all_roles()
            .iter()
            .find(|role| role.role_id == "centre_forward_ip")
            .expect("centre forward role");
        let expected_potential_score =
            score_role(&projected_attributes, centre_forward).map(i64::from);

        let detail = get_player(&conn, 1)
            .expect("get_player")
            .expect("player present");
        assert_eq!(detail.potential_attributes, expected_potential_attributes);
        assert_eq!(detail.role_scores.len(), all_roles().len());
        assert!(detail
            .role_scores
            .iter()
            .all(|role| { role.score.is_some() && role.potential_score.is_some() }));
        let centre_forward = detail
            .role_scores
            .iter()
            .find(|row| row.role_id == "centre_forward_ip")
            .expect("centre forward row");

        assert_eq!(centre_forward.score, Some(50));
        assert_eq!(centre_forward.potential_score, expected_potential_score);
        assert_ne!(centre_forward.potential_score, centre_forward.score);
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
            .find(|row| row.role_id == "deep_lying_playmaker_ip")
            .expect("dlp row");
        assert_eq!(dlp.score, None);
    }
}
