//! Player Search's Moneyball values computed in Rust: statistic percentiles, Moneyball role
//! scores, and tactic slots' Moneyball Scores.
//!
//! A player's statistics come from his qualified sample: this season's, or a kept snapshot's
//! from the last year when this season has under `MIN_SEASON_MINUTES` minutes (see
//! `moneyball::kept_stats`). They rank among that snapshot's comparison groups and move with
//! the strength of the league he played them in. A player without a qualified sample has no
//! Moneyball values.

use std::collections::BTreeMap;
use std::sync::Arc;

use rusqlite::Connection;

use crate::features::moneyball::comparison::{
    scan_statistic_values, statistic_index, ComparisonGroup, ComparisonGroups, GroupsSource,
    PositionFamily, ResolvedGroups,
};
use crate::features::moneyball::json_scan::parse_json_value;
use crate::features::moneyball::kept_stats::KeptStats;
use crate::features::moneyball::league_strength::{shifted, LeagueAdjustment};
use crate::features::moneyball::role_catalog::{definition_for_attribute_role, RoleDefinition};
use crate::features::moneyball::role_score::score_role;
use crate::features::moneyball::MONEYBALL_STATISTIC_KEYS;
use crate::features::planner::fit::{
    custom_metrics_lane_score, moneyball_roles_lane_score, slot_group,
};
use crate::features::planner::tactic::{base_position, PlannerTactic, TacticLane};
use crate::features::player_metrics::resolver::MetricField;

use super::query::Comparison;

/// A snapshot whose statistics score players: the current one, or a kept one that players
/// borrow from.
pub(super) struct SeasonSource {
    snapshot_id: i64,
    groups: Arc<ComparisonGroups>,
    rankings: ResolvedGroups,
    league: LeagueAdjustment,
}

impl SeasonSource {
    /// The group a statistic's percentile ranks player `uid` within under `comparison`.
    pub(super) fn table_group(&self, comparison: Comparison, uid: i64) -> Option<ComparisonGroup> {
        match comparison {
            Comparison::Own => self.groups.own_family(uid).map(ComparisonGroup::family),
            Comparison::Group(group) => Some(group),
        }
    }

    /// A statistic's percentile within `group`, from statistics in `MONEYBALL_STATISTIC_KEYS`
    /// order (NaN when absent).
    pub(super) fn percentile(
        &self,
        group: ComparisonGroup,
        key: &str,
        statistics: &[f64],
    ) -> Option<u8> {
        self.rankings.percentile(group, key, statistics)
    }

    /// The score points a player in `division` gains or loses.
    pub(super) fn league_shift(&self, division: Option<&str>) -> i16 {
        self.league.shift(division)
    }

    /// A role's Moneyball Score: its metrics ranked within the role's family.
    pub(super) fn role_score(
        &self,
        definition: &RoleDefinition,
        statistics: &[f64],
        league_shift: i16,
    ) -> Option<u8> {
        let group =
            PositionFamily::from_id(&definition.position_family).map(ComparisonGroup::family)?;
        score_role(definition, &|key: &str| {
            self.percentile(group, key, statistics)
        })
        .map(|role| shifted(role.score, league_shift))
    }

    /// A tactic slot's Moneyball Score: its chosen metrics' mean percentile within the slot's
    /// group, or its two roles' Moneyball definitions when it has no chosen metrics, as the
    /// Dashboard compares them.
    pub(super) fn lane_score(
        &self,
        lane: &TacticLane,
        statistics: &[f64],
        fit: &PlayerFit,
        league_shift: i16,
    ) -> Option<u8> {
        let lookup = |group, key: &str| self.percentile(group, key, statistics);
        if lane.custom_metric_keys.is_empty() {
            moneyball_roles_lane_score(&lookup, &fit.foot, &fit.positions, lane, league_shift)
        } else {
            custom_metrics_lane_score(&lookup, &fit.foot, &fit.positions, lane, league_shift)
        }
    }
}

