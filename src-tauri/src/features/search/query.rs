use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};

use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use crate::features::moneyball::percentile::{calculate_percentiles, MoneyballNumericStatistics};
use crate::features::moneyball::{role_catalog::builtin_catalog, role_score::score_role};
use crate::features::planner::fit::tactic_adjusted_score;
use crate::features::planner::tactic::{base_position, get_tactic};
use crate::features::player_metrics::{
    club_dna::SCORE_MODEL_VERSION,
    compact::{
        assert_read_models_complete, player_current_column, player_metrics_join,
        player_potential_column, PLAYER_METRICS_ALIAS,
    },
    resolver::{
        parse_requested_fields_for_moneyball, read_dynamic_value, ClubDnaSqlBindings, MetricField,
        TacticGroup,
    },
};

use super::filter::{
    compile_filters, compile_filters_for_moneyball, compile_filters_with_club_dna,
    current_role_ids_from_ast, moneyball_role_ids_from_ast, moneyball_role_rules_match,
    potential_role_ids_from_ast, without_moneyball_role_rules, CombineMode, CompiledFilter,
    FilterAst,
};

pub use crate::features::player_metrics::resolver::DynamicValue;

pub const DEFAULT_PAGE_LIMIT: usize = 50;
pub const MAX_PAGE_LIMIT: usize = 200;
pub const DEFAULT_SUGGEST_LIMIT: usize = 10;
pub const MAX_SUGGEST_LIMIT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchView {
    General,
    Moneyball,
    Shortlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonPool {
    FullCsv,
    Filtered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSuggestHit {
    pub uid: i64,
    pub name: String,
    pub ca: i64,
}

/// Ranked name matches for the active save's current snapshot.
/// Empty/whitespace query → empty list. Order: exact → prefix → contains, then CA desc.
pub fn suggest_players(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<PlayerSuggestHit>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

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
        return Ok(Vec::new());
    };

    let limit = i64::try_from(limit.clamp(1, MAX_SUGGEST_LIMIT))
        .map_err(|_| "suggest limit out of range".to_string())?;
    let escaped = super::filter::escape_like(query);
    let prefix_pattern = format!("{escaped}%");
    let contains_pattern = format!("%{escaped}%");

    let sql = "
        SELECT players.uid, players.name, players.ca
        FROM players
        WHERE players.snapshot_id = ?1
          AND players.name LIKE ?2 ESCAPE '\\' COLLATE NOCASE
        ORDER BY
          CASE
            WHEN players.name = ?3 COLLATE NOCASE THEN 0
            WHEN players.name LIKE ?4 ESCAPE '\\' COLLATE NOCASE THEN 1
            ELSE 2
          END ASC,
          players.ca DESC,
          players.uid ASC
        LIMIT ?5
    ";

    let mut stmt = conn.prepare(sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(
            rusqlite::params![snapshot_id, contains_pattern, query, prefix_pattern, limit],
            |row| {
                Ok(PlayerSuggestHit {
                    uid: row.get(0)?,
                    name: row.get(1)?,
                    ca: row.get(2)?,
                })
            },
        )
        .map_err(|error| error.to_string())?;

    let mut hits = Vec::new();
    for row in rows {
        hits.push(row.map_err(|error| error.to_string())?);
    }
    Ok(hits)
}

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
    /// Whitelisted player metric (`role.*`, `attr.*`, potential role, Position, …).
    Dynamic(MetricField),
}

impl SortField {
    pub const DEFAULT: Self = Self::Ca;

    pub fn parse(value: &str) -> Result<Self, String> {
        Self::parse_for_moneyball(value, false)
    }

    pub fn parse_for_moneyball(value: &str, moneyball: bool) -> Result<Self, String> {
        match value {
            "name" => Ok(Self::Name),
            "age" => Ok(Self::Age),
            "nationality" => Ok(Self::Nationality),
            "club" => Ok(Self::Club),
            "division" => Ok(Self::Division),
            "ca" => Ok(Self::Ca),
            "pa" => Ok(Self::Pa),
            "value" => Ok(Self::Value),
            other => Ok(Self::Dynamic(MetricField::parse_for_moneyball(
                other, moneyball,
            )?)),
        }
    }

    fn sql_expr(&self, club_dna_bindings: Option<ClubDnaSqlBindings>) -> String {
        match self {
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
            Self::Dynamic(field) => {
                field.sql_sort_expression_with_club_dna("players", club_dna_bindings)
            }
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

    fn moneyball_role_id(&self) -> Option<&str> {
        match self {
            Self::Dynamic(field) => field.moneyball_role_id(),
            _ => None,
        }
    }

    fn is_tactic(&self) -> bool {
        matches!(self, Self::Dynamic(field) if field.is_tactic_field())
    }

    fn is_tactic_current(&self) -> bool {
        matches!(self, Self::Dynamic(field) if field.is_tactic_current())
    }

    fn tactic_lane_id(&self) -> Option<&str> {
        match self {
            Self::Dynamic(field) => field.tactic_lane_id(),
            _ => None,
        }
    }

    fn tactic_group(&self) -> Option<TacticGroup> {
        match self {
            Self::Dynamic(field) => field.tactic_group(),
            _ => None,
        }
    }

    fn is_available_in_moneyball(&self) -> bool {
        match self {
            Self::Name
            | Self::Age
            | Self::Nationality
            | Self::Club
            | Self::Division
            | Self::Value => true,
            Self::Dynamic(field) => {
                field.is_tactic_field()
                    || crate::features::player_metrics::resolver::is_moneyball_search_field(
                        field.id(),
                    )
            }
            Self::Ca | Self::Pa => false,
        }
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

#[derive(Debug, Clone, PartialEq)]
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
    /// Values for requested non-basic metric fields (field id → nullable cell).
    pub dynamic_values: BTreeMap<String, Option<DynamicValue>>,
    pub moneyball_percentiles: BTreeMap<String, Option<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchPlayersPage {
    pub players: Vec<PlayerSummary>,
    pub total: i64,
}

pub struct SearchPlayersRequest<'a> {
    pub offset: usize,
    pub limit: usize,
    pub sort_by: SortField,
    pub sort_dir: SortDir,
    pub filter_ast: Option<&'a FilterAst>,
    pub requested_fields: &'a [String],
    pub view: SearchView,
    pub comparison_pool: ComparisonPool,
}

#[cfg(test)]
fn search_players(
    conn: &Connection,
    offset: usize,
    limit: usize,
    sort_by: SortField,
    sort_dir: SortDir,
    filter_ast: Option<&FilterAst>,
    requested_fields: &[String],
) -> Result<SearchPlayersPage, String> {
    search_players_in_view(
        conn,
        SearchPlayersRequest {
            offset,
            limit,
            sort_by,
            sort_dir,
            filter_ast,
            requested_fields,
            view: SearchView::General,
            comparison_pool: ComparisonPool::FullCsv,
        },
    )
}

pub fn search_players_in_view(
    conn: &Connection,
    request: SearchPlayersRequest<'_>,
) -> Result<SearchPlayersPage, String> {
    if request.view != SearchView::Shortlist
        && (request_uses_moneyball_role(&request)
            || (request.view == SearchView::Moneyball && request_uses_tactic(&request)))
    {
        return search_players_with_roles(conn, request);
    }

    let SearchPlayersRequest {
        offset,
        limit,
        sort_by,
        sort_dir,
        filter_ast,
        requested_fields,
        view,
        comparison_pool,
    } = request;
    if view == SearchView::Moneyball && !sort_by.is_available_in_moneyball() {
        return Err("unsupported Moneyball sort field".to_string());
    }
    if view == SearchView::Shortlist {
        let sort_str = match &sort_by {
            SortField::Name => "name",
            SortField::Age => "age",
            SortField::Nationality => "nationality",
            SortField::Club => "club",
            SortField::Division => "division",
            SortField::Ca => "ca",
            SortField::Pa => "pa",
            SortField::Value => "value",
            SortField::Dynamic(field) => field.id(),
        };
        SortField::parse(sort_str)?;
    }
    let requested_fields =
        parse_requested_fields_for_moneyball(requested_fields, view == SearchView::Moneyball)?;
    let dynamic_fields = requested_fields
        .into_iter()
        .filter(|field| !field.is_basic_table_field())
        .collect::<Vec<_>>();
    let moneyball_fields = dynamic_fields
        .iter()
        .filter_map(|field| field.moneyball_key().map(|key| (field.id(), key)))
        .collect::<Vec<_>>();
    let club_dna_filter =
        filter_ast.is_some_and(|ast| ast.rules.iter().any(|rule| rule.field == "club_dna"));
    let club_dna_requested = club_dna_filter
        || sort_by.is_club_dna()
        || dynamic_fields.iter().any(MetricField::is_club_dna);
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

    let compiled = match filter_ast {
        None => None,
        Some(ast) => {
            let compiled = match view {
                SearchView::General | SearchView::Shortlist if club_dna_filter => {
                    compile_filters_with_club_dna(ast, 4, ClubDnaSqlBindings::new(2, 3))?
                }
                SearchView::General | SearchView::Shortlist => compile_filters(ast, 2)?,
                SearchView::Moneyball => compile_filters_for_moneyball(ast, 2, true)?,
            };
            if compiled.sql.is_empty() {
                None
            } else {
                Some(compiled)
            }
        }
    };

    let current_filter_roles = filter_ast
        .map(current_role_ids_from_ast)
        .transpose()?
        .unwrap_or_default();
    let potential_filter_roles = filter_ast
        .map(potential_role_ids_from_ast)
        .transpose()?
        .unwrap_or_default();
    let current_role_sort = sort_by.current_role_id();
    let potential_role_sort = sort_by.potential_role_id();
    let mut uses_current_role_metrics = current_role_sort.is_some()
        || dynamic_fields
            .iter()
            .any(|field| field.current_role_id().is_some())
        || !current_filter_roles.is_empty();
    let mut uses_potential_role_metrics = potential_role_sort.is_some()
        || dynamic_fields
            .iter()
            .any(|field| field.potential_role_id().is_some())
        || !potential_filter_roles.is_empty();
    let uses_tactic_current =
        dynamic_fields.iter().any(|f| f.is_tactic_current()) || sort_by.is_tactic_current();
    let uses_tactic_potential = dynamic_fields
        .iter()
        .any(|f| f.is_tactic_field() && !f.is_tactic_current())
        || (sort_by.is_tactic() && !sort_by.is_tactic_current());
    let uses_tactic = uses_tactic_current || uses_tactic_potential;
    if uses_tactic {
        uses_current_role_metrics = true;
        if uses_tactic_potential {
            uses_potential_role_metrics = true;
        }
    }
    assert_read_models_complete(
        conn,
        snapshot_id,
        uses_current_role_metrics,
        uses_potential_role_metrics,
    )?;
    // Load tactic once when needed for General/Shortlist tactic scoring and validation
    let tactic_for_sql: Option<crate::features::planner::tactic::PlannerTactic> = if uses_tactic {
        let save_id = snapshot_save_id(conn, snapshot_id)?;
        let tactic = get_tactic(conn, save_id).map_err(|e| e.to_string())?;
        Some(tactic)
    } else {
        None
    };

    let club_dna_bindings = club_dna_requested.then(|| {
        if club_dna_filter {
            ClubDnaSqlBindings::new(2, 3)
        } else {
            ClubDnaSqlBindings::new(
                compiled
                    .as_ref()
                    .map_or(2, |filter| filter.params.len() + 2),
                compiled
                    .as_ref()
                    .map_or(3, |filter| filter.params.len() + 3),
            )
        }
    });
    let club_dna_definition_version = club_dna_requested
        .then(|| current_club_dna_definition_version(conn, snapshot_id))
        .transpose()?
        .flatten();
    let club_dna_sort = sort_by.club_dna_sort_identity(club_dna_definition_version);

    let mut from_sql = match view {
        SearchView::General => "FROM players".to_string(),
        SearchView::Shortlist => "FROM players INNER JOIN player_moneyball_stats shortlist ON shortlist.snapshot_id = players.snapshot_id AND shortlist.player_uid = players.uid".to_string(),
        SearchView::Moneyball => "FROM players INNER JOIN player_moneyball_stats moneyball ON moneyball.snapshot_id = players.snapshot_id AND moneyball.player_uid = players.uid AND moneyball.percentiles_json IS NOT NULL".to_string(),
    };
    if uses_current_role_metrics || uses_potential_role_metrics {
        from_sql.push_str(&player_metrics_join(
            "players",
            uses_current_role_metrics,
            uses_potential_role_metrics,
        ));
    }
    if let Some((_, bindings)) = club_dna_sort.zip(club_dna_bindings) {
        from_sql.push_str(&format!(
            " LEFT JOIN club_dna_scores club_dna_sort
              ON club_dna_sort.snapshot_id = players.snapshot_id
              AND club_dna_sort.uid = players.uid
              AND club_dna_sort.definition_version = ?{}
              AND club_dna_sort.score_model_version = ?{}",
            bindings.definition_version, bindings.score_model_version
        ));
    }
    let mut where_sql = "players.snapshot_id = ?1".to_string();
    if let Some(compiled) = &compiled {
        where_sql.push_str(" AND ");
        where_sql.push_str(&compiled.sql);
    }

    let mut bind_values = vec![Value::Integer(snapshot_id)];
    if club_dna_filter {
        bind_values.push(club_dna_definition_version.map_or(Value::Null, Value::Integer));
        bind_values.push(Value::Integer(SCORE_MODEL_VERSION));
    }
    if let Some(compiled) = &compiled {
        bind_values.extend(compiled.params.clone());
    }
    if let Some(identity) = club_dna_sort.filter(|_| !club_dna_filter) {
        bind_values.push(
            identity
                .definition_version
                .map_or(Value::Null, Value::Integer),
        );
        bind_values.push(Value::Integer(identity.score_model_version));
    }
    let filter_bind_values = bind_values.clone();
    let mut select_bind_values = bind_values.clone();
    if club_dna_requested && !club_dna_filter && club_dna_sort.is_none() {
        select_bind_values.push(club_dna_definition_version.map_or(Value::Null, Value::Integer));
        select_bind_values.push(Value::Integer(SCORE_MODEL_VERSION));
    }

    let count_sql = format!("SELECT COUNT(*) {from_sql} WHERE {where_sql}");
    let mut count_stmt = conn
        .prepare(&count_sql)
        .map_err(|error| error.to_string())?;
    let total: i64 = count_stmt
        .query_row(params_from_iter(bind_values.iter()), |row| row.get(0))
        .map_err(|error| error.to_string())?;

    // Whitelisted expr + dir only — never interpolate raw client strings.
    let order_sql = if club_dna_sort.is_some() {
        format!(
            "ORDER BY club_dna_sort.score IS NULL ASC, club_dna_sort.score {}, players.uid ASC",
            sort_dir.sql_keyword()
        )
    } else if sort_by.is_tactic() {
        let tactic = tactic_for_sql.as_ref().expect("tactic loaded for sort");
        let lane_id = sort_by.tactic_lane_id().expect("tactic lane");
        let lane = tactic
            .lanes
            .iter()
            .find(|l| l.lane_id == lane_id)
            .ok_or_else(|| format!("unknown tactic lane: {lane_id}"))?;
        let group = sort_by.tactic_group().expect("tactic group");
        let expr = tactic_sql_expression(lane, group, "players");
        format!(
            "ORDER BY ({expr} IS NULL) ASC, {expr} {}, players.uid ASC",
            sort_dir.sql_keyword()
        )
    } else {
        let sort_expression = sort_by.sql_expr(club_dna_bindings);
        format!(
            "ORDER BY {sort_expression} {}, players.uid ASC",
            sort_dir.sql_keyword()
        )
    };

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
    let limit_index = select_bind_values.len() + 1;
    let offset_index = select_bind_values.len() + 2;
    select_bind_values.push(Value::Integer(limit));
    select_bind_values.push(Value::Integer(offset));

    for field in &dynamic_fields {
        select_sql.push_str(", ");
        if field.is_tactic_field() {
            let tactic = tactic_for_sql.as_ref().expect("tactic loaded for select");
            let lane_id = field.tactic_lane_id().expect("tactic lane");
            let lane = tactic
                .lanes
                .iter()
                .find(|l| l.lane_id == lane_id)
                .ok_or_else(|| format!("unknown tactic lane: {lane_id}"))?;
            let group = field.tactic_group().expect("tactic group");
            let expr = tactic_sql_expression(lane, group, "players");
            select_sql.push_str(&expr);
        } else {
            select_sql.push_str(&field.sql_expression_with_club_dna("players", club_dna_bindings));
        }
    }
    if view == SearchView::Moneyball && comparison_pool == ComparisonPool::FullCsv {
        for (_, key) in &moneyball_fields {
            select_sql.push_str(", ");
            select_sql.push_str(&format!(
                "json_extract(moneyball.percentiles_json, '$.\"{key}\"')"
            ));
        }
    }
    select_sql.push_str(&format!(
        "
             {from_sql}
             WHERE {where_sql}
             {order_sql}
             LIMIT ?{limit_index} OFFSET ?{offset_index}"
    ));

    let mut stmt = conn
        .prepare(&select_sql)
        .map_err(|error| error.to_string())?;

    let mut players = stmt
        .query_map(params_from_iter(select_bind_values.iter()), |row| {
            map_player_summary(
                row,
                &dynamic_fields,
                &moneyball_fields,
                view == SearchView::Moneyball && comparison_pool == ComparisonPool::FullCsv,
            )
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    if view == SearchView::Moneyball && comparison_pool == ComparisonPool::Filtered {
        let mut percentiles = filtered_moneyball_percentiles(
            conn,
            &from_sql,
            &where_sql,
            &filter_bind_values,
            &moneyball_fields,
        )?;
        for player in &mut players {
            player.moneyball_percentiles = percentiles.remove(&player.uid).unwrap_or_default();
        }
    }

    Ok(SearchPlayersPage { players, total })
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

fn request_uses_tactic(request: &SearchPlayersRequest<'_>) -> bool {
    request
        .requested_fields
        .iter()
        .any(|field| field.starts_with("tactic_current.") || field.starts_with("tactic_potential."))
        || request.sort_by.is_tactic()
}

fn snapshot_save_id(conn: &Connection, snapshot_id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT save_id FROM snapshots WHERE id = ?1",
        [snapshot_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

fn moneyball_definition_for_role_and_position(
    role_id: &str,
    base_pos: &str,
) -> Option<&'static crate::features::moneyball::role_catalog::RoleDefinition> {
    let catalog = builtin_catalog().ok()?;
    catalog.definitions.iter().find(|def| {
        def.attribute_role_id.as_deref() == Some(role_id)
            && def.position_tags.contains(&base_pos.to_string())
    })
}

fn tactic_sql_expression(
    lane: &crate::features::planner::tactic::TacticLane,
    group: TacticGroup,
    player_alias: &str,
) -> String {
    let base_ip = base_position(&lane.ip_position);
    let base_oop = base_position(&lane.oop_position);
    let ip_current = player_current_column(&lane.ip_role_id).unwrap_or("goalkeeper_ip");
    let oop_current = player_current_column(&lane.oop_role_id).unwrap_or("goalkeeper_ip");
    let ip_pot = player_potential_column(&lane.ip_role_id)
        .unwrap_or_else(|_| "potential_goalkeeper_ip".to_string());
    let oop_pot = player_potential_column(&lane.oop_role_id)
        .unwrap_or_else(|_| "potential_goalkeeper_ip".to_string());
    let ip_expr = match group {
        TacticGroup::Current => format!("{PLAYER_METRICS_ALIAS}.{ip_current}"),
        TacticGroup::Potential => format!(
            "CASE WHEN {player_alias}.age >= 29 THEN {PLAYER_METRICS_ALIAS}.{ip_current} ELSE {PLAYER_METRICS_ALIAS}.{ip_pot} END"
        ),
    };
    let oop_expr = match group {
        TacticGroup::Current => format!("{PLAYER_METRICS_ALIAS}.{oop_current}"),
        TacticGroup::Potential => format!(
            "CASE WHEN {player_alias}.age >= 29 THEN {PLAYER_METRICS_ALIAS}.{oop_current} ELSE {PLAYER_METRICS_ALIAS}.{oop_pot} END"
        ),
    };
    let weight = lane.ip_weight;
    let oop_weight = 1.0 - weight;
    let blended = format!("ROUND({ip_expr} * {weight} + {oop_expr} * {oop_weight})");
    let fam_ip = format!("json_extract({player_alias}.positions_json, '$.{base_ip}')");
    let fam_oop = format!("json_extract({player_alias}.positions_json, '$.{base_oop}')");
    let foot_mismatch: String = match lane.preferred_foot.as_str() {
        "any" => "0".to_string(),
        "left" => format!("({player_alias}.preferred_foot NOT IN ('left','either'))"),
        "right" => format!("({player_alias}.preferred_foot NOT IN ('right','either'))"),
        "both" => format!("({player_alias}.preferred_foot != 'either')"),
        _ => "0".to_string(),
    };
    let strict_mismatch = if lane.preferred_foot == "any" {
        "0".to_string()
    } else {
        format!(
            "({foot_mismatch} AND '{}' = 'strict')",
            lane.foot_preference
        )
    };
    let foot_soft_penalty = format!("CASE WHEN {foot_mismatch} THEN 5 ELSE 0 END");
    let ip_fam_penalty = format!("CASE WHEN {fam_ip} < 16 THEN 5 ELSE 0 END");
    let oop_fam_penalty = format!("CASE WHEN {fam_oop} < 16 THEN 5 ELSE 0 END");
    let penalty = format!("({foot_soft_penalty} + {ip_fam_penalty} + {oop_fam_penalty})");
    format!(
        "CASE WHEN ({fam_ip} IS NULL OR {fam_ip} < 12 OR {fam_oop} IS NULL OR {fam_oop} < 12) THEN NULL WHEN ({strict_mismatch}) THEN NULL WHEN ({ip_expr} IS NULL OR {oop_expr} IS NULL) THEN NULL ELSE CASE WHEN (({blended}) - ({penalty}) < 0) THEN 0 ELSE CAST((({blended}) - ({penalty})) AS INTEGER) END END"
    )
}

fn request_uses_moneyball_role(request: &SearchPlayersRequest<'_>) -> bool {
    request
        .requested_fields
        .iter()
        .any(|field| field.starts_with("moneyball_role."))
        || request.sort_by.moneyball_role_id().is_some()
        || request.filter_ast.is_some_and(|ast| {
            ast.rules
                .iter()
                .any(|rule| rule.field.starts_with("moneyball_role."))
        })
}

struct RoleSearchCandidate {
    player: PlayerSummary,
    role_statistics: MoneyballNumericStatistics,
    role_percentiles: BTreeMap<String, Option<u8>>,
    role_scores: BTreeMap<String, Option<u8>>,
    positions: BTreeMap<String, Option<i64>>,
    player_foot: String,
    tactic_scores: BTreeMap<String, Option<u8>>,
}

fn search_players_with_roles(
    conn: &Connection,
    request: SearchPlayersRequest<'_>,
) -> Result<SearchPlayersPage, String> {
    let SearchPlayersRequest {
        offset,
        limit,
        sort_by,
        sort_dir,
        filter_ast,
        requested_fields,
        view,
        comparison_pool,
    } = request;
    if view != SearchView::Moneyball {
        return Err("Moneyball role fields require Moneyball search view".to_string());
    }
    if !sort_by.is_available_in_moneyball() {
        return Err("unsupported Moneyball sort field".to_string());
    }

    let requested_fields = parse_requested_fields_for_moneyball(requested_fields, true)?;
    let dynamic_fields = requested_fields
        .into_iter()
        .filter(|field| !field.is_basic_table_field())
        .collect::<Vec<_>>();
    let sql_dynamic_fields = dynamic_fields
        .iter()
        .filter(|field| field.moneyball_role_id().is_none() && !field.is_tactic_field())
        .cloned()
        .collect::<Vec<_>>();
    let moneyball_fields = sql_dynamic_fields
        .iter()
        .filter_map(|field| field.moneyball_key().map(|key| (field.id(), key)))
        .collect::<Vec<_>>();

    let role_filter_ids = filter_ast
        .map(moneyball_role_ids_from_ast)
        .transpose()?
        .unwrap_or_default();
    let mut role_ids = Vec::new();
    for field in &dynamic_fields {
        if let Some(role_id) = field.moneyball_role_id() {
            add_role_once(&mut role_ids, role_id);
        }
    }
    if let Some(role_id) = sort_by.moneyball_role_id() {
        add_role_once(&mut role_ids, role_id);
    }
    for role_id in role_filter_ids {
        add_role_once(&mut role_ids, &role_id);
    }

    let catalog = builtin_catalog()?;
    let definitions = role_ids
        .iter()
        .map(|role_id| {
            catalog
                .definitions
                .iter()
                .find(|definition| definition.id == *role_id)
                .ok_or_else(|| format!("unknown Moneyball role: {role_id}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut role_metric_keys: BTreeSet<String> = definitions
        .iter()
        .flat_map(|definition| definition.metrics.iter().map(|metric| metric.key.clone()))
        .collect();
    let tactic_fields_for_moneyball: Vec<MetricField> = dynamic_fields
        .iter()
        .filter(|f| f.is_tactic_field())
        .cloned()
        .collect();
    let sort_tactic_field: Option<MetricField> = match &sort_by {
        SortField::Dynamic(field) if field.is_tactic_field() => Some(field.clone()),
        _ => None,
    };
    let mut tactic_moneyball_worklist: Vec<MetricField> = tactic_fields_for_moneyball.clone();
    if let Some(sort_field) = &sort_tactic_field {
        if !tactic_moneyball_worklist
            .iter()
            .any(|field| field.id() == sort_field.id())
        {
            tactic_moneyball_worklist.push(sort_field.clone());
        }
    }
    let uses_tactic_moneyball = !tactic_moneyball_worklist.is_empty();

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
    // Load tactic for Moneyball tactic scoring if needed and extend metric keys
    let tactic_for_moneyball: Option<crate::features::planner::tactic::PlannerTactic> =
        if uses_tactic_moneyball {
            let save_id = snapshot_save_id(conn, snapshot_id)?;
            Some(get_tactic(conn, save_id).map_err(|e| e.to_string())?)
        } else {
            None
        };
    if let Some(tactic) = tactic_for_moneyball.as_ref() {
        let mut extra_keys = BTreeSet::new();
        for field in &tactic_moneyball_worklist {
            if let Some(lane_id) = field.tactic_lane_id() {
                if let Some(lane) = tactic.lanes.iter().find(|l| l.lane_id == lane_id) {
                    for (role_id, pos) in [
                        (lane.ip_role_id.as_str(), lane.ip_position.as_str()),
                        (lane.oop_role_id.as_str(), lane.oop_position.as_str()),
                    ] {
                        let base = base_position(pos);
                        if let Some(def) = moneyball_definition_for_role_and_position(role_id, base)
                        {
                            for m in &def.metrics {
                                extra_keys.insert(m.key.clone());
                            }
                        }
                    }
                }
            }
        }
        role_metric_keys.extend(extra_keys);
    }
    let role_metric_keys: Vec<String> = role_metric_keys.into_iter().collect();

    let limit = limit.clamp(1, MAX_PAGE_LIMIT);
    let offset = i64::try_from(offset).map_err(|_| "search offset out of range".to_string())?;
    let limit = i64::try_from(limit).map_err(|_| "search limit out of range".to_string())?;
    let from_sql = "FROM players INNER JOIN player_moneyball_stats moneyball ON moneyball.snapshot_id = players.snapshot_id AND moneyball.player_uid = players.uid AND moneyball.percentiles_json IS NOT NULL".to_string();

    let non_role_ast = match filter_ast {
        Some(ast) => without_moneyball_role_rules(ast)?,
        None => None,
    };
    let compiled_non_role = non_role_ast
        .as_ref()
        .map(|ast| compile_filters_for_moneyball(ast, 2, true))
        .transpose()?;
    let base_where_sql = "players.snapshot_id = ?1".to_string();
    let non_role_where_sql = where_sql_with_filter(&base_where_sql, compiled_non_role.as_ref());
    let has_role_rules = filter_ast.is_some_and(|ast| {
        ast.rules
            .iter()
            .any(|rule| rule.field.starts_with("moneyball_role."))
    });
    let mixed_or =
        has_role_rules && filter_ast.is_some_and(|ast| matches!(ast.combine, CombineMode::Or));

    let mut non_role_bind_values = vec![Value::Integer(snapshot_id)];
    if let Some(compiled) = &compiled_non_role {
        non_role_bind_values.extend(compiled.params.clone());
    }
    let (candidate_where_sql, candidate_bind_values, cohort_where_sql, cohort_bind_values) =
        if mixed_or {
            (
                base_where_sql.clone(),
                vec![Value::Integer(snapshot_id)],
                base_where_sql,
                vec![Value::Integer(snapshot_id)],
            )
        } else {
            (
                non_role_where_sql.clone(),
                non_role_bind_values.clone(),
                non_role_where_sql.clone(),
                non_role_bind_values.clone(),
            )
        };
    let non_role_uids = if mixed_or && compiled_non_role.is_some() {
        query_matching_uids(conn, &from_sql, &non_role_where_sql, &non_role_bind_values)?
    } else {
        HashSet::new()
    };

    let order_sql = if sort_by.moneyball_role_id().is_some() || sort_by.is_tactic() {
        "ORDER BY players.uid ASC".to_string()
    } else {
        format!(
            "ORDER BY {} {}, players.uid ASC",
            sort_by.sql_expr(None),
            sort_dir.sql_keyword()
        )
    };
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
    for field in &sql_dynamic_fields {
        select_sql.push_str(", ");
        select_sql.push_str(&field.sql_expression("players"));
    }
    if comparison_pool == ComparisonPool::FullCsv {
        for (_, key) in &moneyball_fields {
            select_sql.push_str(", ");
            select_sql.push_str(&format!(
                "json_extract(moneyball.percentiles_json, '$.\"{key}\"')"
            ));
        }
    }
    let need_positions = uses_tactic_moneyball;
    if need_positions {
        select_sql.push_str(", players.positions_json");
        select_sql.push_str(", players.preferred_foot");
    }
    for key in &role_metric_keys {
        let column = match comparison_pool {
            ComparisonPool::FullCsv => "percentiles_json",
            ComparisonPool::Filtered => "statistics_json",
        };
        select_sql.push_str(", ");
        select_sql.push_str(&format!("json_extract(moneyball.{column}, '$.\"{key}\"')"));
    }
    select_sql.push_str(&format!(
        " {from_sql} WHERE {candidate_where_sql} {order_sql}"
    ));

    let mut stmt = conn
        .prepare(&select_sql)
        .map_err(|error| error.to_string())?;
    let mut candidates = stmt
        .query_map(params_from_iter(candidate_bind_values.iter()), |row| {
            map_role_search_candidate(
                row,
                &sql_dynamic_fields,
                &moneyball_fields,
                comparison_pool == ComparisonPool::FullCsv,
                &role_metric_keys,
                comparison_pool,
            )
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    if comparison_pool == ComparisonPool::Filtered {
        let statistics_by_player = candidates
            .iter()
            .map(|candidate| (candidate.player.uid, candidate.role_statistics.clone()))
            .collect::<BTreeMap<_, _>>();
        let percentile_scores = calculate_percentiles(&statistics_by_player);
        for candidate in &mut candidates {
            candidate.role_percentiles = percentile_scores
                .get(&candidate.player.uid)
                .cloned()
                .unwrap_or_default();
        }
    }

    for candidate in &mut candidates {
        for (role_id, definition) in role_ids.iter().zip(&definitions) {
            let score =
                score_role(definition, &candidate.role_percentiles).map(|result| result.score);
            candidate.role_scores.insert(role_id.clone(), score);
        }
        if let Some(tactic) = tactic_for_moneyball.as_ref() {
            for field in &tactic_moneyball_worklist {
                if let Some(lane_id) = field.tactic_lane_id() {
                    if let Some(lane) = tactic.lanes.iter().find(|l| l.lane_id == lane_id) {
                        let base_ip = base_position(&lane.ip_position);
                        let base_oop = base_position(&lane.oop_position);
                        let ip_def =
                            moneyball_definition_for_role_and_position(&lane.ip_role_id, base_ip);
                        let oop_def =
                            moneyball_definition_for_role_and_position(&lane.oop_role_id, base_oop);
                        let score = if ip_def.is_none() || oop_def.is_none() {
                            None
                        } else {
                            let ip_score = ip_def.and_then(|def| {
                                score_role(def, &candidate.role_percentiles).map(|r| r.score)
                            });
                            let oop_score = oop_def.and_then(|def| {
                                score_role(def, &candidate.role_percentiles).map(|r| r.score)
                            });
                            tactic_adjusted_score(
                                ip_score,
                                oop_score,
                                lane.ip_weight,
                                &candidate.player_foot,
                                &candidate.positions,
                                lane,
                            )
                        };
                        candidate
                            .tactic_scores
                            .insert(field.id().to_string(), score);
                    }
                }
            }
        }
    }

    if let Some(ast) = filter_ast {
        if has_role_rules {
            let mut filtered_candidates = Vec::with_capacity(candidates.len());
            for candidate in candidates {
                let role_match = moneyball_role_rules_match(ast, &candidate.role_scores)?;
                let keep = if mixed_or {
                    non_role_uids.contains(&candidate.player.uid) || role_match
                } else {
                    role_match
                };
                if keep {
                    filtered_candidates.push(candidate);
                }
            }
            candidates = filtered_candidates;
        }
    }

    if let Some(role_id) = sort_by.moneyball_role_id() {
        candidates.sort_by(|left, right| {
            compare_role_scores(
                left.role_scores.get(role_id).copied().flatten(),
                right.role_scores.get(role_id).copied().flatten(),
                sort_dir,
            )
            .then_with(|| left.player.uid.cmp(&right.player.uid))
        });
    } else if sort_by.is_tactic() {
        if let Some(lane_id) = sort_by.tactic_lane_id() {
            let key = sort_by
                .tactic_group()
                .map(|g| match g {
                    TacticGroup::Current => format!("tactic_current.{lane_id}"),
                    TacticGroup::Potential => format!("tactic_potential.{lane_id}"),
                })
                .unwrap_or_else(|| format!("tactic_current.{lane_id}"));
            candidates.sort_by(|left, right| {
                let left_score = left.tactic_scores.get(&key).copied().flatten();
                let right_score = right.tactic_scores.get(&key).copied().flatten();
                compare_role_scores(left_score, right_score, sort_dir)
                    .then_with(|| left.player.uid.cmp(&right.player.uid))
            });
        }
    }

    if comparison_pool == ComparisonPool::Filtered && !moneyball_fields.is_empty() {
        let mut percentiles = filtered_moneyball_percentiles(
            conn,
            &from_sql,
            &cohort_where_sql,
            &cohort_bind_values,
            &moneyball_fields,
        )?;
        for candidate in &mut candidates {
            candidate.player.moneyball_percentiles = percentiles
                .remove(&candidate.player.uid)
                .unwrap_or_default();
        }
    }

    let total =
        i64::try_from(candidates.len()).map_err(|_| "search total out of range".to_string())?;
    let start = usize::try_from(offset).map_err(|_| "search offset out of range".to_string())?;
    let limit = usize::try_from(limit).map_err(|_| "search limit out of range".to_string())?;
    let players = candidates
        .into_iter()
        .skip(start)
        .take(limit)
        .map(|mut candidate| {
            for field in &dynamic_fields {
                if let Some(role_id) = field.moneyball_role_id() {
                    let value = candidate
                        .role_scores
                        .get(role_id)
                        .copied()
                        .flatten()
                        .map(i64::from)
                        .map(DynamicValue::Integer);
                    candidate
                        .player
                        .dynamic_values
                        .insert(field.id().to_string(), value);
                } else if field.is_tactic_field() {
                    let value = candidate
                        .tactic_scores
                        .get(field.id())
                        .copied()
                        .flatten()
                        .map(i64::from)
                        .map(DynamicValue::Integer);
                    candidate
                        .player
                        .dynamic_values
                        .insert(field.id().to_string(), value);
                }
            }
            candidate.player
        })
        .collect::<Vec<_>>();

    Ok(SearchPlayersPage { players, total })
}

fn where_sql_with_filter(base: &str, compiled: Option<&CompiledFilter>) -> String {
    compiled.map_or_else(
        || base.to_string(),
        |compiled| format!("{base} AND {}", compiled.sql),
    )
}

fn query_matching_uids(
    conn: &Connection,
    from_sql: &str,
    where_sql: &str,
    bind_values: &[Value],
) -> Result<HashSet<i64>, String> {
    let sql = format!("SELECT players.uid {from_sql} WHERE {where_sql}");
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params_from_iter(bind_values.iter()), |row| row.get(0))
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<HashSet<_>, _>>()
        .map_err(|error| error.to_string())
}

fn map_role_search_candidate(
    row: &Row<'_>,
    sql_dynamic_fields: &[MetricField],
    moneyball_fields: &[(&str, &str)],
    include_persisted_percentiles: bool,
    role_metric_keys: &[String],
    comparison_pool: ComparisonPool,
) -> rusqlite::Result<RoleSearchCandidate> {
    let player = map_player_summary(
        row,
        sql_dynamic_fields,
        moneyball_fields,
        include_persisted_percentiles,
    )?;
    let total_cols = row.as_ref().column_count();
    let expected_without_tactic = 11
        + sql_dynamic_fields.len()
        + if include_persisted_percentiles {
            moneyball_fields.len()
        } else {
            0
        }
        + role_metric_keys.len();
    let has_positions = total_cols > expected_without_tactic;
    let mut positions: BTreeMap<String, Option<i64>> = BTreeMap::new();
    let mut player_foot = "right".to_string();
    let mut offset_adjust = 0;
    if has_positions {
        let pos_idx = 11
            + sql_dynamic_fields.len()
            + if include_persisted_percentiles {
                moneyball_fields.len()
            } else {
                0
            };
        let pos_json: String = row.get(pos_idx)?;
        let foot: String = row.get(pos_idx + 1)?;
        if let Ok(map) = serde_json::from_str::<BTreeMap<String, Option<i64>>>(&pos_json) {
            positions = map;
        }
        player_foot = foot;
        offset_adjust = 2;
    }
    let role_metric_start = 11
        + sql_dynamic_fields.len()
        + if include_persisted_percentiles {
            moneyball_fields.len()
        } else {
            0
        }
        + offset_adjust;
    let mut role_statistics = BTreeMap::new();
    let mut role_percentiles = BTreeMap::new();
    for (offset, key) in role_metric_keys.iter().enumerate() {
        let index = role_metric_start + offset;
        match comparison_pool {
            ComparisonPool::FullCsv => {
                role_percentiles.insert(key.clone(), row.get(index)?);
            }
            ComparisonPool::Filtered => {
                role_statistics.insert(key.clone(), row.get(index)?);
            }
        }
    }

    Ok(RoleSearchCandidate {
        player,
        role_statistics,
        role_percentiles,
        role_scores: BTreeMap::new(),
        positions,
        player_foot,
        tactic_scores: BTreeMap::new(),
    })
}

fn compare_role_scores(left: Option<u8>, right: Option<u8>, direction: SortDir) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => match direction {
            SortDir::Asc => left.cmp(&right),
            SortDir::Desc => right.cmp(&left),
        },
    }
}

fn map_player_summary(
    row: &Row<'_>,
    dynamic_fields: &[MetricField],
    moneyball_fields: &[(&str, &str)],
    include_persisted_percentiles: bool,
) -> rusqlite::Result<PlayerSummary> {
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
        let cell = read_dynamic_value(row, idx, field)?;
        dynamic_values.insert(field.id().to_string(), cell);
    }
    let mut moneyball_percentiles = BTreeMap::new();
    if include_persisted_percentiles {
        for (offset, (field_id, _)) in moneyball_fields.iter().enumerate() {
            let index = 11 + dynamic_fields.len() + offset;
            moneyball_percentiles.insert((*field_id).to_string(), row.get(index)?);
        }
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
        moneyball_percentiles,
    })
}

fn filtered_moneyball_percentiles(
    conn: &Connection,
    from_sql: &str,
    where_sql: &str,
    bind_values: &[Value],
    moneyball_fields: &[(&str, &str)],
) -> Result<BTreeMap<i64, BTreeMap<String, Option<u8>>>, String> {
    if moneyball_fields.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut sql = String::from("SELECT players.uid");
    for (_, key) in moneyball_fields {
        sql.push_str(", ");
        sql.push_str(&format!(
            "json_extract(moneyball.statistics_json, '$.\"{key}\"')"
        ));
    }
    sql.push_str(&format!(" {from_sql} WHERE {where_sql}"));
    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(bind_values.iter()), |row| {
            let uid = row.get(0)?;
            let values = moneyball_fields
                .iter()
                .enumerate()
                .map(|(offset, (_, key))| Ok(((*key).to_string(), row.get(offset + 1)?)))
                .collect::<rusqlite::Result<MoneyballNumericStatistics>>()?;
            Ok((uid, values))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<BTreeMap<_, _>, _>>()
        .map_err(|error| error.to_string())?;
    let scores = calculate_percentiles(&rows);
    Ok(scores
        .into_iter()
        .map(|(uid, scores)| {
            (
                uid,
                moneyball_fields
                    .iter()
                    .map(|(field_id, key)| ((*field_id).to_string(), scores[*key]))
                    .collect(),
            )
        })
        .collect())
}

fn add_role_once(role_ids: &mut Vec<String>, role_id: &str) {
    if !role_ids.iter().any(|existing| existing == role_id) {
        role_ids.push(role_id.to_string());
    }
}

fn parse_nationalities(json: &str) -> Result<Vec<String>, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid nationalities_json: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::moneyball::role_catalog::builtin_catalog;
    use crate::features::player_metrics::{
        club_dna::SCORE_MODEL_VERSION, potential_scores::PROJECTION_MODEL_VERSION,
    };
    use crate::features::search::filter::{parse_filter_ast, FilterRule, FilterValue};
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service::{create_save, set_active_save};
    use serde_json::{json, Map, Value};
    use std::path::Path;

    fn search_without_filters(
        conn: &Connection,
        offset: usize,
        limit: usize,
        sort_by: SortField,
        sort_dir: SortDir,
    ) -> Result<SearchPlayersPage, String> {
        search_players_in_view(
            conn,
            SearchPlayersRequest {
                offset,
                limit,
                sort_by,
                sort_dir,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
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
        search_players_in_view(
            conn,
            SearchPlayersRequest {
                offset,
                limit,
                sort_by,
                sort_dir,
                filter_ast: Some(&ast),
                requested_fields: &[],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
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

    fn ingest_players(conn: &mut rusqlite::Connection, players: Vec<Value>) {
        ingest_players_for_game_date(conn, players, "2026-08-14");
    }

    fn ingest_players_for_game_date(
        conn: &mut rusqlite::Connection,
        players: Vec<Value>,
        game_date: &str,
    ) {
        let mut root: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v8.json"))
                .expect("parse golden fixture");
        let mut players = players;
        for player in &mut players {
            complete_position_map(player);
        }
        root["players"] = Value::Array(players);
        root["playerCount"] = json!(root["players"].as_array().unwrap().len());
        root["gameDate"] = json!(game_date);

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let dump_path = temp_dir.path().join("search-dump.json");
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

    fn current_snapshot_id(conn: &Connection) -> i64 {
        conn.query_row("SELECT id FROM snapshots WHERE is_current = 1", [], |row| {
            row.get(0)
        })
        .expect("current snapshot")
    }

    type PotentialState = (
        Vec<(i64, Option<String>, Option<i64>)>,
        Vec<(
            i64,
            Option<crate::features::player_metrics::compact::test_support::CompactRowShape>,
        )>,
    );

    fn potential_state(conn: &Connection) -> PotentialState {
        let snapshot_id = current_snapshot_id(conn);
        let projected = conn
            .prepare(
                "SELECT uid, potential_attributes_json, potential_projection_model_version
                 FROM players WHERE snapshot_id = ?1 ORDER BY uid",
            )
            .expect("prepare projected state")
            .query_map([snapshot_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("query projected state")
            .collect::<Result<_, _>>()
            .expect("read projected state");
        let player_uids = conn
            .prepare("SELECT uid FROM players WHERE snapshot_id = ?1 ORDER BY uid")
            .expect("prepare current player uids")
            .query_map([snapshot_id], |row| row.get::<_, i64>(0))
            .expect("query current player uids")
            .collect::<Result<Vec<_>, _>>()
            .expect("read current player uids");
        let compact_rows = player_uids
            .into_iter()
            .map(|uid| {
                (
                    uid,
                    crate::features::player_metrics::compact::test_support::read_row(
                        conn,
                        snapshot_id,
                        uid,
                    ),
                )
            })
            .collect();
        (projected, compact_rows)
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
            rusqlite::params![score, snapshot_id, uid],
        )
        .expect("update potential role score");
    }

    fn insert_moneyball_row(
        conn: &Connection,
        snapshot_id: i64,
        player_uid: i64,
        percentiles_json: Option<&str>,
    ) {
        insert_moneyball_statistics(
            conn,
            snapshot_id,
            player_uid,
            r#"{"goals":1.5}"#,
            percentiles_json,
        );
    }

    fn insert_moneyball_statistics(
        conn: &Connection,
        snapshot_id: i64,
        player_uid: i64,
        statistics_json: &str,
        percentiles_json: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO player_moneyball_stats (
                snapshot_id, player_uid, statistics_json, percentiles_json
             ) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![snapshot_id, player_uid, statistics_json, percentiles_json],
        )
        .expect("insert Moneyball row");
    }

    fn club_dna_score_rows(conn: &Connection) -> Vec<(i64, i64, i64, i64, Option<i64>)> {
        let mut statement = conn
            .prepare(
                "SELECT snapshot_id, uid, definition_version, score_model_version, score
                 FROM club_dna_scores
                 ORDER BY snapshot_id, uid, definition_version, score_model_version",
            )
            .expect("prepare Club DNA score rows");
        statement
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .expect("read Club DNA score rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect Club DNA score rows")
    }

    fn seed_club_dna_query_players() -> (tempfile::TempDir, Connection) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("club-dna-query.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(60, "Stale model", 100),
                player_template(10, "Present low", 100),
                player_template(50, "Stale definition", 100),
                player_template(30, "Computed null", 100),
                player_template(20, "Present high", 100),
                player_template(40, "Missing", 100),
                player_template(15, "Present low tie", 100),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        let save_id: i64 = conn
            .query_row(
                "SELECT save_id FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("snapshot save");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json, definition_version)
             VALUES (?1, '[\"attr.Acceleration\"]', 2)",
            [save_id],
        )
        .expect("insert Club DNA definition");
        for (uid, definition_version, score_model_version, score) in [
            (10, 2, SCORE_MODEL_VERSION, Some(20)),
            (15, 2, SCORE_MODEL_VERSION, Some(20)),
            (20, 2, SCORE_MODEL_VERSION, Some(80)),
            (30, 2, SCORE_MODEL_VERSION, None),
            (50, 1, SCORE_MODEL_VERSION, Some(95)),
            (60, 2, SCORE_MODEL_VERSION + 1, Some(90)),
        ] {
            conn.execute(
                "INSERT INTO club_dna_scores (
                    snapshot_id, uid, definition_version, score_model_version, score
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    snapshot_id,
                    uid,
                    definition_version,
                    score_model_version,
                    score
                ],
            )
            .expect("insert Club DNA score");
        }
        (temp_dir, conn)
    }

    fn insert_role_row(
        conn: &Connection,
        snapshot_id: i64,
        player_uid: i64,
        role_id: &str,
        raw_value: Option<f64>,
        percentile: Option<u8>,
        null_metric: Option<usize>,
    ) {
        let definition = builtin_catalog()
            .expect("built-in catalog")
            .definitions
            .iter()
            .find(|definition| definition.id == role_id)
            .expect("role definition");
        let mut statistics = Map::new();
        let mut percentiles = Map::new();
        for (index, metric) in definition.metrics.iter().enumerate() {
            statistics.insert(
                metric.key.clone(),
                if null_metric == Some(index) {
                    Value::Null
                } else {
                    json!(raw_value)
                },
            );
            percentiles.insert(
                metric.key.clone(),
                if null_metric == Some(index) {
                    Value::Null
                } else {
                    json!(percentile)
                },
            );
        }
        insert_moneyball_statistics(
            conn,
            snapshot_id,
            player_uid,
            &Value::Object(statistics).to_string(),
            Some(&Value::Object(percentiles).to_string()),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn search_moneyball(
        conn: &Connection,
        offset: usize,
        limit: usize,
        sort_by: SortField,
        sort_dir: SortDir,
        requested_fields: &[String],
        rules: Vec<FilterRule>,
        combine: Option<&str>,
        comparison_pool: ComparisonPool,
    ) -> Result<SearchPlayersPage, String> {
        let ast = if rules.is_empty() {
            None
        } else {
            Some(parse_filter_ast(rules, combine)?)
        };
        search_players_in_view(
            conn,
            SearchPlayersRequest {
                offset,
                limit,
                sort_by,
                sort_dir,
                filter_ast: ast.as_ref(),
                requested_fields,
                view: SearchView::Moneyball,
                comparison_pool,
            },
        )
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
    fn moneyball_view_returns_only_current_scored_players() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-search.db"));
        ingest_players_for_game_date(
            &mut conn,
            vec![player_template(4, "Older scored", 140)],
            "2026-08-14",
        );
        let older_snapshot_id = current_snapshot_id(&conn);
        insert_moneyball_row(&conn, older_snapshot_id, 4, Some(r#"{"goals":50}"#));

        ingest_players_for_game_date(
            &mut conn,
            vec![
                player_template(1, "Current scored", 160),
                player_template(2, "Current unscored", 150),
                player_template(3, "Current absent", 145),
            ],
            "2027-08-14",
        );
        let current_snapshot_id = current_snapshot_id(&conn);
        insert_moneyball_row(&conn, current_snapshot_id, 1, Some(r#"{"goals":50}"#));
        insert_moneyball_row(&conn, current_snapshot_id, 2, None);

        let requested_fields = ["moneyball.goals".to_string()];
        let page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::parse_for_moneyball("moneyball.goals", true).expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("search Moneyball players");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].uid, 1);
        assert_eq!(
            page.players[0].dynamic_values.get("moneyball.goals"),
            Some(&Some(DynamicValue::Real(1.5)))
        );
    }

    #[test]
    fn moneyball_full_csv_pool_returns_persisted_percentiles_only_for_statistics() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-full-csv.db"));
        ingest_players(&mut conn, vec![player_template(1, "Scored", 160)]);
        let snapshot_id = current_snapshot_id(&conn);
        insert_moneyball_statistics(
            &conn,
            snapshot_id,
            1,
            r#"{"goals":12}"#,
            Some(r#"{"goals":83}"#),
        );

        let requested_fields = [
            "moneyball.goals".to_string(),
            "moneyball.minutes".to_string(),
        ];
        let page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::parse_for_moneyball("moneyball.goals", true).expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("search Moneyball players");

        assert_eq!(
            page.players[0].dynamic_values.get("moneyball.goals"),
            Some(&Some(DynamicValue::Real(12.0)))
        );
        assert_eq!(
            page.players[0].moneyball_percentiles.get("moneyball.goals"),
            Some(&Some(83))
        );
        assert!(!page.players[0]
            .moneyball_percentiles
            .contains_key("moneyball.minutes"));
    }

    #[test]
    fn moneyball_filtered_pool_scores_the_full_filtered_cohort_across_pages() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-filtered.db"));
        ingest_players(
            &mut conn,
            (1001..=1101)
                .map(|uid| player_template(uid, &format!("Player {uid:03}"), 150))
                .collect(),
        );
        let snapshot_id = current_snapshot_id(&conn);
        for uid in 1001..=1101 {
            let statistics_json = json!({ "goals": uid - 1001 }).to_string();
            insert_moneyball_statistics(
                &conn,
                snapshot_id,
                uid,
                &statistics_json,
                Some(r#"{"goals":50}"#),
            );
        }

        let requested_fields = ["moneyball.goals".to_string()];
        let sort_by = SortField::parse_for_moneyball("moneyball.goals", true).expect("sort");
        let filter_ast = parse_filter_ast(
            vec![FilterRule {
                field: "moneyball.goals".to_string(),
                op: "gt".to_string(),
                value: FilterValue::Integer(0),
            }],
            None,
        )
        .expect("Moneyball filter");
        let first_page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 50,
                sort_by: sort_by.clone(),
                sort_dir: SortDir::Asc,
                filter_ast: Some(&filter_ast),
                requested_fields: &requested_fields,
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::Filtered,
            },
        )
        .expect("first page");
        let second_page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 50,
                limit: 50,
                sort_by,
                sort_dir: SortDir::Asc,
                filter_ast: Some(&filter_ast),
                requested_fields: &requested_fields,
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::Filtered,
            },
        )
        .expect("second page");

        assert_eq!(first_page.total, 100);
        assert_eq!(first_page.players[0].uid, 1002);
        assert_eq!(
            first_page.players[0]
                .moneyball_percentiles
                .get("moneyball.goals"),
            Some(&Some(0))
        );
        assert_eq!(second_page.players[0].uid, 1052);
        assert_eq!(
            second_page.players[0]
                .moneyball_percentiles
                .get("moneyball.goals"),
            Some(&Some(51))
        );
    }

    #[test]
    fn moneyball_view_rejects_ability_sort_fields() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-sort.db"));
        ingest_players(&mut conn, vec![player_template(1, "Scored", 160)]);
        let snapshot_id = current_snapshot_id(&conn);
        insert_moneyball_row(&conn, snapshot_id, 1, Some(r#"{"goals":50}"#));

        let error = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::Filtered,
            },
        )
        .expect_err("CA must not sort Moneyball Search");

        assert_eq!(error, "unsupported Moneyball sort field");
    }

    #[test]
    fn shortlist_returns_only_current_moneyball_members_with_general_metrics_and_includes_null_percentiles(
    ) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("shortlist-members.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "One", 140),
                player_template(2, "Two", 150),
                player_template(3, "Three", 160),
                player_template(4, "Four", 130),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        // Set role scores for filtering
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", Some(80));
        set_role_score(&conn, 2, "deep_lying_playmaker_ip", Some(40));
        set_role_score(&conn, 3, "deep_lying_playmaker_ip", Some(90));
        set_role_score(&conn, 4, "deep_lying_playmaker_ip", Some(80));
        insert_moneyball_row(&conn, snapshot_id, 1, Some(r#"{"goals":10}"#));
        insert_moneyball_row(&conn, snapshot_id, 3, Some(r#"{"goals":20}"#));
        insert_moneyball_row(&conn, snapshot_id, 2, None);

        let requested_fields = vec![
            "role.deep_lying_playmaker_ip".to_string(),
            "attr.Acceleration".to_string(),
            "ca".to_string(),
        ];
        let page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("search Shortlist");
        assert_eq!(page.total, 3);
        assert_eq!(
            page.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        // General-only requested_fields round-trip (ca is basic, not in dynamic_values)
        assert_eq!(
            page.players[0]
                .dynamic_values
                .get("role.deep_lying_playmaker_ip"),
            Some(&Some(DynamicValue::Integer(90)))
        );
        assert!(page.players[0]
            .dynamic_values
            .contains_key("attr.Acceleration"));
        assert!(!page.players[0].dynamic_values.contains_key("ca"));

        // General returns all 4
        let general_page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("search General");
        assert_eq!(general_page.total, 4);

        // Filter composition via General resolver
        let ast = parse_filter_ast(
            vec![FilterRule {
                field: "role.deep_lying_playmaker_ip".to_string(),
                op: "gt".to_string(),
                value: FilterValue::Integer(50),
            }],
            None,
        )
        .expect("parse filter");
        let filtered = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: Some(&ast),
                requested_fields: &[],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("filtered Shortlist");
        assert_eq!(filtered.total, 2);
        assert_eq!(
            filtered.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![3, 1]
        );

        // Moneyball with percentiles IS NOT NULL would exclude UID 2
        let moneyball_page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::parse_for_moneyball("moneyball.goals", true).expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["moneyball.goals".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("search Moneyball");
        assert_eq!(moneyball_page.total, 2);
        assert!(moneyball_page.players.iter().all(|p| p.uid != 2));
    }

    #[test]
    fn shortlist_respects_snapshot_isolation_and_empty_cohort() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("shortlist-isolation.db"));
        ingest_players(&mut conn, vec![player_template(1, "Active", 150)]);
        let first_save_id: i64 = conn
            .query_row("SELECT id FROM saves WHERE is_active = 1", [], |row| {
                row.get(0)
            })
            .expect("first save id");
        let active_snapshot: i64 = conn
            .query_row(
                "SELECT s.id FROM snapshots s INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1 WHERE s.is_current = 1",
                [],
                |row| row.get(0),
            )
            .expect("first active snapshot");
        insert_moneyball_row(&conn, active_snapshot, 1, Some(r#"{"goals":10}"#));

        let second_save = create_save(&conn, "Second save").expect("create save");
        set_active_save(&mut conn, second_save.id).expect("switch save");
        ingest_players(&mut conn, vec![player_template(2, "Other save", 160)]);
        let other_snapshot: i64 = conn
            .query_row(
                "SELECT s.id FROM snapshots s INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1 WHERE s.is_current = 1",
                [],
                |row| row.get(0),
            )
            .expect("second active snapshot");
        insert_moneyball_row(&conn, other_snapshot, 2, Some(r#"{"goals":20}"#));

        let page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("shortlist other save");
        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].uid, 2);

        // Switch back to first save, ensure isolation
        set_active_save(&mut conn, first_save_id).expect("switch back");
        let page_back = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("shortlist first save");
        assert_eq!(page_back.total, 1);
        assert_eq!(page_back.players[0].uid, 1);

        // Empty cohort returns total 0
        let temp_empty = tempfile::tempdir().expect("temp dir");
        let mut empty_conn = open_migrated(&temp_empty.path().join("shortlist-empty.db"));
        ingest_players(&mut empty_conn, vec![player_template(1, "No cohort", 150)]);
        let empty_page = search_players_in_view(
            &empty_conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("empty shortlist");
        assert_eq!(empty_page.total, 0);
        assert!(empty_page.players.is_empty());
    }

    #[test]
    fn shortlist_comparison_pool_is_independent_on_populated_cohort_with_nullable_member() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("shortlist-pool.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "One", 140),
                player_template(2, "Two", 150),
                player_template(3, "Three", 160),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        insert_moneyball_row(&conn, snapshot_id, 1, Some(r#"{"goals":10}"#));
        insert_moneyball_row(&conn, snapshot_id, 2, None);
        insert_moneyball_row(&conn, snapshot_id, 3, Some(r#"{"goals":20}"#));
        let requested_fields = vec!["role.deep_lying_playmaker_ip".to_string()];
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", Some(80));
        set_role_score(&conn, 2, "deep_lying_playmaker_ip", Some(40));
        set_role_score(&conn, 3, "deep_lying_playmaker_ip", Some(90));

        let full = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("shortlist FullCsv");
        let filtered = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::Filtered,
            },
        )
        .expect("shortlist Filtered");

        assert_eq!(full.total, 3);
        assert_eq!(filtered.total, 3);
        assert_eq!(
            full.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            filtered.players.iter().map(|p| p.uid).collect::<Vec<_>>()
        );
        assert_eq!(
            full.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        for (left, right) in full.players.iter().zip(filtered.players.iter()) {
            assert_eq!(left.dynamic_values, right.dynamic_values);
        }
        assert_eq!(
            full.players[0]
                .dynamic_values
                .get("role.deep_lying_playmaker_ip"),
            Some(&Some(DynamicValue::Integer(90)))
        );
    }

    #[test]
    fn shortlist_rejects_moneyball_only_typed_inputs_with_general_validation_error() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("shortlist-rejects.db"));
        ingest_players(&mut conn, vec![player_template(1, "One", 150)]);
        let snapshot_id = current_snapshot_id(&conn);
        insert_moneyball_row(&conn, snapshot_id, 1, Some(r#"{"goals":10}"#));

        let expected_moneyball_err =
            crate::features::player_metrics::resolver::MetricField::parse_for_moneyball(
                "moneyball.goals",
                false,
            )
            .unwrap_err();
        let expected_role_err =
            crate::features::player_metrics::resolver::MetricField::parse_for_moneyball(
                "moneyball_role.wbl_wbr_wing_back_ip",
                false,
            )
            .unwrap_err();

        for (field, expected) in [
            ("moneyball.goals", expected_moneyball_err.clone()),
            (
                "moneyball_role.wbl_wbr_wing_back_ip",
                expected_role_err.clone(),
            ),
        ] {
            let err = search_players_in_view(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: DEFAULT_PAGE_LIMIT,
                    sort_by: SortField::Ca,
                    sort_dir: SortDir::Desc,
                    filter_ast: None,
                    requested_fields: &[field.to_string()],
                    view: SearchView::Shortlist,
                    comparison_pool: ComparisonPool::FullCsv,
                },
            )
            .expect_err("should reject moneyball requested field");
            assert_eq!(err, expected);
            let general_err =
                crate::features::player_metrics::resolver::parse_requested_fields_for_moneyball(
                    &[field.to_string()],
                    false,
                )
                .unwrap_err();
            assert_eq!(err, general_err);
        }
        for field in ["moneyball.goals", "moneyball_role.wbl_wbr_wing_back_ip"] {
            let ast = parse_filter_ast(
                vec![FilterRule {
                    field: field.to_string(),
                    op: "gt".to_string(),
                    value: FilterValue::Integer(10),
                }],
                None,
            )
            .expect("parse ast");
            let err = search_players_in_view(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: DEFAULT_PAGE_LIMIT,
                    sort_by: SortField::Ca,
                    sort_dir: SortDir::Desc,
                    filter_ast: Some(&ast),
                    requested_fields: &[],
                    view: SearchView::Shortlist,
                    comparison_pool: ComparisonPool::FullCsv,
                },
            )
            .expect_err("should reject moneyball filter");
            let expected = crate::features::search::filter::compile_filters(&ast, 2).unwrap_err();
            assert_eq!(err, expected);
        }
        let moneyball_sort =
            SortField::parse_for_moneyball("moneyball.goals", true).expect("moneyball sort");
        let err = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: moneyball_sort,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect_err("should reject moneyball sort");
        assert_eq!(err, expected_moneyball_err);

        let role_sort = SortField::parse_for_moneyball("moneyball_role.wbl_wbr_wing_back_ip", true)
            .expect("role sort");
        let err = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: role_sort,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect_err("should reject moneyball role sort");
        assert_eq!(err, expected_role_err);

        let mut oversized = vec!["ca".to_string(); 256];
        oversized.push("moneyball_role.wbl_wbr_wing_back_ip".to_string());
        let err = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &oversized,
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect_err("oversized Shortlist with Moneyball role must hit cap");
        let expected_cap =
            crate::features::player_metrics::resolver::parse_requested_fields_for_moneyball(
                &oversized, false,
            )
            .unwrap_err();
        assert_eq!(err, expected_cap);
        assert_eq!(
            err,
            "requested field count exceeds maximum of 256".to_string()
        );

        let ordered = vec![
            "unknown.metric".to_string(),
            "moneyball_role.wbl_wbr_wing_back_ip".to_string(),
        ];
        let err = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &ordered,
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect_err("earlier unknown must win");
        let expected_ordered =
            crate::features::player_metrics::resolver::parse_requested_fields_for_moneyball(
                &ordered, false,
            )
            .unwrap_err();
        assert_eq!(err, expected_ordered);
        assert_eq!(err, "unknown player metric: unknown.metric".to_string());
    }

    #[test]
    fn excludes_players_retained_only_in_an_earlier_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("history-search.db"));
        ingest_players_for_game_date(
            &mut conn,
            vec![player_template(1, "Current player", 150)],
            "2027-08-16",
        );
        ingest_players_for_game_date(
            &mut conn,
            vec![player_template(2, "Removed player", 160)],
            "2026-08-14",
        );

        let page = search_without_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
        )
        .expect("search latest snapshot");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Current player");
        assert!(suggest_players(&conn, "Removed", DEFAULT_SUGGEST_LIMIT)
            .expect("suggest latest snapshot")
            .is_empty());
    }

    #[test]
    fn searches_club_dna_ascending_with_nullable_exact_identity_and_read_only_pages() {
        let (_temp_dir, conn) = seed_club_dna_query_players();
        conn.pragma_update(None, "reverse_unordered_selects", true)
            .expect("reverse unordered ties");
        let score_rows_before = club_dna_score_rows(&conn);
        let score_row_count_before = score_rows_before.len();
        let requested_fields = vec!["club_dna".to_string()];

        let page = search_players(
            &conn,
            0,
            7,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Asc,
            None,
            &requested_fields,
        )
        .expect("search Club DNA ascending");
        assert_eq!(page.total, 7);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [10, 15, 20, 30, 40, 50, 60]
        );
        assert_eq!(
            page.players[0].dynamic_values.get("club_dna"),
            Some(&Some(DynamicValue::Integer(20)))
        );
        assert_eq!(page.players[3].dynamic_values.get("club_dna"), Some(&None));
        assert_eq!(page.players[4].dynamic_values.get("club_dna"), Some(&None));
        assert_eq!(page.players[5].dynamic_values.get("club_dna"), Some(&None));
        assert_eq!(page.players[6].dynamic_values.get("club_dna"), Some(&None));

        let bounded_page = search_players(
            &conn,
            1,
            2,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Asc,
            None,
            &requested_fields,
        )
        .expect("page Club DNA ascending");
        assert_eq!(bounded_page.total, 7);
        assert_eq!(
            bounded_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [15, 20]
        );
        let unavailable_page = search_players(
            &conn,
            3,
            4,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Asc,
            None,
            &requested_fields,
        )
        .expect("page unavailable Club DNA ties");
        assert_eq!(
            unavailable_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [30, 40, 50, 60]
        );
        assert_eq!(club_dna_score_rows(&conn).len(), score_row_count_before);
        assert_eq!(club_dna_score_rows(&conn), score_rows_before);
    }

    #[test]
    fn searches_club_dna_descending_filters_every_operator_and_keeps_scores_read_only() {
        let (_temp_dir, conn) = seed_club_dna_query_players();
        conn.pragma_update(None, "reverse_unordered_selects", true)
            .expect("reverse unordered ties");
        let score_rows_before = club_dna_score_rows(&conn);
        let expected = [
            ("gt", vec![20]),
            ("lt", vec![10, 15]),
            ("eq", vec![10, 15]),
            ("neq", vec![20]),
        ];
        for (operator, expected_uids) in expected {
            let threshold = if operator == "lt" { 30 } else { 20 };
            let ast = parse_filter_ast(
                vec![filter_rule(
                    "club_dna",
                    operator,
                    FilterValue::Integer(threshold),
                )],
                None,
            )
            .expect("parse Club DNA filter");
            let page = search_players(
                &conn,
                0,
                6,
                SortField::parse("club_dna").expect("parse Club DNA sort"),
                SortDir::Desc,
                Some(&ast),
                &[],
            )
            .expect("filter Club DNA");
            assert_eq!(
                page.players
                    .iter()
                    .map(|player| player.uid)
                    .collect::<Vec<_>>(),
                expected_uids,
                "{operator} must ignore null, missing, and stale scores"
            );
        }

        let and_ast = parse_filter_ast(
            vec![
                filter_rule("club_dna", "gt", FilterValue::Integer(20)),
                filter_rule("name", "contains", FilterValue::Text("Present".to_string())),
            ],
            Some("and"),
        )
        .expect("parse Club DNA AND filter");
        let and_page = search_players(
            &conn,
            0,
            6,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Desc,
            Some(&and_ast),
            &[],
        )
        .expect("filter Club DNA with AND");
        assert_eq!(
            and_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [20]
        );

        let or_ast = parse_filter_ast(
            vec![
                filter_rule("club_dna", "eq", FilterValue::Integer(20)),
                filter_rule("name", "contains", FilterValue::Text("Missing".to_string())),
            ],
            Some("or"),
        )
        .expect("parse Club DNA OR filter");
        let or_page = search_players(
            &conn,
            0,
            6,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Desc,
            Some(&or_ast),
            &[],
        )
        .expect("filter Club DNA with OR");
        assert_eq!(
            or_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [10, 15, 40]
        );

        let descending = search_players(
            &conn,
            0,
            7,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Desc,
            None,
            &[],
        )
        .expect("search Club DNA descending");
        assert_eq!(
            descending
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [20, 10, 15, 30, 40, 50, 60]
        );
        let bounded_page = search_players(
            &conn,
            1,
            2,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Desc,
            None,
            &[],
        )
        .expect("page Club DNA descending ties");
        assert_eq!(
            bounded_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [10, 15]
        );
        let unavailable_page = search_players(
            &conn,
            3,
            4,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Desc,
            None,
            &[],
        )
        .expect("page unavailable Club DNA ties");
        assert_eq!(
            unavailable_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [30, 40, 50, 60]
        );
        assert_eq!(club_dna_score_rows(&conn), score_rows_before);
    }

    #[test]
    fn searches_missing_club_dna_definition_as_a_uid_stable_all_null_page() {
        let (_temp_dir, conn) = seed_club_dna_query_players();
        conn.pragma_update(None, "reverse_unordered_selects", true)
            .expect("reverse unordered ties");
        let score_rows_before = club_dna_score_rows(&conn);
        conn.execute("DELETE FROM club_dna_definitions", [])
            .expect("remove Club DNA definition");

        let page = search_players(
            &conn,
            0,
            7,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Desc,
            None,
            &["club_dna".to_string()],
        )
        .expect("search without Club DNA definition");
        assert_eq!(page.total, 7);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [10, 15, 20, 30, 40, 50, 60]
        );
        assert!(page
            .players
            .iter()
            .all(|player| player.dynamic_values.get("club_dna") == Some(&None)));
        assert_eq!(club_dna_score_rows(&conn), score_rows_before);
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
    fn accepts_position_as_a_sortable_display_metric() {
        assert!(SortField::parse("position").is_ok());
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
    fn orders_targeted_scalar_sorts_with_nulls_ties_totals_and_pages() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("targeted-scalar-sorts.db"));
        ingest_players(
            &mut conn,
            [10, 20, 30, 40, 50]
                .into_iter()
                .map(|uid| player_template(uid, &format!("Player {uid}"), 100))
                .collect(),
        );
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, pa, age, market_value_gbp) in [
            (10, 100, Some(20), Some(100)),
            (20, 100, Some(20), Some(100)),
            (30, 150, Some(22), Some(300)),
            (40, 150, None, None),
            (50, 200, Some(22), None),
        ] {
            conn.execute(
                "UPDATE players
                 SET pa = ?1, age = ?2, market_value_gbp = ?3
                 WHERE snapshot_id = ?4 AND uid = ?5",
                rusqlite::params![pa, age, market_value_gbp, snapshot_id, uid],
            )
            .expect("set targeted scalar values");
        }

        for (field, direction, expected, expected_page) in [
            (
                SortField::Pa,
                SortDir::Asc,
                vec![10, 20, 30, 40, 50],
                vec![20, 30],
            ),
            (
                SortField::Pa,
                SortDir::Desc,
                vec![50, 30, 40, 10, 20],
                vec![30, 40],
            ),
            (
                SortField::Age,
                SortDir::Asc,
                vec![40, 10, 20, 30, 50],
                vec![10, 20],
            ),
            (
                SortField::Age,
                SortDir::Desc,
                vec![30, 50, 10, 20, 40],
                vec![50, 10],
            ),
            (
                SortField::Value,
                SortDir::Asc,
                vec![40, 50, 10, 20, 30],
                vec![50, 10],
            ),
            (
                SortField::Value,
                SortDir::Desc,
                vec![30, 10, 20, 40, 50],
                vec![10, 20],
            ),
        ] {
            let page = search_without_filters(&conn, 0, 5, field.clone(), direction)
                .expect("sort targeted scalar values");
            assert_eq!(page.total, 5);
            assert_eq!(
                page.players
                    .iter()
                    .map(|player| player.uid)
                    .collect::<Vec<_>>(),
                expected
            );

            let bounded_page = search_without_filters(&conn, 1, 2, field, direction)
                .expect("page targeted scalar values");
            assert_eq!(bounded_page.total, 5);
            assert_eq!(
                bounded_page
                    .players
                    .iter()
                    .map(|player| player.uid)
                    .collect::<Vec<_>>(),
                expected_page
            );
        }
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
                player_with_deep_fields(
                    4,
                    "Zero MC",
                    130,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 0, "ST": 18 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({ "Consistency": 10 }),
                        personality: json!({ "Ambition": 10 }),
                    },
                ),
                player_with_deep_fields(
                    5,
                    "Null MC",
                    120,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": null, "ST": 18 }),
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

        let inverse = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "position",
                "is_not",
                FilterValue::Text("MC".to_string()),
            )],
            None,
        )
        .expect("inverse position presence");
        assert_eq!(inverse.total, 3);
        assert_eq!(
            inverse
                .players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Striker Only", "Zero MC", "Null MC"]
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
        let column = crate::features::player_metrics::compact::player_current_column(role_id)
            .expect("current role column");
        conn.execute(
            &format!(
                "UPDATE player_role_metrics
                 SET {column} = ?1
                 WHERE snapshot_id = ?2 AND uid = ?3"
            ),
            rusqlite::params![score, snapshot_id, uid],
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
    fn filters_by_new_generic_oop_role_score() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("filtered-new-oop-role.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Shot Stopper", 150),
                player_template(2, "Outfielder", 140),
            ],
        );
        set_role_score(&conn, 1, "goalkeeper_oop", Some(85));
        set_role_score(&conn, 2, "goalkeeper_oop", Some(40));

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "role.goalkeeper_oop",
                "gt",
                FilterValue::Integer(70),
            )],
            None,
        )
        .expect("new OOP role score filter");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Shot Stopper");
    }

    #[test]
    fn potential_role_filter_reads_complete_rows_without_writes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("potential-role-filter.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "Potential target",
                    100,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "GK": 20 }),
                        attributes: json!({ "Positioning": 10, "Concentration": 10 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "Unknown potential",
                    180,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "GK": 20 }),
                        attributes: json!({ "Positioning": null, "Concentration": 20 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
            ],
        );

        let before = potential_state(&conn);
        deny_potential_writes(&conn);
        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "potential_role.line_holding_keeper_oop",
                "gt",
                FilterValue::Integer(0),
            )],
            None,
        )
        .expect("potential role filter");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Potential target");
        assert_eq!(potential_state(&conn), before);
    }

    #[test]
    fn invalid_filter_rules_fail_before_potential_assertion() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("invalid-potential-filter.db"));
        ingest_players(&mut conn, vec![player_template(1, "No cache work", 150)]);

        let snapshot_id = current_snapshot_id(&conn);
        conn.execute(
            "UPDATE player_role_metrics SET projection_model_version = ?2
             WHERE snapshot_id = ?1",
            rusqlite::params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
        )
        .expect("corrupt potential state");
        let before = potential_state(&conn);
        deny_potential_writes(&conn);

        let error = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![
                filter_rule(
                    "potential_role.goalkeeper_ip",
                    "gt",
                    FilterValue::Integer(70),
                ),
                filter_rule("name", "gt", FilterValue::Text("Scout".to_string())),
            ],
            Some("and"),
        )
        .expect_err("invalid filter");
        assert!(error.contains("invalid string filter operator"));
        assert_eq!(potential_state(&conn), before);
    }

    #[test]
    fn non_potential_search_ignores_corrupt_potential_state() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("non-potential-corrupt.db"));
        ingest_players(&mut conn, vec![player_template(1, "Current scalar", 150)]);

        let snapshot_id = current_snapshot_id(&conn);
        conn.execute(
            "UPDATE player_role_metrics SET projection_model_version = ?2
             WHERE snapshot_id = ?1 AND uid = 1",
            rusqlite::params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
        )
        .expect("corrupt unused potential state");
        let before = potential_state(&conn);
        deny_potential_writes(&conn);

        let page = search_without_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
        )
        .expect("read current scalar fields");
        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Current scalar");
        assert_eq!(potential_state(&conn), before);
    }

    #[test]
    fn potential_role_filter_rejects_missing_and_wrong_version_rows_without_writes() {
        for wrong_version in [false, true] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let mut conn = open_migrated(&temp_dir.path().join("potential-role-corrupt.db"));
            ingest_players(&mut conn, vec![player_template(1, "Projected keeper", 100)]);

            let snapshot_id = current_snapshot_id(&conn);
            if wrong_version {
                conn.execute(
                    "UPDATE player_role_metrics SET projection_model_version = ?2
                     WHERE snapshot_id = ?1 AND uid = 1",
                    rusqlite::params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
                )
            } else {
                conn.execute(
                    "DELETE FROM player_role_metrics
                     WHERE snapshot_id = ?1 AND uid = 1",
                    [snapshot_id],
                )
            }
            .expect("corrupt compact row");
            let before = potential_state(&conn);
            deny_potential_writes(&conn);

            assert_eq!(
                search_with_filters(
                    &conn,
                    0,
                    DEFAULT_PAGE_LIMIT,
                    SortField::DEFAULT,
                    SortDir::DEFAULT,
                    vec![filter_rule(
                        "potential_role.line_holding_keeper_oop",
                        "gt",
                        FilterValue::Integer(0),
                    )],
                    None,
                ),
                Err("Current potential snapshot is incomplete".to_string())
            );
            assert_eq!(potential_state(&conn), before);
        }
    }

    #[test]
    fn potential_role_filter_rejects_migrated_score_v1_rows_without_writes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("potential-role-score-v1.db"));
        ingest_players(
            &mut conn,
            vec![player_with_deep_fields(
                1,
                "Migrated scorer",
                100,
                DeepPlayerFields {
                    nationalities: json!(["ENG"]),
                    positions: json!({ "GK": 20 }),
                    attributes: json!({ "Positioning": 10, "Concentration": 10 }),
                    hidden: json!({}),
                    personality: json!({}),
                },
            )],
        );

        // Normal version-2 materialization reads through the potential-only seam.
        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "potential_role.line_holding_keeper_oop",
                "gt",
                FilterValue::Integer(0),
            )],
            None,
        )
        .expect("v2 potential read");
        assert_eq!(page.total, 1);

        // A migrated/seeded score-v1 row keeps projection v2 yet must fail.
        let snapshot_id = current_snapshot_id(&conn);
        conn.execute(
            "UPDATE player_role_metrics SET score_model_version = ?2
             WHERE snapshot_id = ?1 AND uid = 1",
            rusqlite::params![
                snapshot_id,
                crate::features::player_metrics::compact::SCORE_MODEL_VERSION - 1
            ],
        )
        .expect("seed migrated score v1 row");
        let projection_version: i64 = conn
            .query_row(
                "SELECT projection_model_version FROM player_role_metrics
                 WHERE snapshot_id = ?1 AND uid = 1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("read seeded projection version");
        assert_eq!(projection_version, PROJECTION_MODEL_VERSION);
        let before = potential_state(&conn);
        deny_potential_writes(&conn);

        assert_eq!(
            search_with_filters(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::DEFAULT,
                SortDir::DEFAULT,
                vec![filter_rule(
                    "potential_role.line_holding_keeper_oop",
                    "gt",
                    FilterValue::Integer(0),
                )],
                None,
            ),
            Err("Current potential snapshot is incomplete".to_string())
        );
        assert_eq!(potential_state(&conn), before);
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

        let requested_fields = vec![
            "role.deep_lying_playmaker_ip".to_string(),
            "attr.Acceleration".to_string(),
        ];
        let filters = vec![
            filter_rule(
                "role.deep_lying_playmaker_ip",
                "gt",
                FilterValue::Integer(70),
            ),
            filter_rule("attr.Acceleration", "gt", FilterValue::Integer(12)),
        ];
        let ast = parse_filter_ast(filters, None).expect("parse filters");
        let page = search_players(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            Some(&ast),
            &requested_fields,
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
    fn returns_requested_field_families_once_without_tying_them_to_filters() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("requested-fields.db"));
        ingest_players(
            &mut conn,
            vec![player_with_deep_fields(
                1,
                "Metric Target",
                150,
                DeepPlayerFields {
                    nationalities: json!(["ENG"]),
                    positions: json!({ "MC": 16, "AMC": 20, "AMR": 14, "GK": 0, "SW": null }),
                    attributes: json!({ "Acceleration": 16 }),
                    hidden: json!({ "Consistency": 12 }),
                    personality: json!({ "Ambition": 14 }),
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
        conn.execute(
            "UPDATE players
             SET parent_club = 'Parent FC', transfer_listed = 1
             WHERE snapshot_id = ?1 AND uid = 1",
            [snapshot_id],
        )
        .expect("set scalar values");
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", Some(82));

        let requested_fields = vec![
            "parent_club".to_string(),
            "height".to_string(),
            "transfer_listed".to_string(),
            "preferred_foot".to_string(),
            "attr.Acceleration".to_string(),
            "hidden.Consistency".to_string(),
            "personality.Ambition".to_string(),
            "pos.MC".to_string(),
            "pos.AMR".to_string(),
            "pos.GK".to_string(),
            "pos.SW".to_string(),
            "position".to_string(),
            "role.deep_lying_playmaker_ip".to_string(),
            "attr.Acceleration".to_string(),
            "ca".to_string(),
        ];
        let page = search_players(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            None,
            &requested_fields,
        )
        .expect("query requested fields");

        let values = &page.players[0].dynamic_values;
        assert_eq!(values.len(), 13, "duplicates and basic fields are omitted");
        assert_eq!(
            values.get("parent_club"),
            Some(&Some(DynamicValue::Text("Parent FC".to_string())))
        );
        assert_eq!(
            values.get("height"),
            Some(&Some(DynamicValue::Integer(180)))
        );
        assert_eq!(
            values.get("transfer_listed"),
            Some(&Some(DynamicValue::Integer(1)))
        );
        assert_eq!(
            values.get("preferred_foot"),
            Some(&Some(DynamicValue::Text("right".to_string())))
        );
        assert_eq!(
            values.get("attr.Acceleration"),
            Some(&Some(DynamicValue::Integer(16)))
        );
        assert_eq!(
            values.get("hidden.Consistency"),
            Some(&Some(DynamicValue::Integer(12)))
        );
        assert_eq!(
            values.get("personality.Ambition"),
            Some(&Some(DynamicValue::Integer(14)))
        );
        assert_eq!(values.get("pos.MC"), Some(&Some(DynamicValue::Integer(16))));
        assert_eq!(
            values.get("pos.AMR"),
            Some(&Some(DynamicValue::Integer(14)))
        );
        assert_eq!(values.get("pos.GK"), Some(&Some(DynamicValue::Integer(0))));
        assert_eq!(values.get("pos.SW"), Some(&None));
        assert_eq!(
            values.get("position"),
            Some(&Some(DynamicValue::Text("AMC, MC, AMR".to_string())))
        );
        assert_eq!(
            values.get("role.deep_lying_playmaker_ip"),
            Some(&Some(DynamicValue::Integer(82)))
        );
        assert!(!values.contains_key("ca"));
    }

    #[test]
    fn displays_positions_strongest_first_and_sorts_by_the_same_text() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("position-display.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "Midfielder",
                    150,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 16, "AMC": 20, "AML": 14, "AMR": 14, "WBL": 1, "GK": 0, "SW": null, "DL": 0 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "Keeper",
                    160,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "GK": 20 }),
                        attributes: json!({ "Acceleration": 10 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
            ],
        );

        let requested_fields = vec!["position".to_string()];
        let page = search_players(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::parse("position").expect("parse Position sort"),
            SortDir::Asc,
            None,
            &requested_fields,
        )
        .expect("query Position display");

        assert_eq!(
            page.players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            ["Midfielder", "Keeper"]
        );
        assert_eq!(
            page.players[0].dynamic_values.get("position"),
            Some(&Some(DynamicValue::Text(
                "AMC, MC, AMR, AML, WBL".to_string()
            )))
        );
    }

    #[test]
    fn potential_display_reads_complete_rows_without_writes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("potential-page.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "Lower potential",
                    100,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "GK": 20 }),
                        attributes: json!({ "Positioning": 8, "Concentration": 8 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "Higher potential",
                    180,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "GK": 20 }),
                        attributes: json!({ "Positioning": 16, "Concentration": 16 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
            ],
        );
        let requested_fields = vec!["potential_role.line_holding_keeper_oop".to_string()];
        let before = potential_state(&conn);
        deny_potential_writes(&conn);

        let page = search_players(
            &conn,
            0,
            1,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            None,
            &requested_fields,
        )
        .expect("query first potential page");
        assert_eq!(page.players[0].name, "Higher potential");
        assert!(matches!(
            page.players[0]
                .dynamic_values
                .get("potential_role.line_holding_keeper_oop"),
            Some(Some(DynamicValue::Integer(_)))
        ));
        search_players(
            &conn,
            1,
            1,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            None,
            &requested_fields,
        )
        .expect("query second potential page");
        assert_eq!(potential_state(&conn), before);
    }

    #[test]
    fn potential_display_rejects_missing_and_wrong_version_rows_without_writes() {
        for wrong_version in [false, true] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let mut conn = open_migrated(&temp_dir.path().join("potential-display-corrupt.db"));
            ingest_players(&mut conn, vec![player_template(1, "Display", 150)]);

            let snapshot_id = current_snapshot_id(&conn);
            if wrong_version {
                conn.execute(
                    "UPDATE player_role_metrics SET projection_model_version = ?2
                     WHERE snapshot_id = ?1 AND uid = 1",
                    rusqlite::params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
                )
            } else {
                conn.execute(
                    "DELETE FROM player_role_metrics
                     WHERE snapshot_id = ?1 AND uid = 1",
                    [snapshot_id],
                )
            }
            .expect("corrupt compact row");
            let before = potential_state(&conn);
            deny_potential_writes(&conn);

            assert_eq!(
                search_players(
                    &conn,
                    0,
                    DEFAULT_PAGE_LIMIT,
                    SortField::DEFAULT,
                    SortDir::DEFAULT,
                    None,
                    &["potential_role.line_holding_keeper_oop".to_string()],
                ),
                Err("Current potential snapshot is incomplete".to_string())
            );
            assert_eq!(potential_state(&conn), before);
        }
    }

    #[test]
    fn potential_sort_reads_complete_rows_without_writes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("potential-sort.db"));
        ingest_players(
            &mut conn,
            vec![
                player_with_deep_fields(
                    1,
                    "Lower potential",
                    100,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "GK": 20 }),
                        attributes: json!({ "Positioning": 8, "Concentration": 8 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
                player_with_deep_fields(
                    2,
                    "Higher potential",
                    180,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "GK": 20 }),
                        attributes: json!({ "Positioning": 16, "Concentration": 16 }),
                        hidden: json!({}),
                        personality: json!({}),
                    },
                ),
            ],
        );

        let before = potential_state(&conn);
        deny_potential_writes(&conn);
        let page = search_players(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::parse("potential_role.line_holding_keeper_oop")
                .expect("parse potential sort"),
            SortDir::Desc,
            None,
            &[],
        )
        .expect("sort by potential role");

        assert_eq!(
            page.players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            ["Higher potential", "Lower potential"]
        );
        assert_eq!(potential_state(&conn), before);
    }

    #[test]
    fn potential_sort_rejects_missing_and_wrong_version_rows_without_writes() {
        for wrong_version in [false, true] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let mut conn = open_migrated(&temp_dir.path().join("potential-sort-corrupt.db"));
            ingest_players(&mut conn, vec![player_template(1, "Sort", 150)]);

            let snapshot_id = current_snapshot_id(&conn);
            if wrong_version {
                conn.execute(
                    "UPDATE player_role_metrics SET projection_model_version = ?2
                     WHERE snapshot_id = ?1 AND uid = 1",
                    rusqlite::params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
                )
            } else {
                conn.execute(
                    "DELETE FROM player_role_metrics
                     WHERE snapshot_id = ?1 AND uid = 1",
                    [snapshot_id],
                )
            }
            .expect("corrupt compact row");
            let before = potential_state(&conn);
            deny_potential_writes(&conn);

            assert_eq!(
                search_players(
                    &conn,
                    0,
                    DEFAULT_PAGE_LIMIT,
                    SortField::parse("potential_role.line_holding_keeper_oop")
                        .expect("parse potential sort"),
                    SortDir::Desc,
                    None,
                    &[],
                ),
                Err("Current potential snapshot is incomplete".to_string())
            );
            assert_eq!(potential_state(&conn), before);
        }
    }

    #[test]
    fn potential_sort_orders_nullable_ties_with_complete_visible_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("warm-potential-sort.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "First score tie", 180),
                player_template(2, "Second score tie", 170),
                player_template(3, "Lower score", 160),
                player_template(4, "Nullable score", 150),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        let sort_role = "line_holding_keeper_oop";
        let sort_metric = format!("potential_role.{sort_role}");
        let distinct_visible_role = "sweeper_keeper_oop";
        let distinct_visible_metric = format!("potential_role.{distinct_visible_role}");
        for (uid, score) in [(1, Some(80)), (2, Some(80)), (3, Some(40)), (4, None)] {
            set_potential_role_score(&conn, uid, sort_role, score);
        }
        for (uid, score) in [(1, Some(65)), (3, Some(45))] {
            set_potential_role_score(&conn, uid, distinct_visible_role, score);
        }
        let before = potential_state(&conn);
        deny_potential_writes(&conn);
        let sort_by = SortField::parse(&sort_metric).expect("parse potential sort");

        for (direction, expected) in [
            (SortDir::Asc, vec![4, 3, 1, 2]),
            (SortDir::Desc, vec![1, 2, 3, 4]),
        ] {
            let page = search_players(&conn, 0, 4, sort_by.clone(), direction, None, &[])
                .expect("sort warm potential scores");
            assert_eq!(page.total, 4);
            assert_eq!(
                page.players
                    .iter()
                    .map(|player| player.uid)
                    .collect::<Vec<_>>(),
                expected
            );
        }

        let page = search_players(
            &conn,
            1,
            2,
            sort_by,
            SortDir::Asc,
            None,
            &[sort_metric, distinct_visible_metric],
        )
        .expect("page warm potential scores with a distinct visible role");
        assert_eq!(page.total, 4);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [3, 1]
        );
        let compact_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_role_metrics WHERE snapshot_id = ?1",
                [snapshot_id],
                |row| row.get(0),
            )
            .expect("count compact rows");
        assert_eq!(compact_rows, 4);
        let distinct_visible_column =
            crate::features::player_metrics::compact::player_potential_column(
                distinct_visible_role,
            )
            .expect("distinct visible role column");
        let distinct_visible_values = conn
            .prepare(&format!(
                "SELECT uid, {distinct_visible_column}
                 FROM player_role_metrics
                 WHERE snapshot_id = ?1
                 ORDER BY uid ASC"
            ))
            .expect("prepare distinct visible role query")
            .query_map([snapshot_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?))
            })
            .expect("query distinct visible role rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect distinct visible role rows");
        assert_eq!(
            distinct_visible_values,
            [(1, Some(65)), (2, None), (3, Some(45)), (4, None)]
        );
        assert_eq!(potential_state(&conn), before);
    }

    #[test]
    fn club_dna_sort_uses_a_missing_preserving_exact_identity_relation() {
        let source = include_str!("query.rs");
        let query = &source[source
            .find("pub fn search_players_in_view")
            .expect("search query function")
            ..source
                .find("fn current_club_dna_definition_version")
                .expect("following helper")];

        assert!(query.contains("LEFT JOIN club_dna_scores club_dna_sort"));
        assert!(query.contains("club_dna_sort.snapshot_id = players.snapshot_id"));
        assert!(query.contains("club_dna_sort.uid = players.uid"));
        assert!(query.contains("club_dna_sort.definition_version"));
        assert!(query.contains("club_dna_sort.score_model_version"));
        assert!(query.contains("ORDER BY club_dna_sort.score IS NULL ASC"));
    }

    #[test]
    fn rejects_unknown_requested_metric_ids_before_reading_the_snapshot() {
        let conn = Connection::open_in_memory().expect("open database");
        let error = search_players(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            None,
            &["unknown.metric".to_string()],
        )
        .expect_err("reject unknown requested field");

        assert!(error.contains("unknown player metric"));
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

    #[test]
    fn current_role_sort_retains_missing_nullable_duplicate_and_tied_scores() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("current-role-sort.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "First score tie", 180),
                player_template(2, "Second score tie", 170),
                player_template(3, "Nullable score", 160),
                player_template(4, "Missing score", 150),
            ],
        );
        let role_id = "deep_lying_playmaker_ip";
        set_role_score(&conn, 1, role_id, Some(80));
        set_role_score(&conn, 2, role_id, Some(80));
        set_role_score(&conn, 3, role_id, None);
        set_role_score(&conn, 4, role_id, None);
        let sort_by = SortField::parse(&format!("role.{role_id}")).expect("parse role sort");

        for (direction, expected) in [
            (SortDir::Asc, vec![3, 4, 1, 2]),
            (SortDir::Desc, vec![1, 2, 3, 4]),
        ] {
            let page = search_without_filters(&conn, 0, 4, sort_by.clone(), direction)
                .expect("sort current roles");
            assert_eq!(page.total, 4);
            assert_eq!(
                page.players
                    .iter()
                    .map(|player| player.uid)
                    .collect::<Vec<_>>(),
                expected
            );
        }

        let page =
            search_without_filters(&conn, 1, 2, sort_by, SortDir::Asc).expect("page current roles");
        assert_eq!(page.total, 4);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            vec![4, 1]
        );
    }

    #[test]
    fn current_role_sort_reads_complete_requested_potential_fields() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("current-role-potential-page.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "First score tie", 180),
                player_template(2, "Second score tie", 170),
                player_template(3, "Nullable score", 160),
                player_template(4, "Missing score", 150),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        conn.execute(
            "UPDATE players
             SET positions_json = ?1, attributes_json = ?2
             WHERE snapshot_id = ?3 AND uid IN (1, 4)",
            rusqlite::params![
                json!({ "GK": 20 }).to_string(),
                json!({ "Positioning": 16, "Concentration": 16 }).to_string(),
                snapshot_id,
            ],
        )
        .expect("set potential source values");
        let role_id = "deep_lying_playmaker_ip";
        set_role_score(&conn, 1, role_id, Some(80));
        set_role_score(&conn, 2, role_id, Some(80));
        set_role_score(&conn, 3, role_id, None);
        set_role_score(&conn, 4, role_id, None);

        let potential_field = "potential_role.line_holding_keeper_oop".to_string();
        set_potential_role_score(&conn, 1, "line_holding_keeper_oop", Some(70));
        set_potential_role_score(&conn, 4, "line_holding_keeper_oop", Some(70));
        let before = potential_state(&conn);
        deny_potential_writes(&conn);
        let page = search_players(
            &conn,
            1,
            2,
            SortField::parse(&format!("role.{role_id}")).expect("parse role sort"),
            SortDir::Asc,
            None,
            std::slice::from_ref(&potential_field),
        )
        .expect("query current role page with potential field");

        assert_eq!(page.total, 4);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            vec![4, 1]
        );
        assert!(matches!(
            page.players[0].dynamic_values.get(&potential_field),
            Some(Some(DynamicValue::Integer(_)))
        ));
        assert_eq!(potential_state(&conn), before);
    }

    #[test]
    fn returns_moneyball_role_scores_from_full_and_filtered_cohorts() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-cohorts.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Cohort A", 100),
                player_template(2, "Cohort B", 110),
                player_template(3, "Outside", 120),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        let role_id = "mc_central_midfielder_ip";
        insert_role_row(&conn, snapshot_id, 1, role_id, Some(10.0), Some(17), None);
        insert_role_row(&conn, snapshot_id, 2, role_id, Some(20.0), Some(42), None);
        insert_role_row(&conn, snapshot_id, 3, role_id, Some(30.0), Some(83), None);
        let requested_fields = [format!("moneyball_role.{role_id}")];

        let full = search_moneyball(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::Name,
            SortDir::Asc,
            &requested_fields,
            Vec::new(),
            None,
            ComparisonPool::FullCsv,
        )
        .expect("full Moneyball role search");
        assert_eq!(full.total, 3);
        assert_eq!(
            full.players[0]
                .dynamic_values
                .get(&format!("moneyball_role.{role_id}")),
            Some(&Some(DynamicValue::Integer(17)))
        );

        let rules = vec![filter_rule(
            "name",
            "contains",
            FilterValue::Text("Cohort".to_string()),
        )];
        let filtered_first = search_moneyball(
            &conn,
            0,
            1,
            SortField::Name,
            SortDir::Asc,
            &requested_fields,
            rules.clone(),
            None,
            ComparisonPool::Filtered,
        )
        .expect("filtered first page");
        let filtered_second = search_moneyball(
            &conn,
            1,
            1,
            SortField::Name,
            SortDir::Asc,
            &requested_fields,
            rules,
            None,
            ComparisonPool::Filtered,
        )
        .expect("filtered second page");
        assert_eq!(filtered_first.total, 2);
        assert_eq!(filtered_second.total, 2);
        assert_eq!(
            filtered_first.players[0]
                .dynamic_values
                .get(&format!("moneyball_role.{role_id}")),
            Some(&Some(DynamicValue::Integer(0)))
        );
        assert_eq!(
            filtered_second.players[0]
                .dynamic_values
                .get(&format!("moneyball_role.{role_id}")),
            Some(&Some(DynamicValue::Integer(100)))
        );
    }

    #[test]
    fn applies_role_filters_after_scoring_and_unions_mixed_or_matches() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-filters.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "NameOnly", 100),
                player_template(2, "RoleOnly", 110),
                player_template(3, "Neither", 120),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        let role_id = "mc_central_midfielder_ip";
        insert_role_row(&conn, snapshot_id, 1, role_id, Some(10.0), Some(20), None);
        insert_role_row(&conn, snapshot_id, 2, role_id, Some(30.0), Some(90), None);
        insert_role_row(&conn, snapshot_id, 3, role_id, Some(20.0), Some(50), None);
        let requested_fields = [format!("moneyball_role.{role_id}")];

        let and_rules = vec![
            filter_rule("name", "contains", FilterValue::Text("Name".to_string())),
            filter_rule(
                &format!("moneyball_role.{role_id}"),
                "gt",
                FilterValue::Integer(10),
            ),
        ];
        let and_page = search_moneyball(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::Name,
            SortDir::Asc,
            &requested_fields,
            and_rules,
            Some("and"),
            ComparisonPool::Filtered,
        )
        .expect("post-score AND filter");
        assert_eq!(and_page.total, 1);
        assert_eq!(and_page.players[0].uid, 1);
        assert_eq!(
            and_page.players[0]
                .dynamic_values
                .get(&format!("moneyball_role.{role_id}")),
            Some(&Some(DynamicValue::Integer(50)))
        );

        let or_rules = vec![
            filter_rule(
                "name",
                "contains",
                FilterValue::Text("NameOnly".to_string()),
            ),
            filter_rule(
                &format!("moneyball_role.{role_id}"),
                "gt",
                FilterValue::Integer(80),
            ),
        ];
        let or_page = search_moneyball(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::parse_for_moneyball(&format!("moneyball_role.{role_id}"), true)
                .expect("role sort"),
            SortDir::Desc,
            &requested_fields,
            or_rules,
            Some("or"),
            ComparisonPool::Filtered,
        )
        .expect("mixed OR role filter");
        assert_eq!(or_page.total, 2);
        assert_eq!(
            or_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [2, 1]
        );
        assert_eq!(
            or_page.players[0]
                .dynamic_values
                .get(&format!("moneyball_role.{role_id}")),
            Some(&Some(DynamicValue::Integer(100)))
        );
    }

    #[test]
    fn sorts_moneyball_roles_with_nulls_last_and_uid_ties_after_filtering() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-sort.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "One", 100),
                player_template(2, "Two", 110),
                player_template(3, "Three", 120),
                player_template(4, "Missing", 130),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        let role_id = "mc_central_midfielder_ip";
        insert_role_row(&conn, snapshot_id, 1, role_id, Some(1.0), Some(50), None);
        insert_role_row(&conn, snapshot_id, 2, role_id, Some(2.0), Some(50), None);
        insert_role_row(&conn, snapshot_id, 3, role_id, Some(3.0), Some(0), None);
        insert_role_row(&conn, snapshot_id, 4, role_id, None, None, Some(0));
        let requested_fields = [format!("moneyball_role.{role_id}")];
        let sort_by = SortField::parse_for_moneyball(&format!("moneyball_role.{role_id}"), true)
            .expect("role sort");

        let descending = search_moneyball(
            &conn,
            1,
            1,
            sort_by.clone(),
            SortDir::Desc,
            &requested_fields,
            Vec::new(),
            None,
            ComparisonPool::FullCsv,
        )
        .expect("descending role sort");
        assert_eq!(descending.total, 4);
        assert_eq!(descending.players[0].uid, 2);

        let ascending = search_moneyball(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            sort_by,
            SortDir::Asc,
            &requested_fields,
            vec![filter_rule(
                &format!("moneyball_role.{role_id}"),
                "neq",
                FilterValue::Integer(50),
            )],
            None,
            ComparisonPool::FullCsv,
        )
        .expect("ascending role sort and null filter");
        assert_eq!(ascending.total, 1);
        assert_eq!(ascending.players[0].uid, 3);
        assert_eq!(
            ascending.players[0]
                .dynamic_values
                .get(&format!("moneyball_role.{role_id}")),
            Some(&Some(DynamicValue::Integer(0)))
        );
    }

    #[test]
    fn rejects_moneyball_role_queries_outside_moneyball_view() {
        let conn = Connection::open_in_memory().expect("open database");
        let role_field = "moneyball_role.mc_central_midfielder_ip".to_string();
        let error = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::DEFAULT,
                sort_dir: SortDir::DEFAULT,
                filter_ast: None,
                requested_fields: &[role_field],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect_err("General role field");
        assert!(error.contains("Moneyball search view"));
    }

    #[test]
    fn preserves_moneyball_sort_restrictions_for_role_queries() {
        let conn = Connection::open_in_memory().expect("open database");
        let role_field = "moneyball_role.mc_central_midfielder_ip".to_string();
        let error = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::DEFAULT,
                filter_ast: None,
                requested_fields: &[role_field],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect_err("CA sort in Moneyball role query");
        assert!(error.contains("unsupported Moneyball sort field"));
    }

    #[test]
    fn suggest_returns_empty_for_blank_query() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("suggest-blank.db"));
        ingest_players(&mut conn, vec![player_template(1, "Anyone", 150)]);

        let hits = suggest_players(&conn, "   ", DEFAULT_SUGGEST_LIMIT).expect("suggest");
        assert!(hits.is_empty());
    }

    #[test]
    fn suggest_orders_by_match_tier_then_ca_descending() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("suggest-rank.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Alex", 100),
                player_template(2, "Alex", 160),
                player_template(3, "Alexander", 120),
                player_template(4, "Sam Alex", 180),
                player_template(5, "Unrelated", 200),
            ],
        );

        let hits = suggest_players(&conn, "Alex", DEFAULT_SUGGEST_LIMIT).expect("suggest");
        let names: Vec<&str> = hits.iter().map(|hit| hit.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "Alex",      // exact, higher CA
                "Alex",      // exact, lower CA
                "Alexander", // prefix
                "Sam Alex",  // contains
            ]
        );
        assert_eq!(hits[0].uid, 2);
        assert_eq!(hits[0].ca, 160);
        assert!(!names.contains(&"Unrelated"));
    }

    #[test]
    fn tactic_general_current_and_potential_with_age_fallback_and_eligibility_null() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-general.db"));
        // Player 1: eligible GK, age 26, scores 80/60 -> blended 70, no penalty
        // Player 2: ineligible due to low GK familiarity (<12) -> NULL
        // Player 3: age 30, potential should fallback to current (80/60) -> 70, not potential 20/20 -> 20
        // Player 4: strict foot mismatch -> NULL
        let mut p1 = player_template(1, "Eligible", 150);
        p1["age"] = serde_json::json!(26);
        p1["preferredFoot"] = serde_json::json!("right");
        p1["positions"] = serde_json::json!({"GK": 18});
        let mut p2 = player_template(2, "Low Familiarity", 150);
        p2["age"] = serde_json::json!(26);
        p2["preferredFoot"] = serde_json::json!("right");
        p2["positions"] = serde_json::json!({"GK": 10});
        let mut p3 = player_template(3, "Old Potential Fallback", 150);
        p3["age"] = serde_json::json!(30);
        p3["preferredFoot"] = serde_json::json!("right");
        p3["positions"] = serde_json::json!({"GK": 18});
        let mut p4 = player_template(4, "Strict Foot Mismatch", 150);
        p4["age"] = serde_json::json!(26);
        p4["preferredFoot"] = serde_json::json!("left");
        p4["positions"] = serde_json::json!({"GK": 18});
        ingest_players(&mut conn, vec![p1, p2, p3, p4]);
        // Override default tactic foot for mismatch test: set goalkeeper lane to require right strict
        let snapshot_id = current_snapshot_id(&conn);
        let save_id: i64 = conn
            .query_row(
                "SELECT save_id FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |r| r.get(0),
            )
            .expect("save");
        let mut tactic =
            crate::features::planner::tactic::get_tactic(&conn, save_id).expect("tactic");
        tactic.lanes[0].preferred_foot = "right".to_string();
        tactic.lanes[0].foot_preference = "strict".to_string();
        tactic.lanes[0].ip_weight = 0.5;
        crate::features::planner::tactic::save_tactic(&conn, save_id, &tactic)
            .expect("save tactic");
        // Set role scores for GK lane
        for (uid, ip, oop, pot_ip, pot_oop) in [
            (1, Some(80), Some(60), None, None),
            (2, Some(80), Some(60), None, None),
            (3, Some(80), Some(60), Some(20), Some(20)),
            (4, Some(80), Some(60), None, None),
        ] {
            set_role_score(&conn, uid, "goalkeeper_ip", ip);
            set_role_score(&conn, uid, "line_holding_keeper_oop", oop);
            if let Some(v) = pot_ip {
                set_potential_role_score(&conn, uid, "goalkeeper_ip", Some(v));
            }
            if let Some(v) = pot_oop {
                set_potential_role_score(&conn, uid, "line_holding_keeper_oop", Some(v));
            }
        }
        // Current column should give 70 for uid1, None for 2 (familiarity), 70 for 3 (not using potential), None for 4 (strict foot)
        let page_cur = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("tactic current");
        let vals: std::collections::BTreeMap<i64, Option<i64>> = page_cur
            .players
            .iter()
            .map(|p| {
                (
                    p.uid,
                    p.dynamic_values
                        .get("tactic_current.goalkeeper")
                        .and_then(|v| v.as_ref())
                        .and_then(|v| match v {
                            DynamicValue::Integer(i) => Some(*i),
                            _ => None,
                        }),
                )
            })
            .collect();
        assert_eq!(vals.get(&1), Some(&Some(70)));
        assert_eq!(vals.get(&2), Some(&None));
        assert_eq!(vals.get(&3), Some(&Some(70)));
        assert_eq!(vals.get(&4), Some(&None));
        // Potential for uid3 age 30 should fallback to current 70, not potential 20
        let page_pot = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_potential.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_potential.goalkeeper".to_string()],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("tactic potential");
        let vals_pot: std::collections::BTreeMap<i64, Option<i64>> = page_pot
            .players
            .iter()
            .map(|p| {
                (
                    p.uid,
                    p.dynamic_values
                        .get("tactic_potential.goalkeeper")
                        .and_then(|v| v.as_ref())
                        .and_then(|v| match v {
                            DynamicValue::Integer(i) => Some(*i),
                            _ => None,
                        }),
                )
            })
            .collect();
        assert_eq!(vals_pot.get(&3), Some(&Some(70)));
        // Sort nulls last both directions, UID tie
        // Make uid1 and uid3 both 70 -> should be ordered by UID asc when sorted DESC? Actually both same score, uid tie.
        // Add a player with higher score to test ordering
        set_role_score(&conn, 1, "goalkeeper_ip", Some(90));
        set_role_score(&conn, 1, "line_holding_keeper_oop", Some(90));
        // Reset strict foot for sorting test to allow p4 to be scored
        let mut tactic2 =
            crate::features::planner::tactic::get_tactic(&conn, save_id).expect("tactic");
        tactic2.lanes[0].preferred_foot = "any".to_string();
        tactic2.lanes[0].foot_preference = "preferred".to_string();
        crate::features::planner::tactic::save_tactic(&conn, save_id, &tactic2).expect("save");
        // p4 now eligible with 80/60 -> 70; p1 now 90, p3 70
        let page_desc = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("desc");
        // Expected order: 1 (90), then 3 and 4 (70 tie -> UID 3 before 4), then 2 null last
        assert_eq!(
            page_desc.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![1, 3, 4, 2]
        );
        let page_asc = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("asc");
        // ASC: 3,4 (70) then 1 (90) then 2 null last
        assert_eq!(
            page_asc.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![3, 4, 1, 2]
        );
    }

    #[test]
    fn tactic_moneyball_mapped_numeric_and_uncovered_null_with_sort_and_pool() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-moneyball.db"));
        let players = (1..=3)
            .map(|uid| {
                let mut p = player_template(uid, &format!("P{uid}"), 150);
                if uid == 2 {
                    p["positions"] = serde_json::json!({"GK": 10, "ST": 18, "MC": 18});
                } else {
                    p["positions"] = serde_json::json!({"GK": 18, "ST": 18, "MC": 18});
                }
                p["preferredFoot"] = serde_json::json!("right");
                p
            })
            .collect();
        ingest_players(&mut conn, players);
        let snapshot_id = current_snapshot_id(&conn);
        let save_id: i64 = conn
            .query_row(
                "SELECT save_id FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |r| r.get(0),
            )
            .expect("save");
        // Use default tactic for mapped case: goalkeeper lane should be mapped
        // For uncovered, customize centre_forward lane to use second_striker_ip+ST (uncovered)
        let mut tactic =
            crate::features::planner::tactic::get_tactic(&conn, save_id).expect("tactic");
        // Keep goalkeeper as is for mapped test; change centre_forward ip to uncovered
        let cf_idx = tactic
            .lanes
            .iter()
            .position(|l| l.lane_id == "centre_forward")
            .unwrap();
        tactic.lanes[cf_idx].ip_role_id = "second_striker_ip".to_string();
        tactic.lanes[cf_idx].ip_position = "ST".to_string(); // ST base ST -> uncovered with second_striker_ip+ST
        crate::features::planner::tactic::save_tactic(&conn, save_id, &tactic)
            .expect("save uncovered");
        // Insert Moneyball stats for goalkeeper mapped defs (both IP and OOP) and for uncovered (should not affect)
        let catalog = builtin_catalog().expect("catalog");
        let gk_ip_def = catalog
            .definitions
            .iter()
            .find(|d| {
                d.attribute_role_id.as_deref() == Some("goalkeeper_ip")
                    && d.position_tags.contains(&"GK".to_string())
            })
            .expect("gk ip def");
        let gk_oop_def = catalog
            .definitions
            .iter()
            .find(|d| {
                d.attribute_role_id.as_deref() == Some("line_holding_keeper_oop")
                    && d.position_tags.contains(&"GK".to_string())
            })
            .expect("gk oop def");
        for uid in 1..=3 {
            // Build stats that cover both defs' metric keys combined
            let mut stats: std::collections::BTreeMap<String, serde_json::Value> =
                std::collections::BTreeMap::new();
            let mut perc: std::collections::BTreeMap<String, serde_json::Value> =
                std::collections::BTreeMap::new();
            for m in gk_ip_def.metrics.iter().chain(gk_oop_def.metrics.iter()) {
                stats.insert(m.key.clone(), serde_json::json!(10.0 + uid as f64));
                perc.insert(m.key.clone(), serde_json::json!(70));
            }
            let stats_json = serde_json::to_string(&stats).unwrap();
            let perc_json = serde_json::to_string(&perc).unwrap();
            insert_moneyball_statistics(&conn, snapshot_id, uid, &stats_json, Some(&perc_json));
        }
        // Query mapped goalkeeper tactic should be numeric for uid1 and 3, null for uid2
        let page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("moneyball mapped");
        let vals: std::collections::BTreeMap<i64, Option<i64>> = page
            .players
            .iter()
            .map(|p| {
                (
                    p.uid,
                    p.dynamic_values
                        .get("tactic_current.goalkeeper")
                        .and_then(|v| v.as_ref())
                        .and_then(|v| match v {
                            DynamicValue::Integer(i) => Some(*i),
                            _ => None,
                        }),
                )
            })
            .collect();
        assert!(
            vals.get(&1).unwrap().is_some(),
            "uid1 mapped should be numeric"
        );
        assert_eq!(vals.get(&2), Some(&None), "uid2 familiarity low -> null");
        assert!(vals.get(&3).unwrap().is_some());
        // Uncovered centre_forward should be null for all
        let page_uncovered = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.centre_forward", true)
                    .expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.centre_forward".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("uncovered");
        for p in &page_uncovered.players {
            assert_eq!(
                p.dynamic_values.get("tactic_current.centre_forward"),
                Some(&None)
            );
        }
        // Sort nulls last both directions
        let page_desc = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("desc");
        // Highest first, null last, uid tie (uid1 and 3 same score -> uid1 before 3)
        assert_eq!(page_desc.players.last().unwrap().uid, 2);
        assert_eq!(page_desc.players[0].uid, 1);
        assert_eq!(page_desc.players[1].uid, 3);
        let page_filt = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::Filtered,
            },
        )
        .expect("filtered");
        assert_eq!(page_filt.total, 3);
        assert_eq!(
            page_filt.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            page.players.iter().map(|p| p.uid).collect::<Vec<_>>()
        );
    }

    #[test]
    fn tactic_moneyball_sort_only_and_opposite_group_same_lane() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-moneyball-sortonly.db"));
        let players = (1..=2)
            .map(|uid| {
                let mut p = player_template(uid, &format!("P{uid}"), 150);
                p["positions"] = serde_json::json!({"GK": 18, "ST": 18, "MC": 18});
                p["preferredFoot"] = serde_json::json!("right");
                p
            })
            .collect();
        ingest_players(&mut conn, players);
        let snapshot_id = current_snapshot_id(&conn);
        let catalog = builtin_catalog().expect("catalog");
        let gk_ip_def = catalog
            .definitions
            .iter()
            .find(|d| {
                d.attribute_role_id.as_deref() == Some("goalkeeper_ip")
                    && d.position_tags.contains(&"GK".to_string())
            })
            .expect("gk ip");
        let gk_oop_def = catalog
            .definitions
            .iter()
            .find(|d| {
                d.attribute_role_id.as_deref() == Some("line_holding_keeper_oop")
                    && d.position_tags.contains(&"GK".to_string())
            })
            .expect("gk oop");
        for uid in 1..=2 {
            let mut stats = std::collections::BTreeMap::new();
            let mut perc = std::collections::BTreeMap::new();
            for m in gk_ip_def.metrics.iter().chain(gk_oop_def.metrics.iter()) {
                stats.insert(m.key.clone(), serde_json::json!(10.0 + uid as f64 * 10.0));
                perc.insert(m.key.clone(), serde_json::json!(60 + uid * 10));
            }
            insert_moneyball_statistics(
                &conn,
                snapshot_id,
                uid,
                &serde_json::to_string(&stats).unwrap(),
                Some(&serde_json::to_string(&perc).unwrap()),
            );
        }
        // Sort-only: request no tactic field, but sort by tactic_current.goalkeeper
        let page_sort_only = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("sort-only");
        assert_eq!(page_sort_only.total, 2);
        assert_eq!(page_sort_only.players[0].uid, 2);
        assert_eq!(page_sort_only.players[1].uid, 1);
        // Request potential same lane while sorting current: both must be computed independently
        let page_both = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_potential.goalkeeper".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("opposite group");
        for p in &page_both.players {
            assert!(
                p.dynamic_values.contains_key("tactic_potential.goalkeeper"),
                "potential must be present"
            );
            let v = p.dynamic_values.get("tactic_potential.goalkeeper").unwrap();
            assert!(v.is_some(), "potential score should be numeric");
        }
        assert_eq!(page_both.players[0].uid, 2);
        assert_eq!(page_both.players[1].uid, 1);
        // Ensure current and potential keys remain distinct (same lane, different group)
        let page_both_vals = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &[
                    "tactic_current.goalkeeper".to_string(),
                    "tactic_potential.goalkeeper".to_string(),
                ],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("both groups");
        for p in &page_both_vals.players {
            assert!(p.dynamic_values.contains_key("tactic_current.goalkeeper"));
            assert!(p.dynamic_values.contains_key("tactic_potential.goalkeeper"));
            // Moneyball tactic current/potential intentionally share identical cohort score sources,
            // but exact IDs must remain distinct keys; scores should be equal for this cohort.
            assert_eq!(
                p.dynamic_values.get("tactic_current.goalkeeper"),
                p.dynamic_values.get("tactic_potential.goalkeeper")
            );
        }
    }

    #[test]
    fn tactic_general_sql_vs_rust_parity_and_penalty_saturation() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-parity.db"));
        // Player at half-round boundary: ip 81, oop 82, weight 0.5 => 81.5 => 82
        // Familiarity tests: 12..15 penalized, 16 not; soft foot -5; saturation at 0; missing score -> None
        let mut p_half = player_template(1, "HalfRound", 150);
        p_half["age"] = serde_json::json!(26);
        p_half["preferredFoot"] = serde_json::json!("right");
        p_half["positions"] = serde_json::json!({"GK": 16});
        let mut p_fam15 = player_template(2, "Fam15", 150);
        p_fam15["age"] = serde_json::json!(26);
        p_fam15["preferredFoot"] = serde_json::json!("right");
        p_fam15["positions"] = serde_json::json!({"GK": 15});
        let mut p_fam12 = player_template(3, "Fam12", 150);
        p_fam12["age"] = serde_json::json!(26);
        p_fam12["preferredFoot"] = serde_json::json!("right");
        p_fam12["positions"] = serde_json::json!({"GK": 12});
        let mut p_soft = player_template(4, "SoftFoot", 150);
        p_soft["age"] = serde_json::json!(26);
        p_soft["preferredFoot"] = serde_json::json!("left");
        p_soft["positions"] = serde_json::json!({"GK": 15});
        let mut p_saturate = player_template(5, "Saturate", 150);
        p_saturate["age"] = serde_json::json!(26);
        p_saturate["preferredFoot"] = serde_json::json!("left");
        p_saturate["positions"] = serde_json::json!({"GK": 12});
        let mut p_missing = player_template(6, "MissingScore", 150);
        p_missing["age"] = serde_json::json!(26);
        p_missing["preferredFoot"] = serde_json::json!("right");
        p_missing["positions"] = serde_json::json!({"GK": 18});
        ingest_players(
            &mut conn,
            vec![p_half, p_fam15, p_fam12, p_soft, p_saturate, p_missing],
        );
        let snapshot_id = current_snapshot_id(&conn);
        let save_id: i64 = conn
            .query_row(
                "SELECT save_id FROM snapshots WHERE id = ?1",
                [snapshot_id],
                |r| r.get(0),
            )
            .expect("save");
        let mut tactic =
            crate::features::planner::tactic::get_tactic(&conn, save_id).expect("tactic");
        tactic.lanes[0].ip_weight = 0.5;
        tactic.lanes[0].preferred_foot = "right".to_string();
        tactic.lanes[0].foot_preference = "preferred".to_string();
        crate::features::planner::tactic::save_tactic(&conn, save_id, &tactic).expect("save");
        // Set role scores: half-round 81/82, others 70/70 except missing and saturate low
        set_role_score(&conn, 1, "goalkeeper_ip", Some(81));
        set_role_score(&conn, 1, "line_holding_keeper_oop", Some(82));
        for uid in [2, 3, 4] {
            set_role_score(&conn, uid, "goalkeeper_ip", Some(70));
            set_role_score(&conn, uid, "line_holding_keeper_oop", Some(70));
        }
        set_role_score(&conn, 5, "goalkeeper_ip", Some(6));
        set_role_score(&conn, 5, "line_holding_keeper_oop", Some(6));
        set_role_score(&conn, 6, "goalkeeper_ip", None);
        set_role_score(&conn, 6, "line_holding_keeper_oop", Some(70));
        // Compute expected Rust scores
        let lane = tactic.lanes[0].clone();
        let expected_half = crate::features::planner::fit::tactic_adjusted_score(
            Some(81),
            Some(82),
            0.5,
            "right",
            &std::collections::BTreeMap::from([("GK".to_string(), Some(16))]),
            &lane,
        );
        assert_eq!(expected_half, Some(82));
        // Fam15: familiarity 15 both phases -> -10, 70 blended -> 60
        let expected_fam15 = crate::features::planner::fit::tactic_adjusted_score(
            Some(70),
            Some(70),
            0.5,
            "right",
            &std::collections::BTreeMap::from([("GK".to_string(), Some(15))]),
            &lane,
        );
        assert_eq!(expected_fam15, Some(60));
        // Soft foot with 15: -10 fam + -5 foot = -15 => 55
        // But lane uses same GK for both IP/OOP, so familiarity penalty counted twice (once per phase) + soft foot once
        // For p_soft: GK 15 => -5 per phase => -10 + soft -5 => -15 => 55
        let expected_soft = crate::features::planner::fit::tactic_adjusted_score(
            Some(70),
            Some(70),
            0.5,
            "left",
            &std::collections::BTreeMap::from([("GK".to_string(), Some(15))]),
            &lane,
        );
        assert_eq!(expected_soft, Some(55));
        // Saturation: 6 blended -15 => saturates 0
        let expected_sat = crate::features::planner::fit::tactic_adjusted_score(
            Some(6),
            Some(6),
            0.5,
            "left",
            &std::collections::BTreeMap::from([("GK".to_string(), Some(12))]),
            &lane,
        );
        assert_eq!(expected_sat, Some(0));
        // Query via SQL and compare
        let page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::General,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("parity query");
        let vals: std::collections::BTreeMap<i64, Option<i64>> = page
            .players
            .iter()
            .map(|p| {
                (
                    p.uid,
                    p.dynamic_values
                        .get("tactic_current.goalkeeper")
                        .and_then(|v| v.as_ref())
                        .and_then(|v| match v {
                            DynamicValue::Integer(i) => Some(*i),
                            _ => None,
                        }),
                )
            })
            .collect();
        assert_eq!(vals.get(&1), Some(&Some(82)));
        assert_eq!(vals.get(&2), Some(&Some(60)));
        assert_eq!(vals.get(&4), Some(&Some(55)));
        assert_eq!(vals.get(&5), Some(&Some(0)));
        assert_eq!(vals.get(&6), Some(&None));
        // Ensure SQLite NULL handling matches Rust None
        assert_eq!(vals.get(&6), Some(&None));
    }

    #[test]
    fn tactic_completeness_failure_before_score_reads() {
        for use_potential in [false, true] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let mut conn = open_migrated(&temp_dir.path().join("tactic-completeness.db"));
            ingest_players(&mut conn, vec![player_template(1, "One", 150)]);
            let snapshot_id = current_snapshot_id(&conn);
            if use_potential {
                conn.execute(
                    "UPDATE player_role_metrics SET projection_model_version = 999 WHERE snapshot_id = ?1",
                    [snapshot_id],
                )
                .expect("corrupt potential");
            } else {
                conn.execute(
                    "DELETE FROM player_role_metrics WHERE snapshot_id = ?1",
                    [snapshot_id],
                )
                .expect("delete current");
            }
            let field = if use_potential {
                "tactic_potential.goalkeeper"
            } else {
                "tactic_current.goalkeeper"
            };
            let err = search_players_in_view(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: 10,
                    sort_by: SortField::parse(field).expect("sort"),
                    sort_dir: SortDir::Asc,
                    filter_ast: None,
                    requested_fields: &[field.to_string()],
                    view: SearchView::General,
                    comparison_pool: ComparisonPool::FullCsv,
                },
            )
            .expect_err("must fail completeness");
            assert!(
                err.contains("Current") || err.contains("incomplete"),
                "err was {err}"
            );
            // Ensure potential completeness also fails for General sort without requested field (sort-only)
            let err2 = search_players_in_view(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: 10,
                    sort_by: SortField::parse(field).expect("sort"),
                    sort_dir: SortDir::Asc,
                    filter_ast: None,
                    requested_fields: &[],
                    view: SearchView::General,
                    comparison_pool: ComparisonPool::FullCsv,
                },
            )
            .expect_err("sort-only must also fail");
            assert!(err2.contains("Current") || err2.contains("incomplete"));
        }
    }

    #[test]
    fn tactic_shortlist_value_and_sort() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-shortlist.db"));
        let players = (1..=3)
            .map(|uid| {
                let mut p = player_template(uid, &format!("S{uid}"), 150);
                p["positions"] = serde_json::json!({"GK": 18});
                p["preferredFoot"] = serde_json::json!("right");
                p
            })
            .collect();
        ingest_players(&mut conn, players);
        let snapshot_id = current_snapshot_id(&conn);
        for uid in 1..=2 {
            insert_moneyball_row(&conn, snapshot_id, uid, Some(r#"{"goals":10}"#));
        }
        set_role_score(&conn, 1, "goalkeeper_ip", Some(90));
        set_role_score(&conn, 1, "line_holding_keeper_oop", Some(90));
        set_role_score(&conn, 2, "goalkeeper_ip", Some(60));
        set_role_score(&conn, 2, "line_holding_keeper_oop", Some(60));
        set_role_score(&conn, 3, "goalkeeper_ip", Some(80));
        set_role_score(&conn, 3, "line_holding_keeper_oop", Some(80));
        let page = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("shortlist");
        assert_eq!(page.total, 2);
        assert_eq!(page.players[0].uid, 1);
        assert_eq!(page.players[1].uid, 2);
        assert_eq!(
            page.players[0]
                .dynamic_values
                .get("tactic_current.goalkeeper"),
            Some(&Some(DynamicValue::Integer(90)))
        );
        assert_eq!(
            page.players[1]
                .dynamic_values
                .get("tactic_current.goalkeeper"),
            Some(&Some(DynamicValue::Integer(60)))
        );
        // Nulls last check via low familiarity
        conn.execute(
            r#"UPDATE players SET positions_json = '{"GK": 10}' WHERE snapshot_id = ?1 AND uid = 1"#,
            [snapshot_id],
        )
        .expect("make null");
        let page2 = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::Shortlist,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("shortlist nulls last");
        assert_eq!(page2.players.last().unwrap().uid, 1);
    }

    #[test]
    fn tactic_moneyball_filtered_pool_divergent_from_stored_fullcsv() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-filtered-divergent.db"));
        let players = (1..=4)
            .map(|uid| {
                let mut p = player_template(uid, &format!("P{uid}"), 150);
                p["positions"] = serde_json::json!({"GK": 18, "ST": 18, "MC": 18, "DC": 18});
                p["preferredFoot"] = serde_json::json!("right");
                p["age"] = serde_json::json!(26);
                p
            })
            .collect();
        ingest_players(&mut conn, players);
        let snapshot_id = current_snapshot_id(&conn);
        let catalog = builtin_catalog().expect("catalog");
        let gk_ip_def = catalog
            .definitions
            .iter()
            .find(|d| {
                d.attribute_role_id.as_deref() == Some("goalkeeper_ip")
                    && d.position_tags.contains(&"GK".to_string())
            })
            .unwrap();
        let gk_oop_def = catalog
            .definitions
            .iter()
            .find(|d| {
                d.attribute_role_id.as_deref() == Some("line_holding_keeper_oop")
                    && d.position_tags.contains(&"GK".to_string())
            })
            .unwrap();
        let keys: Vec<String> = gk_ip_def
            .metrics
            .iter()
            .chain(gk_oop_def.metrics.iter())
            .map(|m| m.key.clone())
            .collect();
        for (uid, raw) in [(1, 0.0), (2, 10.0), (3, 20.0), (4, 30.0)] {
            let mut stats = std::collections::BTreeMap::new();
            let mut perc = std::collections::BTreeMap::new();
            for k in &keys {
                stats.insert(k.clone(), serde_json::json!(raw));
                perc.insert(k.clone(), serde_json::json!(99));
            }
            insert_moneyball_statistics(
                &conn,
                snapshot_id,
                uid,
                &serde_json::to_string(&stats).unwrap(),
                Some(&serde_json::to_string(&perc).unwrap()),
            );
        }
        // FullCsv uses stored percentiles (99) -> all tactic scores equal high (99 percentile -> high role score)
        let page_full = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::FullCsv,
            },
        )
        .expect("full");
        // All should be numeric and similar (stored 99), order by UID
        assert!(page_full.players.iter().all(|p| p
            .dynamic_values
            .get("tactic_current.goalkeeper")
            .unwrap()
            .is_some()));
        assert_eq!(
            page_full.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
        // Filtered: cohort filtered by raw statistic >5 (excludes uid1), tactic scores computed from filtered cohort percentiles -> uid2 lowest percentile 0, uid4 highest 100
        // Use a moneyball numeric filter via moneyball.* key instead
        let ast2 = parse_filter_ast(
            vec![FilterRule {
                field: format!("moneyball.{}", keys[0]),
                op: "gt".to_string(),
                value: FilterValue::Integer(5),
            }],
            None,
        )
        .expect("filter");
        let page_filtered = search_players_in_view(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse_for_moneyball("tactic_current.goalkeeper", true)
                    .expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: Some(&ast2),
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                view: SearchView::Moneyball,
                comparison_pool: ComparisonPool::Filtered,
            },
        )
        .expect("filtered");
        assert_eq!(page_filtered.total, 3);
        // Filtered scores must be tiered: uid4 highest, uid2 lowest, proving pool selection not stored percentiles
        let scores: Vec<Option<i64>> = page_filtered
            .players
            .iter()
            .map(|p| {
                p.dynamic_values
                    .get("tactic_current.goalkeeper")
                    .and_then(|v| v.as_ref())
                    .and_then(|v| match v {
                        DynamicValue::Integer(i) => Some(*i),
                        _ => None,
                    })
            })
            .collect();
        assert!(
            scores[0].unwrap() > scores[2].unwrap(),
            "highest vs lowest must differ"
        );
        assert_eq!(page_filtered.players[0].uid, 4);
        assert_eq!(page_filtered.players[2].uid, 2);
        assert_ne!(
            page_filtered.players[0]
                .dynamic_values
                .get("tactic_current.goalkeeper"),
            page_full
                .players
                .iter()
                .find(|p| p.uid == 4)
                .unwrap()
                .dynamic_values
                .get("tactic_current.goalkeeper"),
            "filtered vs full must diverge for same player"
        );
    }

    #[test]
    fn suggest_caps_limit_and_escapes_like_wildcards() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("suggest-cap.db"));
        let players = (1..=5)
            .map(|index| player_template(index, &format!("Pat {index}"), 100 + index as i64))
            .collect();
        ingest_players(&mut conn, players);

        let hits = suggest_players(&conn, "Pat", 2).expect("suggest capped");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].name, "Pat 5");
        assert_eq!(hits[1].name, "Pat 4");

        // Literal % in the query must not match every name via LIKE.
        let wild = suggest_players(&conn, "%", DEFAULT_SUGGEST_LIMIT).expect("suggest wildcard");
        assert!(wild.is_empty());
    }
}
