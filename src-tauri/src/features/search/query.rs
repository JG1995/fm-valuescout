use rusqlite::{params, Connection, OptionalExtension, Row};

pub const DEFAULT_PAGE_LIMIT: usize = 50;
pub const MAX_PAGE_LIMIT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSummary {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub birth_year: i64,
    pub birth_day_of_year: i64,
    pub nationalities: Vec<String>,
    pub club: Option<String>,
    pub division: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub market_value_gbp: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPlayersPage {
    pub players: Vec<PlayerSummary>,
    pub total: i64,
}

pub fn search_players(
    conn: &Connection,
    offset: usize,
    limit: usize,
) -> Result<SearchPlayersPage, String> {
    let snapshot_id: Option<i64> = conn
        .query_row(
            "SELECT s.id
             FROM snapshots s
             INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
             WHERE s.is_current = 1
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let Some(snapshot_id) = snapshot_id else {
        return Ok(SearchPlayersPage {
            players: Vec::new(),
            total: 0,
        });
    };

    let limit = limit.clamp(1, MAX_PAGE_LIMIT);
    let offset = i64::try_from(offset).map_err(|_| "search offset out of range".to_string())?;
    let limit = i64::try_from(limit).map_err(|_| "search limit out of range".to_string())?;

    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM players WHERE snapshot_id = ?1",
            params![snapshot_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT
                uid,
                name,
                age,
                birth_year,
                birth_day_of_year,
                nationalities_json,
                current_club,
                division,
                ca,
                pa,
                market_value_gbp
             FROM players
             WHERE snapshot_id = ?1
             ORDER BY ca DESC, uid ASC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|error| error.to_string())?;

    let players = stmt
        .query_map(params![snapshot_id, limit, offset], map_player_summary)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(SearchPlayersPage { players, total })
}

fn map_player_summary(row: &Row<'_>) -> rusqlite::Result<PlayerSummary> {
    let nationalities_json: String = row.get(5)?;
    let nationalities = parse_nationalities(&nationalities_json).map_err(|message| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                message,
            )),
        )
    })?;

    Ok(PlayerSummary {
        uid: row.get(0)?,
        name: row.get(1)?,
        age: row.get(2)?,
        birth_year: row.get(3)?,
        birth_day_of_year: row.get(4)?,
        nationalities,
        club: row.get(6)?,
        division: row.get(7)?,
        ca: row.get(8)?,
        pa: row.get(9)?,
        market_value_gbp: row.get(10)?,
    })
}

fn parse_nationalities(json: &str) -> Result<Vec<String>, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid nationalities_json: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service::{create_save, set_active_save};
    use serde_json::{json, Value};
    use std::path::Path;

    fn open_migrated(db_path: &Path) -> rusqlite::Connection {
        let conn = rusqlite::Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
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
            "teamLevel": "senior"
        })
    }

    fn ingest_players(conn: &mut rusqlite::Connection, players: Vec<Value>) {
        let mut root: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v5.json"))
                .expect("parse golden fixture");
        root["players"] = Value::Array(players);
        root["playerCount"] = json!(root["players"].as_array().unwrap().len());

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let dump_path = temp_dir.path().join("search-dump.json");
        std::fs::write(&dump_path, root.to_string()).expect("write dump");
        ingest_dump_file(conn, &dump_path).expect("ingest dump");
    }

    #[test]
    fn returns_empty_page_when_active_save_has_no_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("no-snapshot.db"));

        let page = search_players(&conn, 0, DEFAULT_PAGE_LIMIT).expect("search players");

        assert_eq!(page.total, 0);
        assert!(page.players.is_empty());
    }

    #[test]
    fn ignores_snapshots_on_inactive_saves() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("inactive-snapshot.db"));
        ingest_players(
            &mut conn,
            vec![player_template(1, "Only On First Save", 150)],
        );

        let second_save = create_save(&conn, "Second save").expect("create save");
        set_active_save(&mut conn, second_save.id).expect("switch save");

        let page = search_players(&conn, 0, DEFAULT_PAGE_LIMIT).expect("search after switch");

        assert_eq!(page.total, 0);
        assert!(page.players.is_empty());
    }

    #[test]
    fn returns_page_ordered_by_ca_descending_with_basic_fields_and_total() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("ordered.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Low CA", 100),
                player_template(2, "High CA", 180),
                player_template(3, "Mid CA", 140),
            ],
        );

        let page = search_players(&conn, 0, DEFAULT_PAGE_LIMIT).expect("search players");

        assert_eq!(page.total, 3);
        assert_eq!(page.players.len(), 3);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.ca)
                .collect::<Vec<_>>(),
            vec![180, 140, 100]
        );
        assert_eq!(page.players[0].uid, 2);
        assert_eq!(page.players[0].name, "High CA");
        assert_eq!(page.players[0].age, Some(26));
        assert_eq!(page.players[0].birth_year, 2000);
        assert_eq!(page.players[0].birth_day_of_year, 100);
        assert_eq!(page.players[0].nationalities, vec!["ENG".to_string()]);
        assert_eq!(page.players[0].club.as_deref(), Some("Test FC"));
        assert_eq!(page.players[0].division.as_deref(), Some("League One"));
        assert_eq!(page.players[0].pa, 190);
        assert_eq!(page.players[0].market_value_gbp, Some(1_000_000));
    }

    #[test]
    fn honours_offset_and_requested_limit() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("paged.db"));
        let players = (1..=5)
            .map(|index| player_template(index, &format!("Player {index}"), 100 + index as i64))
            .collect();
        ingest_players(&mut conn, players);

        let page = search_players(&conn, 2, 2).expect("offset page");
        assert_eq!(page.total, 5);
        assert_eq!(page.players.len(), 2);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.ca)
                .collect::<Vec<_>>(),
            vec![103, 102]
        );
    }

    #[test]
    fn caps_limit_at_max_page_limit() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("cap.db"));
        ingest_players(&mut conn, vec![player_template(1, "Seed", 100)]);

        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");

        let extra = (MAX_PAGE_LIMIT + 5) as i64;
        for uid in 2..=extra {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, ?3, ?3, ?4, 2000, 1, '[]', 'right', '{}', '{}', '{}', '{}')",
                rusqlite::params![snapshot_id, uid, 50 + uid, format!("Extra {uid}")],
            )
            .expect("insert extra player");
        }

        let page = search_players(&conn, 0, MAX_PAGE_LIMIT + 50).expect("capped search");
        assert_eq!(page.total, extra);
        assert_eq!(page.players.len(), MAX_PAGE_LIMIT);
    }
}
