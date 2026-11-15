//! Moneyball Search's computed cohorts: one entry per table query and data generation, built
//! with `search_players_in_view`'s cohort, post-cohort filter, and sort rules.
//!
//! A cohort keeps its members' raw statistics, one column per statistic read once, and ranks
//! them against the snapshot's comparison groups (`moneyball::comparison`) when a filter, sort,
//! or window needs a percentile: the table's comparison for statistic percentiles, each role's
//! family for role scores, and a slot's group for custom tactic scores.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex, PoisonError};

use rusqlite::{params_from_iter, types::Value, Connection, Rows};

use crate::db::result_cache::{Cancellation, ResultCache};
use crate::features::moneyball::comparison::{
    scan_statistic_values, statistic_index, ComparisonGroup, ComparisonGroups, GroupsSource,
    PositionFamily, ResolvedGroups,
};
use crate::features::moneyball::json_scan::parse_json_value;
use crate::features::moneyball::league_strength::{shifted, LeagueAdjustment};
use crate::features::moneyball::role_catalog::{
    builtin_catalog, definition_for_attribute_role, RoleDefinition,
};
use crate::features::moneyball::role_score::score_role;
use crate::features::moneyball::MONEYBALL_STATISTIC_KEYS;
use crate::features::planner::fit::{
    custom_metrics_lane_score, moneyball_roles_lane_score, slot_group,
};
use crate::features::planner::tactic::{base_position, read_tactic, PlannerTactic};
use crate::features::player_metrics::resolver::{MetricField, TacticGroup};

use super::filter::{
    compile_filters_for_moneyball, has_post_cohort_rules, moneyball_percentile_keys_from_ast,
    moneyball_role_ids_from_ast, post_cohort_rules_match, without_post_cohort_rules, CombineMode,
};
use super::query::{
    active_current_snapshot_id, compare_role_scores, moneyball_base_where_sql, query_matching_uids,
    requested_shortlist_missing, snapshot_save_id, where_sql_with_filter, Comparison,
    SearchPlayersPageState, SearchPlayersRequest, SortField,
};

/// Moneyball Search's players: the current snapshot's players with season statistics.
pub(super) const MONEYBALL_FROM_SQL: &str = "FROM players INNER JOIN player_moneyball_stats moneyball ON moneyball.snapshot_id = players.snapshot_id AND moneyball.player_uid = players.uid AND moneyball.percentiles_json IS NOT NULL";

#[cfg(test)]
thread_local! {
    /// Moneyball cohort computations run on this thread.
    pub(super) static COHORT_COMPUTATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Statistic columns read for an entry on this thread.
    pub(super) static STATISTIC_COLUMN_READS: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
    /// Runs once, just before the next cohort query on this thread.
    pub(super) static BEFORE_COHORT_QUERY: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        const { std::cell::RefCell::new(None) };
}

/// One Moneyball table query's cohort and final row order, computed once per data generation.
pub struct MoneyballCohort {
    /// The data generation of the reader that computed it.
    pub(super) generation: u64,
    pub(super) state: SearchPlayersPageState,
    pub(super) snapshot_id: i64,
    /// How the snapshot's league strengths move each player's scores.
    pub(super) league: LeagueAdjustment,
    comparison: Comparison,
    /// The snapshot's comparison groups; `None` unless the state is `Ready`.
    groups: Option<Arc<ComparisonGroups>>,
    /// The comparison cohort in ascending UID order. Statistic columns follow this order.
    uids: Vec<i64>,
    /// Each member's own family, which an `Own` comparison ranks his statistics within.
    own: Vec<Option<PositionFamily>>,
    /// The rows after post-cohort filters and the sort, as indexes into `uids`.
    order: Vec<u32>,
    /// Statistic columns by statistic index, each read once and then kept (NaN when absent).
    /// The entry is shared between concurrent windows, so the lock is held while one is read.
    statistics: Mutex<HashMap<usize, Arc<[f64]>>>,
}

/// Moneyball Search's computed cohorts, keyed by `search_query_fingerprint`.
pub type MoneyballCohortCache = ResultCache<MoneyballCohort>;

