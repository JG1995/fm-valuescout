//! Moneyball Search's computed cohorts: one entry per table query and data generation, built
//! with `search_players_in_view`'s cohort, post-cohort filter, and sort rules.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::sync::{mpsc, Arc, Mutex, PoisonError};

use rusqlite::{params_from_iter, types::Value, Connection, Rows};

use crate::db::result_cache::{Cancellation, ResultCache};
use crate::features::moneyball::percentile::rank_column;
use crate::features::moneyball::role_catalog::{builtin_catalog, RoleDefinition};
use crate::features::moneyball::role_score::{mean_percentile_score, score_role};
use crate::features::planner::fit::{lane_fit_score, tactic_adjusted_score};
use crate::features::planner::tactic::{base_position, read_tactic, PlannerTactic};
use crate::features::player_metrics::resolver::{MetricField, TacticGroup};

use super::filter::{
    compile_filters_for_moneyball, has_post_cohort_rules, moneyball_percentile_keys_from_ast,
    moneyball_role_ids_from_ast, post_cohort_rules_match, without_post_cohort_rules, CombineMode,
};
use super::query::{
    active_current_snapshot_id, compare_role_scores, moneyball_definition_for_role_and_position,
    query_matching_uids, snapshot_save_id, where_sql_with_filter, ComparisonPool,
    SearchPlayersPageState, SearchPlayersRequest, SortField,
};

/// Moneyball Search's players: the current snapshot's players with stored percentiles.
pub(super) const MONEYBALL_FROM_SQL: &str = "FROM players INNER JOIN player_moneyball_stats moneyball ON moneyball.snapshot_id = players.snapshot_id AND moneyball.player_uid = players.uid AND moneyball.percentiles_json IS NOT NULL";

