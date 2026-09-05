use std::collections::{BTreeMap, BTreeSet, HashMap};

use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension, Row};

use super::{fit::tactic_adjusted_score, suggested_training, tactic};

use crate::features::player_metrics::{
    club_dna::SCORE_MODEL_VERSION,
    compact::{
        assert_read_models_complete, player_current_column, player_metrics_join,
        PLAYER_METRICS_ALIAS,
    },
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

    fn potential_role_id(&self) -> Option<&'static str> {
        match self {
            Self::Dynamic(field) => field.potential_role_id(),
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
    pub suggested_training: Option<String>,
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

    let tactic_row_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM planner_tactic_lanes WHERE save_id = ?1",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let tactic = if tactic_row_count == 0 {
        None
    } else {
        Some(tactic::load_tactic(conn, save_id)?)
    };
    let assignments = match tactic {
        None => HashMap::new(),
        Some(_) => conn
            .prepare("SELECT player_uid, lane_id FROM planner_assignments WHERE save_id = ?1")
            .map_err(|error| error.to_string())?
            .query_map(params![save_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|error| error.to_string())?,
    };

    let limit = limit.clamp(1, MAX_SQUAD_PAGE_LIMIT);
    let offset = i64::try_from(offset).map_err(|_| "squad offset out of range".to_string())?;
    let limit = i64::try_from(limit).map_err(|_| "squad limit out of range".to_string())?;
    let current_role_sort = sort_by.current_role_id();
    let potential_role_sort = sort_by.potential_role_id();
    let uses_current_role_metrics = current_role_sort.is_some()
        || dynamic_fields
            .iter()
            .any(|field| field.current_role_id().is_some());
    let uses_potential_role_metrics = potential_role_sort.is_some()
        || dynamic_fields
            .iter()
            .any(|field| field.potential_role_id().is_some());
    assert_read_models_complete(
        conn,
        snapshot_id,
        uses_current_role_metrics,
        uses_potential_role_metrics,
    )?;

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
    let mut from_sql = "FROM players p".to_string();
    if uses_current_role_metrics || uses_potential_role_metrics {
        from_sql.push_str(&player_metrics_join(
            "p",
            uses_current_role_metrics,
            uses_potential_role_metrics,
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
    let order_sql = if club_dna_sort.is_some() {
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
             p.market_value_gbp,
             p.attributes_json{}
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
    let rows = statement
        .query_map(params_from_iter(query_bind_values.iter()), |row| {
            map_player(row, &dynamic_fields)
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    let mut players = Vec::with_capacity(rows.len());
    // Fallback needs current two-phase role metrics, preferred foot, and
    // positions only for page rows that actually use it: unassigned players
    // still developing toward their potential. Assigned and fully developed
    // players keep their existing guards, so their fallback-only inputs are
    // never queried or parsed here; missing or wrong-version compact rows for
    // an eligible row simply leave every lane ineligible via the versioned
    // LEFT JOIN below and yield no suggestion.
    let fallback_uids = rows
        .iter()
        .filter(|(player, _)| {
            tactic.is_some() && !assignments.contains_key(&player.uid) && player.ca < player.pa
        })
        .map(|(player, _)| player.uid)
        .collect::<Vec<_>>();
    let fallback_inputs = load_fallback_inputs(conn, snapshot_id, tactic.as_ref(), &fallback_uids)?;
    for (mut player, attributes) in rows {
        player.suggested_training = suggestion_for_player(
            tactic.as_ref(),
            &assignments,
            player.uid,
            player.ca,
            player.pa,
            &attributes,
            fallback_inputs.get(&player.uid),
        )?;
        players.push(player);
    }

    Ok(SquadPlayersPage { players, total })
}

fn empty_page() -> SquadPlayersPage {
    SquadPlayersPage {
        players: Vec::new(),
        total: 0,
    }
}

fn suggestion_for_player(
    tactic: Option<&tactic::PlannerTactic>,
    assignments: &HashMap<i64, String>,
    player_uid: i64,
    ca: i64,
    pa: i64,
    attributes: &HashMap<String, Option<u8>>,
    fallback: Option<&FallbackInput>,
) -> Result<Option<String>, String> {
    let Some(tactic) = tactic else {
        return Ok(None);
    };
    if let Some(lane_id) = assignments.get(&player_uid) {
        let lane = tactic
            .lanes
            .iter()
            .find(|lane| lane.lane_id == *lane_id)
            .ok_or_else(|| format!("Unknown tactic lane `{lane_id}`"))?;
        if ca >= pa {
            return Ok(None);
        }
        return Ok(suggested_training::suggest_for_lane(attributes, lane).map(str::to_string));
    }
    if ca >= pa {
        return Ok(None);
    }
    let Some(input) = fallback else {
        return Ok(None);
    };
    let Some(lane) = best_fallback_lane(tactic, input) else {
        return Ok(None);
    };
    Ok(suggested_training::suggest_for_lane(attributes, lane).map(str::to_string))
}

struct FallbackInput {
    preferred_foot: String,
    positions: BTreeMap<String, Option<i64>>,
    role_scores: HashMap<String, Option<u8>>,
}

/// Highest current two-phase tactic-adjusted lane in tactic order; ties keep
/// the earlier lane via a strict `>` comparison. `None` when no lane is
/// eligible (missing scores, unfamiliar positions, or strict foot mismatch).
fn best_fallback_lane<'a>(
    tactic: &'a tactic::PlannerTactic,
    input: &FallbackInput,
) -> Option<&'a tactic::TacticLane> {
    let mut best: Option<(&'a tactic::TacticLane, u8)> = None;
    for lane in &tactic.lanes {
        let Some(score) = tactic_adjusted_score(
            input
                .role_scores
                .get(lane.ip_role_id.as_str())
                .copied()
                .flatten(),
            input
                .role_scores
                .get(lane.oop_role_id.as_str())
                .copied()
                .flatten(),
            lane.ip_weight,
            &input.preferred_foot,
            &input.positions,
            lane,
        ) else {
            continue;
        };
        if best
            .as_ref()
            .map_or(true, |(_, best_score)| score > *best_score)
        {
            best = Some((lane, score));
        }
    }
    best.map(|(lane, _)| lane)
}

fn load_fallback_inputs(
    conn: &Connection,
    snapshot_id: i64,
    tactic: Option<&tactic::PlannerTactic>,
    fallback_uids: &[i64],
) -> Result<HashMap<i64, FallbackInput>, String> {
    let Some(tactic) = tactic else {
        return Ok(HashMap::new());
    };
    if fallback_uids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut role_ids = BTreeSet::new();
    for lane in &tactic.lanes {
        role_ids.insert(lane.ip_role_id.as_str());
        role_ids.insert(lane.oop_role_id.as_str());
    }
    let role_ids = role_ids.into_iter().collect::<Vec<_>>();
    let metric_select = role_ids
        .iter()
        .map(|role_id| {
            Ok(format!(
                ", {PLAYER_METRICS_ALIAS}.{}",
                player_current_column(role_id)?
            ))
        })
        .collect::<Result<String, String>>()?;
    let placeholders = fallback_uids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT p.uid, p.preferred_foot, p.positions_json{metric_select}
         FROM players p{}
         WHERE p.snapshot_id = ?1 AND p.uid IN ({placeholders})",
        player_metrics_join("p", true, false)
    );
    let mut values = vec![Value::Integer(snapshot_id)];
    values.extend(fallback_uids.iter().map(|uid| Value::Integer(*uid)));
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let inputs = statement
        .query_map(params_from_iter(values.iter()), |row| {
            let score_values = (0..role_ids.len())
                .map(|index| row.get::<_, Option<u8>>(index + 3))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                score_values,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    inputs
        .into_iter()
        .map(|(uid, preferred_foot, positions_json, score_values)| {
            let positions = serde_json::from_str(&positions_json)
                .map_err(|error| format!("Invalid positions_json for player {uid}: {error}"))?;
            let role_scores = role_ids
                .iter()
                .zip(score_values)
                .map(|(role_id, score)| ((*role_id).to_string(), score))
                .collect();
            Ok((
                uid,
                FallbackInput {
                    preferred_foot,
                    positions,
                    role_scores,
                },
            ))
        })
        .collect()
}

fn map_player(
    row: &Row<'_>,
    dynamic_fields: &[MetricField],
) -> rusqlite::Result<(SquadPlayer, HashMap<String, Option<u8>>)> {
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
    let attributes_json: String = row.get(11)?;
    let attributes: HashMap<String, Option<u8>> =
        serde_json::from_str(&attributes_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                11,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid attributes_json: {error}"),
                )),
            )
        })?;
    let mut dynamic_values = BTreeMap::new();
    for (offset, field) in dynamic_fields.iter().enumerate() {
        dynamic_values.insert(
            field.id().to_string(),
            read_dynamic_value(row, 12 + offset, field)?,
        );
    }

    Ok((
        SquadPlayer {
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
            suggested_training: None,
            dynamic_values,
        },
        attributes,
    ))
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
