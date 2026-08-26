use std::collections::BTreeMap;

use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension, Row};

use crate::features::player_metrics::{
    club_dna::SCORE_MODEL_VERSION,
    potential_scores::assert_snapshot_roles_complete,
    resolver::{
        parse_requested_fields, read_dynamic_value, ClubDnaSqlBindings, DynamicValue, MetricField,
    },
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

    fn sql_expr(&self, club_dna_bindings: Option<ClubDnaSqlBindings>) -> String {
        match self {
            Self::Name => "p.name COLLATE NOCASE".to_string(),
            Self::Age => "p.age".to_string(),
            Self::Nationality => "p.nationalities_json COLLATE NOCASE".to_string(),
            Self::Club => "p.current_club COLLATE NOCASE".to_string(),
            Self::Division => "p.division COLLATE NOCASE".to_string(),
            Self::Ca => "p.ca".to_string(),
            Self::Pa => "p.pa".to_string(),
            Self::Value => "p.market_value_gbp".to_string(),
            Self::Dynamic(field) => field.sql_sort_expression_with_club_dna("p", club_dna_bindings),
        }
    }

    fn is_club_dna(&self) -> bool {
        matches!(self, Self::Dynamic(field) if field.is_club_dna())
    }

    fn potential_role_sort_identity(
        &self,
    ) -> Option<crate::features::player_metrics::resolver::PotentialRoleSortIdentity> {
        match self {
            Self::Dynamic(field) => field.potential_role_sort_identity(),
            _ => None,
        }
    }

    fn current_role_id(&self) -> Option<&'static str> {
        match self {
            Self::Dynamic(field) => field.current_role_id(),
            _ => None,
        }
    }

    fn club_dna_sort_identity(
        &self,
        definition_version: Option<i64>,
    ) -> Option<crate::features::player_metrics::resolver::ClubDnaSortIdentity> {
        match self {
            Self::Dynamic(field) => field.club_dna_sort_identity(definition_version),
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
    let potential_role_sort = sort_by.potential_role_sort_identity();
    let mut potential_role_ids = dynamic_fields
        .iter()
        .filter_map(MetricField::potential_role_id)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if let Some(identity) = potential_role_sort {
        potential_role_ids.push(identity.role_id.to_string());
    }
    assert_snapshot_roles_complete(conn, snapshot_id, &potential_role_ids)?;

    let club_dna_requested =
        sort_by.is_club_dna() || dynamic_fields.iter().any(MetricField::is_club_dna);
    let club_dna_definition_version = club_dna_requested
        .then(|| current_club_dna_definition_version(conn, snapshot_id))
        .transpose()?
        .flatten();
    let club_dna_sort = sort_by.club_dna_sort_identity(club_dna_definition_version);
    let club_dna_bindings = club_dna_requested.then(|| ClubDnaSqlBindings::new(3, 4));
    let mut query_bind_values = vec![Value::Integer(snapshot_id), Value::Integer(save_id)];
    if let Some(identity) = club_dna_sort {
        query_bind_values.push(
            identity
                .definition_version
                .map_or(Value::Null, Value::Integer),
        );
        query_bind_values.push(Value::Integer(identity.score_model_version));
    } else if club_dna_requested {
        query_bind_values.push(club_dna_definition_version.map_or(Value::Null, Value::Integer));
        query_bind_values.push(Value::Integer(SCORE_MODEL_VERSION));
    }

    let membership_sql = "p.snapshot_id = ?1
        AND p.current_club = (
            SELECT club_name FROM managed_club_settings WHERE save_id = ?2
        )";

    let count_sql = format!("SELECT COUNT(*) FROM players p WHERE {membership_sql}");
    let total = conn
        .query_row(&count_sql, params![snapshot_id, save_id], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let current_role_sort = sort_by.current_role_id();
    let mut from_sql = if let Some(role_id) = current_role_sort {
        format!(
            "FROM players p
             LEFT JOIN player_role_scores current_role_sort
               ON current_role_sort.snapshot_id = p.snapshot_id
              AND current_role_sort.role_id = '{role_id}'
              AND current_role_sort.uid = p.uid"
        )
    } else {
        "FROM players p".to_string()
    };
    if let Some(identity) = potential_role_sort {
        from_sql.push_str(&format!(
            " LEFT JOIN player_potential_role_scores potential_role_sort
              ON potential_role_sort.snapshot_id = p.snapshot_id
              AND potential_role_sort.uid = p.uid
              AND potential_role_sort.role_id = '{}'
              AND potential_role_sort.projection_model_version = {}",
            identity.role_id, identity.projection_model_version
        ));
    }
    if let Some((_, bindings)) = club_dna_sort.zip(club_dna_bindings) {
        from_sql.push_str(&format!(
            " LEFT JOIN club_dna_scores club_dna_sort
              ON club_dna_sort.snapshot_id = p.snapshot_id
              AND club_dna_sort.uid = p.uid
              AND club_dna_sort.definition_version = ?{}
              AND club_dna_sort.score_model_version = ?{}",
            bindings.definition_version, bindings.score_model_version
        ));
    }
    let order_sql = if current_role_sort.is_some() {
        format!(
            "ORDER BY current_role_sort.score {}, p.uid ASC",
            sort_dir.sql_keyword()
        )
    } else if potential_role_sort.is_some() {
        format!(
            "ORDER BY potential_role_sort.score {}, p.uid ASC",
            sort_dir.sql_keyword()
        )
    } else if club_dna_sort.is_some() {
        format!(
            "ORDER BY club_dna_sort.score IS NULL ASC, club_dna_sort.score {}, p.uid ASC",
            sort_dir.sql_keyword()
        )
    } else {
        let sort_expression = sort_by.sql_expr(club_dna_bindings);
        format!(
            "ORDER BY {sort_expression} {}, p.uid ASC",
            sort_dir.sql_keyword()
        )
    };
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
         {from_sql}
         WHERE {membership_sql}
             {order_sql}
             LIMIT ?{} OFFSET ?{}",
        dynamic_select_sql(&dynamic_fields, club_dna_bindings),
        query_bind_values.len() + 1,
        query_bind_values.len() + 2
    );
    query_bind_values.push(Value::Integer(limit));
    query_bind_values.push(Value::Integer(offset));
    let mut statement = conn
        .prepare(&select_sql)
        .map_err(|error| error.to_string())?;
    let players = statement
        .query_map(params_from_iter(query_bind_values.iter()), |row| {
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

fn dynamic_select_sql(
    fields: &[MetricField],
    club_dna_bindings: Option<ClubDnaSqlBindings>,
) -> String {
    fields
        .iter()
        .map(|field| {
            format!(
                ", {}",
                field.sql_expression_with_club_dna("p", club_dna_bindings)
            )
        })
        .collect()
}

fn current_club_dna_definition_version(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT definition_version
         FROM club_dna_definitions
         WHERE save_id = (SELECT save_id FROM snapshots WHERE id = ?1)",
        [snapshot_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}
