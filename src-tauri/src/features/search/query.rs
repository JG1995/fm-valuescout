use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use crate::db::result_cache::{Cancellation, ResultCache, TableRegistry, TableTicket};
use crate::db::Db;

use crate::features::moneyball::columns::StatisticColumns;
use crate::features::moneyball::comparison::{
    scan_in_parallel, ComparisonGroup, ComparisonGroupsCache, GroupsSource,
};
use crate::features::moneyball::kept_stats::{
    kept_stats_snapshots, stats_join_sql, KeptStats, STATS_DIVISION_SQL,
};
use crate::features::moneyball::role_catalog::{builtin_catalog, RoleDefinition};
use crate::features::planner::fit::{lane_statistic_keys, FULL_FAMILIARITY};
use crate::features::planner::tactic::{base_position, read_tactic, TacticLane};
use crate::features::player_metrics::{
    club_dna::SCORE_MODEL_VERSION,
    compact::{
        assert_read_models_complete, player_current_column, player_metrics_join,
        player_potential_column, PLAYER_METRICS_ALIAS,
    },
    resolver::{
        parse_search_requested_fields, qualified_season_stats_sql, read_dynamic_value,
        ClubDnaSqlBindings, MetricField, TacticGroup,
    },
};

use super::filter::{
    ast_reads_season_stats, compile_filters, compile_filters_with_club_dna,
    current_role_ids_from_ast, has_post_cohort_rules, moneyball_percentile_keys_from_ast,
    moneyball_role_ids_from_ast, post_cohort_rules_match, potential_role_ids_from_ast,
    without_post_cohort_rules, CombineMode, FilterAst,
};
use super::season_scores::{lane_group, tactic_lane, PlayerFit, SeasonSources};

pub use crate::features::player_metrics::resolver::DynamicValue;

pub const DEFAULT_PAGE_LIMIT: usize = 50;
pub const MAX_PAGE_LIMIT: usize = 200;

/// Whom a statistic's percentile ranks each player against: the peers of his own best
/// position's family, or one group of one or two families for everyone. Moneyball Scores always
/// rank within their roles' families and their slots' groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Own,
    Group(ComparisonGroup),
}

