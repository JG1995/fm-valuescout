use rusqlite::{params, Connection, Row};

pub const DEFAULT_SQUAD_PAGE_LIMIT: usize = 50;
pub const MAX_SQUAD_PAGE_LIMIT: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquadSortField {
    Name,
    Age,
    Nationality,
    Club,
    Division,
    Ca,
    Pa,
    Value,
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
            _ => Err(format!("unknown squad sort field: {value}")),
        }
    }

    fn sql_expr(self) -> &'static str {
        match self {
            Self::Name => "p.name COLLATE NOCASE",
            Self::Age => "p.age",
            Self::Nationality => "p.nationalities_json COLLATE NOCASE",
            Self::Club => "p.current_club COLLATE NOCASE",
            Self::Division => "p.division COLLATE NOCASE",
            Self::Ca => "p.ca",
            Self::Pa => "p.pa",
            Self::Value => "p.market_value_gbp",
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
               AND EXISTS(
                   SELECT 1
                   FROM planner_club_sources source
                   WHERE source.save_id = ?2
                     AND source.club_name = p.current_club
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
) -> Result<SquadPlayersPage, String> {
    let Some(snapshot_id) = super::depth::current_snapshot_id(conn, save_id)? else {
        return Ok(empty_page());
    };
    let is_configured: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM planner_club_settings WHERE save_id = ?1
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
        AND EXISTS(
            SELECT 1
            FROM planner_club_sources source
            WHERE source.save_id = ?2
              AND source.club_name = p.current_club
        )";

    let count_sql = format!("SELECT COUNT(*) FROM players p WHERE {membership_sql}");
    let total = conn
        .query_row(&count_sql, params![snapshot_id, save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let order_sql = format!(
        "ORDER BY {} {}, p.uid ASC",
        sort_by.sql_expr(),
        sort_dir.sql_keyword()
    );
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
             p.market_value_gbp
         FROM players p
         WHERE {membership_sql}
         {order_sql}
         LIMIT ?3 OFFSET ?4"
    );
    let mut statement = conn
        .prepare(&select_sql)
        .map_err(|error| error.to_string())?;
    let players = statement
        .query_map(params![snapshot_id, save_id, limit, offset], map_player)
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

fn map_player(row: &Row<'_>) -> rusqlite::Result<SquadPlayer> {
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
    })
}
