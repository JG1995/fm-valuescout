//! Each tactic slot's recruitment filters, applied to the Dashboard's candidate signings.
//!
//! Most rules compile to SQL and run as one column per slot in the candidate query. A
//! percentile rule ranks the candidate's statistic within the slot's comparison group (see
//! `moneyball::comparison`). Season-stat rules run only while season stats apply.

use rusqlite::{types::Value, Row};

use crate::features::moneyball::comparison::{GroupRankings, ResolvedGroups};
use crate::features::planner::fit::slot_group;
use crate::features::planner::tactic::{recruitment_filter_rules, PlannerTactic};
use crate::features::search::filter::{compile_slot_filters, CompiledFilter, SlotPercentileRule};

pub(super) struct SlotFilters<'a> {
    lanes: Vec<LaneFilters<'a>>,
    /// The statistic keys percentile rules read, in candidate column order.
    statistic_keys: Vec<String>,
    uses_role_scores: bool,
    uses_season_stats: bool,
}

#[derive(Default)]
struct LaneFilters<'a> {
    sql: Option<CompiledFilter>,
    /// Each percentile rule with its key's index in `statistic_keys`.
    percentile_rules: Vec<(usize, SlotPercentileRule)>,
    /// The slot's group, which its percentile rules rank within.
    rankings: Option<&'a GroupRankings>,
}

impl<'a> SlotFilters<'a> {
    /// Compiles the tactic's filters with SQL placeholders from `start_index`. Without
    /// `season_stats`, rules on Moneyball fields are left out; with them, `rankings` must hold
    /// the group of every slot with a percentile rule. `None` when no slot has a rule to run.
    pub(super) fn load(
        tactic: &PlannerTactic,
        start_index: usize,
        season_stats: bool,
        rankings: Option<&'a ResolvedGroups>,
    ) -> Result<Option<Self>, String> {
        let lane_rules = tactic
            .lanes
            .iter()
            .map(|lane| {
                Ok(recruitment_filter_rules(lane)?
                    .into_iter()
                    .filter(|rule| season_stats || !rule.field.starts_with("moneyball."))
                    .collect::<Vec<_>>())
            })
            .collect::<Result<Vec<_>, String>>()?;
        if lane_rules.iter().all(Vec::is_empty) {
            return Ok(None);
        }
        let mut filters = Self {
            lanes: Vec::with_capacity(tactic.lanes.len()),
            statistic_keys: Vec::new(),
            uses_role_scores: false,
            uses_season_stats: false,
        };
        let mut next_index = start_index;
        for (lane, rules) in tactic.lanes.iter().zip(&lane_rules) {
            let plan = compile_slot_filters(rules, next_index)?;
            next_index += plan.sql.as_ref().map_or(0, |sql| sql.params.len());
            filters.uses_role_scores |= plan.uses_role_scores;
            filters.uses_season_stats |=
                plan.uses_season_stats || !plan.percentile_rules.is_empty();
            let percentile_rules = plan
                .percentile_rules
                .into_iter()
                .map(|rule| (filters.statistic_key_index(&rule.key), rule))
                .collect::<Vec<_>>();
            let rankings = match (percentile_rules.is_empty(), slot_group(lane)) {
                (false, Some(group)) => rankings.and_then(|rankings| rankings.get(group)),
                _ => None,
            };
            filters.lanes.push(LaneFilters {
                sql: plan.sql,
                percentile_rules,
                rankings,
            });
        }
        Ok(Some(filters))
    }

    fn statistic_key_index(&mut self, key: &str) -> usize {
        self.statistic_keys
            .iter()
            .position(|candidate| candidate == key)
            .unwrap_or_else(|| {
                self.statistic_keys.push(key.to_string());
                self.statistic_keys.len() - 1
            })
    }

    /// Whether the candidate query must join the compact role metrics row.
    pub(super) fn uses_role_scores(&self) -> bool {
        self.uses_role_scores
    }

    /// Whether the candidate query must join the season statistics row (`moneyball`).
    pub(super) fn uses_season_stats(&self) -> bool {
        self.uses_season_stats
    }

    /// The candidate query's extra columns, which `lane_passes` reads back, and their
    /// parameters in placeholder order.
    pub(super) fn columns(&self) -> (Vec<String>, Vec<Value>) {
        let mut columns = Vec::new();
        let mut params = Vec::new();
        for sql in self.lanes.iter().filter_map(|lane| lane.sql.as_ref()) {
            columns.push(format!("COALESCE({}, 0)", sql.sql));
            params.extend(sql.params.iter().cloned());
        }
        columns.extend(
            self.statistic_keys
                .iter()
                .map(|key| format!("json_extract(moneyball.statistics_json, '$.\"{key}\"')")),
        );
        (columns, params)
    }

    /// Whether the candidate in `row` passes each slot's filters, reading the extra columns
    /// from `first_column`.
    pub(super) fn lane_passes(
        &self,
        row: &Row<'_>,
        first_column: usize,
    ) -> Result<Vec<bool>, String> {
        let read = |error: rusqlite::Error| error.to_string();
        let sql_lane_count = self.lanes.iter().filter(|lane| lane.sql.is_some()).count();
        let statistics = (0..self.statistic_keys.len())
            .map(|index| row.get::<_, Option<f64>>(first_column + sql_lane_count + index))
            .collect::<Result<Vec<_>, _>>()
            .map_err(read)?;
        let mut sql_column = first_column;
        let mut passes = Vec::with_capacity(self.lanes.len());
        for lane in &self.lanes {
            let mut pass = true;
            if lane.sql.is_some() {
                pass = row.get::<_, i64>(sql_column).map_err(read)? != 0;
                sql_column += 1;
            }
            pass = pass
                && lane.percentile_rules.iter().all(|(key_index, rule)| {
                    let percentile = lane.rankings.and_then(|rankings| {
                        rankings
                            .percentile_of(&self.statistic_keys[*key_index], statistics[*key_index])
                    });
                    rule.matches(percentile)
                });
            passes.push(pass);
        }
        Ok(passes)
    }
}
