use std::collections::BTreeMap;

use rusqlite::{params, Connection, Row};

use crate::features::player_metrics::{
    potential_cache::materialize_player_roles,
    resolver::{parse_requested_fields, read_dynamic_value, DynamicValue, MetricField},
};

pub const DEFAULT_SQUAD_PAGE_LIMIT: usize = 50;
pub const MAX_SQUAD_PAGE_LIMIT: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SquadSortField {
    Name,
    Age,
    Nationality,
    Club,
    Division,
    Ca,
    Pa,
    Value,
    Dynamic(MetricField),
}

impl SquadSortField {
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
            other => Ok(Self::Dynamic(MetricField::parse(other)?)),
        }
    }

    fn sql_expr(&self) -> String {
        match self {
            Self::Name => "p.name COLLATE NOCASE".to_string(),
            Self::Age => "p.age".to_string(),
            Self::Nationality => "p.nationalities_json COLLATE NOCASE".to_string(),
            Self::Club => "p.current_club COLLATE NOCASE".to_string(),
            Self::Division => "p.division COLLATE NOCASE".to_string(),
            Self::Ca => "p.ca".to_string(),
            Self::Pa => "p.pa".to_string(),
            Self::Value => "p.market_value_gbp".to_string(),
            Self::Dynamic(field) => field.sql_sort_expression("p"),
        }
    }

    fn potential_role_id(&self) -> Option<&'static str> {
        match self {
            Self::Dynamic(field) => field.potential_role_id(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquadSortDir {
    Asc,
    Desc,
}

impl SquadSortDir {
    pub const DEFAULT: Self = Self::Desc;

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err(format!("unknown squad sort direction: {value}")),
        }
    }

    fn sql_keyword(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SquadPlayer {
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
    pub dynamic_values: BTreeMap<String, Option<DynamicValue>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SquadPlayersPage {
    pub players: Vec<SquadPlayer>,
    pub total: i64,
}

pub fn list_squad_player_uids(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
) -> Result<Vec<i64>, String> {
    let mut statement = conn
        .prepare(
            "SELECT DISTINCT p.uid
             FROM players p
             WHERE p.snapshot_id = ?1
               AND p.current_club = (
                   SELECT club_name FROM managed_club_settings WHERE save_id = ?2
               )
             ORDER BY p.uid ASC",
        )
        .map_err(|error| error.to_string())?;
    let player_uids = statement
        .query_map(params![snapshot_id, save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(player_uids)
}

pub fn list_squad_players(
    conn: &Connection,
    save_id: i64,
    offset: usize,
    limit: usize,
    sort_by: SquadSortField,
    sort_dir: SquadSortDir,
    requested_fields: &[String],
) -> Result<SquadPlayersPage, String> {
    let requested_fields = parse_requested_fields(requested_fields)?;
    let dynamic_fields = requested_fields
        .into_iter()
        .filter(|field| !field.is_basic_table_field())
        .collect::<Vec<_>>();
    let Some(snapshot_id) = super::depth::current_snapshot_id(conn, save_id)? else {
        return Ok(empty_page());
    };
    let is_configured: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM managed_club_settings WHERE save_id = ?1
             )",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !is_configured {
        return Ok(empty_page());
    }

    let limit = limit.clamp(1, MAX_SQUAD_PAGE_LIMIT);
    let offset = i64::try_from(offset).map_err(|_| "squad offset out of range".to_string())?;
    let limit = i64::try_from(limit).map_err(|_| "squad limit out of range".to_string())?;
    let membership_sql = "p.snapshot_id = ?1
        AND p.current_club = (
            SELECT club_name FROM managed_club_settings WHERE save_id = ?2
        )";

    if let Some(role_id) = sort_by.potential_role_id() {
        let player_uids = list_squad_player_uids(conn, save_id, snapshot_id)?;
        materialize_player_roles(conn, snapshot_id, &player_uids, &[role_id.to_string()])?;
    }

    let count_sql = format!("SELECT COUNT(*) FROM players p WHERE {membership_sql}");
    let total = conn
        .query_row(&count_sql, params![snapshot_id, save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let order_sql = format!(
        "ORDER BY {} {}, p.uid ASC",
        sort_by.sql_expr(),
        sort_dir.sql_keyword()
    );
    let potential_display_roles = potential_role_ids(&dynamic_fields);
    if !potential_display_roles.is_empty() {
        let page_uids = query_page_uids(
            conn,
            snapshot_id,
            save_id,
            membership_sql,
            &order_sql,
            limit,
            offset,
        )?;
        materialize_player_roles(conn, snapshot_id, &page_uids, &potential_display_roles)?;
    }
    let select_sql = format!(
        "SELECT
             p.uid,
             p.name,
             p.age,
             p.birth_year,
             p.birth_day_of_year,
             p.nationalities_json,
             p.current_club,
             p.division,
             p.ca,
             p.pa,
             p.market_value_gbp{}
         FROM players p
         WHERE {membership_sql}
         {order_sql}
         LIMIT ?3 OFFSET ?4",
        dynamic_select_sql(&dynamic_fields)
    );
    let mut statement = conn
        .prepare(&select_sql)
        .map_err(|error| error.to_string())?;
    let players = statement
        .query_map(params![snapshot_id, save_id, limit, offset], |row| {
            map_player(row, &dynamic_fields)
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(SquadPlayersPage { players, total })
}

fn empty_page() -> SquadPlayersPage {
    SquadPlayersPage {
        players: Vec::new(),
        total: 0,
    }
}

fn map_player(row: &Row<'_>, dynamic_fields: &[MetricField]) -> rusqlite::Result<SquadPlayer> {
    let nationalities_json: String = row.get(5)?;
    let nationalities = serde_json::from_str(&nationalities_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid nationalities_json: {error}"),
            )),
        )
    })?;
    let mut dynamic_values = BTreeMap::new();
    for (offset, field) in dynamic_fields.iter().enumerate() {
        dynamic_values.insert(
            field.id().to_string(),
            read_dynamic_value(row, 11 + offset, field)?,
        );
    }

    Ok(SquadPlayer {
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

fn dynamic_select_sql(fields: &[MetricField]) -> String {
    fields
        .iter()
        .map(|field| format!(", {}", field.sql_expression("p")))
        .collect()
}

fn potential_role_ids(fields: &[MetricField]) -> Vec<String> {
    let mut role_ids = Vec::new();
    for field in fields {
        let Some(role_id) = field.potential_role_id() else {
            continue;
        };
        if !role_ids.iter().any(|existing| existing == role_id) {
            role_ids.push(role_id.to_string());
        }
    }
    role_ids
}

fn query_page_uids(
    conn: &Connection,
    snapshot_id: i64,
    save_id: i64,
    membership_sql: &str,
    order_sql: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<i64>, String> {
    let sql = format!(
        "SELECT p.uid FROM players p WHERE {membership_sql} {order_sql} LIMIT ?3 OFFSET ?4"
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![snapshot_id, save_id, limit, offset], |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}
