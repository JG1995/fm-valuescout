//! How a player's score in each tactic slot ranks against every player in the save who can
//! play that slot.
//!
//! Some roles score higher than others for every player, so the slot where a player scores
//! highest is not always the one he stands out in. His rank among the players who can play
//! each slot shows that instead.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use rusqlite::{params, Connection};

use crate::features::player_metrics::compact::{
    player_current_column, player_metrics_join, player_potential_column, scores_revision,
    PLAYER_METRICS_ALIAS,
};
use crate::features::scoring::combine::combine_role_scores;

use super::depth::PlannerTeam;
use super::fit::{phase_fit_score, plays_lane, PositionRule};
use super::outlook::{future_weight, Development};
use super::tactic::{PlannerTactic, TacticLane};

/// How many of a slot's players reach each score.
#[derive(Debug, Clone)]
pub(super) struct ScoreDistribution {
    counts: [u32; 256],
    total: u32,
}

impl Default for ScoreDistribution {
    fn default() -> Self {
        Self {
            counts: [0; 256],
            total: 0,
        }
    }
}

impl ScoreDistribution {
    fn add(&mut self, score: Option<u8>) {
        if let Some(score) = score {
            self.counts[usize::from(score)] += 1;
            self.total += 1;
        }
    }

    /// The share of the slot's players who score below `score`, counting half of those level
    /// with it: near 0 for the weakest and near 1 for the best. A slot nobody plays ranks
    /// every score in the middle.
    pub(super) fn rank(&self, score: u8) -> f64 {
        if self.total == 0 {
            return 0.5;
        }
        let below: u32 = self.counts[..usize::from(score)].iter().sum();
        let level = self.counts[usize::from(score)];
        (f64::from(below) + f64::from(level) / 2.0) / f64::from(self.total)
    }
}

/// One tactic lane's score distributions: its fit score, then each phase's fit score.
#[derive(Debug, Clone, Default)]
pub(super) struct LaneRankings {
    pub(super) current: ScoreDistribution,
    pub(super) potential: ScoreDistribution,
    pub(super) ip_current: ScoreDistribution,
    pub(super) ip_potential: ScoreDistribution,
    pub(super) oop_current: ScoreDistribution,
    pub(super) oop_potential: ScoreDistribution,
}

impl LaneRankings {
    /// Counts one player's four role scores for the lane: current IP and OOP, then potential
    /// IP and OOP.
    fn add_player(
        &mut self,
        lane: &TacticLane,
        player_foot: &str,
        positions: &BTreeMap<String, Option<i64>>,
        scores: &[Option<u8>],
        position_rule: PositionRule,
    ) {
        let plays_lane = plays_lane(positions, lane);
        let lane_fit = |ip_score, oop_score| {
            if !plays_lane {
                return None;
            }
            position_rule.lane_score(
                combine_role_scores(ip_score, oop_score, lane.ip_weight),
                player_foot,
                positions,
                lane,
            )
        };
        let phase_fit = |score, position| {
            phase_fit_score(
                score,
                player_foot,
                positions,
                position,
                &lane.preferred_foot,
                &lane.foot_preference,
            )
        };
        let current = lane_fit(scores[0], scores[1]);
        self.current.add(current);
        // A lane without a potential score promises no growth.
        self.potential
            .add(lane_fit(scores[2], scores[3]).or(current));
        self.ip_current.add(phase_fit(scores[0], &lane.ip_position));
        self.oop_current
            .add(phase_fit(scores[1], &lane.oop_position));
        self.ip_potential
            .add(phase_fit(scores[2], &lane.ip_position));
        self.oop_potential
            .add(phase_fit(scores[3], &lane.oop_position));
    }
}

/// Every tactic lane's rankings, in the tactic's lane order.
#[derive(Debug, Clone)]
pub(super) struct SlotRankings {
    lanes: Vec<LaneRankings>,
}

impl SlotRankings {
    pub(super) fn lane(&self, lane_index: usize) -> &LaneRankings {
        &self.lanes[lane_index]
    }