#[cfg(test)]
thread_local! {
    /// Moneyball cohort computations run on this thread.
    pub(super) static COHORT_COMPUTATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Percentile columns ranked or read for an entry on this thread.
    pub(super) static PERCENTILE_COLUMN_COMPUTATIONS: std::cell::Cell<usize> =
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
    pool: ComparisonPool,
    /// The comparison cohort in ascending UID order. Percentile columns follow this order.
    uids: Vec<i64>,
    /// The rows after post-cohort filters and the sort, as indexes into `uids`.
    order: Vec<u32>,
    /// Percentile columns by statistic key, each ranked or read once and then kept. The entry
    /// is shared between concurrent windows, so the lock is held while a column is computed.
    percentiles: Mutex<HashMap<String, Arc<[Option<u8>]>>>,
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

/// Percentile columns for a set of keys, read from one entry.
pub(super) struct PercentileColumns(Vec<(String, Arc<[Option<u8>]>)>);

impl PercentileColumns {
    /// One cohort member's percentiles for these keys.
    pub(super) fn member(&self, member: u32) -> BTreeMap<String, Option<u8>> {
        self.0
            .iter()
            .map(|(key, column)| (key.clone(), column[member as usize]))
            .collect()
    }
}

impl MoneyballCohort {
    /// Computes the cohort, its build-time percentile columns, and its final order. Role,
    /// tactic, and custom tactic scores are computed only for the sort and post-cohort filters.
    pub(super) fn compute(
        conn: &Connection,
        generation: u64,
        request: &SearchPlayersRequest<'_>,
        cancellation: &Cancellation,
    ) -> Result<Self, String> {
        #[cfg(test)]
        COHORT_COMPUTATIONS.with(|count| count.set(count.get() + 1));
        let SearchPlayersRequest {
            sort_by,
            sort_dir,
            filter_ast,
            comparison_pool,
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
            pool: *comparison_pool,
            uids: Vec::new(),
            order: Vec::new(),
            percentiles: Mutex::default(),
        };
        let Some(snapshot_id) = active_current_snapshot_id(conn)? else {
            return Ok(cohort);
        };
        cohort.state = SearchPlayersPageState::Ready;
        cohort.snapshot_id = snapshot_id;
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
        let base_where_sql = "players.snapshot_id = ?1";
        let non_role_where_sql = where_sql_with_filter(base_where_sql, compiled_non_role.as_ref());
        let mut non_role_bind_values = vec![Value::Integer(snapshot_id)];
        if let Some(compiled) = &compiled_non_role {
            non_role_bind_values.extend(compiled.params.iter().cloned());
        }
        let post_cohort_ast = filter_ast.filter(|ast| has_post_cohort_rules(ast));
        let mixed_or = post_cohort_ast.is_some_and(|ast| matches!(ast.combine, CombineMode::Or));
        let (cohort_where_sql, cohort_bind_values) = if mixed_or {
            (
                base_where_sql.to_string(),
                vec![Value::Integer(snapshot_id)],
            )
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
        #[cfg(test)]
        if let Some(hook) = BEFORE_COHORT_QUERY.with(|hook| hook.borrow_mut().take()) {
            hook();
        }
        cancellation.check()?;
        let rows = conn
            .prepare(&format!(
                "SELECT players.uid{fit_columns} {MONEYBALL_FROM_SQL} WHERE {cohort_where_sql} {order_sql}"
            ))
            .map_err(|error| error.to_string())?
            .query_map(params_from_iter(cohort_bind_values.iter()), |row| {
                let fit = if sort_tactic.is_some() {
                    Some(PlayerFit::from_columns(&row.get::<_, String>(1)?, row.get(2)?))
                } else {
                    None
                };
                Ok((row.get::<_, i64>(0)?, fit))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        cancellation.check()?;

        cohort.uids = rows.iter().map(|(uid, _)| *uid).collect();
        cohort.uids.sort_unstable();
        let mut fits = Vec::with_capacity(rows.len());
        fits.resize_with(rows.len(), || None);
        let mut order = Vec::with_capacity(rows.len());
        for (uid, fit) in rows {
            let member = cohort.member(uid).expect("a cohort row is a member");
            fits[member as usize] = fit;
            order.push(member);
        }

        let mut build_keys = BTreeSet::new();
        for definition in sort_role
            .iter()
            .chain(filter_roles.iter().map(|(definition, _)| definition))
        {
            build_keys.extend(definition.metrics.iter().map(|metric| metric.key.clone()));
        }
        build_keys.extend(
            filter_ast
                .map(moneyball_percentile_keys_from_ast)
                .unwrap_or_default(),
        );
        if let Some((field, tactic)) = &sort_tactic {
            build_keys.extend(tactic_metric_keys(field, tactic));
        }
        let columns =
            cohort.percentile_columns(conn, &build_keys.into_iter().collect::<Vec<_>>())?;
        cancellation.check()?;

        if let Some(ast) = post_cohort_ast {
            let mut kept = Vec::with_capacity(order.len());
            for member in order {
                let percentiles = columns.member(member);
                let scores = filter_roles
                    .iter()
                    .map(|(definition, role_id)| {
                        let score = score_role(definition, &percentiles).map(|role| role.score);
                        (role_id.clone(), score)
                    })
                    .collect();
                let post_cohort_match = post_cohort_rules_match(ast, &scores, &percentiles)?;
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
            let percentiles = columns.member(member);
            match (sort_role, &sort_tactic) {
                (Some(definition), _) => {
                    score_role(definition, &percentiles).map(|role| role.score)
                }
                (None, Some((field, tactic))) => {
                    let fit = fits[member as usize]
                        .as_ref()
                        .expect("tactic sorts read fit");
                    tactic_score(field, tactic, &percentiles, fit)
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

    /// The percentile columns for `keys`, computing each missing one once: a Filtered pool
    /// ranks the cohort's statistics, and All players reads the stored percentiles. `conn`
    /// must read this entry's data generation.
    pub(super) fn percentile_columns(
        &self,
        conn: &Connection,
        keys: &[String],
    ) -> Result<PercentileColumns, String> {
        let mut memo = self
            .percentiles
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut missing = Vec::new();
        for key in keys {
            if !memo.contains_key(key) && !missing.contains(&key) {
                missing.push(key);
            }
        }
        if !missing.is_empty() {
            #[cfg(test)]
            PERCENTILE_COLUMN_COMPUTATIONS.with(|count| count.set(count.get() + missing.len()));
            let columns = self.read_columns(conn, &missing)?;
            for (key, column) in missing.into_iter().zip(columns) {
                memo.insert(key.clone(), column);
            }
        }
        Ok(PercentileColumns(
            keys.iter()
                .map(|key| (key.clone(), Arc::clone(&memo[key])))
                .collect(),
        ))
    }

    fn read_columns(
        &self,
        conn: &Connection,
        keys: &[&String],
    ) -> Result<Vec<Arc<[Option<u8>]>>, String> {
        let source = match self.pool {
            ComparisonPool::Filtered => "statistics_json",
            ComparisonPool::FullCsv => "percentiles_json",
        };
        // One read and scan of each member's object: `json_extract` per key re-reads and
        // re-parses the object for every key. Rows outside the cohort are skipped.
        let mut statement = conn
            .prepare(&format!(
                "SELECT player_uid, {source} FROM player_moneyball_stats WHERE snapshot_id = ?1"
            ))
            .map_err(|error| error.to_string())?;
        let mut rows = statement
            .query([self.snapshot_id])
            .map_err(|error| error.to_string())?;
        match self.pool {
            ComparisonPool::Filtered => {
                let values = self.scan_members(&mut rows, keys, |text| text.parse::<f64>().ok())?;
                Ok(keys
                    .iter()
                    .zip(values)
                    .map(|(key, values)| rank_column(&values, key).into())
                    .collect())
            }
            ComparisonPool::FullCsv => {
                let columns = self.scan_members(&mut rows, keys, |text| text.parse::<u8>().ok())?;
                Ok(columns.into_iter().map(Into::into).collect())
            }
        }
    }

    /// Each member's values for `keys`, one column per key, from rows of UIDs and JSON
    /// objects. Worker threads scan the objects while this thread reads the rows.
    fn scan_members<T: Copy + Send>(
        &self,
        rows: &mut Rows<'_>,
        keys: &[&String],
        parse: fn(&str) -> Option<T>,
    ) -> Result<Vec<Vec<Option<T>>>, String> {
        const CHUNK_ROWS: usize = 1024;
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
                    values.resize(start + keys.len(), None);
                    members.push(member);
                    result = scan_object_values(&json, keys, |index, text| {
                        values[start + index] = parse_value(text, parse)?;
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
            let mut columns = vec![vec![None; self.uids.len()]; keys.len()];
            for handle in handles {
                let (members, values) = handle.join().expect("a Moneyball scan worker panicked")?;
                for (member, values) in members.into_iter().zip(values.chunks_exact(keys.len())) {
                    for (column, value) in columns.iter_mut().zip(values) {
                        column[member as usize] = *value;
                    }
                }
            }
            read.map(|()| columns)
        })
    }
}

/// Calls `found` with the index and raw value text of each of `keys` that one stored Moneyball
/// JSON object holds. Its values are numbers or null.
fn scan_object_values(
    json: &str,
    keys: &[&String],
    mut found: impl FnMut(usize, &str) -> Result<(), String>,
) -> Result<(), String> {
    let malformed = || "malformed Moneyball JSON object".to_string();
    let bytes = json.as_bytes();
    let mut at = 0;
    let skip_whitespace = |at: &mut usize| {
        while bytes.get(*at).is_some_and(u8::is_ascii_whitespace) {
            *at += 1;
        }
    };
    skip_whitespace(&mut at);
    if bytes.get(at) != Some(&b'{') {
        return Err(malformed());
    }
    at += 1;
    skip_whitespace(&mut at);
    if bytes.get(at) == Some(&b'}') {
        return Ok(());
    }
    loop {
        skip_whitespace(&mut at);
        if bytes.get(at) != Some(&b'"') {
            return Err(malformed());
        }
        let key_start = at + 1;
        at = key_start;
        loop {
            match bytes.get(at) {
                Some(b'"') => break,
                Some(b'\\') => at += 2,
                Some(_) => at += 1,
                None => return Err(malformed()),
            }
        }
        let key = &json[key_start..at];
        at += 1;
        skip_whitespace(&mut at);
        if bytes.get(at) != Some(&b':') {
            return Err(malformed());
        }
        at += 1;
        skip_whitespace(&mut at);
        let value_start = at;
        while bytes
            .get(at)
            .is_some_and(|byte| !matches!(byte, b',' | b'}') && !byte.is_ascii_whitespace())
        {
            at += 1;
        }
        if let Some(index) = keys.iter().position(|wanted| wanted.as_str() == key) {
            found(index, &json[value_start..at])?;
        }
        skip_whitespace(&mut at);
        match bytes.get(at) {
            Some(b',') => at += 1,
            Some(b'}') => return Ok(()),
            _ => return Err(malformed()),
        }
    }
}

/// A stored value: `null`, or a number. Rust parses numbers to the nearest value, as SQLite's
/// JSON functions do for the at most 17 significant digits that stored values carry.
fn parse_value<T>(text: &str, parse: fn(&str) -> Option<T>) -> Result<Option<T>, String> {
    if text == "null" {
        return Ok(None);
    }
    parse(text)
        .map(Some)
        .ok_or_else(|| format!("malformed Moneyball value: {text}"))
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
        moneyball_definition_for_role_and_position(role_id, base_position(position))
    })
    .flat_map(|definition| definition.metrics.iter().map(|metric| metric.key.clone()))
    .collect()
}

/// A Moneyball tactic column's score from a player's percentiles and fit: the custom metrics'
/// mean, or the slot's two role scores when both have Moneyball definitions.
pub(super) fn tactic_score(
    field: &MetricField,
    tactic: &PlannerTactic,
    percentiles: &BTreeMap<String, Option<u8>>,
    fit: &PlayerFit,
) -> Option<u8> {
    let lane = tactic_lane(field, tactic)?;
    if field.tactic_group() == Some(TacticGroup::Custom) {
        let base = mean_percentile_score(&lane.custom_metric_keys, percentiles);
        return lane_fit_score(base, &fit.foot, &fit.positions, lane);
    }
    let definition = |role_id: &str, position: &str| -> Option<&RoleDefinition> {
        moneyball_definition_for_role_and_position(role_id, base_position(position))
    };
    let ip_definition = definition(&lane.ip_role_id, &lane.ip_position)?;
    let oop_definition = definition(&lane.oop_role_id, &lane.oop_position)?;
    tactic_adjusted_score(
        score_role(ip_definition, percentiles).map(|role| role.score),
        score_role(oop_definition, percentiles).map(|role| role.score),
        lane.ip_weight,
        &fit.foot,
        &fit.positions,
        lane,
    )
}

fn tactic_lane<'a>(
    field: &MetricField,
    tactic: &'a PlannerTactic,
) -> Option<&'a crate::features::planner::tactic::TacticLane> {
    let lane_id = field.tactic_lane_id()?;
    tactic.lanes.iter().find(|lane| lane.lane_id == lane_id)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{parse_value, scan_object_values};

    #[test]
    fn scanned_values_equal_sqlite_json_extract() {
        let json = r#" { "goals" : 0.30000000000000004 , "own\"goals":7,"xg":-1.5e-3,
            "minutes":null, "saves" :12345678901234567 } "#;
        let keys = ["goals", "xg", "minutes", "saves", "assists"].map(str::to_string);
        let keys = keys.iter().collect::<Vec<_>>();
        let mut scanned = vec![None; keys.len()];
        scan_object_values(json, &keys, |index, text| {
            scanned[index] = parse_value(text, |text| text.parse::<f64>().ok())?;
            Ok(())
        })
        .expect("scan object");

        let conn = Connection::open_in_memory().expect("open database");
        let extracted = keys
            .iter()
            .map(|key| {
                conn.query_row(
                    &format!("SELECT json_extract(?1, '$.\"{key}\"')"),
                    [json],
                    |row| row.get::<_, Option<f64>>(0),
                )
                .expect("extract value")
            })
            .collect::<Vec<_>>();
        assert_eq!(scanned, extracted);
        assert_eq!(scanned[0], Some(0.30000000000000004));
        assert_eq!(scanned[2], None);
    }
}
