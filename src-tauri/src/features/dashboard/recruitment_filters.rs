//! Each tactic slot's recruitment filters, applied to the Dashboard's candidate signings.
//!
//! Most rules compile to SQL and run as one column per slot in the candidate query. A
//! percentile rule ranks the candidate's statistic among the slot's position peers: the
//! players with season statistics rated at least 16 at either of the slot's two positions.

use rusqlite::{types::Value, Connection, Row};

use crate::features::moneyball::percentile::percentile_in_population;
use crate::features::planner::tactic::{base_position, recruitment_filter_rules, PlannerTactic};
use crate::features::player_metrics::compact::player_metrics_join;
use crate::features::search::filter::{compile_slot_filters, CompiledFilter, SlotPercentileRule};

/// The lowest familiarity at which a player counts among a slot's position peers.
const PEER_FAMILIARITY: i64 = 16;

pub(super) struct SlotFilters {
    lanes: Vec<LaneFilters>,
    /// The statistic keys percentile rules read, in candidate column order.
    statistic_keys: Vec<String>,
    uses_role_scores: bool,
    uses_season_stats: bool,
}

#[derive(Default)]
struct LaneFilters {
    sql: Option<CompiledFilter>,
    /// Each percentile rule with its key's index in `statistic_keys` and the peers' values of
    /// that statistic, sorted.
    percentile_rules: Vec<(usize, SlotPercentileRule, Vec<f64>)>,
}

impl SlotFilters {
    /// Compiles the tactic's filters with SQL placeholders from `start_index` and ranks the
    /// position peers its percentile rules need. `None` when no slot has filters.
    pub(super) fn load(
        conn: &Connection,
        snapshot_id: i64,
        tactic: &PlannerTactic,
        start_index: usize,
    ) -> Result<Option<Self>, String> {
        if tactic
            .lanes
            .iter()
            .all(|lane| lane.recruitment_filters.is_empty())
        {
            return Ok(None);
        }
        let mut filters = Self {
            lanes: Vec::with_capacity(tactic.lanes.len()),
            statistic_keys: Vec::new(),
            uses_role_scores: false,
            uses_season_stats: false,
        };
        let mut next_index = start_index;
        for lane in &tactic.lanes {
            let plan = compile_slot_filters(&recruitment_filter_rules(lane)?, next_index)?;
            next_index += plan.sql.as_ref().map_or(0, |sql| sql.params.len());
            filters.uses_role_scores |= plan.uses_role_scores;
            filters.uses_season_stats |=
                plan.uses_season_stats || !plan.percentile_rules.is_empty();
            let percentile_rules = plan
                .percentile_rules
                .into_iter()
                .map(|rule| (filters.statistic_key_index(&rule.key), rule, Vec::new()))
                .collect();
            filters.lanes.push(LaneFilters {
                sql: plan.sql,
                percentile_rules,
            });
        }
        filters.rank_peers(conn, snapshot_id, tactic)?;
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

    /// Reads every season-statistics row once and gives each percentile rule its slot's
    /// peers' values.
    fn rank_peers(
        &mut self,
        conn: &Connection,
        snapshot_id: i64,
        tactic: &PlannerTactic,
    ) -> Result<(), String> {
        let mut positions = Vec::new();
        for (lane, filters) in tactic.lanes.iter().zip(&self.lanes) {
            if filters.percentile_rules.is_empty() {
                continue;
            }
            for position in [
                base_position(&lane.ip_position),
                base_position(&lane.oop_position),
            ] {
                if !positions.contains(&position) {
                    positions.push(position);
                }
            }
        }
        if positions.is_empty() {
            return Ok(());
        }
        // Positions come from validated tactic placements and keys from the statistic
        // catalog, so both are safe to embed in the JSON paths.
        let columns = positions
            .iter()
            .map(|position| format!("json_extract(players.positions_json, '$.{position}')"))
            .chain(
                self.statistic_keys
                    .iter()
                    .map(|key| format!("json_extract(moneyball.statistics_json, '$.\"{key}\"')")),
            )
            .collect::<Vec<_>>()
            .join(", ");
        let mut statement = conn
            .prepare(&format!(
                "SELECT {columns}
                 FROM player_moneyball_stats moneyball
                 INNER JOIN players ON players.snapshot_id = moneyball.snapshot_id
                    AND players.uid = moneyball.player_uid
                 WHERE moneyball.snapshot_id = ?1 AND moneyball.percentiles_json IS NOT NULL"
            ))
            .map_err(|error| error.to_string())?;
        let mut rows = statement
            .query([snapshot_id])
            .map_err(|error| error.to_string())?;
        while let Some(row) = rows.next().map_err(|error| error.to_string())? {
            let familiarity = |position: &str| -> Result<i64, String> {
                let index = positions
                    .iter()
                    .position(|candidate| *candidate == position)
                    .expect("ranked position");
                row.get::<_, Option<i64>>(index)
                    .map(|value| value.unwrap_or(0))
                    .map_err(|error| error.to_string())
            };
            for (lane, filters) in tactic.lanes.iter().zip(&mut self.lanes) {
                if filters.percentile_rules.is_empty()
                    || (familiarity(base_position(&lane.ip_position))? < PEER_FAMILIARITY
                        && familiarity(base_position(&lane.oop_position))? < PEER_FAMILIARITY)
                {
                    continue;
                }
                for (key_index, _, peers) in &mut filters.percentile_rules {
                    let value: Option<f64> = row
                        .get(positions.len() + *key_index)
                        .map_err(|error| error.to_string())?;
                    peers.extend(value.filter(|value| value.is_finite()));
                }
            }
        }
        for filters in &mut self.lanes {
            for (_, _, peers) in &mut filters.percentile_rules {
                peers.sort_by(f64::total_cmp);
            }
        }
        Ok(())
    }

    /// The joins the candidate query needs on top of its own, given which table it already has.
    pub(super) fn extra_joins(&self, has_role_scores: bool, has_season_stats: bool) -> String {
        let mut joins = String::new();
        if self.uses_role_scores && !has_role_scores {
            joins.push_str(&player_metrics_join("players", true, false));
        }
        if self.uses_season_stats && !has_season_stats {
            joins.push_str(
                " LEFT JOIN player_moneyball_stats moneyball
                   ON moneyball.snapshot_id = players.snapshot_id
                  AND moneyball.player_uid = players.uid",
            );
        }
        joins
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
                && lane
                    .percentile_rules
                    .iter()
                    .all(|(key_index, rule, peers)| {
                        let percentile = statistics[*key_index].and_then(|value| {
                            percentile_in_population(peers, value, &self.statistic_keys[*key_index])
                        });
                        rule.matches(percentile)
                    });
            passes.push(pass);
        }
        Ok(passes)
    }
}