/// The current snapshot's statistics source, then the kept snapshots', latest first.
pub(super) struct SeasonSources(Vec<SeasonSource>);

impl SeasonSources {
    /// Loads each source's comparison groups and league adjustment, with rankings for every
    /// family and for `groups_to_rank`.
    pub(super) fn load(
        conn: &Connection,
        groups: &GroupsSource<'_>,
        snapshot_id: i64,
        kept: &[KeptStats],
        groups_to_rank: &[ComparisonGroup],
    ) -> Result<Self, String> {
        let mut sources = Vec::with_capacity(kept.len() + 1);
        for (source_id, kept_date) in std::iter::once((snapshot_id, None)).chain(
            kept.iter()
                .map(|kept| (kept.snapshot_id, Some(&kept.stats_date))),
        ) {
            let source_groups = match kept_date {
                None => groups.load(conn, source_id)?,
                Some(stats_date) => groups.load_kept(conn, source_id, stats_date)?,
            };
            let rankings =
                ResolvedGroups::resolve(&source_groups, conn, groups_to_rank.iter().copied())?;
            sources.push(SeasonSource {
                snapshot_id: source_id,
                groups: source_groups,
                rankings,
                league: LeagueAdjustment::load(conn, source_id)?,
            });
        }
        Ok(Self(sources))
    }

    pub(super) fn get(&self, snapshot_id: i64) -> Option<&SeasonSource> {
        self.0
            .iter()
            .find(|source| source.snapshot_id == snapshot_id)
    }
}

/// Which statistics a request reads, indexed like `MONEYBALL_STATISTIC_KEYS`.
pub(super) fn needed_statistics<'a>(
    keys: impl IntoIterator<Item = &'a str>,
) -> Result<Vec<bool>, String> {
    let mut needed = vec![false; MONEYBALL_STATISTIC_KEYS.len()];
    for key in keys {
        let index =
            statistic_index(key).ok_or_else(|| format!("unknown Moneyball statistic: {key}"))?;
        needed[index] = true;
    }
    Ok(needed)
}

/// The `needed` statistics of one stored statistics object, in `MONEYBALL_STATISTIC_KEYS`
/// order; the others are NaN.
pub(super) fn scan_needed_statistics(json: &str, needed: &[bool]) -> Result<Box<[f64]>, String> {
    let mut values = vec![f64::NAN; MONEYBALL_STATISTIC_KEYS.len()].into_boxed_slice();
    scan_statistic_values(json, |index, text| {
        if needed[index] {
            if let Some(value) = parse_json_value(text, |text| text.parse::<f64>().ok())? {
                values[index] = value;
            }
        }
        Ok(())
    })?;
    Ok(values)
}

/// The positions and preferred foot that a slot's fit adjustment reads.
pub(super) struct PlayerFit {
    positions: BTreeMap<String, Option<i64>>,
    foot: String,
}

impl PlayerFit {
    pub(super) fn from_columns(positions_json: &str, foot: String) -> Self {
        Self {
            positions: serde_json::from_str(positions_json).unwrap_or_default(),
            foot,
        }
    }
}

/// The tactic slot a tactic column scores.
pub(super) fn tactic_lane<'a>(
    field: &MetricField,
    tactic: &'a PlannerTactic,
) -> Option<&'a TacticLane> {
    let lane_id = field.tactic_lane_id()?;
    tactic.lanes.iter().find(|lane| lane.lane_id == lane_id)
}

/// The statistics a slot's Moneyball Score reads.
pub(super) fn lane_statistic_keys(lane: &TacticLane) -> Vec<String> {
    if !lane.custom_metric_keys.is_empty() {
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

/// The group a slot's chosen metrics rank within; role definitions rank within their own
/// families, which every source ranks anyway.
pub(super) fn lane_group(lane: &TacticLane) -> Option<ComparisonGroup> {
    if lane.custom_metric_keys.is_empty() {
        None
    } else {
        slot_group(lane)
    }
}
