//! Each tactic slot's recruitment filters, applied to the Dashboard's candidate signings.
//!
//! Most rules compile to SQL and run as one column per slot in the candidate query. A
//! percentile rule ranks the candidate's statistic within the slot's comparison group (see
//! `moneyball::comparison`), read from his statistics row. Season-stat rules run only
//! while season stats apply.

use rusqlite::{types::Value, Row};

use crate::features::moneyball::comparison::{statistic_index, ComparisonGroup, ResolvedGroups};
use crate::features::planner::fit::slot_group;
use crate::features::planner::tactic::{recruitment_filter_rules, PlannerTactic};
use crate::features::search::filter::{compile_slot_filters, CompiledFilter, SlotPercentileRule};

pub(super) struct SlotFilters {
    lanes: Vec<LaneFilters>,
    uses_role_scores: bool,
    uses_season_stats: bool,
}

#[derive(Default)]
struct LaneFilters {
    sql: Option<CompiledFilter>,
    percentile_rules: Vec<SlotPercentileRule>,
    /// The slot's group, which its percentile rules rank within.
    group: Option<ComparisonGroup>,
}

impl SlotFilters {
    /// Compiles the tactic's filters with SQL placeholders from `start_index`. Without
    /// `season_stats`, rules on Moneyball fields are left out. `None` when no slot has a rule
    /// to run.
    pub(super) fn load(
        tactic: &PlannerTactic,
        start_index: usize,
        season_stats: bool,
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
            let group = if plan.percentile_rules.is_empty() {
                None
            } else {
                slot_group(lane)
            };
            filters.lanes.push(LaneFilters {
                sql: plan.sql,
                percentile_rules: plan.percentile_rules,
                group,
            });
        }
        Ok(Some(filters))
    }

    /// Whether the candidate query must join the compact role metrics row.
    pub(super) fn uses_role_scores(&self) -> bool {
        self.uses_role_scores
    }

    /// Whether the candidate query must join the season statistics row (`moneyball`).
    pub(super) fn uses_season_stats(&self) -> bool {
        self.uses_season_stats
    }

    /// The statistics the percentile rules rank, which `lane_passes` reads from the
    /// candidate's statistics.
    pub(super) fn ranked_statistics(&self) -> impl Iterator<Item = &str> {
        self.lanes
            .iter()
            .flat_map(|lane| lane.percentile_rules.iter().map(|rule| rule.key.as_str()))
    }

    /// The candidate query's extra columns, one for each slot with SQL rules, which
    /// `sql_passes` reads back, and their parameters in placeholder order.
    pub(super) fn columns(&self) -> (Vec<String>, Vec<Value>) {
        let mut columns = Vec::new();
        let mut params = Vec::new();
        for sql in self.lanes.iter().filter_map(|lane| lane.sql.as_ref()) {
            columns.push(format!("COALESCE({}, 0)", sql.sql));
            params.extend(sql.params.iter().cloned());
        }
        (columns, params)
    }

    /// Whether the candidate in `row` passes each slot's SQL rules, reading the extra columns
    /// from `first_column`. A slot without SQL rules passes.
    pub(super) fn sql_passes(
        &self,
        row: &Row<'_>,
        first_column: usize,
    ) -> rusqlite::Result<Vec<bool>> {
        let mut sql_column = first_column;
        self.lanes
            .iter()
            .map(|lane| {
                if lane.sql.is_none() {
                    return Ok(true);
                }
                let pass = row.get::<_, i64>(sql_column)? != 0;
                sql_column += 1;
                Ok(pass)
            })
            .collect()
    }

    /// Whether the candidate passes each slot's filters, given `sql_passes` for his row.
    /// `statistics` are his statistics in `MONEYBALL_STATISTIC_KEYS` order, if he has any,
    /// holding at least `ranked_statistics`, and `rankings` those of the snapshot they come
    /// from, holding the group of every slot with a percentile rule.
    pub(super) fn lane_passes(
        &self,
        sql_passes: &[bool],
        statistics: Option<&[f64]>,
        rankings: Option<&ResolvedGroups>,
    ) -> Vec<bool> {
        self.lanes
            .iter()
            .zip(sql_passes)
            .map(|(lane, sql_pass)| {
                *sql_pass
                    && lane.percentile_rules.iter().all(|rule| {
                        let percentile = lane
                            .group
                            .and_then(|group| rankings?.get(group))
                            .and_then(|rankings| {
                                let index = statistic_index(&rule.key)?;
                                rankings.percentile(index, Some(statistics?[index]))
                            });
                        rule.matches(percentile)
                    })
            })
            .collect()
    }
}