impl Comparison {
    /// `own`, a role catalog family ID, or two family IDs joined by `+`.
    pub fn parse(value: &str) -> Result<Self, String> {
        if value == "own" {
            return Ok(Self::Own);
        }
        ComparisonGroup::parse(value)
            .map(Self::Group)
            .ok_or_else(|| format!("unknown Moneyball comparison: {value}"))
    }
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
    /// Any other Player Search field (`role.*`, `attr.*`, statistics, Moneyball Scores, …).
    Dynamic(MetricField),
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
            other => Ok(Self::Dynamic(MetricField::parse_search(other)?)),
        }
    }

    pub(super) fn sql_expr(&self, club_dna_bindings: Option<ClubDnaSqlBindings>) -> String {
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

    /// Whether the sort is a Moneyball Score, which Rust computes and SQL cannot order by.
    fn is_moneyball_score(&self) -> bool {
        matches!(self, Self::Dynamic(field) if field.is_moneyball_score())
    }

    /// Whether the sort reads the season statistics row aliased `moneyball`.
    fn reads_season_stats(&self) -> bool {
        matches!(self, Self::Dynamic(field) if field.reads_season_stats())
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

    pub(super) fn sql_keyword(self) -> &'static str {
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
    pub current_club_uid: Option<i64>,
    pub division: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub market_value_gbp: Option<i64>,
    /// Values for requested non-basic metric fields (field id → nullable cell).
    pub dynamic_values: BTreeMap<String, Option<DynamicValue>>,
    /// The requested statistics' percentiles, for a player with a qualified sample.
    pub moneyball_percentiles: BTreeMap<String, Option<u8>>,
    /// The in-game date of the kept snapshot whose statistics the player's season values come
    /// from, when this season has too few minutes; `None` for this season's.
    pub moneyball_stats_date: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchPlayersPage {
    pub players: Vec<PlayerSummary>,
    pub total: i64,
    /// `NoShortlist` means the requested shortlist is not one of the current snapshot's
    /// save's shortlists, so the frontend drops its selection. A shortlist whose players are
    /// not in the snapshot pages as an empty `Ready`.
    pub state: SearchPlayersPageState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchPlayersPageState {
    Ready,
    NoCurrentSnapshot,
    NoShortlist,
}

pub struct SearchPlayersRequest<'a> {
    pub offset: usize,
    pub limit: usize,
    pub sort_by: SortField,
    pub sort_dir: SortDir,
    pub filter_ast: Option<&'a FilterAst>,
    pub requested_fields: &'a [String],
    /// Whom statistic percentiles rank each player against.
    pub comparison: Comparison,
    /// Restrict the results to this player shortlist of the save.
    pub shortlist_id: Option<i64>,
}

/// One table query's complete row order, computed once per data generation.
pub struct PlayerOrder {
    /// The data generation of the reader that computed it.
    generation: u64,
    state: SearchPlayersPageState,
    snapshot_id: i64,
    uids: Vec<i64>,
}

/// Player Search's computed orders, keyed by [`search_query_fingerprint`].
pub type PlayerOrderCache = ResultCache<PlayerOrder>;

/// Identifies a table query by every input that changes its rows or their order. The offset,
/// limit, and requested fields only choose which rows and columns a window shows.
pub fn search_query_fingerprint(request: &SearchPlayersRequest<'_>) -> String {
    format!(
        "{:?}|{:?}|{:?}|{:?}|{:?}",
        request.filter_ast,
        request.sort_by,
        request.sort_dir,
        request.comparison,
        request.shortlist_id,
    )
}

/// Serves one window from its query's computed order, computing the order on a miss. With a
/// ticket, the computation is its table's in `registry`, so a newer request for the table can
/// supersede it. Returns the same page as [`search_players_uncached`].
pub fn search_players(
    db: &Db,
    orders: &PlayerOrderCache,
    groups: &ComparisonGroupsCache,
    registry: &TableRegistry,
    ticket: Option<&TableTicket>,
    request: SearchPlayersRequest<'_>,
) -> Result<SearchPlayersPage, String> {
    let fields = window_fields(request.requested_fields)?;
    let fingerprint = search_query_fingerprint(&request);
    let mut generation = db.generation();
    loop {
        let order = orders.get_or_compute(&fingerprint, generation, || {
            let reader = db.read()?;
            let order = registry.run(ticket, &reader, |cancellation| {
                compute_order(
                    &reader,
                    reader.generation(),
                    &GroupsSource::Cached(groups, reader.generation()),
                    &request,
                    cancellation,
                )
            })?;
            Ok::<_, String>((reader.generation(), order))
        })?;
        let reader = db.read()?;
        if reader.generation() > order.generation {
            // Data changed since the order was read, so the window reads a fresh order.
            generation = reader.generation();
            continue;
        }
        return read_window(
            &reader,
            &GroupsSource::Cached(groups, reader.generation()),
            &order,
            &request,
            &fields,
        );
    }
}

/// One window of a table query, computed without a cache.
#[cfg(test)]
pub fn search_players_uncached(
    conn: &Connection,
    request: SearchPlayersRequest<'_>,
) -> Result<SearchPlayersPage, String> {
    let fields = window_fields(request.requested_fields)?;
    let order = compute_order(
        conn,
        0,
        &GroupsSource::Uncached,
        &request,
        &Cancellation::never(),
    )?;
    read_window(conn, &GroupsSource::Uncached, &order, &request, &fields)
}

/// A request's validated fields beyond the basic columns every row reads.
fn window_fields(requested_fields: &[String]) -> Result<Vec<MetricField>, String> {
    Ok(parse_search_requested_fields(requested_fields)?
        .into_iter()
        .filter(|field| !field.is_basic_table_field())
        .collect())
}

/// Computes a query's complete row order: in SQL, or in Rust when a rule or the sort is a
/// Moneyball Score or a percentile.
fn compute_order(
    conn: &Connection,
    generation: u64,
    groups: &GroupsSource<'_>,
    request: &SearchPlayersRequest<'_>,
    cancellation: &Cancellation,
) -> Result<PlayerOrder, String> {
    #[cfg(test)]
    tests::ORDER_COMPUTATIONS.with(|count| count.set(count.get() + 1));
    let empty = |state| PlayerOrder {
        generation,
        state,
        snapshot_id: 0,
        uids: Vec::new(),
    };
    let Some(snapshot_id) = active_current_snapshot_id(conn)? else {
        return Ok(empty(SearchPlayersPageState::NoCurrentSnapshot));
    };
    if requested_shortlist_missing(conn, snapshot_id, request.shortlist_id)? {
        return Ok(empty(SearchPlayersPageState::NoShortlist));
    }
    let scored = request.filter_ast.is_some_and(has_post_cohort_rules)
        || request.sort_by.is_moneyball_score();
    let uids = if scored {
        scored_order(conn, snapshot_id, groups, request, cancellation)?
    } else {
        sql_order(conn, snapshot_id, request, cancellation)?
    };
    Ok(PlayerOrder {
        generation,
        state: SearchPlayersPageState::Ready,
        snapshot_id,
        uids,
    })
}

/// The FROM, WHERE, and ORDER BY of an order's query: the snapshot `?1`, the shortlist, the
/// SQL rules, and the joins they and the sort read.
struct OrderQuery {
    from_sql: String,
    /// The compiled SQL rules, if any.
    filter_sql: Option<String>,
    order_sql: String,
    bind_values: Vec<Value>,
}

/// Builds an order's query from its SQL rules `sql_ast`. A Moneyball Score sort orders by UID,
/// for Rust to sort stably. With `season_stats`, it joins the statistics row whatever the rules
/// and sort read.
fn order_query(
    conn: &Connection,
    snapshot_id: i64,
    request: &SearchPlayersRequest<'_>,
    sql_ast: Option<&FilterAst>,
    kept_ids: &[i64],
    season_stats: bool,
) -> Result<OrderQuery, String> {
    let sort_by = &request.sort_by;
    let club_dna_filter =
        sql_ast.is_some_and(|ast| ast.rules.iter().any(|rule| rule.field == "club_dna"));
    let compiled = match sql_ast {
        None => None,
        Some(ast) => {
            let compiled = if club_dna_filter {
                compile_filters_with_club_dna(ast, 4, ClubDnaSqlBindings::new(2, 3))?
            } else {
                compile_filters(ast, 2)?
            };
            (!compiled.sql.is_empty()).then_some(compiled)
        }
    };
    let (uses_current_role_metrics, uses_potential_role_metrics) =
        role_metric_requirements(sql_ast, sort_by, &[])?;
    assert_read_models_complete(
        conn,
        snapshot_id,
        uses_current_role_metrics,
        uses_potential_role_metrics,
    )?;

    let tactic = (sort_by.is_tactic() && !sort_by.is_moneyball_score())
        .then(|| read_tactic(conn, snapshot_save_id(conn, snapshot_id)?))
        .transpose()?;
    let club_dna_bindings = (club_dna_filter || sort_by.is_club_dna()).then(|| {
        if club_dna_filter {
            ClubDnaSqlBindings::new(2, 3)
        } else {
            let filter_params = compiled.as_ref().map_or(0, |filter| filter.params.len());
            ClubDnaSqlBindings::new(filter_params + 2, filter_params + 3)
        }
    });
    let club_dna_definition_version = club_dna_bindings
        .is_some()
        .then(|| current_club_dna_definition_version(conn, snapshot_id))
        .transpose()?
        .flatten();
    let club_dna_sort = sort_by.club_dna_sort_identity(club_dna_definition_version);

    // Without statistics, SQLite seeks an unfiltered snapshot through `idx_players_snapshot_ca`
    // and visits its rows in CA order, one random table read each. Unless a covering index
    // already holds the sort key and UID, an unfiltered order reads the snapshot in primary-key
    // order, which follows storage order. Filters keep the planner's choice, which can use
    // their own indexes.
    let reads_in_key_order = compiled.is_none()
        && !matches!(
            sort_by,
            SortField::Pa | SortField::Age | SortField::Value | SortField::Club
        );
    let mut from_sql = if let Some(shortlist_id) = request.shortlist_id {
        shortlist_from_sql(shortlist_id)
    } else if reads_in_key_order {
        "FROM players INDEXED BY sqlite_autoindex_players_1".to_string()
    } else {
        "FROM players".to_string()
    };
    if uses_current_role_metrics || uses_potential_role_metrics {
        from_sql.push_str(&player_metrics_join(
            "players",
            uses_current_role_metrics,
            uses_potential_role_metrics,
        ));
    }
    if season_stats || sql_ast.is_some_and(ast_reads_season_stats) || sort_by.reads_season_stats() {
        from_sql.push_str(&stats_join_sql("players", kept_ids));
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
    let mut bind_values = vec![Value::Integer(snapshot_id)];
    if club_dna_filter {
        bind_values.push(club_dna_definition_version.map_or(Value::Null, Value::Integer));
        bind_values.push(Value::Integer(SCORE_MODEL_VERSION));
    }
    if let Some(compiled) = &compiled {
        bind_values.extend(compiled.params.iter().cloned());
    }
    if let Some(identity) = club_dna_sort.filter(|_| !club_dna_filter) {
        bind_values.push(
            identity
                .definition_version
                .map_or(Value::Null, Value::Integer),
        );
        bind_values.push(Value::Integer(identity.score_model_version));
    }

    // Whitelisted expr + dir only — never interpolate raw client strings.
    let dir = request.sort_dir.sql_keyword();
    let order_sql = if sort_by.is_moneyball_score() {
        "ORDER BY players.uid ASC".to_string()
    } else if club_dna_sort.is_some() {
        format!(
            "ORDER BY club_dna_sort.score IS NULL ASC, club_dna_sort.score {dir}, players.uid ASC"
        )
    } else if let Some(tactic) = &tactic {
        let lane_id = sort_by.tactic_lane_id().expect("tactic lane");
        let lane = tactic
            .lanes
            .iter()
            .find(|l| l.lane_id == lane_id)
            .ok_or_else(|| format!("unknown tactic lane: {lane_id}"))?;
        let group = sort_by.tactic_group().expect("tactic group");
        let expr = tactic_sql_expression(lane, group, "players");
        format!("ORDER BY ({expr} IS NULL) ASC, {expr} {dir}, players.uid ASC")
    } else if sort_by.reads_season_stats() {
        // Most players have no qualified sample; they follow those who do in either direction.
        let expr = sort_by.sql_expr(None);
        format!("ORDER BY ({expr}) IS NULL ASC, {expr} {dir}, players.uid ASC")
    } else {
        format!(
            "ORDER BY {} {dir}, players.uid ASC",
            sort_by.sql_expr(club_dna_bindings)
        )
    };
    Ok(OrderQuery {
        from_sql,
        filter_sql: compiled.map(|compiled| compiled.sql),
        order_sql,
        bind_values,
    })
}

/// A query's order when SQL can sort and filter every row.
fn sql_order(
    conn: &Connection,
    snapshot_id: i64,
    request: &SearchPlayersRequest<'_>,
    cancellation: &Cancellation,
) -> Result<Vec<i64>, String> {
    let reads_stats = request.filter_ast.is_some_and(ast_reads_season_stats)
        || request.sort_by.reads_season_stats();
    let kept_ids = if reads_stats {
        kept_ids(&kept_stats_snapshots(conn, snapshot_id)?)
    } else {
        Vec::new()
    };
    let query = order_query(
        conn,
        snapshot_id,
        request,
        request.filter_ast,
        &kept_ids,
        false,
    )?;
    let where_sql = match &query.filter_sql {
        Some(filter) => format!("players.snapshot_id = ?1 AND {filter}"),
        None => "players.snapshot_id = ?1".to_string(),
    };
    #[cfg(test)]
    if let Some(hook) = tests::BEFORE_ORDER_QUERY.with(|hook| hook.borrow_mut().take()) {
        hook();
    }
    cancellation.check()?;
    conn.prepare(&format!(
        "SELECT players.uid {} WHERE {where_sql} {}",
        query.from_sql, query.order_sql
    ))
    .map_err(|error| error.to_string())?
    .query_map(params_from_iter(query.bind_values.iter()), |row| row.get(0))
    .map_err(|error| error.to_string())?
    .collect::<Result<Vec<i64>, _>>()
    .map_err(|error| error.to_string())
}

/// One row of a scored order's query.
struct ScoredRow {
    /// The row's place in the SQL order.
    index: usize,
    uid: i64,
    /// Whether the SQL rules keep the row; under OR the Moneyball rules can keep it as well.
    sql_match: bool,
    /// The snapshot of the statistics row, when the player has a qualified sample.
    source_id: Option<i64>,
    /// The statistics the request reads, when the player has a qualified sample.
    statistics: Option<Box<[f64]>>,
    division: Option<String>,
    fit: Option<PlayerFit>,
}

/// A query's order when a rule or the sort is a Moneyball Score or a statistic's percentile.
/// SQL selects the rows its rules allow; Rust scores the players with a qualified sample,
/// applies the Moneyball rules, and sorts by a Moneyball Score.
fn scored_order(
    conn: &Connection,
    snapshot_id: i64,
    groups: &GroupsSource<'_>,
    request: &SearchPlayersRequest<'_>,
    cancellation: &Cancellation,
) -> Result<Vec<i64>, String> {
    let catalog = builtin_catalog()?;
    let definition = |role_id: &str| {
        catalog
            .definitions
            .iter()
            .find(|definition| definition.id == role_id)
            .ok_or_else(|| format!("unknown Moneyball role: {role_id}"))
    };
    let post_ast = request.filter_ast.filter(|ast| has_post_cohort_rules(ast));
    let sql_ast = request
        .filter_ast
        .map(without_post_cohort_rules)
        .transpose()?
        .flatten();
    // Under OR, a player kept by an SQL rule stays whatever his Moneyball values.
    let mixed_or = post_ast.is_some_and(|ast| ast.combine == CombineMode::Or) && sql_ast.is_some();
    let sort_role = request
        .sort_by
        .moneyball_role_id()
        .map(definition)
        .transpose()?;
    let filter_roles = post_ast
        .map(moneyball_role_ids_from_ast)
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .map(|role_id| Ok((definition(&role_id)?, role_id)))
        .collect::<Result<Vec<(&RoleDefinition, String)>, String>>()?;
    let percentile_keys = post_ast
        .map(moneyball_percentile_keys_from_ast)
        .unwrap_or_default();
    let sort_tactic = match &request.sort_by {
        SortField::Dynamic(field) if field.tactic_group() == Some(TacticGroup::Moneyball) => {
            Some(read_tactic(conn, snapshot_save_id(conn, snapshot_id)?)?)
        }
        _ => None,
    };
    let sort_lane = match (&request.sort_by, &sort_tactic) {
        (SortField::Dynamic(field), Some(tactic)) => Some(
            tactic_lane(field, tactic)
                .ok_or_else(|| format!("unknown tactic lane: {}", field.id()))?,
        ),
        _ => None,
    };

    let mut keys = BTreeSet::new();
    for definition in sort_role
        .iter()
        .chain(filter_roles.iter().map(|(definition, _)| definition))
    {
        keys.extend(definition.metrics.iter().map(|metric| metric.key.clone()));
    }
    keys.extend(percentile_keys.iter().cloned());
    if let Some(lane) = sort_lane {
        keys.extend(lane_statistic_keys(lane));
    }
    let needed = StatisticColumns::of_keys(keys.iter().map(String::as_str))?;
    let kept = kept_stats_snapshots(conn, snapshot_id)?;
    let groups_to_rank = match request.comparison {
        Comparison::Group(group) => Some(group),
        Comparison::Own => None,
    }
    .into_iter()
    .chain(sort_lane.and_then(lane_group))
    .collect::<Vec<_>>();
    let sources = SeasonSources::load(conn, groups, snapshot_id, &kept, &groups_to_rank)?;
    cancellation.check()?;

    let query = order_query(
        conn,
        snapshot_id,
        request,
        sql_ast.as_ref(),
        &kept_ids(&kept),
        true,
    )?;
    let qualified = qualified_season_stats_sql();
    let (match_column, mut where_sql) = match (&query.filter_sql, mixed_or) {
        (Some(filter), true) => (
            format!("CASE WHEN {filter} THEN 1 ELSE 0 END"),
            "players.snapshot_id = ?1".to_string(),
        ),
        (Some(filter), false) => (
            "1".to_string(),
            format!("players.snapshot_id = ?1 AND {filter}"),
        ),
        (None, _) => ("1".to_string(), "players.snapshot_id = ?1".to_string()),
    };
    // Only a qualified sample passes a Moneyball rule, so without an SQL rule under OR to keep
    // him, a player without one is never a row.
    if post_ast.is_some() && !mixed_or {
        where_sql.push_str(&format!(" AND {qualified}"));
    }
    let fit_columns = if sort_lane.is_some() {
        ", players.positions_json, players.preferred_foot"
    } else {
        ""
    };
    let first_statistic_column = if sort_lane.is_some() { 6 } else { 4 };
    let sql = format!(
        "SELECT players.uid, {match_column},
                CASE WHEN {qualified} THEN moneyball.snapshot_id END,
                CASE WHEN {qualified} THEN {STATS_DIVISION_SQL} END{fit_columns}{}
         {} WHERE {where_sql} {}",
        needed.appended_sql("moneyball"),
        query.from_sql,
        query.order_sql
    );
    #[cfg(test)]
    if let Some(hook) = tests::BEFORE_ORDER_QUERY.with(|hook| hook.borrow_mut().take()) {
        hook();
    }
    cancellation.check()?;
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let mut rows = statement
        .query(params_from_iter(query.bind_values.iter()))
        .map_err(|error| error.to_string())?;
    let next_index = Cell::new(0);
    let read = |row: &Row<'_>| {
        let index = next_index.get();
        next_index.set(index + 1);
        let source_id: Option<i64> = row.get(2)?;
        Ok(ScoredRow {
            index,
            uid: row.get(0)?,
            sql_match: row.get::<_, i64>(1)? == 1,
            source_id,
            statistics: source_id
                .map(|_| needed.read(row, first_statistic_column))
                .transpose()?,
            division: row.get(3)?,
            fit: if sort_lane.is_some() {
                Some(PlayerFit::from_columns(
                    &row.get::<_, String>(4)?,
                    row.get(5)?,
                ))
            } else {
                None
            },
        })
    };
    let scan = |row: ScoredRow| -> Result<(usize, i64, bool, Option<u8>), String> {
        let sample = row
            .source_id
            .and_then(|id| sources.get(id))
            .zip(row.statistics.as_ref());
        let league_shift = sample.as_ref().map_or(0, |(source, _)| {
            source.league_shift(row.division.as_deref())
        });
        let role_score = |definition: &RoleDefinition| {
            sample.as_ref().and_then(|(source, statistics)| {
                source.role_score(definition, statistics, league_shift)
            })
        };
        let keep = match post_ast {
            None => true,
            Some(ast) => {
                let scores = filter_roles
                    .iter()
                    .map(|(definition, role_id)| (role_id.clone(), role_score(definition)))
                    .collect();
                let percentiles = percentile_keys
                    .iter()
                    .map(|key| {
                        let percentile = sample.as_ref().and_then(|(source, statistics)| {
                            let group = source.table_group(request.comparison, row.uid)?;
                            source.percentile(group, key, statistics)
                        });
                        (key.clone(), percentile)
                    })
                    .collect();
                let passes = post_cohort_rules_match(ast, &scores, &percentiles)?;
                passes || (mixed_or && row.sql_match)
            }
        };
        let score = match (sort_role, sort_lane) {
            (Some(definition), _) => role_score(definition),
            (None, Some(lane)) => sample.as_ref().and_then(|(source, statistics)| {
                let fit = row.fit.as_ref().expect("a tactic sort reads fit");
                source.lane_score(lane, statistics, fit, league_shift)
            }),
            (None, None) => None,
        };
        Ok((row.index, row.uid, keep, score))
    };
    let mut scored = scan_in_parallel(&mut rows, read, scan)?;
    cancellation.check()?;
    scored.sort_unstable_by_key(|(index, ..)| *index);
    scored.retain(|(_, _, keep, _)| *keep);
    if request.sort_by.is_moneyball_score() {
        // Rows arrive in UID order, so a stable sort breaks ties by UID.
        scored.sort_by(|(_, _, _, left), (_, _, _, right)| {
            compare_role_scores(*left, *right, request.sort_dir)
        });
    }
    Ok(scored.into_iter().map(|(_, uid, ..)| uid).collect())
}

fn kept_ids(kept: &[KeptStats]) -> Vec<i64> {
    kept.iter().map(|kept| kept.snapshot_id).collect()
}

/// Restricts the results to shortlist `shortlist_id`'s players by UID. The ID is a typed
/// integer written as a literal, so it adds no bind and every parameter index stays unchanged.
/// Callers first check that the shortlist belongs to the snapshot's save.
fn shortlist_from_sql(shortlist_id: i64) -> String {
    format!("FROM players INNER JOIN player_shortlist_entries shortlist ON shortlist.player_uid = players.uid AND shortlist.shortlist_id = {shortlist_id}")
}

pub(super) fn active_current_snapshot_id(conn: &Connection) -> Result<Option<i64>, String> {
    conn.query_row(
        "SELECT s.id
         FROM snapshots s
         INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
         WHERE s.is_current = 1
         LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

/// Whether a shortlist is requested that is not one of the snapshot's save's shortlists. Such
/// a request must restrict to nothing rather than to another save's players.
pub(super) fn requested_shortlist_missing(
    conn: &Connection,
    snapshot_id: i64,
    shortlist_id: Option<i64>,
) -> Result<bool, String> {
    let Some(shortlist_id) = shortlist_id else {
        return Ok(false);
    };
    conn.query_row(
        "SELECT NOT EXISTS(
            SELECT 1 FROM player_shortlists
            WHERE id = ?2 AND save_id = (SELECT save_id FROM snapshots WHERE id = ?1)
         )",
        [snapshot_id, shortlist_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

/// Whether a request reads current and potential compact role metrics, through its SQL rules,
/// sort, tactic columns, or requested fields.
fn role_metric_requirements(
    sql_ast: Option<&FilterAst>,
    sort_by: &SortField,
    fields: &[MetricField],
) -> Result<(bool, bool), String> {
    let current_filter_roles = sql_ast
        .map(current_role_ids_from_ast)
        .transpose()?
        .unwrap_or_default();
    let potential_filter_roles = sql_ast
        .map(potential_role_ids_from_ast)
        .transpose()?
        .unwrap_or_default();
    let is_role_tactic = |field: &MetricField| {
        field.is_tactic_field() && field.tactic_group() != Some(TacticGroup::Moneyball)
    };
    let sort_role_tactic = matches!(sort_by, SortField::Dynamic(field) if is_role_tactic(field));
    let mut uses_current = sort_by.current_role_id().is_some()
        || fields.iter().any(|field| field.current_role_id().is_some())
        || !current_filter_roles.is_empty();
    let mut uses_potential = sort_by.potential_role_id().is_some()
        || fields
            .iter()
            .any(|field| field.potential_role_id().is_some())
        || !potential_filter_roles.is_empty();
    let uses_tactic_current =
        fields.iter().any(MetricField::is_tactic_current) || sort_by.is_tactic_current();
    let uses_tactic_potential = fields
        .iter()
        .any(|field| is_role_tactic(field) && !field.is_tactic_current())
        || (sort_role_tactic && !sort_by.is_tactic_current());
    if uses_tactic_current || uses_tactic_potential {
        uses_current = true;
        uses_potential |= uses_tactic_potential;
    }
    Ok((uses_current, uses_potential))
}

/// Reads the display columns and requested fields of one window of a computed order, in the
/// order's sequence, and computes its Moneyball values from each player's qualified sample.
fn read_window(
    conn: &Connection,
    groups: &GroupsSource<'_>,
    order: &PlayerOrder,
    request: &SearchPlayersRequest<'_>,
    fields: &[MetricField],
) -> Result<SearchPlayersPage, String> {
    if order.state != SearchPlayersPageState::Ready {
        return Ok(SearchPlayersPage {
            players: Vec::new(),
            total: 0,
            state: order.state,
        });
    }
    let snapshot_id = order.snapshot_id;
    let sql_fields = fields
        .iter()
        .filter(|field| !field.is_moneyball_score())
        .cloned()
        .collect::<Vec<_>>();
    let statistic_fields = fields
        .iter()
        .filter_map(|field| field.moneyball_key().map(|key| (field.id(), key)))
        .collect::<Vec<_>>();
    let catalog = builtin_catalog()?;
    let role_fields = fields
        .iter()
        .filter_map(|field| {
            field
                .moneyball_role_id()
                .map(|role_id| (field.id(), role_id))
        })
        .map(|(field_id, role_id)| {
            catalog
                .definitions
                .iter()
                .find(|definition| definition.id == role_id)
                .map(|definition| (field_id, definition))
                .ok_or_else(|| format!("unknown Moneyball role: {role_id}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let tactic = fields
        .iter()
        .any(MetricField::is_tactic_field)
        .then(|| read_tactic(conn, snapshot_save_id(conn, snapshot_id)?))
        .transpose()?;
    let lane_fields = match &tactic {
        Some(tactic) => fields
            .iter()
            .filter(|field| field.tactic_group() == Some(TacticGroup::Moneyball))
            .map(|field| {
                tactic_lane(field, tactic)
                    .map(|lane| (field.id(), lane))
                    .ok_or_else(|| format!("unknown tactic lane: {}", field.id()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let scores_needed =
        !statistic_fields.is_empty() || !role_fields.is_empty() || !lane_fields.is_empty();
    let reads_stats = scores_needed || fields.iter().any(MetricField::reads_season_stats);
    let mut keys = statistic_fields
        .iter()
        .map(|(_, key)| (*key).to_string())
        .collect::<Vec<_>>();
    for (_, definition) in &role_fields {
        keys.extend(definition.metrics.iter().map(|metric| metric.key.clone()));
    }
    for (_, lane) in &lane_fields {
        keys.extend(lane_statistic_keys(lane));
    }
    let needed = StatisticColumns::of_keys(keys.iter().map(String::as_str))?;

    let (uses_current_role_metrics, uses_potential_role_metrics) =
        role_metric_requirements(None, &SortField::Ca, fields)?;
    assert_read_models_complete(
        conn,
        snapshot_id,
        uses_current_role_metrics,
        uses_potential_role_metrics,
    )?;
    let mut bind_values = vec![Value::Integer(snapshot_id)];
    let club_dna_bindings = fields
        .iter()
        .any(MetricField::is_club_dna)
        .then(|| ClubDnaSqlBindings::new(2, 3));
    if club_dna_bindings.is_some() {
        let definition_version = current_club_dna_definition_version(conn, snapshot_id)?;
        bind_values.push(definition_version.map_or(Value::Null, Value::Integer));
        bind_values.push(Value::Integer(SCORE_MODEL_VERSION));
    }
    let kept = if reads_stats {
        kept_stats_snapshots(conn, snapshot_id)?
    } else {
        Vec::new()
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
                players.current_club_uid,
                players.division,
                players.ca,
                players.pa,
                players.market_value_gbp",
    );
    for field in &sql_fields {
        select_sql.push_str(", ");
        if let Some(tactic) = tactic.as_ref().filter(|_| field.is_tactic_field()) {
            let lane_id = field.tactic_lane_id().expect("tactic lane");
            let lane = tactic
                .lanes
                .iter()
                .find(|l| l.lane_id == lane_id)
                .ok_or_else(|| format!("unknown tactic lane: {lane_id}"))?;
            let group = field.tactic_group().expect("tactic group");
            select_sql.push_str(&tactic_sql_expression(lane, group, "players"));
        } else {
            select_sql.push_str(&field.sql_expression_with_club_dna("players", club_dna_bindings));
        }
    }
    let qualified = qualified_season_stats_sql();
    if reads_stats {
        select_sql.push_str(&format!(
            ", CASE WHEN {qualified} THEN moneyball.snapshot_id END"
        ));
    }
    if scores_needed {
        select_sql.push_str(&format!(
            ", CASE WHEN {qualified} THEN {STATS_DIVISION_SQL} END"
        ));
    }
    if !lane_fields.is_empty() {
        select_sql.push_str(", players.positions_json, players.preferred_foot");
    }
    if scores_needed {
        select_sql.push_str(&needed.appended_sql("moneyball"));
    }
    select_sql.push_str(" FROM players");
    if uses_current_role_metrics || uses_potential_role_metrics {
        select_sql.push_str(&player_metrics_join(
            "players",
            uses_current_role_metrics,
            uses_potential_role_metrics,
        ));
    }
    if reads_stats {
        select_sql.push_str(&stats_join_sql("players", &kept_ids(&kept)));
    }

    let limit = request.limit.clamp(1, MAX_PAGE_LIMIT);
    let start = request.offset.min(order.uids.len());
    let window = &order.uids[start..(start + limit).min(order.uids.len())];
    let first_uid_index = bind_values.len() + 1;
    let uid_placeholders = (0..window.len())
        .map(|index| format!("?{}", first_uid_index + index))
        .collect::<Vec<_>>()
        .join(", ");
    bind_values.extend(window.iter().map(|uid| Value::Integer(*uid)));
    select_sql.push_str(&format!(
        " WHERE players.snapshot_id = ?1 AND players.uid IN ({uid_placeholders})"
    ));

    let source_index = 12 + sql_fields.len();
    let division_index = source_index + 1;
    let fit_index = source_index + 2;
    let first_statistic_index = fit_index + if lane_fields.is_empty() { 0 } else { 2 };
    let mut rows = conn
        .prepare(&select_sql)
        .map_err(|error| error.to_string())?
        .query_map(params_from_iter(bind_values.iter()), |row| {
            let player = map_player_summary(row, &sql_fields)?;
            let source_id: Option<i64> = if reads_stats {
                row.get(source_index)?
            } else {
                None
            };
            let (statistics, division): (Option<Box<[f64]>>, Option<String>) =
                match source_id.filter(|_| scores_needed) {
                    Some(_) => (
                        Some(needed.read(row, first_statistic_index)?),
                        row.get(division_index)?,
                    ),
                    None => (None, None),
                };
            let fit = if lane_fields.is_empty() {
                None
            } else {
                Some(PlayerFit::from_columns(
                    &row.get::<_, String>(fit_index)?,
                    row.get(fit_index + 1)?,
                ))
            };
            Ok((player.uid, (player, source_id, statistics, division, fit)))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|error| error.to_string())?;

    let sources = if scores_needed {
        let groups_to_rank = match request.comparison {
            Comparison::Group(group) => Some(group),
            Comparison::Own => None,
        }
        .into_iter()
        .chain(lane_fields.iter().filter_map(|(_, lane)| lane_group(lane)))
        .collect::<Vec<_>>();
        Some(SeasonSources::load(
            conn,
            groups,
            snapshot_id,
            &kept,
            &groups_to_rank,
        )?)
    } else {
        None
    };
    let mut players = Vec::with_capacity(window.len());
    for uid in window {
        let Some((mut player, source_id, statistics, division, fit)) = rows.remove(uid) else {
            continue;
        };
        player.moneyball_stats_date = source_id
            .filter(|id| *id != snapshot_id)
            .and_then(|id| kept.iter().find(|kept| kept.snapshot_id == id))
            .map(|kept| kept.stats_date.clone());
        if let Some(sources) = &sources {
            let sample = source_id
                .and_then(|id| sources.get(id))
                .zip(statistics.as_ref());
            let league_shift = sample
                .as_ref()
                .map_or(0, |(source, _)| source.league_shift(division.as_deref()));
            for (field_id, key) in &statistic_fields {
                let percentile = sample.as_ref().and_then(|(source, statistics)| {
                    let group = source.table_group(request.comparison, player.uid)?;
                    source.percentile(group, key, statistics)
                });
                player
                    .moneyball_percentiles
                    .insert((*field_id).to_string(), percentile);
            }
            let score_value = |score: Option<u8>| score.map(i64::from).map(DynamicValue::Integer);
            for (field_id, definition) in &role_fields {
                let score = sample.as_ref().and_then(|(source, statistics)| {
                    source.role_score(definition, statistics, league_shift)
                });
                player
                    .dynamic_values
                    .insert((*field_id).to_string(), score_value(score));
            }
            for (field_id, lane) in &lane_fields {
                let score = match (&sample, &fit) {
                    (Some((source, statistics)), Some(fit)) => {
                        source.lane_score(lane, statistics, fit, league_shift)
                    }
                    _ => None,
                };
                player
                    .dynamic_values
                    .insert((*field_id).to_string(), score_value(score));
            }
        }
        players.push(player);
    }
    Ok(SearchPlayersPage {
        players,
        total: i64::try_from(order.uids.len()).map_err(|error| error.to_string())?,
        state: SearchPlayersPageState::Ready,
    })
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

pub(super) fn snapshot_save_id(conn: &Connection, snapshot_id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT save_id FROM snapshots WHERE id = ?1",
        [snapshot_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

/// A tactic slot's Role Score or Potential Role Score in SQL. Moneyball Scores are Rust's.
fn tactic_sql_expression(lane: &TacticLane, group: TacticGroup, player_alias: &str) -> String {
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
        TacticGroup::Moneyball => unreachable!("Moneyball Scores are computed in Rust"),
    };
    let oop_expr = match group {
        TacticGroup::Current => format!("{PLAYER_METRICS_ALIAS}.{oop_current}"),
        TacticGroup::Potential => format!(
            "CASE WHEN {player_alias}.age >= 29 THEN {PLAYER_METRICS_ALIAS}.{oop_current} ELSE {PLAYER_METRICS_ALIAS}.{oop_pot} END"
        ),
        TacticGroup::Moneyball => unreachable!("Moneyball Scores are computed in Rust"),
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
    let ip_fam_penalty = format!("CASE WHEN {fam_ip} < {FULL_FAMILIARITY} THEN 5 ELSE 0 END");
    let oop_fam_penalty = format!("CASE WHEN {fam_oop} < {FULL_FAMILIARITY} THEN 5 ELSE 0 END");
    let penalty = format!("({foot_soft_penalty} + {ip_fam_penalty} + {oop_fam_penalty})");
    format!(
        "CASE WHEN ({fam_ip} IS NULL OR {fam_ip} < 12 OR {fam_oop} IS NULL OR {fam_oop} < 12) THEN NULL WHEN ({strict_mismatch}) THEN NULL WHEN ({ip_expr} IS NULL OR {oop_expr} IS NULL) THEN NULL ELSE CASE WHEN (({blended}) - ({penalty}) < 0) THEN 0 ELSE CAST((({blended}) - ({penalty})) AS INTEGER) END END"
    )
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
        let idx = 12 + offset;
        let cell = read_dynamic_value(row, idx, field)?;
        dynamic_values.insert(field.id().to_string(), cell);
    }

    Ok(PlayerSummary {
        uid: row.get(0)?,
        name: row.get(1)?,
        age: row.get(2)?,
        birth_year: row.get(3)?,
        birth_day_of_year: row.get(4)?,
        nationalities,
        club: row.get(6)?,
        current_club_uid: row.get(7)?,
        division: row.get(8)?,
        ca: row.get(9)?,
        pa: row.get(10)?,
        market_value_gbp: row.get(11)?,
        dynamic_values,
        moneyball_percentiles: BTreeMap::new(),
        moneyball_stats_date: None,
    })
}

fn parse_nationalities(json: &str) -> Result<Vec<String>, String> {
    serde_json::from_str(json).map_err(|error| format!("invalid nationalities_json: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::moneyball::comparison::{ComparisonGroup, PositionFamily};
    use crate::features::moneyball::league_strength::{self, test_support::measure_leagues};
    use crate::features::moneyball::persistence::test_support::insert_statistics_row;
    use crate::features::moneyball::role_catalog::builtin_catalog;
    use crate::features::player_metrics::{
        club_dna::SCORE_MODEL_VERSION, potential_scores::PROJECTION_MODEL_VERSION,
    };
    use crate::features::search::filter::{parse_filter_ast, FilterRule, FilterValue};
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service::{create_save, set_active_save};
    use serde_json::{json, Map, Value};
    use std::cell::{Cell, RefCell};
    use std::path::Path;

    thread_local! {
        /// General order computations run on this thread.
        pub(super) static ORDER_COMPUTATIONS: Cell<usize> = const { Cell::new(0) };
        /// Runs once, just before the next General order query on this thread.
        pub(super) static BEFORE_ORDER_QUERY: RefCell<Option<Box<dyn FnOnce()>>> =
            const { RefCell::new(None) };
    }

    fn search_players_page(
        conn: &Connection,
        offset: usize,
        limit: usize,
        sort_by: SortField,
        sort_dir: SortDir,
        filter_ast: Option<&FilterAst>,
        requested_fields: &[String],
    ) -> Result<SearchPlayersPage, String> {
        search_players_uncached(
            conn,
            SearchPlayersRequest {
                offset,
                limit,
                sort_by,
                sort_dir,
                filter_ast,
                requested_fields,
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
    }

    fn search_without_filters(
        conn: &Connection,
        offset: usize,
        limit: usize,
        sort_by: SortField,
        sort_dir: SortDir,
    ) -> Result<SearchPlayersPage, String> {
        search_players_uncached(
            conn,
            SearchPlayersRequest {
                offset,
                limit,
                sort_by,
                sort_dir,
                filter_ast: None,
                requested_fields: &[],
                comparison: Comparison::Own,
                shortlist_id: None,
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
        search_players_uncached(
            conn,
            SearchPlayersRequest {
                offset,
                limit,
                sort_by,
                sort_dir,
                filter_ast: Some(&ast),
                requested_fields: &[],
                comparison: Comparison::Own,
                shortlist_id: None,
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
            "appearance": null,
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
            "currentClubUid": null,
            "parentClub": null,
            "parentClubUid": null,
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
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v11.json"))
                .expect("parse golden fixture");
        let mut players = players;
        for player in &mut players {
            complete_position_map(player);
            let object = player.as_object_mut().expect("player object");
            object.entry("currentClubUid").or_insert(Value::Null);
            object.entry("parentClubUid").or_insert(Value::Null);
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

    /// Adds `uids` to the Targets shortlist of the current snapshot's save and returns the
    /// shortlist's ID.
    fn seed_player_shortlist(conn: &Connection, uids: &[i64]) -> i64 {
        seed_named_player_shortlist(conn, "Targets", uids)
    }

    /// Adds `uids` to the shortlist called `name` of the current snapshot's save, creating
    /// it when needed, and returns its ID.
    fn seed_named_player_shortlist(conn: &Connection, name: &str, uids: &[i64]) -> i64 {
        let shortlist_id = conn
            .query_row(
                "INSERT INTO player_shortlists (save_id, name)
                 SELECT save_id, ?1 FROM snapshots WHERE is_current = 1
                 ON CONFLICT (save_id, name) DO UPDATE SET name = name
                 RETURNING id",
                [name],
                |row| row.get(0),
            )
            .expect("store shortlist");
        for uid in uids {
            conn.execute(
                "INSERT INTO player_shortlist_entries (shortlist_id, player_uid) VALUES (?1, ?2)",
                [shortlist_id, *uid],
            )
            .expect("seed shortlist");
        }
        shortlist_id
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
                "SELECT p.uid, a.attributes_json, a.projection_model_version
                 FROM players p
                 LEFT JOIN player_projected_attributes a
                   ON a.snapshot_id = p.snapshot_id AND a.uid = p.uid
                 WHERE p.snapshot_id = ?1 ORDER BY p.uid",
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
            "CREATE TRIGGER deny_projected_inserts
             BEFORE INSERT ON player_projected_attributes
             BEGIN SELECT RAISE(ABORT, 'potential player writes are forbidden'); END;
             CREATE TRIGGER deny_projected_updates
             BEFORE UPDATE ON player_projected_attributes
             BEGIN SELECT RAISE(ABORT, 'potential player writes are forbidden'); END;
             CREATE TRIGGER deny_projected_deletes
             BEFORE DELETE ON player_projected_attributes
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
        // 900 minutes: enough to count among the player's comparison groups' peers.
        insert_statistics_row(
            conn,
            snapshot_id,
            player_uid,
            Some(900),
            statistics_json,
            percentiles_json,
        );
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
        comparison: Comparison,
    ) -> Result<SearchPlayersPage, String> {
        let ast = if rules.is_empty() {
            None
        } else {
            Some(parse_filter_ast(rules, combine)?)
        };
        search_players_uncached(
            conn,
            SearchPlayersRequest {
                offset,
                limit,
                sort_by,
                sort_dir,
                filter_ast: ast.as_ref(),
                requested_fields,
                comparison,
                shortlist_id: None,
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
    fn statistics_come_from_the_current_snapshots_rows_and_every_player_is_a_row() {
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
        let page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::parse("moneyball.goals").expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("search Moneyball players");

        // The unscored and absent players follow the scored one; the older snapshot's player
        // is not a row.
        assert_eq!(page.total, 3);
        assert_eq!(
            page.players
                .iter()
                .map(|player| (player.uid, player.dynamic_values["moneyball.goals"].clone()))
                .collect::<Vec<_>>(),
            [(1, Some(DynamicValue::Real(1.5))), (2, None), (3, None)]
        );
    }

    #[test]
    fn moneyball_percentiles_rank_each_player_within_his_own_family_or_the_chosen_one() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-comparison.db"));
        let mut striker = player_template(3, "Striker", 160);
        striker["positions"] = json!({ "ST": 20 });
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Scorer", 160),
                player_template(2, "Passer", 160),
                striker,
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, goals) in [(1, 12), (2, 4), (3, 100)] {
            insert_moneyball_statistics(
                &conn,
                snapshot_id,
                uid,
                &json!({ "goals": goals }).to_string(),
                Some("{}"),
            );
        }

        let requested_fields = [
            "moneyball.goals".to_string(),
            "moneyball.minutes".to_string(),
        ];
        let page = |comparison| {
            search_players_uncached(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: DEFAULT_PAGE_LIMIT,
                    sort_by: SortField::parse("moneyball.goals").expect("sort"),
                    sort_dir: SortDir::Desc,
                    filter_ast: None,
                    requested_fields: &requested_fields,
                    comparison,
                    shortlist_id: None,
                },
            )
            .expect("search Moneyball players")
        };
        let goals_percentiles = |page: &SearchPlayersPage| {
            page.players
                .iter()
                .map(|player| {
                    (
                        player.uid,
                        player
                            .moneyball_percentiles
                            .get("moneyball.goals")
                            .copied()
                            .flatten(),
                    )
                })
                .collect::<Vec<_>>()
        };

        // On their own, the central midfielders rank among each other and the striker alone.
        let own = page(Comparison::Own);
        assert_eq!(
            own.players[1].dynamic_values.get("moneyball.goals"),
            Some(&Some(DynamicValue::Real(12.0)))
        );
        assert_eq!(
            goals_percentiles(&own),
            [(3, Some(50)), (1, Some(100)), (2, Some(0))]
        );
        assert!(!own.players[0]
            .moneyball_percentiles
            .contains_key("moneyball.minutes"));
        assert!(own
            .players
            .iter()
            .all(|player| player.moneyball_stats_date.is_none()));
        // Compared with central midfielders, the striker is ranked without joining them.
        assert_eq!(
            goals_percentiles(&page(Comparison::Group(ComparisonGroup::family(
                PositionFamily::CentralMidfielder
            )))),
            [(3, Some(100)), (1, Some(100)), (2, Some(0))]
        );
    }

    #[test]
    fn a_short_sample_has_no_moneyball_values_and_every_moneyball_rule_drops_him() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-short-sample.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Low", 100),
                player_template(2, "High", 100),
                player_template(3, "Short", 100),
                player_template(4, "Unplayed", 100),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, goals) in [(1, 1), (2, 5), (3, 9)] {
            let statistics = json!({ "goals": goals }).to_string();
            insert_moneyball_statistics(&conn, snapshot_id, uid, &statistics, Some("{}"));
        }
        conn.execute(
            "UPDATE player_moneyball_stats SET minutes = 449 WHERE player_uid = 3",
            [],
        )
        .expect("shorten a sample");
        let fields = [
            "moneyball.goals".to_string(),
            "moneyball.minutes".to_string(),
        ];
        let page = |rules: Vec<FilterRule>| {
            search_moneyball(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::Name,
                SortDir::Asc,
                &fields,
                rules,
                None,
                Comparison::Own,
            )
            .expect("page")
        };
        let rows = |page: &SearchPlayersPage| {
            page.players
                .iter()
                .map(|player| {
                    (
                        player.uid,
                        player.dynamic_values["moneyball.goals"].clone(),
                        player.dynamic_values["moneyball.minutes"].clone(),
                        player.moneyball_percentiles["moneyball.goals"],
                    )
                })
                .collect::<Vec<_>>()
        };

        // Every player is a row. The peers are 1 and 5, so 5 ranks at 100; the short sample
        // and the player who has not played have no statistics and no percentile, though
        // their minutes still show.
        assert_eq!(
            rows(&page(Vec::new())),
            [
                (
                    2,
                    Some(DynamicValue::Real(5.0)),
                    Some(DynamicValue::Integer(900)),
                    Some(100)
                ),
                (
                    1,
                    Some(DynamicValue::Real(1.0)),
                    Some(DynamicValue::Integer(900)),
                    Some(0)
                ),
                (3, None, Some(DynamicValue::Integer(449)), None),
                (4, None, Some(DynamicValue::Integer(0)), None),
            ]
        );
        let uids = |page: SearchPlayersPage| {
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>()
        };
        // An "at most" rule on a statistic or its percentile drops them too.
        assert_eq!(
            uids(page(vec![filter_rule(
                "moneyball.goals",
                "lte",
                FilterValue::Integer(20)
            )])),
            [2, 1]
        );
        assert_eq!(
            uids(page(vec![filter_rule(
                "moneyball.goals",
                "pct_lte",
                FilterValue::Integer(100)
            )])),
            [2, 1]
        );
        // Playing time counts a missing row as 0 minutes.
        assert_eq!(
            uids(page(vec![filter_rule(
                "moneyball.minutes",
                "lte",
                FilterValue::Integer(500)
            )])),
            [3, 4]
        );
        // Under OR, a player rule keeps a player without a sample.
        let either = search_moneyball(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::Name,
            SortDir::Asc,
            &fields,
            vec![
                filter_rule("name", "is", FilterValue::Text("Unplayed".to_string())),
                filter_rule("moneyball.goals", "pct_gte", FilterValue::Integer(100)),
            ],
            Some("or"),
            Comparison::Own,
        )
        .expect("OR page");
        assert_eq!(uids(either), [2, 4]);
    }

    #[test]
    fn percentiles_can_rank_within_two_families_together() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-two-families.db"));
        let mut striker = player_template(3, "Striker", 100);
        striker["positions"] = json!({ "ST": 18 });
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Low", 100),
                player_template(2, "High", 100),
                striker,
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, goals) in [(1, 1), (2, 5), (3, 9)] {
            let statistics = json!({ "goals": goals }).to_string();
            insert_moneyball_statistics(&conn, snapshot_id, uid, &statistics, Some("{}"));
        }
        let percentiles = |comparison: &str| {
            search_moneyball(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::Name,
                SortDir::Asc,
                &["moneyball.goals".to_string()],
                Vec::new(),
                None,
                Comparison::parse(comparison).expect("comparison"),
            )
            .expect("page")
            .players
            .iter()
            .map(|player| (player.uid, player.moneyball_percentiles["moneyball.goals"]))
            .collect::<Vec<_>>()
        };

        // Apart, the striker is alone among strikers; together, all three are peers.
        assert_eq!(
            percentiles("own"),
            [(2, Some(100)), (1, Some(0)), (3, Some(50))]
        );
        assert_eq!(
            percentiles("central_midfielder+striker"),
            [(2, Some(50)), (1, Some(0)), (3, Some(100))]
        );
    }

    #[test]
    fn a_short_sample_borrows_a_kept_snapshots_qualified_sample() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-borrowed.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Borrower", 100),
                player_template(2, "Regular", 100),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, goals) in [(1, 50), (2, 100)] {
            let statistics = json!({ "goals": goals }).to_string();
            insert_moneyball_statistics(&conn, snapshot_id, uid, &statistics, Some("{}"));
        }
        conn.execute(
            "UPDATE player_moneyball_stats SET minutes = 200 WHERE player_uid = 1",
            [],
        )
        .expect("shorten a sample");
        let kept = crate::features::moneyball::kept_stats::test_support::add_kept_snapshot(
            &conn,
            snapshot_id,
            "2026-05-20",
            &[1, 2],
        );
        conn.execute(
            "UPDATE snapshots SET moneyball_stats_game_date = '2026-05-20' WHERE id = ?1",
            [kept],
        )
        .expect("date kept stats");
        for (uid, goals) in [(1, 10), (2, 5)] {
            let statistics = json!({ "goals": goals }).to_string();
            insert_moneyball_statistics(&conn, kept, uid, &statistics, Some("{}"));
        }

        let page = search_moneyball(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::parse("moneyball.goals").expect("sort"),
            SortDir::Desc,
            &[
                "moneyball.goals".to_string(),
                "moneyball.minutes".to_string(),
            ],
            vec![filter_rule(
                "moneyball.goals",
                "pct_gte",
                FilterValue::Integer(50),
            )],
            None,
            Comparison::Own,
        )
        .expect("page");

        // His kept 10 goals and 900 minutes stand in, ranked among that season's 5 and 10.
        // The regular, alone among this season's peers, ranks at 50.
        assert_eq!(page.players.len(), 2);
        let borrower = page
            .players
            .iter()
            .find(|player| player.uid == 1)
            .expect("borrower");
        assert_eq!(
            borrower.dynamic_values["moneyball.goals"],
            Some(DynamicValue::Real(10.0))
        );
        assert_eq!(
            borrower.dynamic_values["moneyball.minutes"],
            Some(DynamicValue::Integer(900))
        );
        assert_eq!(borrower.moneyball_percentiles["moneyball.goals"], Some(100));
        assert_eq!(borrower.moneyball_stats_date.as_deref(), Some("2026-05-20"));
        assert_eq!(page.players[0].uid, 2);
        assert_eq!(page.players[0].moneyball_stats_date, None);
    }

    #[test]
    fn moneyball_percentiles_rank_against_the_whole_group_whatever_the_filters() {
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
            insert_moneyball_statistics(&conn, snapshot_id, uid, &statistics_json, Some("{}"));
        }

        let requested_fields = ["moneyball.goals".to_string()];
        let sort_by = SortField::parse("moneyball.goals").expect("sort");
        let filter_ast = parse_filter_ast(
            vec![FilterRule {
                field: "moneyball.goals".to_string(),
                op: "gt".to_string(),
                value: FilterValue::Integer(0),
            }],
            None,
        )
        .expect("Moneyball filter");
        let first_page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 50,
                sort_by: sort_by.clone(),
                sort_dir: SortDir::Asc,
                filter_ast: Some(&filter_ast),
                requested_fields: &requested_fields,
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("first page");
        let second_page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 50,
                limit: 50,
                sort_by,
                sort_dir: SortDir::Asc,
                filter_ast: Some(&filter_ast),
                requested_fields: &requested_fields,
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("second page");

        // The filter leaves out the player with no goals, but he still counts among the
        // central midfielders who rank the rest: one goal is above one of the 101.
        assert_eq!(first_page.total, 100);
        assert_eq!(first_page.players[0].uid, 1002);
        assert_eq!(
            first_page.players[0]
                .moneyball_percentiles
                .get("moneyball.goals"),
            Some(&Some(1))
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
    fn moneyball_view_filters_and_sorts_ability_and_contract_fields() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-ability.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Low", 140),
                player_template(2, "High", 170),
                player_template(3, "Mid", 155),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        for uid in 1..=3 {
            insert_moneyball_row(&conn, snapshot_id, uid, Some(r#"{"goals":50}"#));
        }
        let filter_ast = parse_filter_ast(
            vec![FilterRule {
                field: "ca".to_string(),
                op: "gte".to_string(),
                value: FilterValue::Integer(150),
            }],
            None,
        )
        .expect("CA filter");
        let requested_fields = ["wage".to_string(), "reputation".to_string()];

        let page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::parse("pa").expect("PA sort"),
                sort_dir: SortDir::Desc,
                filter_ast: Some(&filter_ast),
                requested_fields: &requested_fields,
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("CA and PA work in Moneyball Search");

        assert_eq!(page.total, 2);
        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        assert!(page.players[0].dynamic_values.contains_key("wage"));
        assert!(page.players[0].dynamic_values.contains_key("reputation"));
    }

    #[test]
    fn general_shortlist_restriction_returns_only_saved_current_uids_with_filter_and_paging() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("general-shortlist.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "One", 130),
                player_template(2, "Two", 140),
                player_template(3, "Three", 150),
                player_template(4, "Four", 160),
            ],
        );
        set_role_score(&conn, 1, "deep_lying_playmaker_ip", Some(80));
        set_role_score(&conn, 2, "deep_lying_playmaker_ip", Some(40));
        set_role_score(&conn, 3, "deep_lying_playmaker_ip", Some(90));
        set_role_score(&conn, 4, "deep_lying_playmaker_ip", Some(80));
        // Saved UIDs 1..=3 plus a stale UID with no current-snapshot player, and another
        // shortlist of the same save holding only UID 4.
        let shortlist = Some(seed_player_shortlist(&conn, &[1, 2, 3, 99]));
        let loans = seed_named_player_shortlist(&conn, "Loans", &[4]);
        let requested_fields = vec!["role.deep_lying_playmaker_ip".to_string()];

        // Unrestricted General still returns every current player.
        let unrestricted = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("unrestricted General");
        assert_eq!(unrestricted.total, 4);

        // Restricted General returns only saved current UIDs; the stale UID drops out.
        let restricted = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &requested_fields,
                comparison: Comparison::Own,
                shortlist_id: shortlist,
            },
        )
        .expect("restricted General");
        assert_eq!(restricted.total, 3);
        assert_eq!(
            restricted.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        assert_eq!(
            restricted.players[0]
                .dynamic_values
                .get("role.deep_lying_playmaker_ip"),
            Some(&Some(DynamicValue::Integer(90)))
        );

        // Another shortlist of the same save restricts to its own players only.
        let other = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                comparison: Comparison::Own,
                shortlist_id: Some(loans),
            },
        )
        .expect("other shortlist");
        assert_eq!(
            other.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![4]
        );

        // The restriction composes with one General filter.
        let ast = parse_filter_ast(
            vec![FilterRule {
                field: "role.deep_lying_playmaker_ip".to_string(),
                op: "gt".to_string(),
                value: FilterValue::Integer(50),
            }],
            None,
        )
        .expect("parse filter");
        let filtered = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: Some(&ast),
                requested_fields: &[],
                comparison: Comparison::Own,
                shortlist_id: shortlist,
            },
        )
        .expect("filtered restricted General");
        assert_eq!(filtered.total, 2);
        assert_eq!(
            filtered.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![3, 1]
        );

        // Totals survive paging.
        let paged = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 1,
                limit: 1,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: Some(&ast),
                requested_fields: &[],
                comparison: Comparison::Own,
                shortlist_id: shortlist,
            },
        )
        .expect("paged restricted General");
        assert_eq!(paged.total, 2);
        assert_eq!(
            paged.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![1]
        );

        // An empty saved list restricts to nothing.
        conn.execute("DELETE FROM player_shortlist_entries", [])
            .expect("clear shortlist");
        let empty = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                comparison: Comparison::Own,
                shortlist_id: shortlist,
            },
        )
        .expect("empty restricted General");
        assert_eq!(empty.total, 0);
        assert!(empty.players.is_empty());
    }

    #[test]
    fn a_shortlist_of_another_save_or_an_unknown_one_reports_no_shortlist() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("shortlist-probe.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "One", 130),
                player_template(2, "Two", 140),
            ],
        );

        let probe = |conn: &Connection, shortlist_id: i64| {
            search_players_uncached(
                conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: 1,
                    sort_by: SortField::Ca,
                    sort_dir: SortDir::Desc,
                    filter_ast: None,
                    requested_fields: &[],
                    comparison: Comparison::Own,
                    shortlist_id: Some(shortlist_id),
                },
            )
            .expect("shortlist probe")
        };

        // An unknown ID, and another save's shortlist of current players, restrict to nothing.
        let unknown = probe(&conn, 1);
        assert_eq!(unknown.state, SearchPlayersPageState::NoShortlist);
        conn.execute_batch(
            "INSERT INTO saves (id, name, is_active) VALUES (99, 'Other save', 0);
             INSERT INTO player_shortlists (id, save_id, name) VALUES (7, 99, 'Other');
             INSERT INTO player_shortlist_entries (shortlist_id, player_uid) VALUES (7, 1);",
        )
        .expect("seed another save's shortlist");
        let foreign = probe(&conn, 7);
        assert_eq!(foreign.state, SearchPlayersPageState::NoShortlist);
        assert_eq!(foreign.total, 0);
        assert!(foreign.players.is_empty());

        // A shortlist whose players are not in the snapshot stays ready with an empty page.
        let shortlist_id = seed_player_shortlist(&conn, &[99]);
        let stale_only = probe(&conn, shortlist_id);
        assert_eq!(stale_only.state, SearchPlayersPageState::Ready);
        assert_eq!(stale_only.total, 0);
        assert!(stale_only.players.is_empty());

        // A linked uid that resolves to a current player reports ready with matches.
        seed_player_shortlist(&conn, &[1]);
        let matched = probe(&conn, shortlist_id);
        assert_eq!(matched.state, SearchPlayersPageState::Ready);
        assert_eq!(matched.total, 1);
        assert_eq!(matched.players[0].uid, 1);
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
        conn.execute("UPDATE snapshots SET is_kept = 1", [])
            .expect("keep the later snapshot");
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
    }

    #[test]
    fn searches_club_dna_ascending_with_nullable_exact_identity_and_read_only_pages() {
        let (_temp_dir, conn) = seed_club_dna_query_players();
        conn.pragma_update(None, "reverse_unordered_selects", true)
            .expect("reverse unordered ties");
        let score_rows_before = club_dna_score_rows(&conn);
        let score_row_count_before = score_rows_before.len();
        let requested_fields = vec!["club_dna".to_string()];

        let page = search_players_page(
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

        let bounded_page = search_players_page(
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
        let unavailable_page = search_players_page(
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
            let page = search_players_page(
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
        let and_page = search_players_page(
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
        let or_page = search_players_page(
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

        let descending = search_players_page(
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
        let bounded_page = search_players_page(
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
        let unavailable_page = search_players_page(
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

        let page = search_players_page(
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

    /// What the realistic-signing rule reads of a player besides his ability.
    struct Standing {
        age: i64,
        /// The player's potential ability; his current ability when `None`.
        pa: Option<i64>,
        /// The club he plays for: its name, UID, and team reputation.
        club: Option<(&'static str, i64, i64)>,
        /// The club that holds his contract when he is out on loan.
        parent_club_uid: Option<i64>,
        reputation: (i64, i64),
    }

    fn player_with_standing(uid: u64, name: &str, ca: i64, standing: Standing) -> Value {
        let mut player = player_template(uid, name, ca);
        let object = player.as_object_mut().expect("player object");
        object.insert("age".to_string(), json!(standing.age));
        object.insert("pa".to_string(), json!(standing.pa.unwrap_or(ca)));
        let (club, club_uid, club_reputation) = match standing.club {
            Some((club, uid, reputation)) => (json!(club), json!(uid), json!(reputation)),
            None => (Value::Null, Value::Null, Value::Null),
        };
        object.insert("currentClub".to_string(), club);
        object.insert("currentClubUid".to_string(), club_uid.clone());
        object.insert(
            "parentClubUid".to_string(),
            standing.parent_club_uid.map_or(club_uid, |uid| json!(uid)),
        );
        object.insert("clubReputation".to_string(), club_reputation);
        object.insert(
            "reputation".to_string(),
            json!({ "current": standing.reputation.0, "world": standing.reputation.1 }),
        );
        player
    }

    const SMALL_CLUB: (&str, i64, i64) = ("Small FC", 500, 4000);
    const SMALLER_CLUB: (&str, i64, i64) = ("Smaller FC", 600, 3800);
    const BIGGER_CLUB: (&str, i64, i64) = ("Bigger FC", 700, 5500);
    const GIANT_CLUB: (&str, i64, i64) = ("Giant FC", 900, 9800);

    /// Small FC's twelve players of ability 100 (UIDs 101–112) and nine players elsewhere.
    /// UIDs 2, 4, and 5 are realistic signings for Small FC; 1, 3, 6, 7, and 9 are not. UID 8
    /// is Small FC's own player, so he is neither.
    fn seed_signing_targets(conn: &mut rusqlite::Connection) {
        let at = |club, reputation| Standing {
            age: 27,
            pa: None,
            club: Some(club),
            parent_club_uid: None,
            reputation,
        };
        let mut players = (101..=112)
            .map(|uid| player_with_standing(uid, "Squad", 100, at(SMALL_CLUB, (3500, 3200))))
            .collect::<Vec<_>>();
        players.extend([
            player_with_standing(1, "Superstar", 190, at(GIANT_CLUB, (9500, 9700))),
            player_with_standing(2, "Peer", 105, at(SMALLER_CLUB, (3300, 3000))),
            player_with_standing(
                3,
                "Better at a bigger club",
                112,
                at(BIGGER_CLUB, (4800, 4500)),
            ),
            player_with_standing(
                4,
                "Fringe at a bigger club",
                85,
                at(("Big FC", 800, 4400), (3500, 3200)),
            ),
            player_with_standing(
                5,
                "Free agent",
                100,
                Standing {
                    age: 29,
                    pa: None,
                    club: None,
                    parent_club_uid: None,
                    reputation: (3500, 3300),
                },
            ),
            player_with_standing(
                6,
                "Free-agent star",
                160,
                Standing {
                    age: 30,
                    pa: None,
                    club: None,
                    parent_club_uid: None,
                    reputation: (8000, 8200),
                },
            ),
            // Peer's ability, but a prospect counts some of his potential as his level.
            player_with_standing(
                7,
                "Prospect",
                105,
                Standing {
                    age: 19,
                    pa: Some(165),
                    ..at(SMALLER_CLUB, (3300, 3000))
                },
            ),
            // Small FC's own player, far above its level, out on loan.
            player_with_standing(
                8,
                "Loaned out",
                150,
                Standing {
                    parent_club_uid: Some(SMALL_CLUB.1),
                    ..at(BIGGER_CLUB, (5000, 4800))
                },
            ),
            // Within the club's reach, but by less than the game's "Interested" level asks.
            player_with_standing(
                9,
                "Free agent who is only slightly interested",
                108,
                Standing {
                    age: 29,
                    pa: None,
                    club: None,
                    parent_club_uid: None,
                    reputation: (3500, 3300),
                },
            ),
        ]);
        ingest_players(conn, players);
    }

    fn manage_club(conn: &Connection, club: (&str, i64, i64), with_uid: bool) {
        let save_id: i64 = conn
            .query_row(
                "SELECT save_id FROM snapshots WHERE is_current = 1",
                [],
                |row| row.get(0),
            )
            .expect("save id");
        conn.execute(
            "INSERT INTO managed_club_settings (save_id, club_name, club_uid) VALUES (?1, ?2, ?3)",
            rusqlite::params![save_id, club.0, with_uid.then_some(club.1)],
        )
        .expect("choose managed club");
    }

    fn realistic_signing_uids(conn: &Connection, op: &str, value: bool) -> Vec<i64> {
        let page = search_with_filters(
            conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![
                filter_rule("realistic_signing", op, FilterValue::Bool(value)),
                // Leaves out Small FC's twelve squad players.
                filter_rule("name", "is_not", FilterValue::Text("Squad".to_string())),
            ],
            Some("and"),
        )
        .expect("realistic signing filter");
        let mut uids = page
            .players
            .iter()
            .map(|player| player.uid)
            .collect::<Vec<_>>();
        uids.sort_unstable();
        uids
    }

    #[test]
    fn realistic_signings_are_players_the_managed_club_could_attract() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("realistic-signings.db"));
        seed_signing_targets(&mut conn);
        manage_club(&conn, SMALL_CLUB, true);

        assert_eq!(realistic_signing_uids(&conn, "is", true), vec![2, 4, 5]);
        assert_eq!(
            realistic_signing_uids(&conn, "is_not", false),
            vec![2, 4, 5]
        );
        // The loaned-out player is the club's own, so his score does not make him unrealistic.
        assert_eq!(
            realistic_signing_uids(&conn, "is", false),
            vec![1, 3, 6, 7, 9]
        );
        assert_eq!(
            realistic_signing_uids(&conn, "is_not", true),
            vec![1, 3, 6, 7, 9]
        );

        // A club cannot sign its own players, so its squad matches neither way, although each
        // of them is within its reach.
        for realistic in [true, false] {
            let squad = search_with_filters(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::DEFAULT,
                SortDir::DEFAULT,
                vec![
                    filter_rule("realistic_signing", "is", FilterValue::Bool(realistic)),
                    filter_rule("name", "is", FilterValue::Text("Squad".to_string())),
                ],
                Some("and"),
            )
            .expect("own squad");
            assert_eq!(squad.total, 0, "realistic signing is {realistic}");
        }
        // Without the rule they are listed as usual.
        let squad = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule(
                "name",
                "is",
                FilterValue::Text("Squad".to_string()),
            )],
            Some("and"),
        )
        .expect("own squad without the rule");
        assert_eq!(squad.total, 12);
    }

    #[test]
    fn a_bigger_club_can_attract_better_players() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("realistic-signings-giant.db"));
        seed_signing_targets(&mut conn);
        // Giant FC's only player is the superstar, so its level is his ability. He is its own
        // player, so he is not a signing.
        manage_club(&conn, GIANT_CLUB, true);

        assert_eq!(
            realistic_signing_uids(&conn, "is", true),
            vec![2, 3, 4, 5, 6, 7, 8, 9]
        );
    }

    #[test]
    fn realistic_signings_match_nobody_without_a_managed_club_uid() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("realistic-signings-no-club.db"));
        seed_signing_targets(&mut conn);

        assert!(realistic_signing_uids(&conn, "is", true).is_empty());
        assert!(realistic_signing_uids(&conn, "is_not", true).is_empty());

        // A club chosen before club UIDs were recorded has no squad to measure.
        manage_club(&conn, SMALL_CLUB, false);
        assert!(realistic_signing_uids(&conn, "is", true).is_empty());
        assert!(realistic_signing_uids(&conn, "is_not", true).is_empty());
    }

    #[test]
    fn moneyball_view_filters_realistic_signings() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("realistic-signings-moneyball.db"));
        seed_signing_targets(&mut conn);
        manage_club(&conn, SMALL_CLUB, true);
        let snapshot_id = current_snapshot_id(&conn);
        for uid in 1..=9 {
            insert_moneyball_row(&conn, snapshot_id, uid, Some(r#"{"goals":50}"#));
        }

        for comparison in [
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
            Comparison::Own,
        ] {
            let page = search_moneyball(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::parse("ca").expect("CA sort"),
                SortDir::Asc,
                &[],
                vec![filter_rule(
                    "realistic_signing",
                    "is",
                    FilterValue::Bool(true),
                )],
                None,
                comparison,
            )
            .expect("Moneyball realistic signing filter");
            assert_eq!(
                page.players
                    .iter()
                    .map(|player| player.uid)
                    .collect::<Vec<_>>(),
                vec![4, 5, 2]
            );
        }
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
                        positions: json!({ "MC": 14, "ST": 18 }),
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
                    6,
                    "Cutoff MC",
                    145,
                    DeepPlayerFields {
                        nationalities: json!(["ENG"]),
                        positions: json!({ "MC": 15, "ST": 1 }),
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

        // FM rates every position from 1, so a Position rule only counts familiarity 15+.
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
            vec!["Natural MC", "Cutoff MC"]
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
        assert_eq!(inverse.total, 4);
        assert_eq!(
            inverse
                .players
                .iter()
                .map(|player| player.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Striker Only", "Fringe MC", "Zero MC", "Null MC"]
        );

        let suitability = search_with_filters(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::DEFAULT,
            SortDir::DEFAULT,
            vec![filter_rule("pos.MC", "gt", FilterValue::Integer(16))],
            None,
        )
        .expect("position suitability");
        assert_eq!(suitability.total, 1);
        assert_eq!(suitability.players[0].name, "Natural MC");

        let names = |rules: Vec<FilterRule>| {
            search_with_filters(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::DEFAULT,
                SortDir::DEFAULT,
                rules,
                None,
            )
            .expect("position search")
            .players
            .into_iter()
            .map(|player| player.name)
            .collect::<Vec<_>>()
        };
        let positions = |keys: &[&str]| {
            FilterValue::List(
                keys.iter()
                    .map(|key| FilterValue::Text((*key).to_string()))
                    .collect(),
            )
        };
        // Any of: familiarity 15+ at either key; DM 12 does not count.
        assert_eq!(
            names(vec![filter_rule(
                "position",
                "in",
                positions(&["MC", "DM"])
            )]),
            vec!["Natural MC", "Cutoff MC"]
        );
        assert_eq!(
            names(vec![filter_rule("position", "not_in", positions(&["ST"]))]),
            vec!["Natural MC", "Cutoff MC"]
        );
        assert_eq!(
            names(vec![filter_rule(
                "pos.MC",
                "between",
                FilterValue::List(vec![FilterValue::Integer(14), FilterValue::Integer(15)])
            )]),
            vec!["Cutoff MC", "Fringe MC"]
        );
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
        let page = search_players_page(
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
        let page = search_players_page(
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
        let page = search_players_page(
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

        let page = search_players_page(
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
        search_players_page(
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
                search_players_page(
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
        let page = search_players_page(
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
                search_players_page(
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
            let page = search_players_page(&conn, 0, 4, sort_by.clone(), direction, None, &[])
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

        let page = search_players_page(
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
            .find("fn order_query(")
            .expect("order query function")
            ..source
                .find("/// A query's order when SQL can sort and filter every row.")
                .expect("following function")];

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
        let error = search_players_page(
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
        let page = search_players_page(
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
    fn moneyball_role_scores_rank_within_the_roles_family_whatever_the_filters() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-role-orders.db"));
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

        // The table's comparison ranks statistics, not roles: the central midfielder role
        // ranks the three central midfielders' 10, 20, and 30 at 0, 50, and 100.
        for comparison in [
            Comparison::Own,
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
        ] {
            let full = search_moneyball(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::Name,
                SortDir::Asc,
                &requested_fields,
                Vec::new(),
                None,
                comparison,
            )
            .expect("full Moneyball role search");
            assert_eq!(full.total, 3);
            assert_eq!(
                full.players
                    .iter()
                    .map(
                        |player| player.dynamic_values[&format!("moneyball_role.{role_id}")]
                            .clone()
                    )
                    .collect::<Vec<_>>(),
                [0, 50, 100].map(|score| Some(DynamicValue::Integer(score)))
            );
        }

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
            Comparison::Own,
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
            Comparison::Own,
        )
        .expect("filtered second page");
        // The filter leaves out the best player, who still ranks the others.
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
            Some(&Some(DynamicValue::Integer(50)))
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

        // The three central midfielders' 10, 30, and 20 score 0, 100, and 50.
        let and_rules = vec![
            filter_rule("name", "contains", FilterValue::Text("Only".to_string())),
            filter_rule(
                &format!("moneyball_role.{role_id}"),
                "lt",
                FilterValue::Integer(50),
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
            Comparison::Own,
        )
        .expect("post-score AND filter");
        assert_eq!(and_page.total, 1);
        assert_eq!(and_page.players[0].uid, 1);
        assert_eq!(
            and_page.players[0]
                .dynamic_values
                .get(&format!("moneyball_role.{role_id}")),
            Some(&Some(DynamicValue::Integer(0)))
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
            SortField::parse(&format!("moneyball_role.{role_id}")).expect("role sort"),
            SortDir::Desc,
            &requested_fields,
            or_rules,
            Some("or"),
            Comparison::Own,
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
    fn filters_moneyball_percentiles_within_the_tables_comparison() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-percentile-filters.db"));
        let mut striker = player_template(5, "Cohort striker", 100);
        striker["positions"] = json!({ "ST": 20 });
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Cohort low", 100),
                player_template(2, "Cohort high", 100),
                player_template(3, "Outside top", 100),
                player_template(4, "Cohort missing", 100),
                striker,
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, statistics) in [
            (1, r#"{"goals":1}"#),
            (2, r#"{"goals":5}"#),
            (3, r#"{"goals":9}"#),
            (4, r#"{"goals":null}"#),
            (5, r#"{"goals":10}"#),
        ] {
            insert_moneyball_statistics(&conn, snapshot_id, uid, statistics, Some("{}"));
        }
        let requested_fields = ["moneyball.goals".to_string()];
        let search = |op: &str, target: i64, comparison: Comparison| {
            search_moneyball(
                &conn,
                0,
                DEFAULT_PAGE_LIMIT,
                SortField::Name,
                SortDir::Asc,
                &requested_fields,
                vec![
                    filter_rule("name", "contains", FilterValue::Text("Cohort".to_string())),
                    filter_rule("moneyball.goals", op, FilterValue::Integer(target)),
                ],
                Some("and"),
                comparison,
            )
            .expect("percentile filter")
        };
        let uids = |page: &SearchPlayersPage| {
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>()
        };

        // The central midfielders' 1, 5, and 9 rank at 0, 50, and 100, though the name
        // filter leaves out the 9; the striker, alone in his family, ranks at 50.
        let own = search("pct_gte", 50, Comparison::Own);
        assert_eq!(uids(&own), [2, 5]);
        assert_eq!(
            own.players[0].moneyball_percentiles.get("moneyball.goals"),
            Some(&Some(50))
        );
        // Compared with central midfielders, the striker's 10 is the best.
        assert_eq!(
            uids(&search(
                "pct_gte",
                90,
                Comparison::Group(ComparisonGroup::family(PositionFamily::CentralMidfielder))
            )),
            [5]
        );
        // A player without the statistic matches neither bound.
        assert_eq!(uids(&search("pct_lte", 50, Comparison::Own)), [2, 1, 5]);
    }

    #[test]
    fn unions_percentile_rules_under_or_and_ranks_lower_is_better_metrics_as_best() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("moneyball-percentile-or.db"));
        ingest_players(
            &mut conn,
            vec![
                player_template(1, "Named", 100),
                player_template(2, "Fast scorer", 100),
                player_template(3, "Slow scorer", 100),
            ],
        );
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, minutes_per_goal) in [(1, 300), (2, 60), (3, 400)] {
            insert_moneyball_statistics(
                &conn,
                snapshot_id,
                uid,
                &json!({ "minutes_per_goal": minutes_per_goal }).to_string(),
                Some(r#"{"minutes_per_goal":50}"#),
            );
        }

        let page = search_moneyball(
            &conn,
            0,
            DEFAULT_PAGE_LIMIT,
            SortField::Name,
            SortDir::Asc,
            &[],
            vec![
                filter_rule("name", "contains", FilterValue::Text("Named".to_string())),
                filter_rule(
                    "moneyball.minutes_per_goal",
                    "pct_gte",
                    FilterValue::Integer(90),
                ),
            ],
            Some("or"),
            Comparison::Own,
        )
        .expect("mixed OR percentile filter");

        assert_eq!(
            page.players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            [2, 1]
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
        // Two tied central midfielders above a third score 50, 50, and 0.
        insert_role_row(&conn, snapshot_id, 1, role_id, Some(2.0), Some(50), None);
        insert_role_row(&conn, snapshot_id, 2, role_id, Some(2.0), Some(50), None);
        insert_role_row(&conn, snapshot_id, 3, role_id, Some(1.0), Some(0), None);
        insert_role_row(&conn, snapshot_id, 4, role_id, None, None, Some(0));
        let requested_fields = [format!("moneyball_role.{role_id}")];
        let sort_by = SortField::parse(&format!("moneyball_role.{role_id}")).expect("role sort");

        let descending = search_moneyball(
            &conn,
            1,
            1,
            sort_by.clone(),
            SortDir::Desc,
            &requested_fields,
            Vec::new(),
            None,
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
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
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
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
    fn one_request_reads_player_fields_beside_moneyball_values() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("merged-fields.db"));
        let players = (1..=3)
            .map(|uid| {
                let mut player = player_template(uid, &format!("P{uid}"), 100);
                player["attributes"] = json!({ "Acceleration": 10 + uid });
                player
            })
            .collect();
        ingest_players(&mut conn, players);
        let snapshot_id = current_snapshot_id(&conn);
        for (uid, value) in [(1, 1.0), (2, 4.0)] {
            insert_role_row(
                &conn,
                snapshot_id,
                uid,
                MONEYBALL_ROLE,
                Some(value),
                None,
                None,
            );
        }
        let role = moneyball_role_field();
        let fields = [
            "attr.Acceleration".to_string(),
            role.clone(),
            "moneyball.minutes".to_string(),
        ];
        let filter = parse_filter_ast(
            vec![
                filter_rule("attr.Acceleration", "gte", FilterValue::Integer(12)),
                filter_rule(&role, "gte", FilterValue::Integer(0)),
            ],
            Some("and"),
        )
        .expect("filter");

        let page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: DEFAULT_PAGE_LIMIT,
                sort_by: SortField::parse("attr.Acceleration").expect("attribute sort"),
                sort_dir: SortDir::Desc,
                filter_ast: Some(&filter),
                requested_fields: &fields,
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("merged page");

        // UID 3 has the pace but no season stats, so the Moneyball rule drops him; UID 1 lacks
        // the pace.
        assert_eq!(
            page.players
                .iter()
                .map(|player| (
                    player.uid,
                    player.dynamic_values["attr.Acceleration"].clone(),
                    player.dynamic_values[&role].is_some(),
                ))
                .collect::<Vec<_>>(),
            [(2, Some(DynamicValue::Integer(12)), true)]
        );
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
        let page_cur = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
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
        let page_pot = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_potential.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_potential.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
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
        let page_desc = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("desc");
        // Expected order: 1 (90), then 3 and 4 (70 tie -> UID 3 before 4), then 2 null last
        assert_eq!(
            page_desc.players.iter().map(|p| p.uid).collect::<Vec<_>>(),
            vec![1, 3, 4, 2]
        );
        let page_asc = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
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
    fn tactic_moneyball_mapped_numeric_and_uncovered_null_with_sort() {
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
            // UIDs 1 and 3, the keepers, tie; UID 2 is rated too low to keep goal.
            for m in gk_ip_def.metrics.iter().chain(gk_oop_def.metrics.iter()) {
                stats.insert(
                    m.key.clone(),
                    serde_json::json!(if uid == 2 { 10.0 } else { 11.0 }),
                );
                perc.insert(m.key.clone(), serde_json::json!(70));
            }
            let stats_json = serde_json::to_string(&stats).unwrap();
            let perc_json = serde_json::to_string(&perc).unwrap();
            insert_moneyball_statistics(&conn, snapshot_id, uid, &stats_json, Some(&perc_json));
        }
        // Query mapped goalkeeper tactic should be numeric for uid1 and 3, null for uid2
        let page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_moneyball.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_moneyball.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
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
                        .get("tactic_moneyball.goalkeeper")
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
        let page_uncovered = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_moneyball.centre_forward").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_moneyball.centre_forward".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("uncovered");
        for p in &page_uncovered.players {
            assert_eq!(
                p.dynamic_values.get("tactic_moneyball.centre_forward"),
                Some(&None)
            );
        }
        // Sort nulls last both directions
        let page_desc = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_moneyball.goalkeeper").expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_moneyball.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("desc");
        // Highest first, null last, uid tie (uid1 and 3 same score -> uid1 before 3)
        assert_eq!(page_desc.players.last().unwrap().uid, 2);
        assert_eq!(page_desc.players[0].uid, 1);
        assert_eq!(page_desc.players[1].uid, 3);
        let page_filt = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_moneyball.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_moneyball.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
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
    fn tactic_moneyball_sort_without_chosen_metrics_scores_the_slots_roles() {
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
        // Sort-only: request no tactic field, but sort by the goalkeeper slot's Moneyball
        // Score, which has no chosen metrics and so scores its roles' Moneyball definitions.
        let page_sort_only = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_moneyball.goalkeeper").expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &[],
                comparison: Comparison::Own,
                shortlist_id: None,
            },
        )
        .expect("sort-only");
        assert_eq!(page_sort_only.total, 2);
        assert_eq!(page_sort_only.players[0].uid, 2);
        assert_eq!(page_sort_only.players[1].uid, 1);
        // The old Custom group is gone.
        assert_eq!(
            SortField::parse("tactic_custom.goalkeeper").unwrap_err(),
            "unknown player metric: tactic_custom.goalkeeper"
        );
    }

    fn tactic_scores_by_uid(page: &SearchPlayersPage, field: &str) -> Vec<(i64, Option<i64>)> {
        page.players
            .iter()
            .map(|player| {
                let value = match player.dynamic_values.get(field) {
                    Some(Some(DynamicValue::Integer(value))) => Some(*value),
                    Some(None) => None,
                    other => panic!("unexpected {field} value: {other:?}"),
                };
                (player.uid, value)
            })
            .collect()
    }

    #[test]
    fn tactic_moneyball_custom_scores_rank_within_the_slots_group_with_null_last_sort() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-moneyball-custom.db"));
        // uid 2 cannot play GK (familiarity 10); uid 3 is GK 14, a 5-point
        // penalty in each of the goalkeeper slot's IP and OOP positions.
        let players = (1..=5)
            .map(|uid| {
                let mut p = player_template(uid, &format!("P{uid}"), 150);
                let goalkeeper = match uid {
                    2 => 10,
                    3 => 14,
                    _ => 18,
                };
                p["positions"] = serde_json::json!({"GK": goalkeeper, "MC": 18});
                p
            })
            .collect();
        ingest_players(&mut conn, players);
        let snapshot_id = current_snapshot_id(&conn);
        let save_id = snapshot_save_id(&conn, snapshot_id).expect("save");
        let mut tactic =
            crate::features::planner::tactic::get_tactic(&conn, save_id).expect("tactic");
        assert_eq!(tactic.lanes[0].lane_id, "goalkeeper");
        // goals_conceded_per_90 is lower-is-better, so fewer goals rank higher.
        tactic.lanes[0].custom_metric_keys = vec![
            "save_ratio".to_string(),
            "goals_conceded_per_90".to_string(),
        ];
        crate::features::planner::tactic::save_tactic(&conn, save_id, &tactic)
            .expect("save metrics");
        // uid 4 has no goals_conceded_per_90 statistic.
        let rows = [
            (1, 0.5, Some(1.0)),
            (2, 0.1, Some(2.0)),
            (3, 0.95, Some(3.0)),
            (4, 0.9, None),
            (5, 0.5, Some(2.0)),
        ];
        for (uid, save_ratio, conceded) in rows {
            let mut statistics = serde_json::json!({ "save_ratio": save_ratio });
            if let Some(value) = conceded {
                statistics["goals_conceded_per_90"] = serde_json::json!(value);
            }
            insert_moneyball_statistics(
                &conn,
                snapshot_id,
                uid,
                &statistics.to_string(),
                Some("{}"),
            );
        }
        let custom_page = |conn: &Connection, field: &str, sort_dir, comparison| {
            search_players_uncached(
                conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: 10,
                    sort_by: SortField::parse(field).expect("custom sort"),
                    sort_dir,
                    filter_ast: None,
                    requested_fields: &[field.to_string()],
                    comparison,
                    shortlist_id: None,
                },
            )
            .expect("custom search")
        };
        let field = "tactic_moneyball.goalkeeper";

        // The slot ranks among keepers rated 15 or more: UIDs 1, 4, and 5. Their save
        // ratios of 0.5, 0.9, and 0.5 put UIDs 1 and 5 at 0, and UID 3's 0.95 above them
        // all; their goals conceded of 1.0 and 2.0 put UID 1 at 100 and UID 5 at 0, and
        // UID 3's 3.0 below both. UID 3 averages 50, minus 10 for GK 14. Nulls stay last.
        let full_desc = custom_page(&conn, field, SortDir::Desc, Comparison::Own);
        assert_eq!(
            tactic_scores_by_uid(&full_desc, field),
            [
                (1, Some(50)),
                (3, Some(40)),
                (5, Some(0)),
                (2, None),
                (4, None)
            ]
        );
        let full_asc = custom_page(&conn, field, SortDir::Asc, Comparison::Own);
        assert_eq!(
            tactic_scores_by_uid(&full_asc, field),
            [
                (5, Some(0)),
                (3, Some(40)),
                (1, Some(50)),
                (2, None),
                (4, None)
            ]
        );

        // The table's comparison ranks statistic columns, not slot scores.
        let compared_with_strikers = custom_page(
            &conn,
            field,
            SortDir::Desc,
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
        );
        assert_eq!(
            tactic_scores_by_uid(&compared_with_strikers, field),
            tactic_scores_by_uid(&full_desc, field)
        );

        // The custom score covers both phases and ignores the IP/OOP weight.
        tactic.lanes[0].ip_weight = 0.9;
        crate::features::planner::tactic::save_tactic(&conn, save_id, &tactic)
            .expect("save weight");
        let reweighted = custom_page(
            &conn,
            field,
            SortDir::Desc,
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
        );
        assert_eq!(
            tactic_scores_by_uid(&reweighted, field),
            tactic_scores_by_uid(&full_desc, field)
        );

        // A slot without metrics has no custom score.
        let empty_field = "tactic_moneyball.left_back";
        let empty = custom_page(
            &conn,
            empty_field,
            SortDir::Desc,
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
        );
        assert_eq!(
            tactic_scores_by_uid(&empty, empty_field),
            [(1, None), (2, None), (3, None), (4, None), (5, None)]
        );
    }

    #[test]
    fn tactic_general_sql_vs_rust_parity_and_penalty_saturation() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tactic-parity.db"));
        // Player at half-round boundary: ip 81, oop 82, weight 0.5 => 81.5 => 82
        // Familiarity tests: 12..14 penalized, 15 not; soft foot -5; saturation at 0; missing score -> None
        let mut p_half = player_template(1, "HalfRound", 150);
        p_half["age"] = serde_json::json!(26);
        p_half["preferredFoot"] = serde_json::json!("right");
        p_half["positions"] = serde_json::json!({"GK": 15});
        let mut p_fam14 = player_template(2, "Fam14", 150);
        p_fam14["age"] = serde_json::json!(26);
        p_fam14["preferredFoot"] = serde_json::json!("right");
        p_fam14["positions"] = serde_json::json!({"GK": 14});
        let mut p_fam12 = player_template(3, "Fam12", 150);
        p_fam12["age"] = serde_json::json!(26);
        p_fam12["preferredFoot"] = serde_json::json!("right");
        p_fam12["positions"] = serde_json::json!({"GK": 12});
        let mut p_soft = player_template(4, "SoftFoot", 150);
        p_soft["age"] = serde_json::json!(26);
        p_soft["preferredFoot"] = serde_json::json!("left");
        p_soft["positions"] = serde_json::json!({"GK": 14});
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
            vec![p_half, p_fam14, p_fam12, p_soft, p_saturate, p_missing],
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
            &std::collections::BTreeMap::from([("GK".to_string(), Some(15))]),
            &lane,
        );
        assert_eq!(expected_half, Some(82));
        // Fam14: familiarity 14 both phases -> -10, 70 blended -> 60
        let expected_fam14 = crate::features::planner::fit::tactic_adjusted_score(
            Some(70),
            Some(70),
            0.5,
            "right",
            &std::collections::BTreeMap::from([("GK".to_string(), Some(14))]),
            &lane,
        );
        assert_eq!(expected_fam14, Some(60));
        // Soft foot with 14: -10 fam + -5 foot = -15 => 55
        // But lane uses same GK for both IP/OOP, so familiarity penalty counted twice (once per phase) + soft foot once
        // For p_soft: GK 14 => -5 per phase => -10 + soft -5 => -15 => 55
        let expected_soft = crate::features::planner::fit::tactic_adjusted_score(
            Some(70),
            Some(70),
            0.5,
            "left",
            &std::collections::BTreeMap::from([("GK".to_string(), Some(14))]),
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
        let page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::Ca,
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: None,
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
            let err = search_players_uncached(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: 10,
                    sort_by: SortField::parse(field).expect("sort"),
                    sort_dir: SortDir::Asc,
                    filter_ast: None,
                    requested_fields: &[field.to_string()],
                    comparison: Comparison::Own,
                    shortlist_id: None,
                },
            )
            .expect_err("must fail completeness");
            assert!(
                err.contains("Current") || err.contains("incomplete"),
                "err was {err}"
            );
            // Ensure potential completeness also fails for General sort without requested field (sort-only)
            let err2 = search_players_uncached(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: 10,
                    sort_by: SortField::parse(field).expect("sort"),
                    sort_dir: SortDir::Asc,
                    filter_ast: None,
                    requested_fields: &[],
                    comparison: Comparison::Own,
                    shortlist_id: None,
                },
            )
            .expect_err("sort-only must also fail");
            assert!(err2.contains("Current") || err2.contains("incomplete"));
        }
    }

    #[test]
    fn tactic_general_shortlist_value_and_sort() {
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
        let shortlist = Some(seed_player_shortlist(&conn, &[1, 2]));
        set_role_score(&conn, 1, "goalkeeper_ip", Some(90));
        set_role_score(&conn, 1, "line_holding_keeper_oop", Some(90));
        set_role_score(&conn, 2, "goalkeeper_ip", Some(60));
        set_role_score(&conn, 2, "line_holding_keeper_oop", Some(60));
        set_role_score(&conn, 3, "goalkeeper_ip", Some(80));
        set_role_score(&conn, 3, "line_holding_keeper_oop", Some(80));
        let page = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Desc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: shortlist,
            },
        )
        .expect("restricted general");
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
        let page2 = search_players_uncached(
            &conn,
            SearchPlayersRequest {
                offset: 0,
                limit: 10,
                sort_by: SortField::parse("tactic_current.goalkeeper").expect("sort"),
                sort_dir: SortDir::Asc,
                filter_ast: None,
                requested_fields: &["tactic_current.goalkeeper".to_string()],
                comparison: Comparison::Own,
                shortlist_id: shortlist,
            },
        )
        .expect("restricted general nulls last");
        assert_eq!(page2.players.last().unwrap().uid, 1);
    }

    #[test]
    fn tactic_moneyball_role_scores_ignore_the_tables_filters() {
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
            let stats = keys
                .iter()
                .map(|key| (key.clone(), serde_json::json!(raw)))
                .collect::<std::collections::BTreeMap<_, _>>();
            insert_moneyball_statistics(
                &conn,
                snapshot_id,
                uid,
                &serde_json::to_string(&stats).unwrap(),
                Some("{}"),
            );
        }
        let page = |filter_ast: Option<&FilterAst>| {
            search_players_uncached(
                &conn,
                SearchPlayersRequest {
                    offset: 0,
                    limit: 10,
                    sort_by: SortField::parse("tactic_moneyball.goalkeeper").expect("sort"),
                    sort_dir: SortDir::Desc,
                    filter_ast,
                    requested_fields: &["tactic_moneyball.goalkeeper".to_string()],
                    comparison: Comparison::Own,
                    shortlist_id: None,
                },
            )
            .expect("goalkeeper slot page")
        };
        let scores = |page: &SearchPlayersPage| {
            page.players
                .iter()
                .map(|player| {
                    let score = player
                        .dynamic_values
                        .get("tactic_moneyball.goalkeeper")
                        .cloned()
                        .flatten();
                    (player.uid, score)
                })
                .collect::<Vec<_>>()
        };

        // The four keepers rank among each other, so their scores are tiered.
        let unfiltered = scores(&page(None));
        assert_eq!(
            unfiltered.iter().map(|(uid, _)| *uid).collect::<Vec<_>>(),
            [4, 3, 2, 1]
        );
        assert!(unfiltered.iter().all(|(_, score)| score.is_some()));
        assert_ne!(unfiltered[0].1, unfiltered[3].1);
        // A filter that leaves out the weakest keeper keeps everyone else's score.
        let filter = parse_filter_ast(
            vec![FilterRule {
                field: format!("moneyball.{}", keys[0]),
                op: "gt".to_string(),
                value: FilterValue::Integer(5),
            }],
            None,
        )
        .expect("filter");
        assert_eq!(scores(&page(Some(&filter))), unfiltered[..3]);
    }

    struct GeneralShape {
        name: &'static str,
        sort: &'static str,
        dir: SortDir,
        filters: Option<FilterAst>,
        fields: Vec<String>,
        shortlist_id: Option<i64>,
    }

    impl GeneralShape {
        fn new(name: &'static str, sort: &'static str, dir: SortDir, fields: &[&str]) -> Self {
            Self {
                name,
                sort,
                dir,
                filters: None,
                fields: fields.iter().map(|field| field.to_string()).collect(),
                shortlist_id: None,
            }
        }

        fn filtered(self, rules: Vec<FilterRule>, combine: &str) -> Self {
            Self {
                filters: Some(parse_filter_ast(rules, Some(combine)).expect("filters")),
                ..self
            }
        }

        fn request(&self, offset: usize, limit: usize) -> SearchPlayersRequest<'_> {
            SearchPlayersRequest {
                offset,
                limit,
                sort_by: SortField::parse(self.sort).expect("sort"),
                sort_dir: self.dir,
                filter_ast: self.filters.as_ref(),
                requested_fields: &self.fields,
                comparison: Comparison::Own,
                shortlist_id: self.shortlist_id,
            }
        }
    }

    /// A database with readers whose players tie and miss values across every General sort.
    fn open_general_equivalence_db() -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = crate::db::open(&temp_dir.path().join(crate::db::APP_DB_FILE)).expect("open db");
        let names = [
            "Zed", "alpha", "Bravo", "charlie", "Delta", "echo", "Alpha", "foxtrot", "Golf",
            "hotel", "India",
        ];
        let abilities = [150, 120, 150, 90, 120, 150, 180, 90, 120, 150, 100];
        let players = (1..=11)
            .map(|uid| {
                let index = uid as usize - 1;
                let mut player = player_template(uid, names[index], abilities[index]);
                player["age"] = json!(18 + (uid as i64 * 5) % 14);
                player["marketValueGbp"] = json!((uid as i64 % 4) * 500_000);
                player["currentClub"] = match uid % 4 {
                    0 => json!("alpha FC"),
                    1 => json!("Alpha FC"),
                    2 => json!("Beta"),
                    _ => json!(null),
                };
                player["attributes"] = if uid % 5 == 0 {
                    json!({})
                } else {
                    json!({ "Acceleration": 8 + uid % 4 })
                };
                player["positions"] = match uid % 3 {
                    0 => json!({ "MC": 18 }),
                    1 => json!({ "GK": 18 }),
                    _ => json!({ "GK": 14, "MC": 12 }),
                };
                player
            })
            .collect();
        ingest_players(&mut db.0.lock().expect("lock writer"), players);

        let writer = db.0.lock().expect("lock writer");
        for (uid, score) in [
            (1, Some(70)),
            (2, Some(70)),
            (4, None),
            (7, Some(85)),
            (8, Some(40)),
        ] {
            set_role_score(&writer, uid, "goalkeeper_ip", score);
            set_role_score(&writer, uid, "line_holding_keeper_oop", score);
            set_potential_role_score(&writer, uid, "goalkeeper_ip", score.map(|value| value + 5));
            set_potential_role_score(
                &writer,
                uid,
                "line_holding_keeper_oop",
                score.map(|value| value + 5),
            );
        }
        let snapshot_id = current_snapshot_id(&writer);
        writer
            .execute(
                "INSERT INTO club_dna_definitions (save_id, attribute_ids_json, definition_version)
                 SELECT save_id, '[\"attr.Acceleration\"]', 2 FROM snapshots WHERE id = ?1",
                [snapshot_id],
            )
            .expect("insert Club DNA definition");
        for (uid, definition_version, score) in [
            (1, 2, Some(30)),
            (3, 2, Some(30)),
            (6, 2, None),
            (9, 1, Some(99)),
            (10, 2, Some(75)),
        ] {
            writer
                .execute(
                    "INSERT INTO club_dna_scores (
                        snapshot_id, uid, definition_version, score_model_version, score
                     ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        snapshot_id,
                        uid,
                        definition_version,
                        SCORE_MODEL_VERSION,
                        score
                    ],
                )
                .expect("insert Club DNA score");
        }
        drop(writer);
        (temp_dir, db)
    }

    /// Joins three-row cached windows and compares each with the uncached page at its offset.
    fn assert_cached_windows_match(db: &Db, orders: &PlayerOrderCache, shape: &GeneralShape) {
        let registry = TableRegistry::default();
        let complete = search_players_uncached(
            &db.read().expect("reader"),
            shape.request(0, MAX_PAGE_LIMIT),
        )
        .expect("uncached complete page");
        let mut joined = Vec::new();
        for offset in (0..complete.players.len() + 3).step_by(3) {
            let cached = search_players(
                db,
                orders,
                &ComparisonGroupsCache::default(),
                &registry,
                None,
                shape.request(offset, 3),
            )
            .unwrap_or_else(|error| panic!("{}: {error}", shape.name));
            let uncached =
                search_players_uncached(&db.read().expect("reader"), shape.request(offset, 3))
                    .expect("uncached page");
            assert_eq!(cached, uncached, "{} at offset {offset}", shape.name);
            joined.extend(cached.players);
        }
        assert_eq!(joined, complete.players, "{} joined windows", shape.name);
    }

    #[test]
    fn cached_general_windows_equal_uncached_pages_for_every_shape() {
        let (_temp_dir, db) = open_general_equivalence_db();
        let orders = PlayerOrderCache::new(4);
        let shapes = vec![
            GeneralShape::new("CA", "ca", SortDir::Desc, &["height"]),
            GeneralShape::new("name", "name", SortDir::Asc, &[]),
            GeneralShape::new("PA", "pa", SortDir::Asc, &[]),
            GeneralShape::new("age", "age", SortDir::Desc, &[]),
            GeneralShape::new("value", "value", SortDir::Desc, &[]),
            GeneralShape::new("club", "club", SortDir::Asc, &[]),
            GeneralShape::new(
                "current role",
                "role.goalkeeper_ip",
                SortDir::Desc,
                &["role.goalkeeper_ip", "potential_role.goalkeeper_ip"],
            ),
            GeneralShape::new(
                "potential role",
                "potential_role.goalkeeper_ip",
                SortDir::Asc,
                &["potential_role.goalkeeper_ip"],
            ),
            GeneralShape::new("Club DNA", "club_dna", SortDir::Desc, &["club_dna"]),
            GeneralShape::new(
                "tactic column",
                "tactic_current.goalkeeper",
                SortDir::Desc,
                &["tactic_current.goalkeeper", "tactic_potential.goalkeeper"],
            ),
            GeneralShape::new(
                "attribute",
                "attr.Acceleration",
                SortDir::Asc,
                &["attr.Acceleration", "position"],
            ),
            GeneralShape::new("AND filter", "pa", SortDir::Desc, &["club_dna"]).filtered(
                vec![
                    filter_rule("ca", "gte", FilterValue::Integer(120)),
                    filter_rule("age", "lte", FilterValue::Integer(26)),
                ],
                "and",
            ),
            GeneralShape::new("OR filter", "value", SortDir::Asc, &["role.goalkeeper_ip"])
                .filtered(
                    vec![
                        filter_rule("ca", "eq", FilterValue::Integer(90)),
                        filter_rule("role.goalkeeper_ip", "gte", FilterValue::Integer(70)),
                    ],
                    "or",
                ),
            GeneralShape::new("Club DNA filter", "club_dna", SortDir::Asc, &["club_dna"]).filtered(
                vec![filter_rule("club_dna", "gte", FilterValue::Integer(10))],
                "and",
            ),
            GeneralShape::new("empty", "ca", SortDir::Desc, &[]).filtered(
                vec![filter_rule("ca", "gte", FilterValue::Integer(999))],
                "and",
            ),
        ];
        for shape in &shapes {
            assert_cached_windows_match(&db, &orders, shape);
        }

        // Shortlist 1 is unknown until it is seeded, then restricts; another save's
        // shortlist reports no_shortlist on both paths.
        let shortlist = GeneralShape {
            shortlist_id: Some(1),
            ..GeneralShape::new("shortlist", "ca", SortDir::Desc, &["role.goalkeeper_ip"])
        };
        assert_cached_windows_match(&db, &orders, &shortlist);
        {
            let writer = db.0.lock().expect("lock writer");
            assert_eq!(seed_player_shortlist(&writer, &[2, 3, 5, 99]), 1);
            writer
                .execute_batch(
                    "INSERT INTO saves (id, name, is_active) VALUES (99, 'Other save', 0);
                     INSERT INTO player_shortlists (id, save_id, name) VALUES (7, 99, 'Other');
                     INSERT INTO player_shortlist_entries (shortlist_id, player_uid)
                     VALUES (7, 2);",
                )
                .expect("seed another save's shortlist");
        }
        assert_cached_windows_match(&db, &orders, &shortlist);
        assert_cached_windows_match(
            &db,
            &orders,
            &GeneralShape {
                shortlist_id: Some(7),
                ..GeneralShape::new("foreign shortlist", "ca", SortDir::Desc, &[])
            },
        );
    }

    #[test]
    fn later_windows_reuse_the_order_until_a_write() {
        let (_temp_dir, db) = open_general_equivalence_db();
        let orders = PlayerOrderCache::new(4);
        let registry = TableRegistry::default();
        let shape = GeneralShape::new("CA", "ca", SortDir::Desc, &["height"]);
        let with_column = GeneralShape::new("CA", "ca", SortDir::Desc, &["attr.Acceleration"]);
        ORDER_COMPUTATIONS.with(|count| count.set(0));

        search_players(
            &db,
            &orders,
            &ComparisonGroupsCache::default(),
            &registry,
            None,
            shape.request(0, 3),
        )
        .expect("first window");
        search_players(
            &db,
            &orders,
            &ComparisonGroupsCache::default(),
            &registry,
            None,
            shape.request(6, 3),
        )
        .expect("later window");
        search_players(
            &db,
            &orders,
            &ComparisonGroupsCache::default(),
            &registry,
            None,
            with_column.request(3, 3),
        )
        .expect("window with another column");
        assert_eq!(ORDER_COMPUTATIONS.with(Cell::get), 1);

        db.0.lock()
            .expect("lock writer")
            .execute("UPDATE players SET ca = 200 WHERE uid = 4", [])
            .expect("change a player");
        let fresh = search_players(
            &db,
            &orders,
            &ComparisonGroupsCache::default(),
            &registry,
            None,
            shape.request(0, 3),
        )
        .expect("window after a write");
        assert_eq!(ORDER_COMPUTATIONS.with(Cell::get), 2);
        assert_eq!(fresh.players[0].uid, 4);
        assert_eq!(
            fresh,
            search_players_uncached(&db.read().expect("reader"), shape.request(0, 3))
                .expect("uncached page")
        );
    }

    #[test]
    fn a_newer_general_query_supersedes_a_running_one_and_completes() {
        let (_temp_dir, db) = open_general_equivalence_db();
        let orders = PlayerOrderCache::new(4);
        let registry = TableRegistry::default();
        let older = GeneralShape::new("CA", "ca", SortDir::Desc, &[]);
        let newer = || GeneralShape::new("name", "name", SortDir::Asc, &[]);
        let older_ticket =
            registry.arrive("search", &search_query_fingerprint(&older.request(0, 3)));

        let (sender, receiver) = std::sync::mpsc::channel();
        {
            let (db, orders, registry) = (db.clone(), orders.clone(), registry.clone());
            BEFORE_ORDER_QUERY.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    let newer = newer();
                    let ticket =
                        registry.arrive("search", &search_query_fingerprint(&newer.request(0, 3)));
                    sender
                        .send(search_players(
                            &db,
                            &orders,
                            &ComparisonGroupsCache::default(),
                            &registry,
                            Some(&ticket),
                            newer.request(0, 3),
                        ))
                        .expect("send newer page");
                }));
            });
        }

        let superseded = search_players(
            &db,
            &orders,
            &ComparisonGroupsCache::default(),
            &registry,
            Some(&older_ticket),
            older.request(0, 3),
        );
        assert_eq!(
            superseded,
            Err(crate::db::result_cache::SUPERSEDED.to_string())
        );
        let newer = newer();
        assert_eq!(
            receiver.recv().expect("newer page"),
            search_players_uncached(&db.read().expect("reader"), newer.request(0, 3))
        );
    }

    const MONEYBALL_ROLE: &str = "mc_central_midfielder_ip";

    fn moneyball_role_field() -> String {
        format!("moneyball_role.{MONEYBALL_ROLE}")
    }

    struct MoneyballShape {
        name: &'static str,
        sort: String,
        dir: SortDir,
        comparison: Comparison,
        filters: Option<FilterAst>,
        fields: Vec<String>,
        shortlist_id: Option<i64>,
    }

    impl MoneyballShape {
        fn new(name: &'static str, sort: &str, dir: SortDir, fields: &[&str]) -> Self {
            Self {
                name,
                sort: sort.to_string(),
                dir,
                comparison: Comparison::Own,
                filters: None,
                fields: fields.iter().map(|field| field.to_string()).collect(),
                shortlist_id: None,
            }
        }

        /// Restricts to shortlist 1, the first one a test seeds.
        fn shortlisted(self) -> Self {
            Self {
                shortlist_id: Some(1),
                ..self
            }
        }

        fn filtered(self, rules: Vec<FilterRule>, combine: &str) -> Self {
            Self {
                filters: Some(parse_filter_ast(rules, Some(combine)).expect("filters")),
                ..self
            }
        }

        fn compared_with(&self, comparison: Comparison) -> Self {
            Self {
                name: self.name,
                sort: self.sort.clone(),
                dir: self.dir,
                comparison,
                filters: self.filters.clone(),
                fields: self.fields.clone(),
                shortlist_id: self.shortlist_id,
            }
        }

        fn request(&self, offset: usize, limit: usize) -> SearchPlayersRequest<'_> {
            SearchPlayersRequest {
                offset,
                limit,
                sort_by: SortField::parse(&self.sort).expect("sort"),
                sort_dir: self.dir,
                filter_ast: self.filters.as_ref(),
                requested_fields: &self.fields,
                comparison: self.comparison,
                shortlist_id: self.shortlist_id,
            }
        }
    }

    /// A database with readers whose Moneyball statistics and stored percentiles tie and miss
    /// values, with a custom goalkeeper slot and an uncovered centre-forward slot. Its players
    /// are spread over a strong league, a weaker one, and none.
    fn open_moneyball_equivalence_db() -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = crate::db::open(&temp_dir.path().join(crate::db::APP_DB_FILE)).expect("open db");
        let names = [
            "Zed", "alpha", "Bravo", "charlie", "Delta", "echo", "Alpha", "foxtrot", "Golf",
            "hotel", "India", "Juliet",
        ];
        let abilities = [150, 120, 150, 90, 120, 150, 180, 90, 120, 150, 100, 110];
        let players = (1..=12)
            .map(|uid| {
                let index = uid as usize - 1;
                let mut player = player_template(uid, names[index], abilities[index]);
                player["age"] = json!(18 + (uid as i64 * 5) % 14);
                player["positions"] = match uid % 4 {
                    0 => json!({ "GK": 18, "MC": 18 }),
                    1 => json!({ "GK": 14, "MC": 12 }),
                    2 => json!({ "GK": 10, "ST": 18 }),
                    _ => json!({ "GK": 18, "ST": 16 }),
                };
                player["preferredFoot"] = json!(if uid % 3 == 0 { "left" } else { "right" });
                player["division"] = match uid % 3 {
                    0 => json!("Top"),
                    1 => json!("Middle"),
                    _ => Value::Null,
                };
                player
            })
            .collect();
        ingest_players(&mut db.0.lock().expect("lock writer"), players);

        let writer = db.0.lock().expect("lock writer");
        let snapshot_id = current_snapshot_id(&writer);
        measure_leagues(
            &writer,
            snapshot_id,
            &[("Top", 150.0), ("Middle", 140.0), ("Weak", 130.0)],
        );
        let save_id = snapshot_save_id(&writer, snapshot_id).expect("save");
        let mut tactic =
            crate::features::planner::tactic::get_tactic(&writer, save_id).expect("tactic");
        tactic.lanes[0].custom_metric_keys = vec![
            "save_ratio".to_string(),
            "goals_conceded_per_90".to_string(),
        ];
        let centre_forward = tactic
            .lanes
            .iter()
            .position(|lane| lane.lane_id == "centre_forward")
            .expect("centre-forward lane");
        tactic.lanes[centre_forward].ip_role_id = "second_striker_ip".to_string();
        tactic.lanes[centre_forward].ip_position = "ST".to_string();
        crate::features::planner::tactic::save_tactic(&writer, save_id, &tactic)
            .expect("save tactic");

        let catalog = builtin_catalog().expect("catalog");
        let mut keys = BTreeSet::from(
            [
                "goals",
                "minutes_per_goal",
                "save_ratio",
                "goals_conceded_per_90",
                "xg",
            ]
            .map(str::to_string),
        );
        for definition in catalog.definitions.iter().filter(|definition| {
            definition.id == MONEYBALL_ROLE
                || (definition.position_tags.contains(&"GK".to_string())
                    && matches!(
                        definition.attribute_role_id.as_deref(),
                        Some("goalkeeper_ip" | "line_holding_keeper_oop")
                    ))
        }) {
            keys.extend(definition.metrics.iter().map(|metric| metric.key.clone()));
        }
        // UID 11 has no stored percentiles and UID 12 no statistics, so neither is in a cohort.
        for uid in 1..=11_usize {
            let mut statistics = Map::new();
            let mut percentiles = Map::new();
            for (index, key) in keys.iter().enumerate() {
                let seed = uid * 7 + index * 3;
                statistics.insert(
                    key.clone(),
                    if seed % 6 == 0 {
                        Value::Null
                    } else {
                        json!((seed % 5) as f64 * 1.5)
                    },
                );
                percentiles.insert(
                    key.clone(),
                    if (uid + index) % 7 == 0 {
                        Value::Null
                    } else {
                        json!((uid * 13 + index * 7) % 101)
                    },
                );
            }
            let percentiles = Value::Object(percentiles).to_string();
            insert_moneyball_statistics(
                &writer,
                snapshot_id,
                uid as i64,
                &Value::Object(statistics).to_string(),
                (uid != 11).then_some(percentiles.as_str()),
            );
        }
        drop(writer);
        (temp_dir, db)
    }

    /// Joins three-row cached windows and compares each with the uncached page at its offset.
    fn assert_cached_moneyball_windows_match(
        db: &Db,
        orders: &PlayerOrderCache,
        shape: &MoneyballShape,
    ) {
        let registry = TableRegistry::default();
        let name = format!("{} ({:?})", shape.name, shape.comparison);
        let complete = search_players_uncached(
            &db.read().expect("reader"),
            shape.request(0, MAX_PAGE_LIMIT),
        )
        .unwrap_or_else(|error| panic!("{name} uncached: {error}"));
        let mut joined = Vec::new();
        for offset in (0..complete.players.len() + 3).step_by(3) {
            let cached = search_players(
                db,
                orders,
                &ComparisonGroupsCache::default(),
                &registry,
                None,
                shape.request(offset, 3),
            )
            .unwrap_or_else(|error| panic!("{name}: {error}"));
            let uncached =
                search_players_uncached(&db.read().expect("reader"), shape.request(offset, 3))
                    .expect("uncached page");
            assert_eq!(cached, uncached, "{name} at offset {offset}");
            joined.extend(cached.players);
        }
        assert_eq!(joined, complete.players, "{name} joined windows");
    }

    #[test]
    fn moneyball_scores_move_with_each_players_league_until_the_save_turns_it_off() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = crate::db::open(&temp_dir.path().join(crate::db::APP_DB_FILE)).expect("open db");
        let mut players = vec![
            player_template(1, "Top league", 100),
            player_template(2, "Middle league", 100),
            player_template(3, "No league", 100),
        ];
        players[0]["division"] = json!("Top");
        players[1]["division"] = json!("Middle");
        players[2]["division"] = Value::Null;
        ingest_players(&mut db.0.lock().expect("lock writer"), players);
        let save_id = {
            let writer = db.0.lock().expect("lock writer");
            let snapshot_id = current_snapshot_id(&writer);
            measure_leagues(
                &writer,
                snapshot_id,
                &[("Top", 150.0), ("Middle", 120.0), ("Weak", 90.0)],
            );
            // Every player has the same statistics, so the same score of 50 before his
            // league moves it.
            for uid in 1..=3 {
                insert_role_row(
                    &writer,
                    snapshot_id,
                    uid,
                    MONEYBALL_ROLE,
                    Some(4.0),
                    None,
                    None,
                );
            }
            snapshot_save_id(&writer, snapshot_id).expect("save")
        };
        let role = moneyball_role_field();
        // Each returned player's role score, the same from the cached and uncached paths.
        let scores = |rules: Vec<FilterRule>| {
            let shape = MoneyballShape::new("league", &role, SortDir::Desc, &[&role])
                .compared_with(Comparison::Group(ComparisonGroup::family(
                    PositionFamily::Striker,
                )));
            let shape = if rules.is_empty() {
                shape
            } else {
                shape.filtered(rules, "and")
            };
            let cached = search_players(
                &db,
                &PlayerOrderCache::new(2),
                &ComparisonGroupsCache::default(),
                &TableRegistry::default(),
                None,
                shape.request(0, 10),
            )
            .expect("cached page");
            let uncached =
                search_players_uncached(&db.read().expect("reader"), shape.request(0, 10))
                    .expect("uncached page");
            assert_eq!(cached, uncached);
            cached
                .players
                .iter()
                .map(|player| (player.uid, player.dynamic_values[&role].clone()))
                .collect::<Vec<_>>()
        };
        let score = |value: i64| Some(DynamicValue::Integer(value));

        // The middle league is 30 points weaker than the top one, which costs 15. A player
        // with no league counts as in the weakest measured one, 60 points weaker.
        assert_eq!(
            scores(vec![]),
            [(1, score(50)), (2, score(35)), (3, score(20))]
        );
        assert_eq!(
            scores(vec![filter_rule(&role, "gte", FilterValue::Integer(30))]),
            [(1, score(50)), (2, score(35))]
        );

        league_strength::set_enabled(&db.0.lock().expect("lock writer"), save_id, false)
            .expect("turn the adjustment off");
        assert_eq!(
            scores(vec![]),
            [(1, score(50)), (2, score(50)), (3, score(50))]
        );
    }

    #[test]
    fn cached_moneyball_windows_equal_uncached_pages_for_every_shape() {
        let (_temp_dir, db) = open_moneyball_equivalence_db();
        let orders = PlayerOrderCache::new(2);
        let role = moneyball_role_field();
        let role = role.as_str();
        let shapes = vec![
            MoneyballShape::new(
                "statistic sort",
                "moneyball.goals",
                SortDir::Desc,
                &[
                    "moneyball.goals",
                    "moneyball.minutes_per_goal",
                    "moneyball.xg",
                    "height",
                ],
            ),
            MoneyballShape::new(
                "lower-is-better sort",
                "moneyball.minutes_per_goal",
                SortDir::Asc,
                &["moneyball.minutes_per_goal"],
            ),
            MoneyballShape::new("CA ties", "ca", SortDir::Desc, &[]),
            MoneyballShape::new("role column", "name", SortDir::Asc, &[role]),
            MoneyballShape::new(
                "name filter",
                "name",
                SortDir::Asc,
                &[role, "moneyball.goals"],
            )
            .filtered(
                vec![filter_rule(
                    "name",
                    "contains",
                    FilterValue::Text("a".to_string()),
                )],
                "and",
            ),
            MoneyballShape::new("role sort and filter", role, SortDir::Desc, &[role]).filtered(
                vec![
                    filter_rule("name", "contains", FilterValue::Text("a".to_string())),
                    filter_rule(role, "gt", FilterValue::Integer(10)),
                ],
                "and",
            ),
            MoneyballShape::new("mixed OR", role, SortDir::Desc, &[role, "moneyball.goals"])
                .filtered(
                    vec![
                        filter_rule("name", "contains", FilterValue::Text("alpha".to_string())),
                        filter_rule(role, "gt", FilterValue::Integer(60)),
                    ],
                    "or",
                ),
            MoneyballShape::new("role nulls", role, SortDir::Asc, &[role]).filtered(
                vec![filter_rule(role, "neq", FilterValue::Integer(50))],
                "and",
            ),
            MoneyballShape::new("pct_gte", "name", SortDir::Asc, &["moneyball.goals"]).filtered(
                vec![
                    filter_rule("ca", "gte", FilterValue::Integer(100)),
                    filter_rule("moneyball.goals", "pct_gte", FilterValue::Integer(50)),
                ],
                "and",
            ),
            MoneyballShape::new(
                "pct_lte",
                "moneyball.goals",
                SortDir::Desc,
                &["moneyball.goals"],
            )
            .filtered(
                vec![filter_rule(
                    "moneyball.goals",
                    "pct_lte",
                    FilterValue::Integer(40),
                )],
                "and",
            ),
            MoneyballShape::new("pct_between", "name", SortDir::Desc, &["moneyball.xg"]).filtered(
                vec![filter_rule(
                    "moneyball.xg",
                    "pct_between",
                    FilterValue::List(vec![FilterValue::Integer(20), FilterValue::Integer(70)]),
                )],
                "and",
            ),
            MoneyballShape::new("lower-is-better OR", "name", SortDir::Asc, &[]).filtered(
                vec![
                    filter_rule("name", "contains", FilterValue::Text("Zed".to_string())),
                    filter_rule(
                        "moneyball.minutes_per_goal",
                        "pct_gte",
                        FilterValue::Integer(70),
                    ),
                ],
                "or",
            ),
            MoneyballShape::new(
                "tactic sort",
                "tactic_current.goalkeeper",
                SortDir::Asc,
                &["tactic_current.goalkeeper", "tactic_current.centre_forward"],
            ),
            MoneyballShape::new(
                "custom tactic sort",
                "tactic_moneyball.goalkeeper",
                SortDir::Desc,
                &["tactic_moneyball.goalkeeper", "moneyball.save_ratio"],
            ),
            MoneyballShape::new(
                "uncovered tactic sort",
                "tactic_current.centre_forward",
                SortDir::Desc,
                &["tactic_current.centre_forward"],
            ),
            MoneyballShape::new(
                "display-only scores",
                "moneyball.xg",
                SortDir::Desc,
                &[
                    "tactic_current.goalkeeper",
                    "tactic_moneyball.goalkeeper",
                    role,
                ],
            ),
            MoneyballShape::new(
                "statistic filter with tactic sort",
                "tactic_current.goalkeeper",
                SortDir::Desc,
                &["tactic_current.goalkeeper", "moneyball.goals"],
            )
            .filtered(
                vec![filter_rule(
                    "moneyball.goals",
                    "gt",
                    FilterValue::Integer(1),
                )],
                "and",
            ),
            MoneyballShape::new("empty", "ca", SortDir::Desc, &[role]).filtered(
                vec![filter_rule("ca", "gte", FilterValue::Integer(999))],
                "and",
            ),
        ];
        for shape in &shapes {
            for comparison in [
                Comparison::Own,
                Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
            ] {
                assert_cached_moneyball_windows_match(
                    &db,
                    &orders,
                    &shape.compared_with(comparison),
                );
            }
        }

        // Shortlist shapes report no_shortlist alike before a link, then restrict alike.
        let shortlist_shapes = [
            MoneyballShape::new("shortlist", "ca", SortDir::Desc, &["moneyball.goals"]),
            MoneyballShape::new("shortlist role sort", role, SortDir::Desc, &[role]),
            MoneyballShape::new("shortlist mixed OR", role, SortDir::Desc, &[role]).filtered(
                vec![
                    filter_rule("name", "contains", FilterValue::Text("a".to_string())),
                    filter_rule(role, "gt", FilterValue::Integer(40)),
                ],
                "or",
            ),
            MoneyballShape::new(
                "shortlist tactic sort",
                "tactic_moneyball.goalkeeper",
                SortDir::Desc,
                &["tactic_moneyball.goalkeeper", "moneyball.save_ratio"],
            ),
        ]
        .map(MoneyballShape::shortlisted);
        let assert_shortlist_shapes = || {
            for shape in &shortlist_shapes {
                for comparison in [
                    Comparison::Own,
                    Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
                ] {
                    assert_cached_moneyball_windows_match(
                        &db,
                        &orders,
                        &shape.compared_with(comparison),
                    );
                }
            }
        };
        assert_shortlist_shapes();
        assert_eq!(
            seed_player_shortlist(&db.0.lock().expect("lock writer"), &[1, 2, 4, 7, 9, 11, 99]),
            1
        );
        assert_shortlist_shapes();

        let (_empty_dir, empty_db) = {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let db =
                crate::db::open(&temp_dir.path().join(crate::db::APP_DB_FILE)).expect("open db");
            (temp_dir, db)
        };
        assert_cached_moneyball_windows_match(
            &empty_db,
            &PlayerOrderCache::new(2),
            &MoneyballShape::new("no snapshot", "ca", SortDir::Desc, &[role]),
        );
    }

    #[test]
    fn moneyball_shortlist_restricts_rows_but_not_the_comparison_groups() {
        let (_temp_dir, db) = open_moneyball_equivalence_db();
        let orders = PlayerOrderCache::new(4);
        let registry = TableRegistry::default();
        let role = moneyball_role_field();
        let fields = [role.as_str(), "moneyball.goals", "moneyball.xg"];
        let shape = |filters: Vec<FilterRule>, combine: &str, comparison: Comparison| {
            MoneyballShape::new("shortlist", "ca", SortDir::Desc, &fields)
                .filtered(filters, combine)
                .compared_with(comparison)
        };
        let page = |shape: &MoneyballShape| {
            search_players(
                &db,
                &orders,
                &ComparisonGroupsCache::default(),
                &registry,
                None,
                shape.request(0, 50),
            )
            .expect("Moneyball page")
        };
        let ca = |op: &str, value: i64| vec![filter_rule("ca", op, FilterValue::Integer(value))];

        let unlinked = page(&shape(ca("lte", 120), "and", Comparison::Own).shortlisted());
        assert_eq!(unlinked.state, SearchPlayersPageState::NoShortlist);
        assert!(unlinked.players.is_empty());

        // UID 11 has no stored percentiles and UID 99 is not in the snapshot. Of the
        // shortlist, CA 120 or less leaves UIDs 2, 5, 9, and 11. Another shortlist of the save
        // holds UIDs 4, 8, and 12, which also have CA 120 or less, and must not leak in.
        let shortlist = [1, 2, 5, 7, 9, 11, 99];
        {
            let writer = db.0.lock().expect("lock writer");
            assert_eq!(seed_player_shortlist(&writer, &shortlist), 1);
            seed_named_player_shortlist(&writer, "Loans", &[4, 8, 12]);
        }
        let in_shortlist = |page: SearchPlayersPage| {
            page.players
                .into_iter()
                .filter(|player| shortlist.contains(&player.uid))
                .collect::<Vec<_>>()
        };

        let filtered = page(&shape(ca("lte", 120), "and", Comparison::Own).shortlisted());
        assert_eq!(filtered.state, SearchPlayersPageState::Ready);
        assert_eq!(
            filtered
                .players
                .iter()
                .map(|player| player.uid)
                .collect::<Vec<_>>(),
            vec![2, 5, 9, 11]
        );
        assert_eq!(filtered.total, 4);
        // Percentiles and role scores rank within the same groups with or without the
        // shortlist, for either comparison; only the rows are restricted.
        for comparison in [
            Comparison::Own,
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
        ] {
            let shortlisted = page(&shape(ca("lte", 120), "and", comparison).shortlisted());
            let shortlist_off = page(&shape(ca("lte", 120), "and", comparison));
            assert_eq!(shortlisted.players, in_shortlist(shortlist_off));
        }

        // A mixed OR filter keeps the shortlist as its base.
        let mixed_or = vec![
            filter_rule("name", "contains", FilterValue::Text("e".to_string())),
            filter_rule(&role, "gt", FilterValue::Integer(40)),
        ];
        let mixed = page(
            &shape(
                mixed_or.clone(),
                "or",
                Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
            )
            .shortlisted(),
        );
        let mixed_off = page(&shape(
            mixed_or,
            "or",
            Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
        ));
        assert!(mixed_off
            .players
            .iter()
            .any(|player| !shortlist.contains(&player.uid)));
        assert!(!mixed.players.is_empty());
        assert_eq!(mixed.players, in_shortlist(mixed_off));
    }

    #[test]
    fn cached_moneyball_windows_equal_uncached_pages_across_several_scan_chunks() {
        // More cohort members than two of the Moneyball scan's 1,024-row chunks, so workers
        // scan full chunks whose values merge back by member.
        const PLAYERS: usize = 2_100;
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = crate::db::open(&temp_dir.path().join(crate::db::APP_DB_FILE)).expect("open db");
        // UIDs start past the golden dump's staff UID.
        const FIRST_UID: usize = 1_000;
        let uids = FIRST_UID..FIRST_UID + PLAYERS;
        let players = uids
            .clone()
            .map(|uid| {
                player_template(uid as u64, &format!("P{uid}"), 80 + (uid as i64 * 37) % 100)
            })
            .collect();
        ingest_players(&mut db.0.lock().expect("lock writer"), players);

        let writer = db.0.lock().expect("lock writer");
        let snapshot_id = current_snapshot_id(&writer);
        let catalog = builtin_catalog().expect("catalog");
        let mut keys = BTreeSet::from(["goals", "minutes_per_goal", "xg"].map(str::to_string));
        let role_definition = catalog
            .definitions
            .iter()
            .find(|definition| definition.id == MONEYBALL_ROLE)
            .expect("role");
        keys.extend(
            role_definition
                .metrics
                .iter()
                .map(|metric| metric.key.clone()),
        );
        let transaction = writer.unchecked_transaction().expect("transaction");
        // The last UID has no stored percentiles and the one before it no row, so the scans
        // skip both mid-chunk.
        let last = uids.end - 1;
        for uid in uids.filter(|&uid| uid != last - 1) {
            let mut statistics = Map::new();
            let mut percentiles = Map::new();
            for (index, key) in keys.iter().enumerate() {
                let seed = uid * 7 + index * 3;
                statistics.insert(
                    key.clone(),
                    if (uid + index * 5) % 9 == 0 {
                        Value::Null
                    } else {
                        json!((seed % 23) as f64 * 0.5)
                    },
                );
                percentiles.insert(
                    key.clone(),
                    if (uid + index) % 7 == 0 {
                        Value::Null
                    } else {
                        json!((uid * 13 + index * 7) % 101)
                    },
                );
            }
            let percentiles = Value::Object(percentiles).to_string();
            insert_moneyball_statistics(
                &transaction,
                snapshot_id,
                uid as i64,
                &Value::Object(statistics).to_string(),
                (uid != last).then_some(percentiles.as_str()),
            );
        }
        transaction.commit().expect("commit Moneyball rows");
        drop(writer);

        let orders = PlayerOrderCache::new(2);
        let registry = TableRegistry::default();
        let role = moneyball_role_field();
        let shapes = [
            MoneyballShape::new(
                "statistic sort",
                "moneyball.goals",
                SortDir::Desc,
                &["moneyball.goals", "moneyball.minutes_per_goal"],
            ),
            MoneyballShape::new("role sort", &role, SortDir::Desc, &[&role]),
        ];
        for shape in &shapes {
            for comparison in [
                Comparison::Own,
                Comparison::Group(ComparisonGroup::family(PositionFamily::Striker)),
            ] {
                let shape = shape.compared_with(comparison);
                let name = format!("{} ({comparison:?})", shape.name);
                let mut joined = 0;
                for offset in (0..PLAYERS).step_by(MAX_PAGE_LIMIT) {
                    let cached = search_players(
                        &db,
                        &orders,
                        &ComparisonGroupsCache::default(),
                        &registry,
                        None,
                        shape.request(offset, MAX_PAGE_LIMIT),
                    )
                    .unwrap_or_else(|error| panic!("{name}: {error}"));
                    let uncached = search_players_uncached(
                        &db.read().expect("reader"),
                        shape.request(offset, MAX_PAGE_LIMIT),
                    )
                    .expect("uncached page");
                    assert_eq!(cached, uncached, "{name} at offset {offset}");
                    joined += cached.players.len();
                }
                assert_eq!(joined, PLAYERS, "{name} rows");
            }
        }
    }

    #[test]
    fn scored_windows_reuse_the_order_and_an_added_column_reads_only_its_window() {
        let (_temp_dir, db) = open_moneyball_equivalence_db();
        let orders = PlayerOrderCache::new(2);
        let registry = TableRegistry::default();
        let role = moneyball_role_field();
        let shape = MoneyballShape::new("role sort", &role, SortDir::Desc, &[&role]);
        let with_column =
            MoneyballShape::new("role sort", &role, SortDir::Desc, &[&role, "moneyball.xg"]);
        let window = |shape: &MoneyballShape, offset: usize| {
            search_players(
                &db,
                &orders,
                &ComparisonGroupsCache::default(),
                &registry,
                None,
                shape.request(offset, 3),
            )
            .expect("window")
        };
        ORDER_COMPUTATIONS.with(|count| count.set(0));

        window(&shape, 0);
        window(&shape, 6);
        let pages = [3, 0, 6].map(|offset| (offset, window(&with_column, offset)));
        assert_eq!(ORDER_COMPUTATIONS.with(Cell::get), 1);
        for (offset, page) in pages {
            assert_eq!(
                page,
                search_players_uncached(
                    &db.read().expect("reader"),
                    with_column.request(offset, 3)
                )
                .expect("uncached page")
            );
        }
        ORDER_COMPUTATIONS.with(|count| count.set(1));

        db.0.lock()
            .expect("lock writer")
            .execute(
                "UPDATE player_moneyball_stats SET minutes = 100 WHERE player_uid = 1",
                [],
            )
            .expect("shorten a sample");
        let fresh = window(&shape, 0);
        assert_eq!(ORDER_COMPUTATIONS.with(Cell::get), 2);
        assert_eq!(
            fresh,
            search_players_uncached(&db.read().expect("reader"), shape.request(0, 3))
                .expect("uncached page")
        );
    }

    #[test]
    fn a_newer_moneyball_query_supersedes_a_running_one_and_completes() {
        let (_temp_dir, db) = open_moneyball_equivalence_db();
        let orders = PlayerOrderCache::new(2);
        let registry = TableRegistry::default();
        let role = moneyball_role_field();
        let older = MoneyballShape::new("role sort", &role, SortDir::Desc, &[&role]);
        let newer = || MoneyballShape::new("name", "name", SortDir::Asc, &[]);
        let older_ticket =
            registry.arrive("search", &search_query_fingerprint(&older.request(0, 3)));

        let (sender, receiver) = std::sync::mpsc::channel();
        {
            let (db, orders, registry) = (db.clone(), orders.clone(), registry.clone());
            BEFORE_ORDER_QUERY.with(|hook| {
                *hook.borrow_mut() = Some(Box::new(move || {
                    let newer = newer();
                    let ticket =
                        registry.arrive("search", &search_query_fingerprint(&newer.request(0, 3)));
                    sender
                        .send(search_players(
                            &db,
                            &orders,
                            &ComparisonGroupsCache::default(),
                            &registry,
                            Some(&ticket),
                            newer.request(0, 3),
                        ))
                        .expect("send newer page");
                }));
            });
        }

        let superseded = search_players(
            &db,
            &orders,
            &ComparisonGroupsCache::default(),
            &registry,
            Some(&older_ticket),
            older.request(0, 3),
        );
        assert_eq!(
            superseded,
            Err(crate::db::result_cache::SUPERSEDED.to_string())
        );
        let newer = newer();
        assert_eq!(
            receiver.recv().expect("newer page"),
            search_players_uncached(&db.read().expect("reader"), newer.request(0, 3))
        );
    }
}
