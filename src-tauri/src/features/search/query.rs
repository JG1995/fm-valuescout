use std::collections::BTreeMap;

use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use super::filter::{compile_filters, dynamic_fields_from_ast, field_value_sql, FilterAst};

pub const DEFAULT_PAGE_LIMIT: usize = 50;
pub const MAX_PAGE_LIMIT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortField {
    Name,
    Age,
    Nationality,
    Club,
    Division,
    Ca,
    Pa,
    Value,
    /// Whitelisted filter field id (`role.*`, `attr.*`, scalar non-basics, …).
    Dynamic(String),
}

impl SortField {
    pub const DEFAULT: Self = Self::Ca;

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "name" => Ok(Self::Name),
            "age" => Ok(Self::Age),
            "nationality" => Ok(Self::Nationality),
            "club" => Ok(Self::Club),
            "division" => Ok(Self::Division),
            "ca" => Ok(Self::Ca),
            "pa" => Ok(Self::Pa),
            "value" => Ok(Self::Value),
            other => {
                // Reject fields that cannot produce a single sortable expression.
                let _ = field_value_sql(other)?;
                Ok(Self::Dynamic(other.to_string()))
            }
        }
    }

    fn sql_expr(&self) -> Result<String, String> {
        Ok(match self {
            Self::Name => "name COLLATE NOCASE".to_string(),
            Self::Age => "age".to_string(),
            // ponytail: sort by raw nationalities_json text, not display join order
            // Upgrade to first-nationality / normalized key if multi-nation sort UX matters
            Self::Nationality => "nationalities_json COLLATE NOCASE".to_string(),
            Self::Club => "current_club COLLATE NOCASE".to_string(),
            Self::Division => "division COLLATE NOCASE".to_string(),
            Self::Ca => "ca".to_string(),
            Self::Pa => "pa".to_string(),
            Self::Value => "market_value_gbp".to_string(),
            Self::Dynamic(field) => field_value_sql(field)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub const DEFAULT: Self = Self::Desc;

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(format!("unknown search sort direction: {value}")),
        }
    }

    fn sql_keyword(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicValue {
    Integer(i64),
    Text(String),
}

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
    /// Values for active non-basic filter fields (field id → nullable cell).
    pub dynamic_values: BTreeMap<String, Option<DynamicValue>>,
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
    sort_by: SortField,
    sort_dir: SortDir,
    filter_ast: Option<&FilterAst>,
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

    let dynamic_fields = filter_ast.map(dynamic_fields_from_ast).unwrap_or_default();

    let compiled = match filter_ast {
        None => None,
        Some(ast) => {
            let compiled = compile_filters(ast, 2)?;
            if compiled.sql.is_empty() {
                None
            } else {
                Some(compiled)
            }
        }
    };

    let mut where_sql = "players.snapshot_id = ?1".to_string();
    if let Some(compiled) = &compiled {
        where_sql.push_str(" AND ");
        where_sql.push_str(&compiled.sql);
    }

    let mut bind_values = vec![Value::Integer(snapshot_id)];
    if let Some(compiled) = &compiled {
        bind_values.extend(compiled.params.clone());
    }

    let count_sql = format!("SELECT COUNT(*) FROM players WHERE {where_sql}");
    let mut count_stmt = conn
        .prepare(&count_sql)
        .map_err(|error| error.to_string())?;
    let total: i64 = count_stmt
        .query_row(params_from_iter(bind_values.iter()), |row| row.get(0))
        .map_err(|error| error.to_string())?;

    let limit_index = bind_values.len() + 1;
    let offset_index = bind_values.len() + 2;
    bind_values.push(Value::Integer(limit));
    bind_values.push(Value::Integer(offset));

    // Whitelisted expr + dir only — never interpolate raw client strings.
    let order_sql = format!(
        "ORDER BY {} {}, players.uid ASC",
        sort_by.sql_expr()?,
        sort_dir.sql_keyword()
    );

    let mut select_sql = String::from(
        "SELECT
                players.uid,
                players.name,
                players.age,
                players.birth_year,
                players.birth_day_of_year,
                players.nationalities_json,
                players.current_club,
                players.division,
                players.ca,
                players.pa,
                players.market_value_gbp",
    );
    for field in &dynamic_fields {
        select_sql.push_str(", ");
        select_sql.push_str(&field_value_sql(field)?);
    }
    select_sql.push_str(&format!(
        "
             FROM players
             WHERE {where_sql}
             {order_sql}
             LIMIT ?{limit_index} OFFSET ?{offset_index}"
    ));

    let mut stmt = conn
        .prepare(&select_sql)
        .map_err(|error| error.to_string())?;

    let players = stmt
        .query_map(params_from_iter(bind_values.iter()), |row| {
            map_player_summary(row, &dynamic_fields)
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(SearchPlayersPage { players, total })
}

fn map_player_summary(row: &Row<'_>, dynamic_fields: &[String]) -> rusqlite::Result<PlayerSummary> {
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

    let mut dynamic_values = BTreeMap::new();
    for (offset, field) in dynamic_fields.iter().enumerate() {
        let idx = 11 + offset;
        let cell = read_dynamic_cell(row, idx, field)?;
        dynamic_values.insert(field.clone(), cell);
    }

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
        dynamic_values,
    })
}

fn read_dynamic_cell(
    row: &Row<'_>,
    idx: usize,
    field: &str,
) -> rusqlite::Result<Option<DynamicValue>> {
    // Integer-like fields (scores, attrs, scalars, bools as 0/1).
    if field.starts_with("role.")
        || field.starts_with("attr.")
        || field.starts_with("hidden.")
        || field.starts_with("personality.")
        || field.starts_with("pos.")
        || matches!(
            field,
            "height"
                | "wage"
                | "reputation"
                | "world_reputation"
                | "birth_year"
                | "contract_year"
                | "transfer_listed"
                | "loan_listed"
                | "not_for_sale"
                | "set_for_release"
                | "on_loan"
        )
    {
        let value: Option<i64> = row.get(idx)?;
        return Ok(value.map(DynamicValue::Integer));
    }

    let value: Option<String> = row.get(idx)?;
    Ok(value.map(DynamicValue::Text))
}

fn parse_nationalities(json: &str) -> Result<Vec<String>, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid nationalities_json: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::search::filter::{parse_filter_ast, FilterRule, FilterValue};
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service::{create_save, set_active_save};
    use serde_json::{json, Value};
    use std::path::Path;

    fn search_without_filters(
        conn: &Connection,
        offset: usize,
        limit: usize,
        sort_by: SortField,
        sort_dir: SortDir,
    ) -> Result<SearchPlayersPage, String> {
        search_players(conn, offset, limit, sort_by, sort_dir, None)
    }

    fn search_with_filters(
        conn: &Connection,
        offset: usize,
        limit: usize,
        sort_by: SortField,
        sort_dir: SortDir,
        rules: Vec<FilterRule>,
        combine: Option<&str>,
    ) -> Result<SearchPlayersPage, String> {
        let ast = parse_filter_ast(rules, combine)?;
        search_players(conn, offset, limit, sort_by, sort_dir, Some(&ast))
    }

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

        let page = search_without_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
        )
        .expect("search players");

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

        let page = search_without_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
        )
        .expect("search after switch");

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

        let page = search_without_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
        )
        .expect("search players");

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

        let page = search_without_filters(&conn, 2, 2, SortField::DEFAULT, SortDir::DEFAULT)
            .expect("offset page");
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

        let page = search_without_filters(
            &conn,
            0,
            MAX_PAGE_LIMIT + 50,
            SortField::DEFAULT,
            SortDir::DEFAULT,
        )
        .expect("capped search");
        assert_eq!(page.total, extra);
        assert_eq!(page.players.len(), MAX_PAGE_LIMIT);
    }

    #[test]
    fn rejects_unknown_sort_field() {
        assert!(SortField::parse("not_a_column").is_err());
        assert!(SortField::parse("ca; DROP TABLE players").is_err());
        assert!(SortDir::parse("sideways").is_err());
    }

    #[test]
    fn orders_by_whitelisted_name_ascending() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("sort-name.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Charlie", 100),
                player_template(2, "Alice", 180),
                player_template(3, "Bob", 140),
            ],
        );

        let page =
            search_without_filters(&conn, 0, DEFAULT_PAGE_LIMIT, SortField::Name, SortDir::Asc)
                .expect("sort by name");

        assert_eq!(
            page.players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Alice", "Bob", "Charlie"]
        );
    }

    #[test]
    fn orders_by_pa_ascending_when_requested() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("sort-pa.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Low", 100),
                player_template(2, "High", 180),
                player_template(3, "Mid", 140),
            ],
        );

        let page =
            search_without_filters(&conn, 0, DEFAULT_PAGE_LIMIT, SortField::Pa, SortDir::Asc)
                .expect("sort by pa");

        assert_eq!(
            page.players
                .iter()
                .map(|player| player.pa)
                .collect::<Vec<_>>(),
            vec![110, 150, 190]
        );
    }

    fn filter_rule(field: &str, op: &str, value: FilterValue) -> FilterRule {
        FilterRule {
            field: field.to_string(),
            op: op.to_string(),
            value,
        }
    }

    #[test]
    fn filters_players_by_ca_and_name_with_and_combine() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-and.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Alpha Star", 180),
                player_template(2, "Beta Star", 120),
                player_template(3, "Alpha Bench", 150),
            ],
        );

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![
                filter_rule("ca", "gt", FilterValue::Integer(140)),
                filter_rule("name", "contains", FilterValue::Text("Alpha".to_string())),
            ],
            Some("and"),
        )
        .expect("filtered search");

        assert_eq!(page.total, 2);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Alpha Star", "Alpha Bench"]
        );
    }

    #[test]
    fn filters_players_with_or_combine() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-or.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Low", 100),
                player_template(2, "High", 180),
                player_template(3, "Mid", 140),
            ],
        );

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![
                filter_rule("ca", "eq", FilterValue::Integer(100)),
                filter_rule("ca", "eq", FilterValue::Integer(180)),
            ],
            Some("or"),
        )
        .expect("or filtered search");

        assert_eq!(page.total, 2);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.ca)
                .collect::<Vec<_>>(),
            vec![180, 100]
        );
    }

    #[test]
    fn excludes_nullable_integers_from_integer_filters() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-null-age.db"));
        ingest_players(&mut conn, vec![player_template(1, "Known Age", 150)]);

        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year, age,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, 2, 120, 130, 'Unknown Age', 2000, 1, NULL, '[]', 'right', '{}', '{}', '{}', '{}')",
            rusqlite::params![snapshot_id],
        )
        .expect("insert null age");

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule("age", "gt", FilterValue::Integer(20))],
            None,
        )
        .expect("age filter");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Known Age");
    }

    #[test]
    fn excludes_null_boolean_from_is_not_filter() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-null-bool.db"));
        ingest_players(&mut conn, vec![player_template(1, "Listed", 150)]);

        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");
        conn.execute(
            "UPDATE players SET transfer_listed = NULL WHERE snapshot_id = ?1 AND uid = 1",
            rusqlite::params![snapshot_id],
        )
        .expect("null transfer listed");

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "transfer_listed",
                "is_not",
                FilterValue::Bool(true),
            )],
            None,
        )
        .expect("bool is_not filter");

        assert_eq!(page.total, 0);
    }

    struct DeepPlayerFields {
        nationalities: Value,
        positions: Value,
        attributes: Value,
        hidden: Value,
        personality: Value,
    }

    fn player_with_deep_fields(uid: u64, name: &str, ca: i64, deep: DeepPlayerFields) -> Value {
        let mut player = player_template(uid, name, ca);
        player["nationalities"] = deep.nationalities;
        player["positions"] = deep.positions;
        player["attributes"] = deep.attributes;
        player["hiddenAttributes"] = deep.hidden;
        player["personality"] = deep.personality;
        player
    }

    #[test]
    fn filters_by_attribute_json_extract_and_excludes_null_attr() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-attr.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "Fast",
                    150,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 18 }),
                        attributes: json!({ "Acceleration": 16, "Pace": 14 }),
                        hidden: json!({ "Consistency": 12 }),
                        personality: json!({ "Ambition": 14 }),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "Slow",
                    140,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 18 }),
                        attributes: json!({ "Acceleration": 8, "Pace": 14 }),
                        hidden: json!({ "Consistency": 12 }),
                        personality: json!({ "Ambition": 14 }),
                    },
                ),
                player_with_deep_fields(
                    3,
                    "Unknown Accel",
                    160,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "ST": 15 }),
                        attributes: json!({ "Acceleration": null, "Pace": 18 }),
                        hidden: json!({ "Consistency": 12 }),
                        personality: json!({ "Ambition": 14 }),
                    },
                ),
            ],
        );

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "attr.Acceleration",
                "gt",
                FilterValue::Integer(12),
            )],
            None,
        )
        .expect("attr filter");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Fast");
    }

    #[test]
    fn filters_nationality_when_any_list_element_matches() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-nation.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "Dual",
                    150,
                    DeepPlayerFields {
                        nationalities: json!(["SCO", "ENG"]),
                        positions: json!({ "MC": 18 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "Welsh",
                    140,
                    DeepPlayerFields {
                        nationalities: json!(["WAL"]),
                        positions: json!({ "MC": 18 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
            ],
        );

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "nationality",
                "is",
                FilterValue::Text("ENG".to_string()),
            )],
            None,
        )
        .expect("nationality filter");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Dual");
    }

    #[test]
    fn filters_position_presence_and_suitability() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-pos.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "Natural MC",
                    150,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 18, "DM": 12 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "Fringe MC",
                    140,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 10, "ST": 18 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
                player_with_deep_fields(
                    3,
                    "Striker Only",
                    160,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "ST": 20 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
            ],
        );

        let presence = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "position",
                "is",
                FilterValue::Text("MC".to_string()),
            )],
            None,
        )
        .expect("position presence");
        assert_eq!(presence.total, 2);
        assert_eq!(
            presence
                .players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Natural MC", "Fringe MC"]
        );

        let suitability = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule("pos.MC", "gt", FilterValue::Integer(15))],
            None,
        )
        .expect("position suitability");
        assert_eq!(suitability.total, 1);
        assert_eq!(suitability.players[0].name, "Natural MC");
    }

    #[test]
    fn position_contains_matches_exact_key_not_substring() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-pos-exact.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "True MC",
                    150,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 18 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "AMC Only",
                    140,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "AMC": 18 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
            ],
        );

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "position",
                "contains",
                FilterValue::Text("MC".to_string()),
            )],
            None,
        )
        .expect("exact position contains");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "True MC");
    }

    #[test]
    fn attribute_filter_on_two_thousand_players_stays_interactive() {
        // ponytail: no JSON expression indexes for attribute/position filters
        // Upgrade to generated columns / indexes if attr filter p95 exceeds ~200ms on a full ~180k snapshot
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-attr-timing.db"));
        ingest_players(
            &mut conn,
            vec![player_with_deep_fields(
                1,
                "Seed",
                150,
                DeepPlayerFields {
                    nationalities: json!(["ENG"]),
                    positions: json!({ "MC": 18 }),
                    attributes: json!({ "Acceleration": 16 }),
                    hidden: json!({ "Consistency": 10 }),
                    personality: json!({ "Ambition": 10 }),
                },
            )],
        );

        let snapshot_id: i64 = conn
            .query_row(
                "SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("snapshot id");

        let attrs = r#"{"Acceleration":10,"Pace":12}"#;
        for uid in 2..=2000 {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, 100, 110, ?3, 2000, 1, '[\"ENG\"]', 'right', '{\"MC\":10}', ?4, '{}', '{}')",
                rusqlite::params![snapshot_id, uid, format!("P{uid}"), attrs],
            )
            .expect("insert bulk player");
        }

        let started = std::time::Instant::now();
        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "attr.Acceleration",
                "gt",
                FilterValue::Integer(12),
            )],
            None,
        )
        .expect("timed attr filter");
        let elapsed = started.elapsed();

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Seed");
        assert!(
            elapsed.as_millis() < 500,
            "attr json_extract filter on 2k players took {:?}; investigate indexes",
            elapsed
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
            rusqlite::params![score, snapshot_id, uid, role_id],
        )
        .expect("update role score");
    }

    #[test]
    fn filters_by_role_score_and_excludes_null_score() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-role.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "High DLP", 150),
                player_template(2, "Low DLP", 140),
                player_template(3, "Null DLP", 160),
            ],
        );
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", Some(85));
        set_role_score(&conn, 2, "deep_lying_playmaker_ip", Some(40));
        set_role_score(&conn, 3, "deep_lying_playmaker_ip", None);

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "role.deep_lying_playmaker_ip",
                "gt",
                FilterValue::Integer(70),
            )],
            None,
        )
        .expect("role score filter");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "High DLP");
    }

    #[test]
    fn returns_dynamic_values_for_active_non_basic_filter_fields() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("dynamic-cols.db"));
        ingest_players(
            &mut conn,
            vec![player_with_deep_fields(
                1,
                "Scout Target",
                150,
                DeepPlayerFields {
                    nationalities: json!(["ENG"]),
                    positions: json!({ "MC": 18 }),
                    attributes: json!({ "Acceleration": 16 }),
                    hidden: json!({ "Consistency": 10 }),
                    personality: json!({ "Ambition": 10 }),
                },
            )],
        );
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", Some(82));

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![
                filter_rule(
                    "role.deep_lying_playmaker_ip",
                    "gt",
                    FilterValue::Integer(70),
                ),
                filter_rule("attr.Acceleration", "gt", FilterValue::Integer(12)),
            ],
            None,
        )
        .expect("dynamic values search");

        assert_eq!(page.total, 1);
        let player = &page.players[0];
        assert_eq!(
            player.dynamic_values.get("role.deep_lying_playmaker_ip"),
            Some(&Some(DynamicValue::Integer(82)))
        );
        assert_eq!(
            player.dynamic_values.get("attr.Acceleration"),
            Some(&Some(DynamicValue::Integer(16)))
        );
        assert!(
            !player.dynamic_values.contains_key("ca"),
            "basic fields must not appear in dynamic_values"
        );
    }

    #[test]
    fn orders_by_role_score_when_sort_field_is_role() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("sort-role.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Low Role", 180),
                player_template(2, "High Role", 100),
            ],
        );
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", Some(40));
        set_role_score(&conn, 2, "deep_lying_playmaker_ip", Some(90));

        let sort_by = SortField::parse("role.deep_lying_playmaker_ip").expect("parse role sort");
        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            sort_by,
            SortDir::Desc,
            vec![filter_rule(
                "role.deep_lying_playmaker_ip",
                "gt",
                FilterValue::Integer(0),
            )],
            None,
        )
        .expect("sort by role");

        assert_eq!(
            page.players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            vec!["High Role", "Low Role"]
        );
    }
}