    /// The lane the player stands out in for `team`, from his current and potential fit
    /// scores in each lane he can play. Each score is ranked against the save, and `team`
    /// weighs the potential rank by its focus on the future and the player's development.
    /// Level lanes go to the higher current score, then the earlier lane.
    pub(super) fn standout_lane(
        &self,
        team: PlannerTeam,
        development: &Development,
        lane_scores: &[Option<(u8, u8)>],
    ) -> Option<usize> {
        self.lane_by_rank(future_weight(team, development), lane_scores)
    }

    /// The lane the player is growing into: the one where his potential fit score ranks
    /// highest against the save, whatever he scores now. Level lanes go as in
    /// [`Self::standout_lane`].
    pub(super) fn development_lane(&self, lane_scores: &[Option<(u8, u8)>]) -> Option<usize> {
        self.lane_by_rank(1.0, lane_scores)
    }

    /// The lane where the player's ranks stand highest, weighing his potential rank by `future`
    /// and his current rank by the rest.
    fn lane_by_rank(&self, future: f64, lane_scores: &[Option<(u8, u8)>]) -> Option<usize> {
        let mut standout: Option<(usize, f64, u8)> = None;
        for (lane_index, scores) in lane_scores.iter().enumerate() {
            let Some((current, potential)) = *scores else {
                continue;
            };
            let lane = &self.lanes[lane_index];
            let rank = (1.0 - future) * lane.current.rank(current)
                + future * lane.potential.rank(potential);
            let stands_out_more = standout.map_or(true, |(_, best_rank, best_current)| {
                rank > best_rank || (rank == best_rank && current > best_current)
            });
            if stands_out_more {
                standout = Some((lane_index, rank, current));
            }
        }
        standout.map(|(lane_index, _, _)| lane_index)
    }
}

/// What a kept [`SlotRankings`] was read from: everything a ranking reads.
///
/// A snapshot's players, positions, and feet are written once, when Load Data ingests it under
/// a token of its own. Its role scores change later only through `compact`, which counts those
/// writes. So this key holds across every other write, such as a Planner assignment or a
/// setting, where a data generation would not.
#[derive(PartialEq, Eq)]
struct RankingsKey {
    database: String,
    /// The snapshot's own token. A snapshot's row ID can return after a deletion.
    snapshot_token: String,
    scores_revision: u64,
    any_position: bool,
    /// Every lane's roles, positions, weight, and foot rule: all a ranking reads of the tactic.
    tactic: String,
}

impl RankingsKey {
    /// The key on a read-only connection, or `None` for the writer, which always ranks in full
    /// because it may hold changes it has yet to commit.
    fn for_reader(
        conn: &Connection,
        snapshot_id: i64,
        tactic: &PlannerTactic,
        position_rule: PositionRule,
    ) -> Option<Self> {
        if !conn.is_readonly(rusqlite::MAIN_DB).ok()? {
            return None;
        }
        // Read before the ranking, so a write that lands meanwhile ranks again.
        let scores_revision = scores_revision(conn);
        Some(Self {
            database: conn.path()?.to_string(),
            snapshot_token: conn
                .query_row(
                    "SELECT context_token FROM snapshots WHERE id = ?1",
                    [snapshot_id],
                    |row| row.get(0),
                )
                .ok()?,
            scores_revision,
            any_position: position_rule == PositionRule::AnyPosition,
            tactic: tactic
                .lanes
                .iter()
                .map(|lane| {
                    format!(
                        "{}|{}|{}|{}|{}|{}|{};",
                        lane.ip_position,
                        lane.ip_role_id,
                        lane.oop_position,
                        lane.oop_role_id,
                        lane.ip_weight.to_bits(),
                        lane.preferred_foot,
                        lane.foot_preference
                    )
                })
                .collect(),
        })
    }
}

/// The rankings last read, newest last. A handful covers both position rules of the open
/// database; tests open more.
const KEPT_RANKINGS: usize = 4;

type KeptRankings = Vec<(RankingsKey, Arc<SlotRankings>)>;

fn kept_rankings() -> MutexGuard<'static, KeptRankings> {
    static KEPT: OnceLock<Mutex<KeptRankings>> = OnceLock::new();
    KEPT.get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// [`load_slot_rankings`], kept for readers until the snapshot, its role scores, or the tactic