/// The positions and preferred foot that a tactic score's fit adjustment reads.
pub(super) struct PlayerFit {
    pub(super) positions: BTreeMap<String, Option<i64>>,
    pub(super) foot: String,
}

impl PlayerFit {
    pub(super) fn from_columns(positions_json: &str, foot: String) -> Self {
        Self {
            positions: serde_json::from_str(positions_json).unwrap_or_default(),
            foot,
        }
    }
}

/// Statistic columns for a set of keys, read from one entry, by statistic index.
pub(super) struct StatisticColumns(Vec<Option<Arc<[f64]>>>);

impl StatisticColumns {
    fn value(&self, index: usize, member: u32) -> Option<f64> {
        self.0[index]
            .as_ref()
            .map(|column| column[member as usize])
            .filter(|value| value.is_finite())
    }
}

/// A cohort member's percentiles: his statistics in `columns`, ranked within `rankings`.
pub(super) struct MemberPercentiles<'a> {
    pub(super) rankings: &'a ResolvedGroups,
    pub(super) columns: &'a StatisticColumns,
    pub(super) member: u32,
}

impl MemberPercentiles<'_> {
    pub(super) fn percentile(&self, group: ComparisonGroup, key: &str) -> Option<u8> {
        let index = statistic_index(key)?;
        self.rankings
            .get(group)?
            .percentile(index, self.columns.value(index, self.member))
    }

    /// A role's score, its metrics ranked within the role's family.
    pub(super) fn role_score(&self, definition: &RoleDefinition) -> Option<u8> {
        let group = role_group(definition)?;
        score_role(definition, &|key: &str| self.percentile(group, key)).map(|role| role.score)
    }
}

/// The group a role's metrics rank within: its family's.
pub(super) fn role_group(definition: &RoleDefinition) -> Option<ComparisonGroup> {
    PositionFamily::from_id(&definition.position_family).map(ComparisonGroup::family)
}

