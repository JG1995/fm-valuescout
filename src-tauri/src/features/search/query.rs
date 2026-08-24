use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};

use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use crate::features::moneyball::percentile::{calculate_percentiles, MoneyballNumericStatistics};
use crate::features::moneyball::{role_catalog::builtin_catalog, role_score::score_role};
use crate::features::player_metrics::club_dna::SCORE_MODEL_VERSION;
use crate::features::player_metrics::{
    potential_cache::{materialize_player_roles, materialize_snapshot_roles},
    resolver::{
        parse_requested_fields_for_moneyball, read_dynamic_value, ClubDnaSqlBindings, MetricField,
    },
};

use super::filter::{
    compile_filters, compile_filters_for_moneyball, compile_filters_with_club_dna,
    moneyball_role_ids_from_ast, moneyball_role_rules_match, potential_role_ids_from_ast,
    without_moneyball_role_rules, CombineMode, CompiledFilter, FilterAst,
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

    fn moneyball_role_id(&self) -> Option<&str> {
        match self {
            Self::Dynamic(field) => field.moneyball_role_id(),
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
                crate::features::player_metrics::resolver::is_moneyball_search_field(field.id())
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
    if request_uses_moneyball_role(&request) {
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
                SearchView::General if club_dna_filter => {
                    compile_filters_with_club_dna(ast, 4, ClubDnaSqlBindings::new(2, 3))?
                }
                SearchView::General => compile_filters(ast, 2)?,
                SearchView::Moneyball => compile_filters_for_moneyball(ast, 2, true)?,
            };
            if compiled.sql.is_empty() {
                None
            } else {
                Some(compiled)
            }
        }
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

    let mut full_snapshot_roles = filter_ast
        .map(potential_role_ids_from_ast)
        .transpose()?
        .unwrap_or_default();
    if let Some(role_id) = sort_by.potential_role_id() {
        add_role_once(&mut full_snapshot_roles, role_id);
    }
    if !full_snapshot_roles.is_empty() {
        materialize_snapshot_roles(conn, snapshot_id, &full_snapshot_roles)?;
    }

    let from_sql = match view {
        SearchView::General => "FROM players".to_string(),
        SearchView::Moneyball => "FROM players INNER JOIN player_moneyball_stats moneyball ON moneyball.snapshot_id = players.snapshot_id AND moneyball.player_uid = players.uid AND moneyball.percentiles_json IS NOT NULL".to_string(),
    };
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
    let filter_bind_values = bind_values.clone();
    let mut select_bind_values = bind_values.clone();
    if club_dna_requested && !club_dna_filter {
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
    let sort_expression = sort_by.sql_expr(club_dna_bindings);
    let order_sql = if sort_by.is_club_dna() {
        format!(
            "ORDER BY ({sort_expression}) IS NULL ASC, {sort_expression} {}, players.uid ASC",
            sort_dir.sql_keyword()
        )
    } else {
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
    let potential_display_roles = potential_role_ids(&dynamic_fields);
    if !potential_display_roles.is_empty() {
        let page_uids = query_page_uids(
            conn,
            &from_sql,
            &where_sql,
            &select_bind_values,
            &order_sql,
            limit,
            offset,
        )?;
        materialize_player_roles(conn, snapshot_id, &page_uids, &potential_display_roles)?;
    }

    let limit_index = select_bind_values.len() + 1;
    let offset_index = select_bind_values.len() + 2;
    select_bind_values.push(Value::Integer(limit));
    select_bind_values.push(Value::Integer(offset));

    for field in &dynamic_fields {
        select_sql.push_str(", ");
        select_sql.push_str(&field.sql_expression_with_club_dna("players", club_dna_bindings));
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
        .filter(|field| field.moneyball_role_id().is_none())
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
    let role_metric_keys = definitions
        .iter()
        .flat_map(|definition| definition.metrics.iter().map(|metric| metric.key.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

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

    let order_sql = if sort_by.moneyball_role_id().is_some() {
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
    let role_metric_start = 11
        + sql_dynamic_fields.len()
        + if include_persisted_percentiles {
            moneyball_fields.len()
        } else {
            0
        };
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

fn potential_role_ids(fields: &[MetricField]) -> Vec<String> {
    let mut role_ids = Vec::new();
    for field in fields {
        if let Some(role_id) = field.potential_role_id() {
            add_role_once(&mut role_ids, role_id);
        }
    }
    role_ids
}

fn query_page_uids(
    conn: &Connection,
    from_sql: &str,
    where_sql: &str,
    bind_values: &[Value],
    order_sql: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<i64>, String> {
    let limit_index = bind_values.len() + 1;
    let offset_index = bind_values.len() + 2;
    let sql = format!(
        "SELECT players.uid {from_sql} WHERE {where_sql} {order_sql} LIMIT ?{limit_index} OFFSET ?{offset_index}"
    );
    let mut values = bind_values.to_vec();
    values.push(Value::Integer(limit));
    values.push(Value::Integer(offset));
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), |row| row.get(0))
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn parse_nationalities(json: &str) -> Result<Vec<String>, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid nationalities_json: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::moneyball::role_catalog::builtin_catalog;
    use crate::features::player_metrics::club_dna::SCORE_MODEL_VERSION;
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
                player_template(10, "Present low", 100),
                player_template(20, "Present high", 100),
                player_template(30, "Computed null", 100),
                player_template(40, "Missing", 100),
                player_template(50, "Stale definition", 100),
                player_template(60, "Stale model", 100),
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
        let score_rows_before = club_dna_score_rows(&conn);
        let requested_fields = vec!["club_dna".to_string()];

        let page = search_players(
            &conn,
            0,
            6,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Asc,
            None,
            &requested_fields,
        )
        .expect("search Club DNA ascending");
        assert_eq!(page.total, 6);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [10, 20, 30, 40, 50, 60]
        );
        assert_eq!(
            page.players[0].dynamic_values.get("club_dna"),
            Some(&Some(DynamicValue::Integer(20)))
        );
        assert_eq!(page.players[2].dynamic_values.get("club_dna"), Some(&None));
        assert_eq!(page.players[3].dynamic_values.get("club_dna"), Some(&None));
        assert_eq!(page.players[4].dynamic_values.get("club_dna"), Some(&None));
        assert_eq!(page.players[5].dynamic_values.get("club_dna"), Some(&None));

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
        assert_eq!(bounded_page.total, 6);
        assert_eq!(
            bounded_page
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [20, 30]
        );
        assert_eq!(club_dna_score_rows(&conn), score_rows_before);
    }

    #[test]
    fn searches_club_dna_descending_filters_every_operator_and_keeps_scores_read_only() {
        let (_temp_dir, conn) = seed_club_dna_query_players();
        let score_rows_before = club_dna_score_rows(&conn);
        let expected = [
            ("gt", vec![20]),
            ("lt", vec![10]),
            ("eq", vec![10]),
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
            [10, 40]
        );

        let descending = search_players(
            &conn,
            0,
            6,
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
            [20, 10, 30, 40, 50, 60]
        );
        assert_eq!(club_dna_score_rows(&conn), score_rows_before);
    }

    #[test]
    fn searches_missing_club_dna_definition_as_a_uid_stable_all_null_page() {
        let (_temp_dir, conn) = seed_club_dna_query_players();
        let score_rows_before = club_dna_score_rows(&conn);
        conn.execute("DELETE FROM club_dna_definitions", [])
            .expect("remove Club DNA definition");

        let page = search_players(
            &conn,
            0,
            6,
            SortField::parse("club_dna").expect("parse Club DNA sort"),
            SortDir::Desc,
            None,
            &["club_dna".to_string()],
        )
        .expect("search without Club DNA definition");
        assert_eq!(page.total, 6);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [10, 20, 30, 40, 50, 60]
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
    fn potential_role_filter_materializes_the_full_snapshot_and_reuses_cached_rows() {
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

        let rules = vec![filter_rule(
            "potential_role.line_holding_keeper_oop",
            "gt",
            FilterValue::Integer(0),
        )];
        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            rules.clone(),
            None,
        )
        .expect("potential role filter");

        assert_eq!(page.total, 1);
        assert_eq!(page.players[0].name, "Potential target");
        let cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores
                 WHERE role_id = 'line_holding_keeper_oop'",
                [],
                |row| row.get(0),
            )
            .expect("count materialized rows");
        assert_eq!(cache_rows, 2, "the filter needs the whole snapshot");
        let null_scores: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores
                 WHERE role_id = 'line_holding_keeper_oop' AND score IS NULL",
                [],
                |row| row.get(0),
            )
            .expect("count cached null scores");
        assert_eq!(null_scores, 1);

        conn.execute(
            "UPDATE players SET attributes_json = '{invalid JSON}'
             WHERE snapshot_id = (SELECT id FROM snapshots WHERE is_current = 1 LIMIT 1)",
            [],
        )
        .expect("make a recalculation observable");

        search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            rules,
            None,
        )
        .expect("repeat potential role filter");
        let repeated_cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores
                 WHERE role_id = 'line_holding_keeper_oop'",
                [],
                |row| row.get(0),
            )
            .expect("count reused cache rows");
        assert_eq!(repeated_cache_rows, 2);
    }

    #[test]
    fn invalid_filter_rules_do_not_materialize_potential_cache_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("invalid-potential-filter.db"));
        ingest_players(&mut conn, vec![player_template(1, "No cache work", 150)]);

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
        let cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores",
                [],
                |row| row.get(0),
            )
            .expect("count cache rows");
        assert_eq!(cache_rows, 0);
    }

    #[test]
    fn potential_role_filters_materialize_each_requested_role_and_replace_stale_rows() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("potential-role-stale.db"));
        ingest_players(
            &mut conn,
            vec![player_with_deep_fields(
                1,
                "Projected keeper",
                100,
                DeepPlayerFields {
                    nationalities: json!(["ENG"]),
                    positions: json!({ "GK": 20 }),
                    attributes: json!({
                        "Positioning": 10,
                        "Concentration": 10,
                        "RushingOut": 10,
                        "Anticipation": 10,
                        "Decisions": 10,
                    }),
                    hidden: json!({}),
                    personality: json!({}),
                },
            )],
        );
        let rules = vec![
            filter_rule(
                "potential_role.line_holding_keeper_oop",
                "gt",
                FilterValue::Integer(0),
            ),
            filter_rule(
                "potential_role.sweeper_keeper_oop",
                "gt",
                FilterValue::Integer(0),
            ),
        ];

        let page = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            rules.clone(),
            Some("and"),
        )
        .expect("materialize multiple potential roles");
        assert_eq!(page.total, 1);
        let cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores",
                [],
                |row| row.get(0),
            )
            .expect("count cached roles");
        assert_eq!(cache_rows, 2);

        conn.execute(
            "UPDATE player_potential_role_scores
             SET score = 99, projection_model_version = 2
             WHERE role_id = 'line_holding_keeper_oop'",
            [],
        )
        .expect("mark cache row stale");

        search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            rules,
            Some("and"),
        )
        .expect("replace stale cache row");
        let (score, version): (Option<i64>, i64) = conn
            .query_row(
                "SELECT score, projection_model_version
                 FROM player_potential_role_scores
                 WHERE role_id = 'line_holding_keeper_oop'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read refreshed cache row");
        assert!(score.is_some_and(|value| value < 99));
        assert_eq!(version, 1);
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
                "AMC, MC, AML, AMR, WBL".to_string()
            )))
        );
    }

    #[test]
    fn potential_display_materializes_only_requested_page_players() {
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
        let first_page_cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores",
                [],
                |row| row.get(0),
            )
            .expect("count first page cache rows");
        assert_eq!(first_page_cache_rows, 1);

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
        let second_page_cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores",
                [],
                |row| row.get(0),
            )
            .expect("count second page cache rows");
        assert_eq!(second_page_cache_rows, 2);
    }

    #[test]
    fn potential_sort_materializes_the_full_search_cohort_before_ordering() {
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
        let cache_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM player_potential_role_scores",
                [],
                |row| row.get(0),
            )
            .expect("count sorted cohort cache rows");
        assert_eq!(cache_rows, 2);
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