/// change ([`RankingsKey`]). The Training workspace, the Squad overview, the Planner board, and
/// Best role fit all ask for the same rankings, and each ranking reads the whole snapshot. Two
/// readers that miss together rank once: the second waits for the first.
pub(super) fn slot_rankings(
    conn: &Connection,
    snapshot_id: i64,
    tactic: &PlannerTactic,
    position_rule: PositionRule,
) -> Result<Arc<SlotRankings>, String> {
    let Some(key) = RankingsKey::for_reader(conn, snapshot_id, tactic, position_rule) else {
        return load_slot_rankings(conn, snapshot_id, tactic, position_rule).map(Arc::new);
    };
    let find_kept = || {
        kept_rankings()
            .iter()
            .find(|(kept, _)| *kept == key)
            .map(|(_, rankings)| Arc::clone(rankings))
    };
    if let Some(rankings) = find_kept() {
        return Ok(rankings);
    }
    static RANKING: Mutex<()> = Mutex::new(());
    let _ranking = RANKING.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(rankings) = find_kept() {
        return Ok(rankings);
    }
    let rankings = Arc::new(load_slot_rankings(
        conn,
        snapshot_id,
        tactic,
        position_rule,
    )?);
    let mut kept = kept_rankings();
    // Rankings of this database's earlier snapshots and scores will not be asked for again.
    kept.retain(|(other, _)| {
        other.database != key.database
            || (other.snapshot_token == key.snapshot_token
                && other.scores_revision == key.scores_revision)
    });
    if kept.len() >= KEPT_RANKINGS {
        kept.remove(0);
    }
    kept.push((key, Arc::clone(&rankings)));
    Ok(rankings)
}

/// Ranks every lane of `tactic` over all of the snapshot's players. A player counts in a lane
/// when he can play its positions and has the role scores, and `position_rule` scores him
/// there as Optimize does. Every call reads the whole snapshot once; readers ask
/// [`slot_rankings`] instead.
pub(super) fn load_slot_rankings(
    conn: &Connection,
    snapshot_id: i64,
    tactic: &PlannerTactic,
    position_rule: PositionRule,
) -> Result<SlotRankings, String> {
    // The foot and positions, then each lane's scores.
    const SCORE_COLUMN: usize = 2;
    // Each lane's current IP and OOP scores, then its potential IP and OOP scores.
    const SCORES_PER_LANE: usize = 4;

    let mut metric_columns = Vec::with_capacity(tactic.lanes.len() * SCORES_PER_LANE);
    for lane in &tactic.lanes {
        metric_columns.push(player_current_column(&lane.ip_role_id)?.to_string());
        metric_columns.push(player_current_column(&lane.oop_role_id)?.to_string());
        metric_columns.push(player_potential_column(&lane.ip_role_id)?);
        metric_columns.push(player_potential_column(&lane.oop_role_id)?);
    }
    let metric_select = metric_columns
        .iter()
        .map(|column| format!(", {PLAYER_METRICS_ALIAS}.{column}"))
        .collect::<String>();
    let sql = format!(
        "SELECT p.preferred_foot, p.positions_json{metric_select}
         FROM players p{}
         WHERE p.snapshot_id = ?1",
        player_metrics_join("p", true, true)
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let mut rows = statement
        .query(params![snapshot_id])
        .map_err(|error| error.to_string())?;

    let mut lanes = vec![LaneRankings::default(); tactic.lanes.len()];
    let mut scores = vec![None; metric_columns.len()];
    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let player_foot = row
            .get_ref(0)
            .and_then(|value| Ok(value.as_str()?))
            .map_err(|error| error.to_string())?;
        let positions = row
            .get_ref(1)
            .and_then(|value| Ok(value.as_str()?))
            .map_err(|error| error.to_string())
            .and_then(|json| {
                serde_json::from_str::<BTreeMap<String, Option<i64>>>(json)
                    .map_err(|error| error.to_string())
            })?;
        for (index, score) in scores.iter_mut().enumerate() {
            *score = row
                .get::<_, Option<u8>>(SCORE_COLUMN + index)
                .map_err(|error| error.to_string())?;
        }
        for ((lane, rankings), lane_scores) in tactic
            .lanes
            .iter()
            .zip(&mut lanes)
            .zip(scores.chunks_exact(SCORES_PER_LANE))
        {
            rankings.add_player(lane, player_foot, &positions, lane_scores, position_rule);
        }
    }
    Ok(SlotRankings { lanes })
}