impl MoneyballCohort {
    /// Computes the cohort and its final order. Role, tactic, and custom tactic scores are
    /// computed only for the sort and post-cohort filters.
    pub(super) fn compute(
        conn: &Connection,
        generation: u64,
        groups: &GroupsSource<'_>,
        request: &SearchPlayersRequest<'_>,
        cancellation: &Cancellation,
    ) -> Result<Self, String> {
        #[cfg(test)]
        COHORT_COMPUTATIONS.with(|count| count.set(count.get() + 1));
        let SearchPlayersRequest {
            sort_by,
            sort_dir,
            filter_ast,
            comparison,
            shortlist_id,
            ..
        } = request;
        let catalog = builtin_catalog()?;
        let role_definition = |role_id: &str| {
            catalog
                .definitions
                .iter()
                .find(|definition| definition.id == role_id)
                .ok_or_else(|| format!("unknown Moneyball role: {role_id}"))
        };
        let sort_role = sort_by
            .moneyball_role_id()
            .map(role_definition)
            .transpose()?;
        let filter_roles = filter_ast
            .map(moneyball_role_ids_from_ast)
            .transpose()?
            .unwrap_or_default()
            .into_iter()
            .map(|role_id| Ok((role_definition(&role_id)?, role_id)))
            .collect::<Result<Vec<_>, String>>()?;

        let mut cohort = Self {
            generation,
            state: SearchPlayersPageState::NoCurrentSnapshot,
            snapshot_id: 0,
            league: LeagueAdjustment::default(),
            comparison: *comparison,
            groups: None,
            uids: Vec::new(),
            own: Vec::new(),
            order: Vec::new(),
            statistics: Mutex::default(),
        };
        let Some(snapshot_id) = active_current_snapshot_id(conn)? else {
            return Ok(cohort);
        };
        cohort.snapshot_id = snapshot_id;
        if requested_shortlist_missing(conn, snapshot_id, *shortlist_id)? {
            cohort.state = SearchPlayersPageState::NoShortlist;
            return Ok(cohort);
        }
        cohort.state = SearchPlayersPageState::Ready;
        cohort.league = LeagueAdjustment::load(conn, snapshot_id)?;
        let groups = groups.load(conn, snapshot_id)?;
        cancellation.check()?;
        let sort_tactic = match sort_by {
            SortField::Dynamic(field) if field.is_tactic_field() => Some((
                field,
                read_tactic(conn, snapshot_save_id(conn, snapshot_id)?)?,
            )),
            _ => None,
        };

        // Under mixed OR, the comparison cohort is the whole snapshot, and a member stays when
        // it matches the SQL rules or the post-cohort rules.
        let non_role_ast = filter_ast
            .map(without_post_cohort_rules)
            .transpose()?
            .flatten();
        let compiled_non_role = non_role_ast
            .as_ref()
            .map(|ast| compile_filters_for_moneyball(ast, 2, true))
            .transpose()?;
        let base_where_sql = moneyball_base_where_sql(*shortlist_id);
        let non_role_where_sql = where_sql_with_filter(&base_where_sql, compiled_non_role.as_ref());
        let mut non_role_bind_values = vec![Value::Integer(snapshot_id)];
        if let Some(compiled) = &compiled_non_role {
            non_role_bind_values.extend(compiled.params.iter().cloned());
        }
        let post_cohort_ast = filter_ast.filter(|ast| has_post_cohort_rules(ast));
        let mixed_or = post_cohort_ast.is_some_and(|ast| matches!(ast.combine, CombineMode::Or));
        let (cohort_where_sql, cohort_bind_values) = if mixed_or {
            (base_where_sql, vec![Value::Integer(snapshot_id)])
        } else {
            (non_role_where_sql.clone(), non_role_bind_values.clone())
        };
        let non_role_uids = if mixed_or && compiled_non_role.is_some() {
            query_matching_uids(
                conn,
                MONEYBALL_FROM_SQL,
                &non_role_where_sql,
                &non_role_bind_values,
            )?
        } else {
            HashSet::new()
        };
        cancellation.check()?;

        // Score sorts start from UID order and sort stably in Rust; other sorts are SQL's.
        let order_sql = if sort_role.is_some() || sort_by.is_tactic() {
            "ORDER BY players.uid ASC".to_string()
        } else {
            format!(
                "ORDER BY {} {}, players.uid ASC",
                sort_by.sql_expr(None),
                sort_dir.sql_keyword()
            )
        };
        let fit_columns = if sort_tactic.is_some() {
            ", players.positions_json, players.preferred_foot"
        } else {
            ""
        };
        // A sort or filter on an adjusted score reads each member's league to adjust it.
        let scored = cohort.league.applies()
            && (sort_role.is_some() || sort_tactic.is_some() || !filter_roles.is_empty());
        let (division_column, division_index) = match (scored, sort_tactic.is_some()) {
            (false, _) => ("", 0),
            (true, false) => (", players.division", 1),
            (true, true) => (", players.division", 3),
        };
        #[cfg(test)]
        if let Some(hook) = BEFORE_COHORT_QUERY.with(|hook| hook.borrow_mut().take()) {
            hook();
        }
        cancellation.check()?;
        let rows = conn
            .prepare(&format!(
                "SELECT players.uid{fit_columns}{division_column} {MONEYBALL_FROM_SQL} WHERE {cohort_where_sql} {order_sql}"
            ))
            .map_err(|error| error.to_string())?
            .query_map(params_from_iter(cohort_bind_values.iter()), |row| {
                let fit = if sort_tactic.is_some() {
                    Some(PlayerFit::from_columns(&row.get::<_, String>(1)?, row.get(2)?))
                } else {
                    None
                };
                let shift = if scored {
                    let division: Option<String> = row.get(division_index)?;
                    cohort.league.shift(division.as_deref())
                } else {
                    0
                };
                Ok((row.get::<_, i64>(0)?, fit, shift))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        cancellation.check()?;

        cohort.uids = rows.iter().map(|(uid, ..)| *uid).collect();
        cohort.uids.sort_unstable();
        cohort.own = cohort
            .uids
            .iter()
            .map(|uid| groups.own_family(*uid))
            .collect();
        cohort.groups = Some(Arc::clone(&groups));
        let mut fits = Vec::with_capacity(rows.len());
        fits.resize_with(rows.len(), || None);
        let mut shifts = vec![0_i16; rows.len()];
        let mut order = Vec::with_capacity(rows.len());
        for (uid, fit, shift) in rows {
            let member = cohort.member(uid).expect("a cohort row is a member");
            fits[member as usize] = fit;
            shifts[member as usize] = shift;
            order.push(member);
        }

        let mut build_keys = BTreeSet::new();
        for definition in sort_role
            .iter()
            .chain(filter_roles.iter().map(|(definition, _)| definition))
        {
            build_keys.extend(definition.metrics.iter().map(|metric| metric.key.clone()));
        }
        let percentile_keys = filter_ast
            .map(moneyball_percentile_keys_from_ast)
            .unwrap_or_default();
        build_keys.extend(percentile_keys.iter().cloned());
        if let Some((field, tactic)) = &sort_tactic {
            build_keys.extend(tactic_metric_keys(field, tactic));
        }
        let columns =
            cohort.statistic_columns(conn, &build_keys.into_iter().collect::<Vec<_>>())?;
        let rankings = ResolvedGroups::resolve(
            &groups,
            conn,
            sort_tactic
                .iter()
                .filter_map(|(field, tactic)| tactic_slot_group(field, tactic)),
        )?;
        cancellation.check()?;

        if let Some(ast) = post_cohort_ast {
            let mut kept = Vec::with_capacity(order.len());
            for member in order {
                let percentiles = MemberPercentiles {
                    rankings: &rankings,
                    columns: &columns,
                    member,
                };
                let scores = filter_roles
                    .iter()
                    .map(|(definition, role_id)| {
                        let score = percentiles
                            .role_score(definition)
                            .map(|score| shifted(score, shifts[member as usize]));
                        (role_id.clone(), score)
                    })
                    .collect();
                let group = cohort.table_group(member);
                let statistic_percentiles = percentile_keys
                    .iter()
                    .map(|key| {
                        let percentile = group.and_then(|group| percentiles.percentile(group, key));
                        (key.clone(), percentile)
                    })
                    .collect();
                let post_cohort_match =
                    post_cohort_rules_match(ast, &scores, &statistic_percentiles)?;
                if post_cohort_match
                    || (mixed_or && non_role_uids.contains(&cohort.uids[member as usize]))
                {
                    kept.push(member);
                }
            }
            order = kept;
            cancellation.check()?;
        }

        let sort_score = |member: u32| -> Option<u8> {
            let percentiles = MemberPercentiles {
                rankings: &rankings,
                columns: &columns,
                member,
            };
            match (sort_role, &sort_tactic) {
                (Some(definition), _) => percentiles
                    .role_score(definition)
                    .map(|score| shifted(score, shifts[member as usize])),
                (None, Some((field, tactic))) => {
                    let fit = fits[member as usize]
                        .as_ref()
                        .expect("tactic sorts read fit");
                    tactic_score(field, tactic, &percentiles, fit, shifts[member as usize])
                }
                (None, None) => None,
            }
        };
        if sort_role.is_some() || sort_tactic.is_some() {
            let mut scores = vec![None; cohort.uids.len()];
            for &member in &order {
                scores[member as usize] = sort_score(member);
            }
            order.sort_by(|&left, &right| {
                compare_role_scores(scores[left as usize], scores[right as usize], *sort_dir)
                    .then_with(|| cohort.uids[left as usize].cmp(&cohort.uids[right as usize]))
            });
        }
        cohort.order = order;
        Ok(cohort)
    }

    /// The number of rows the query returns.
    pub(super) fn total(&self) -> usize {
        self.order.len()
    }

    /// The cohort members shown by one window, in row order.
    pub(super) fn window(&self, offset: usize, limit: usize) -> &[u32] {
        let start = offset.min(self.order.len());
        &self.order[start..(start + limit).min(self.order.len())]
    }

    pub(super) fn uid(&self, member: u32) -> i64 {
        self.uids[member as usize]
    }

    fn member(&self, uid: i64) -> Option<u32> {
        self.uids
            .binary_search(&uid)
            .ok()
            .map(|member| member as u32)
    }

    /// The group the table's statistic percentiles rank a member within: his own family's,
    /// or the family the table compares everyone with.
    pub(super) fn table_group(&self, member: u32) -> Option<ComparisonGroup> {
        match self.comparison {
            Comparison::Own => self.own[member as usize].map(ComparisonGroup::family),
            Comparison::Family(family) => Some(ComparisonGroup::family(family)),
        }
    }

    /// The rankings a window looks up: every family's and the given slot groups'. `conn`
    /// must read this entry's data generation.
    pub(super) fn rankings(
        &self,
        conn: &Connection,
        slots: impl IntoIterator<Item = ComparisonGroup>,
    ) -> Result<ResolvedGroups, String> {
        let groups = self
            .groups
            .as_ref()
            .ok_or_else(|| "a Moneyball cohort without a snapshot has no groups".to_string())?;
        ResolvedGroups::resolve(groups, conn, slots)
    }

    /// The statistic columns for `keys`, reading each missing one once. `conn` must read this
    /// entry's data generation.
    pub(super) fn statistic_columns(
        &self,
        conn: &Connection,
        keys: &[String],
    ) -> Result<StatisticColumns, String> {
        let indexes = keys
            .iter()
            .map(|key| {
                statistic_index(key).ok_or_else(|| format!("unknown Moneyball statistic: {key}"))
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        let mut memo = self
            .statistics
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let missing = indexes
            .iter()
            .copied()
            .filter(|index| !memo.contains_key(index))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            #[cfg(test)]
            STATISTIC_COLUMN_READS.with(|count| count.set(count.get() + missing.len()));
            let columns = self.read_columns(conn, &missing)?;
            for (index, column) in missing.into_iter().zip(columns) {
                memo.insert(index, column);
            }
        }
        let mut columns = vec![None; MONEYBALL_STATISTIC_KEYS.len()];
        for index in indexes {
            columns[index] = Some(Arc::clone(&memo[&index]));
        }
        Ok(StatisticColumns(columns))
    }

    fn read_columns(
        &self,
        conn: &Connection,
        indexes: &[usize],
    ) -> Result<Vec<Arc<[f64]>>, String> {
        // One read and scan of each member's object: `json_extract` per key re-reads and
        // re-parses the object for every key. Rows outside the cohort are skipped.
        let mut statement = conn
            .prepare(
                "SELECT player_uid, statistics_json FROM player_moneyball_stats WHERE snapshot_id = ?1",
            )
            .map_err(|error| error.to_string())?;
        let mut rows = statement
            .query([self.snapshot_id])
            .map_err(|error| error.to_string())?;
        let columns = self.scan_members(&mut rows, indexes)?;
        Ok(columns.into_iter().map(Into::into).collect())
    }

    /// Each member's values of the statistics at `indexes`, one column per index (NaN when
    /// absent), from rows of UIDs and statistics objects. Worker threads scan the objects
    /// while this thread reads the rows.
    fn scan_members(
        &self,
        rows: &mut Rows<'_>,
        indexes: &[usize],
    ) -> Result<Vec<Vec<f64>>, String> {
        const CHUNK_ROWS: usize = 1024;
        let mut slot_of = vec![None; MONEYBALL_STATISTIC_KEYS.len()];
        for (slot, index) in indexes.iter().enumerate() {
            slot_of[*index] = Some(slot);
        }
        let workers = std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(8);
        let (sender, receiver) = mpsc::sync_channel::<Vec<(u32, String)>>(workers * 2);
        let receiver = Mutex::new(receiver);
        let scan_chunks = || {
            let mut members = Vec::new();
            let mut values = Vec::new();
            let mut result = Ok(());
            // A worker drains every chunk even after an error, so the reader never blocks.
            while let Ok(chunk) = receiver
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .recv()
            {
                for (member, json) in chunk {
                    if result.is_err() {
                        break;
                    }
                    let start = values.len();
                    values.resize(start + indexes.len(), f64::NAN);
                    members.push(member);
                    result = scan_statistic_values(&json, |index, text| {
                        if let Some(slot) = slot_of[index] {
                            if let Some(value) =
                                parse_json_value(text, |text| text.parse::<f64>().ok())?
                            {
                                values[start + slot] = value;
                            }
                        }
                        Ok(())
                    });
                }
            }
            result.map(|()| (members, values))
        };
        std::thread::scope(|scope| {
            let handles = (0..workers)
                .map(|_| scope.spawn(scan_chunks))
                .collect::<Vec<_>>();
            let read = (|| {
                let mut chunk = Vec::with_capacity(CHUNK_ROWS);
                while let Some(row) = rows.next().map_err(|error| error.to_string())? {
                    let Some(member) = self.member(row.get(0).map_err(|error| error.to_string())?)
                    else {
                        continue;
                    };
                    chunk.push((member, row.get(1).map_err(|error| error.to_string())?));
                    if chunk.len() == CHUNK_ROWS {
                        let full = std::mem::replace(&mut chunk, Vec::with_capacity(CHUNK_ROWS));
                        sender.send(full).map_err(|error| error.to_string())?;
                    }
                }
                sender.send(chunk).map_err(|error| error.to_string())
            })();
            drop(sender);
            let mut columns = vec![vec![f64::NAN; self.uids.len()]; indexes.len()];
            for handle in handles {
                let (members, values) = handle.join().expect("a Moneyball scan worker panicked")?;
                for (member, values) in members
                    .into_iter()
                    .zip(values.chunks_exact(indexes.len().max(1)))
                {
                    for (column, value) in columns.iter_mut().zip(values) {
                        column[member as usize] = *value;
                    }
                }
            }
            read.map(|()| columns)
        })
    }
}

/// The statistic keys a Moneyball tactic column's score reads.
pub(super) fn tactic_metric_keys(field: &MetricField, tactic: &PlannerTactic) -> Vec<String> {
    let Some(lane) = tactic_lane(field, tactic) else {
        return Vec::new();
    };
    if field.tactic_group() == Some(TacticGroup::Custom) {
        return lane.custom_metric_keys.clone();
    }
    [
        (lane.ip_role_id.as_str(), lane.ip_position.as_str()),
        (lane.oop_role_id.as_str(), lane.oop_position.as_str()),
    ]
    .into_iter()
    .filter_map(|(role_id, position)| {
        definition_for_attribute_role(role_id, base_position(position))
    })
    .flat_map(|definition| definition.metrics.iter().map(|metric| metric.key.clone()))
    .collect()
}

/// The slot group a custom tactic column ranks its metrics within. Role columns need none:
/// their roles rank within families.
pub(super) fn tactic_slot_group(
    field: &MetricField,
    tactic: &PlannerTactic,
) -> Option<ComparisonGroup> {
    if field.tactic_group() != Some(TacticGroup::Custom) {
        return None;
    }
    slot_group(tactic_lane(field, tactic)?)
}

/// A Moneyball tactic column's score from a member's percentiles, fit, and league shift: the
/// custom metrics' mean, or the slot's two role scores when both have Moneyball definitions.
pub(super) fn tactic_score(
    field: &MetricField,
    tactic: &PlannerTactic,
    percentiles: &MemberPercentiles<'_>,
    fit: &PlayerFit,
    league_shift: i16,
) -> Option<u8> {
    let lane = tactic_lane(field, tactic)?;
    let lookup = |group, key: &str| percentiles.percentile(group, key);
    if field.tactic_group() == Some(TacticGroup::Custom) {
        custom_metrics_lane_score(&lookup, &fit.foot, &fit.positions, lane, league_shift)
    } else {
        moneyball_roles_lane_score(&lookup, &fit.foot, &fit.positions, lane, league_shift)
    }
}

fn tactic_lane<'a>(
    field: &MetricField,
    tactic: &'a PlannerTactic,
) -> Option<&'a crate::features::planner::tactic::TacticLane> {
    let lane_id = field.tactic_lane_id()?;
    tactic.lanes.iter().find(|lane| lane.lane_id == lane_id)
}
