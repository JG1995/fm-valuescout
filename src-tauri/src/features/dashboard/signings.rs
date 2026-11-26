//! Recommended signings: the managed club's starting XI per tactic slot, and the players who
//! would improve it, taken from the club's realistic signings or from one shortlist.

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, HashSet};

use rusqlite::{params_from_iter, types::Value, Connection, Row};

use crate::features::managed_club::service::{managed_club_player_sql, managed_club_sql};
use crate::features::moneyball::columns::StatisticColumns;
pub(crate) use crate::features::moneyball::comparison::MIN_SEASON_MINUTES;
use crate::features::moneyball::comparison::{
    scan_in_parallel, statistic_index, GroupsSource, ResolvedGroups,
};
use crate::features::moneyball::kept_stats::{
    kept_stats_snapshots, stats_join_sql, STATS_DIVISION_SQL,
};
use crate::features::moneyball::league_strength::LeagueAdjustment;
use crate::features::moneyball::role_catalog::definition_for_attribute_role;
use crate::features::planner::fit::{
    custom_metrics_lane_score, lane_statistic_keys, moneyball_roles_lane_score, slot_group,
    tactic_adjusted_score,
};
use crate::features::planner::optimizer::{allocate_lanes, OptimizerCandidate};
use crate::features::planner::tactic::{
    base_position, default_tactic, read_tactic, PlannerTactic, TacticLane,
};
use crate::features::player_metrics::compact::{
    assert_read_models_complete, player_current_column, player_metrics_join, PLAYER_METRICS_ALIAS,
    SCORE_MODEL_VERSION,
};
use crate::features::search::realistic_signing;

use super::club_snapshot_id;
use super::recruitment_filters::SlotFilters;

/// The most ranked recommendations one read returns: as many as the dashboard shows at once.
pub(crate) const MAX_RECOMMENDATIONS: usize = 12;
const MAX_OTHER_LANES: usize = 2;
/// The oldest age a candidate is compared at his full score, as in Search's saved filters.
const FULL_SCORE_AGE: i64 = 28;

/// Which scores compare a candidate with the club's player in a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScoreMode {
    /// The slot's attribute role scores, as in Search's Tactic (Current) columns.
    General,
    /// The slot's Moneyball score from season statistics.
    Moneyball,
}

impl ScoreMode {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "general" => Ok(Self::General),
            "moneyball" => Ok(Self::Moneyball),
            _ => Err(format!("unknown dashboard score mode: {value}")),
        }
    }
}

/// What a slot's score is made of in the requested mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScoreBasis {
    Attributes,
    /// The slot's chosen Moneyball metrics.
    MoneyballCustom,
    /// The Moneyball definitions of the slot's roles, used when it has no chosen metrics.
    MoneyballRoles,
    /// The slot has no chosen metrics and a role without a Moneyball definition.
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IncumbentSource {
    /// The player assigned to the slot in the Planner's first string.
    Planner,
    /// The best remaining club player, allocated as the Planner's Optimize does.
    BestFit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LineupPlayer {
    pub(crate) uid: i64,
    pub(crate) name: String,
    pub(crate) score: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LineupSlot {
    pub(crate) lane: TacticLane,
    pub(crate) score_basis: ScoreBasis,
    pub(crate) incumbent: Option<(LineupPlayer, IncumbentSource)>,
    /// The best club player for the slot outside the starting XI.
    pub(crate) cover: Option<LineupPlayer>,
    /// The Planner's first-string player for the slot, when the mode gives them no score
    /// there and another player stands in.
    pub(crate) unscored_planner_player: Option<String>,
    /// The candidate who would improve the slot most. Slots answer independently, so one
    /// player can be the best signing for several.
    pub(crate) best_signing: Option<Recommendation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OtherLane {
    pub(crate) lane_id: String,
    pub(crate) improvement: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Recommendation {
    pub(crate) uid: i64,
    pub(crate) name: String,
    pub(crate) age: Option<i64>,
    pub(crate) club: Option<String>,
    pub(crate) club_uid: Option<i64>,
    pub(crate) market_value_gbp: Option<i64>,
    /// Season minutes behind a Moneyball score; absent in General mode.
    pub(crate) minutes: Option<i64>,
    /// The in-game date of the kept snapshot's statistics behind a Moneyball score, when the
    /// player has under `MIN_SEASON_MINUTES` minutes this season and borrows them.
    pub(crate) borrowed_stats_date: Option<String>,
    pub(crate) lane_id: String,
    /// His score in the slot after `age_penalty`.
    pub(crate) score: u8,
    /// Points above the slot's incumbent, or the whole score when the slot has none.
    pub(crate) improvement: u8,
    /// The points his scores lost for his age; see [`age_penalty`].
    pub(crate) age_penalty: u8,
    /// Further slots the player would improve, best first.
    pub(crate) other_lanes: Vec<OtherLane>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Signings {
    /// Whether the save's tactic still equals the default one.
    pub(crate) tactic_is_default: bool,
    /// False only in Moneyball mode when the snapshot has no season statistics.
    pub(crate) scores_available: bool,
    pub(crate) slots: Vec<LineupSlot>,
    /// How many players are compared: the requested shortlist's players in the current
    /// snapshot outside the managed club. Realistic signings are not counted, so without a
    /// shortlist this is `None` when the club has any and `Some(0)` when it has none. In
    /// Moneyball mode a realistic signing also needs season statistics.
    pub(crate) candidates: Option<usize>,
    pub(crate) total_recommendations: usize,
    pub(crate) recommendations: Vec<Recommendation>,
    /// Candidates who would improve a considered slot but are held back from every slot they
    /// would improve: by its recruitment filters, or by too few minutes.
    pub(crate) filtered_out: usize,
    /// Of `filtered_out`, those held back by having under `MIN_SEASON_MINUTES` minutes.
    pub(crate) under_minimum_minutes: usize,
    /// Whether the snapshot has season statistics, or a kept snapshot has some to lend.
    pub(crate) season_stats_available: bool,
    /// Whether season stats applied: candidates needed `MIN_SEASON_MINUTES` minutes and the
    /// slots' season-stat filters ran. Always in Moneyball mode when the snapshot has them.
    pub(crate) season_stats_applied: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SigningsRequest<'a> {
    pub(crate) mode: ScoreMode,
    /// One of the save's shortlists, or the club's realistic signings when absent.
    pub(crate) shortlist_id: Option<i64>,
    /// Restricts the ranked recommendations to one slot. Each slot's best signing is unaffected.
    pub(crate) lane_id: Option<&'a str>,
    /// Whether General mode applies season stats; Moneyball mode always does.
    pub(crate) season_stats: bool,
}

struct ScoredPlayer {
    uid: i64,
    name: String,
    age: Option<i64>,
    club: Option<String>,
    club_uid: Option<i64>,
    market_value_gbp: Option<i64>,
    minutes: Option<i64>,
    /// The date of the kept statistics behind his Moneyball scores; `None` in General mode.
    borrowed_stats_date: Option<String>,
    /// The player's score in each tactic slot, in lane order, at his full ability.
    lane_scores: Vec<Option<u8>>,
    /// The points a candidate's scores lose for his age; zero for the club's own players.
    age_penalty: u8,
    /// Whether the player may be recommended for each slot: he has enough minutes and passes
    /// its recruitment filters. Always true for the club's own players.
    lane_allowed: Vec<bool>,
    /// Whether season stats applied and the player has under `MIN_SEASON_MINUTES` minutes.
    under_minimum_minutes: bool,
}

/// One player's row of the candidate query, read before he is scored.
struct CandidateRow {
    uid: i64,
    name: String,
    age: Option<i64>,
    club: Option<String>,
    club_uid: Option<i64>,
    market_value_gbp: Option<i64>,
    foot: String,
    positions_json: String,
    /// General mode: the two role scores of each slot, in lane order.
    role_scores: Vec<Option<u8>>,
    /// The minutes of his statistics row, when the query reads them.
    minutes: Option<i64>,
    /// The statistics the request scores or ranks, when the query reads them and he has a
    /// statistics row.
    statistics: Option<Box<[f64]>>,
    /// Moneyball mode: the league he played his statistics in.
    division: Option<String>,
    /// The snapshot his statistics come from, when the query reads them.
    source_id: Option<i64>,
    /// `SlotFilters::sql_passes`, for a candidate of a tactic with recruitment filters.
    sql_passes: Option<Vec<bool>>,
}

/// A snapshot whose season statistics score players: the current one, or a kept one that
/// players under `MIN_SEASON_MINUTES` minutes borrow from (see `moneyball::kept_stats`).
struct StatsSource {
    snapshot_id: i64,
    /// The in-game date of a kept snapshot's statistics; `None` for the current snapshot.
    kept_date: Option<String>,
    /// The comparison groups its statistics rank within, when the request ranks any.
    rankings: Option<ResolvedGroups>,
    /// How its league strengths move Moneyball scores.
    league: LeagueAdjustment,
}

#[derive(Clone, Copy)]
enum PlayerScope<'a> {
    Club,
    /// The players of one of the save's shortlists outside the managed club.
    Shortlist(i64),
    /// The club's realistic signings outside it. In General mode these are only the ones who
    /// could improve a slot, given each slot's incumbent score.
    RealisticSignings(&'a [Option<u8>]),
}

/// A save's club and candidates scored in every tactic slot: everything a signings answer
/// needs except the slot it is restricted to. Scoring the candidates is the slow part, so the
/// app keeps this per data generation (`dashboard::commands::SigningsCache`) and answers a
/// slot change from it.
pub(crate) struct ScoredSignings {
    tactic: PlannerTactic,
    mode: ScoreMode,
    scores_available: bool,
    season_stats_available: bool,
    season_stats_applied: bool,
    club: Vec<ScoredPlayer>,
    lineup: Lineup,
    incumbent_scores: Vec<Option<u8>>,
    /// The candidates who would improve a slot. The rest never appear in an answer.
    candidates: Vec<ScoredPlayer>,
    /// `Signings::candidates`.
    candidate_count: Option<usize>,
}

/// The save's starting XI and recommended signings, or `None` without a current snapshot or a
/// configured managed club.
#[cfg(test)]
pub(crate) fn get_signings(
    conn: &Connection,
    groups: &GroupsSource<'_>,
    save_id: i64,
    request: SigningsRequest<'_>,
) -> Result<Option<Signings>, String> {
    score_signings(conn, groups, save_id, request)?
        .map(|scored| scored.answer(request.lane_id))
        .transpose()
}

/// Scores the save's club and the request's candidates, whatever slot it names, or `None`
/// without a current snapshot or a configured managed club. It only reads, so a save without
/// stored lanes uses the default tactic in memory.
pub(crate) fn score_signings(
    conn: &Connection,
    groups: &GroupsSource<'_>,
    save_id: i64,
    request: SigningsRequest<'_>,
) -> Result<Option<ScoredSignings>, String> {
    let Some(snapshot_id) = club_snapshot_id(conn, save_id)? else {
        return Ok(None);
    };
    let tactic = read_tactic(conn, save_id)?;
    let mode = request.mode;
    let kept = kept_stats_snapshots(conn, snapshot_id)?;
    let season_stats_available = !kept.is_empty() || snapshot_has_percentiles(conn, snapshot_id)?;
    let scores_available = match mode {
        ScoreMode::General => {
            assert_read_models_complete(conn, snapshot_id, true, false)?;
            true
        }
        ScoreMode::Moneyball => season_stats_available,
    };
    let season_stats =
        season_stats_available && (mode == ScoreMode::Moneyball || request.season_stats);

    // Moneyball scores and percentile filters rank statistics within comparison groups: the
    // roles' families, and the slot's group for custom metrics and filters.
    let ranks_slot = |lane: &TacticLane| {
        (mode == ScoreMode::Moneyball && !lane.custom_metric_keys.is_empty())
            || (season_stats && has_percentile_filter(lane))
    };
    let ranks =
        season_stats && (mode == ScoreMode::Moneyball || tactic.lanes.iter().any(ranks_slot));
    let slots = tactic
        .lanes
        .iter()
        .filter(|lane| ranks_slot(lane))
        .filter_map(slot_group)
        .collect::<Vec<_>>();
    let source = |snapshot_id: i64, kept_date: Option<String>| {
        let rankings = if !ranks {
            None
        } else {
            let groups = match &kept_date {
                None => groups.load(conn, snapshot_id)?,
                Some(stats_date) => groups.load_kept(conn, snapshot_id, stats_date)?,
            };
            Some(ResolvedGroups::resolve(
                &groups,
                conn,
                slots.iter().copied(),
            )?)
        };
        let league = match mode {
            ScoreMode::General => LeagueAdjustment::default(),
            ScoreMode::Moneyball => LeagueAdjustment::load(conn, snapshot_id)?,
        };
        Ok::<_, String>(StatsSource {
            snapshot_id,
            kept_date,
            rankings,
            league,
        })
    };
    // The current snapshot first, then the kept snapshots players can borrow from.
    let mut sources = vec![source(snapshot_id, None)?];
    if season_stats {
        for kept in kept {
            sources.push(source(kept.snapshot_id, Some(kept.stats_date))?);
        }
    }
    let load = |scope| {
        load_players(
            conn,
            save_id,
            snapshot_id,
            scope,
            &tactic,
            mode,
            season_stats,
            &sources,
        )
    };
    let club = load(PlayerScope::Club)?;
    let lineup = starting_lineup(&tactic, &club, &first_string_assignments(conn, save_id)?);
    let incumbent_scores = lineup
        .incumbents
        .iter()
        .enumerate()
        .map(|(lane_index, incumbent)| {
            incumbent.and_then(|(player_index, _)| club[player_index].lane_scores[lane_index])
        })
        .collect::<Vec<_>>();
    let scope = match request.shortlist_id {
        Some(shortlist_id) => PlayerScope::Shortlist(shortlist_id),
        None => PlayerScope::RealisticSignings(&incumbent_scores),
    };
    let mut candidates = load(scope)?;
    let candidate_count = match request.shortlist_id {
        Some(_) => Some(candidates.len()),
        None if !candidates.is_empty() => None,
        None if has_realistic_signings(conn, save_id, snapshot_id, mode, &sources)? => None,
        None => Some(0),
    };
    candidates.retain(|player| {
        (0..tactic.lanes.len())
            .any(|lane_index| improvement(player, &incumbent_scores, lane_index).is_some())
    });

    Ok(Some(ScoredSignings {
        tactic,
        mode,
        scores_available,
        season_stats_available,
        season_stats_applied: season_stats,
        club,
        lineup,
        incumbent_scores,
        candidates,
        candidate_count,
    }))
}

impl ScoredSignings {
    /// The starting XI and the recommended signings, ranked for the slot `lane_id` or for
    /// every slot.
    pub(crate) fn answer(&self, lane_id: Option<&str>) -> Result<Signings, String> {
        let Self {
            tactic,
            club,
            lineup,
            incumbent_scores,
            candidates,
            ..
        } = self;
        let lane_filter = lane_id
            .map(|lane_id| {
                tactic
                    .lanes
                    .iter()
                    .position(|lane| lane.lane_id == lane_id)
                    .ok_or_else(|| format!("unknown tactic lane: {lane_id}"))
            })
            .transpose()?;
        let mut recommendations = candidates
            .iter()
            .filter_map(|player| recommend(player, tactic, incumbent_scores, lane_filter))
            .collect::<Vec<_>>();
        let held_back = candidates
            .iter()
            .filter(|player| {
                let mut improving = (0..tactic.lanes.len())
                    .filter(|&lane_index| lane_filter.map_or(true, |filter| filter == lane_index))
                    .filter(|&lane_index| {
                        improvement(player, incumbent_scores, lane_index).is_some()
                    })
                    .peekable();
                improving.peek().is_some()
                    && improving.all(|lane_index| !player.lane_allowed[lane_index])
            })
            .collect::<Vec<_>>();
        let filtered_out = held_back.len();
        let under_minimum_minutes = held_back
            .iter()
            .filter(|player| player.under_minimum_minutes)
            .count();
        recommendations.sort_by(by_improvement);
        let total_recommendations = recommendations.len();
        recommendations.truncate(MAX_RECOMMENDATIONS);

        let slots = tactic
            .lanes
            .iter()
            .enumerate()
            .map(|(lane_index, lane)| LineupSlot {
                lane: lane.clone(),
                score_basis: score_basis(lane, self.mode),
                incumbent: lineup.incumbents[lane_index].and_then(|(player_index, source)| {
                    lineup_player(&club[player_index], lane_index).map(|player| (player, source))
                }),
                cover: cover(club, lineup, lane_index),
                unscored_planner_player: lineup.unscored_planner_players[lane_index].clone(),
                best_signing: candidates
                    .iter()
                    .filter_map(|player| {
                        recommend(player, tactic, incumbent_scores, Some(lane_index))
                    })
                    .min_by(by_improvement),
            })
            .collect();

        Ok(Signings {
            tactic_is_default: *tactic == default_tactic(),
            scores_available: self.scores_available,
            slots,
            candidates: self.candidate_count,
            total_recommendations,
            recommendations,
            filtered_out,
            under_minimum_minutes,
            season_stats_available: self.season_stats_available,
            season_stats_applied: self.season_stats_applied,
        })
    }
}

/// Orders the larger improvement first, then the higher score, then by name.
fn by_improvement(left: &Recommendation, right: &Recommendation) -> Ordering {
    right
        .improvement
        .cmp(&left.improvement)
        .then_with(|| right.score.cmp(&left.score))
        .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        .then_with(|| left.uid.cmp(&right.uid))
}

fn score_basis(lane: &TacticLane, mode: ScoreMode) -> ScoreBasis {
    if mode == ScoreMode::General {
        return ScoreBasis::Attributes;
    }
    if !lane.custom_metric_keys.is_empty() {
        return ScoreBasis::MoneyballCustom;
    }
    let has_definition = |role_id: &str, position: &str| {
        definition_for_attribute_role(role_id, base_position(position)).is_some()
    };
    if has_definition(&lane.ip_role_id, &lane.ip_position)
        && has_definition(&lane.oop_role_id, &lane.oop_position)
    {
        ScoreBasis::MoneyballRoles
    } else {
        ScoreBasis::Unavailable
    }
}

/// Whether one of the slot's recruitment filters ranks a statistic within the slot's group.
fn has_percentile_filter(lane: &TacticLane) -> bool {
    lane.recruitment_filters
        .iter()
        .any(|filter| filter.op.starts_with("pct_"))
}

fn snapshot_has_percentiles(conn: &Connection, snapshot_id: i64) -> Result<bool, String> {
    conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM player_moneyball_stats
             WHERE snapshot_id = ?1 AND percentiles_json IS NOT NULL
         )",
        [snapshot_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

/// The lane and player of each assignment in the first string of the save's first Planner
/// team, read without creating the Planner's defaults.
fn first_string_assignments(conn: &Connection, save_id: i64) -> Result<Vec<(String, i64)>, String> {
    conn.prepare(
        "SELECT lane_id, player_uid
         FROM planner_assignments
         WHERE string_id = (
             SELECT id
             FROM planner_strings
             WHERE save_id = ?1
             ORDER BY CASE team WHEN 'senior' THEN 0 WHEN 'reserves' THEN 1 ELSE 2 END,
                      string_order
             LIMIT 1
         )
         ORDER BY id",
    )
    .map_err(|error| error.to_string())?
    .query_map([save_id], |row| Ok((row.get(0)?, row.get(1)?)))
    .map_err(|error| error.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| error.to_string())
}

/// SQL that is true for a `players` row neither at the managed club nor on loan from it.
fn outside_managed_club_sql() -> String {
    format!(
        "{} IS NOT 1 AND NOT (players.on_loan IS 1 AND {} IS 1)",
        managed_club_player_sql("players", "?2"),
        managed_club_sql("players.parent_club", "players.parent_club_uid", "?2")
    )
}

/// Only a player with season statistics has a Moneyball score to compare.
const HAS_PERCENTILES_SQL: &str = "moneyball.percentiles_json IS NOT NULL";

/// The kept snapshots of `sources`, latest first.
fn kept_snapshot_ids(sources: &[StatsSource]) -> Vec<i64> {
    sources[1..]
        .iter()
        .map(|source| source.snapshot_id)
        .collect()
}

/// The tactic's role score columns, two for each slot in lane order.
fn role_score_columns(tactic: &PlannerTactic) -> Result<Vec<String>, String> {
    let mut columns = Vec::with_capacity(tactic.lanes.len() * 2);
    for lane in &tactic.lanes {
        for role_id in [&lane.ip_role_id, &lane.oop_role_id] {
            columns.push(format!(
                "{PLAYER_METRICS_ALIAS}.{}",
                player_current_column(role_id)?
            ));
        }
    }
    Ok(columns)
}

/// SQL that is false for a player who cannot improve any slot in General mode, so the many
/// realistic signings who cannot are never read or scored.
///
/// A slot's score never exceeds the higher of its two role scores: the blend lies between
/// them, and the fit adjustment and the age penalty only lower it. A player therefore improves
/// a slot only when one of his role scores there exceeds the incumbent's score, or zero for an
/// empty slot.
fn could_improve_sql(role_score_columns: &[String], incumbent_scores: &[Option<u8>]) -> String {
    let better = role_score_columns
        .chunks(2)
        .zip(incumbent_scores)
        .flat_map(|(roles, incumbent)| {
            let incumbent = incumbent.unwrap_or(0);
            roles
                .iter()
                .map(move |role| format!("{role} > {incumbent}"))
        })
        .collect::<Vec<_>>();
    format!("({})", better.join(" OR "))
}

/// Whether the club has any realistic signing to compare, whatever the slots' incumbents.
fn has_realistic_signings(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    mode: ScoreMode,
    sources: &[StatsSource],
) -> Result<bool, String> {
    let (join, comparable) = match mode {
        ScoreMode::General => (String::new(), "1"),
        ScoreMode::Moneyball => (
            stats_join_sql("players", &kept_snapshot_ids(sources)),
            HAS_PERCENTILES_SQL,
        ),
    };
    conn.query_row(
        &format!(
            "SELECT EXISTS(
                 SELECT 1 FROM players{join}
                 WHERE players.snapshot_id = ?1 AND {comparable} AND {} AND {}
             )",
            realistic_signing::clause(true),
            outside_managed_club_sql()
        ),
        [snapshot_id, save_id],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

/// Reads the scope's players with their score in every tactic slot. Every statement binds
/// the snapshot as `?1`, where the realistic-signing rule reads it, and the save as `?2`.
/// With `season_stats`, a candidate needs `MIN_SEASON_MINUTES` minutes and the slots'
/// season-stat filters run; without it they are left out. Each player's statistics come from
/// one of `sources`, whose rankings Moneyball scores and percentile filters read.
#[allow(clippy::too_many_arguments)]
fn load_players(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    scope: PlayerScope<'_>,
    tactic: &PlannerTactic,
    mode: ScoreMode,
    season_stats: bool,
    sources: &[StatsSource],
) -> Result<Vec<ScoredPlayer>, String> {
    let kept_snapshot_ids = kept_snapshot_ids(sources);
    let stats_join = stats_join_sql("players", &kept_snapshot_ids);
    let role_score_columns = match mode {
        ScoreMode::General => role_score_columns(tactic)?,
        ScoreMode::Moneyball => Vec::new(),
    };
    let (score_columns, mut from_sql) = match mode {
        ScoreMode::General => (
            role_score_columns.join(", "),
            format!("players{}", player_metrics_join("players", true, false)),
        ),
        ScoreMode::Moneyball => (
            format!("moneyball.minutes, {STATS_DIVISION_SQL}, moneyball.snapshot_id"),
            format!("players{stats_join}"),
        ),
    };
    let mut bind_values = vec![Value::Integer(snapshot_id), Value::Integer(save_id)];
    let scope_sql = match scope {
        PlayerScope::Club => managed_club_player_sql("players", "?2"),
        PlayerScope::Shortlist(shortlist_id) => {
            bind_values.push(Value::Integer(shortlist_id));
            format!(
                "players.uid IN (
                     SELECT entry.player_uid
                     FROM player_shortlist_entries entry
                     INNER JOIN player_shortlists shortlist ON shortlist.id = entry.shortlist_id
                     WHERE shortlist.save_id = ?2 AND shortlist.id = ?3
                 )
                 AND {}",
                outside_managed_club_sql()
            )
        }
        PlayerScope::RealisticSignings(incumbent_scores) => {
            // Most players are realistic signings for most clubs, so each mode reads its
            // scores first and a player's row only once those scores can be compared. SQLite
            // keeps a CROSS JOIN's order.
            let comparable = match mode {
                ScoreMode::General => {
                    from_sql = format!(
                        "player_role_metrics {PLAYER_METRICS_ALIAS}
                         CROSS JOIN players
                           ON players.snapshot_id = {PLAYER_METRICS_ALIAS}.snapshot_id
                          AND players.uid = {PLAYER_METRICS_ALIAS}.uid"
                    );
                    format!(
                        "{PLAYER_METRICS_ALIAS}.snapshot_id = ?1
                         AND {PLAYER_METRICS_ALIAS}.score_model_version = {SCORE_MODEL_VERSION}
                         AND {}",
                        could_improve_sql(&role_score_columns, incumbent_scores)
                    )
                }
                ScoreMode::Moneyball if kept_snapshot_ids.is_empty() => {
                    from_sql = "player_moneyball_stats moneyball
                         CROSS JOIN players
                           ON players.snapshot_id = moneyball.snapshot_id
                          AND players.uid = moneyball.player_uid"
                        .to_string();
                    format!("moneyball.snapshot_id = ?1 AND {HAS_PERCENTILES_SQL}")
                }
                // A player without statistics this season can borrow a kept snapshot's, so
                // the scan starts from everyone with statistics in either.
                ScoreMode::Moneyball => {
                    let kept = kept_snapshot_ids
                        .iter()
                        .map(i64::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    from_sql = format!(
                        "(SELECT player_uid FROM player_moneyball_stats WHERE snapshot_id = ?1
                          UNION
                          SELECT player_uid FROM player_moneyball_stats
                          WHERE snapshot_id IN ({kept}) AND minutes >= {MIN_SEASON_MINUTES}
                         ) stats_players
                         CROSS JOIN players
                           ON players.snapshot_id = ?1
                          AND players.uid = stats_players.player_uid{stats_join}"
                    );
                    HAS_PERCENTILES_SQL.to_string()
                }
            };
            format!(
                "{comparable} AND {} AND {}",
                realistic_signing::clause(true),
                outside_managed_club_sql()
            )
        }
    };
    let candidates = !matches!(scope, PlayerScope::Club);
    let filters = if candidates {
        SlotFilters::load(tactic, bind_values.len() + 1, season_stats)?
    } else {
        None
    };
    let applies_minimum = candidates && season_stats;
    let lane_count = tactic.lanes.len();
    let first_filter_column = 8 + match mode {
        ScoreMode::General => lane_count * 2,
        ScoreMode::Moneyball => 3,
    };
    let mut extra_columns = Vec::new();
    if let Some(filters) = &filters {
        let (columns, params) = filters.columns();
        extra_columns.extend(columns);
        bind_values.extend(params);
    }
    // General mode reads season stats only for the minimum minutes or a filter.
    let general_reads_stats = mode == ScoreMode::General
        && (applies_minimum || filters.as_ref().is_some_and(SlotFilters::uses_season_stats));
    if general_reads_stats {
        from_sql.push_str(&stats_join);
    }
    if mode == ScoreMode::Moneyball && filters.as_ref().is_some_and(SlotFilters::uses_role_scores) {
        from_sql.push_str(&player_metrics_join("players", true, false));
    }
    // Where the statistics row's other values are read: Moneyball mode's score columns hold
    // them all, and General mode adds only the ones it needs.
    let mut extra_column = |needed: bool, column: &str| {
        needed.then(|| {
            extra_columns.push(column.to_string());
            first_filter_column + extra_columns.len() - 1
        })
    };
    let (minutes_column, source_column) = match mode {
        ScoreMode::General => (
            extra_column(applies_minimum, "moneyball.minutes"),
            extra_column(general_reads_stats, "moneyball.snapshot_id"),
        ),
        ScoreMode::Moneyball => (Some(8), Some(10)),
    };
    // The statistics the slots' Moneyball scores and percentile rules read, after every other
    // column. A key outside the catalog has no column and ranks nobody.
    let mut statistic_keys = filters
        .iter()
        .flat_map(SlotFilters::ranked_statistics)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if mode == ScoreMode::Moneyball {
        statistic_keys.extend(tactic.lanes.iter().flat_map(lane_statistic_keys));
    }
    let statistics = StatisticColumns::of_keys(
        statistic_keys
            .iter()
            .map(String::as_str)
            .filter(|key| statistic_index(key).is_some()),
    )?;
    let first_statistic_column = first_filter_column + extra_columns.len();
    let filter_columns = extra_columns
        .iter()
        .map(|column| format!(", {column}"))
        .chain([statistics.appended_sql("moneyball")])
        .collect::<String>();
    // The club's order settles ties when its lineup is allocated. Candidates are ranked by
    // `by_improvement`, a total order, so sorting the many realistic signings is left out.
    let order_sql = match scope {
        PlayerScope::Club | PlayerScope::Shortlist(_) => "ORDER BY players.uid",
        PlayerScope::RealisticSignings(_) => "",
    };
    let sql = format!(
        "SELECT players.uid, players.name, players.age, players.current_club,
                players.current_club_uid, players.market_value_gbp, players.preferred_foot,
                players.positions_json, {score_columns}{filter_columns}
         FROM {from_sql}
         WHERE players.snapshot_id = ?1 AND {scope_sql}
         {order_sql}"
    );
    let score_bases = tactic
        .lanes
        .iter()
        .map(|lane| score_basis(lane, mode))
        .collect::<Vec<_>>();
    let optional = |row: &Row<'_>, column: Option<usize>| {
        column.map_or(Ok(None), |column| row.get::<_, Option<i64>>(column))
    };
    let read = |row: &Row<'_>| {
        let source_id = optional(row, source_column)?;
        Ok(CandidateRow {
            uid: row.get(0)?,
            name: row.get(1)?,
            age: row.get(2)?,
            club: row.get(3)?,
            club_uid: row.get(4)?,
            market_value_gbp: row.get(5)?,
            foot: row.get(6)?,
            positions_json: row.get(7)?,
            role_scores: match mode {
                ScoreMode::General => (8..first_filter_column)
                    .map(|column| row.get(column))
                    .collect::<rusqlite::Result<_>>()?,
                ScoreMode::Moneyball => Vec::new(),
            },
            minutes: optional(row, minutes_column)?,
            statistics: source_id
                .filter(|_| statistics.len() > 0)
                .map(|_| statistics.read(row, first_statistic_column))
                .transpose()?,
            division: match mode {
                ScoreMode::General => None,
                ScoreMode::Moneyball => row.get(9)?,
            },
            source_id,
            sql_passes: filters
                .as_ref()
                .map(|filters| filters.sql_passes(row, first_filter_column))
                .transpose()?,
        })
    };
    let score = |row: CandidateRow| {
        let source = row
            .source_id
            .and_then(|id| sources.iter().find(|source| source.snapshot_id == id))
            .unwrap_or(&sources[0]);
        let positions = serde_json::from_str::<BTreeMap<String, Option<i64>>>(&row.positions_json)
            .map_err(|error| error.to_string())?;
        let statistics = row.statistics;
        let lane_scores = match mode {
            ScoreMode::General => tactic
                .lanes
                .iter()
                .zip(row.role_scores.chunks(2))
                .map(|(lane, roles)| {
                    tactic_adjusted_score(
                        roles[0],
                        roles[1],
                        lane.ip_weight,
                        &row.foot,
                        &positions,
                        lane,
                    )
                })
                .collect(),
            ScoreMode::Moneyball => {
                let league_shift = source.league.shift(row.division.as_deref());
                tactic
                    .lanes
                    .iter()
                    .zip(&score_bases)
                    .map(|(lane, score_basis)| {
                        let statistics = statistics.as_deref()?;
                        let rankings = source.rankings.as_ref()?;
                        let percentiles =
                            |group, key: &str| rankings.percentile(group, key, statistics);
                        match score_basis {
                            ScoreBasis::MoneyballCustom => custom_metrics_lane_score(
                                &percentiles,
                                &row.foot,
                                &positions,
                                lane,
                                league_shift,
                            ),
                            ScoreBasis::MoneyballRoles => moneyball_roles_lane_score(
                                &percentiles,
                                &row.foot,
                                &positions,
                                lane,
                                league_shift,
                            ),
                            ScoreBasis::Attributes | ScoreBasis::Unavailable => None,
                        }
                    })
                    .collect()
            }
        };
        let under_minimum_minutes =
            applies_minimum && row.minutes.unwrap_or(0) < MIN_SEASON_MINUTES;
        let lane_allowed = match (&filters, &row.sql_passes) {
            _ if under_minimum_minutes => vec![false; lane_count],
            (Some(filters), Some(sql_passes)) => {
                filters.lane_passes(sql_passes, statistics.as_deref(), source.rankings.as_ref())
            }
            _ => vec![true; lane_count],
        };
        Ok::<_, String>(ScoredPlayer {
            uid: row.uid,
            name: row.name,
            age: row.age,
            club: row.club,
            club_uid: row.club_uid,
            market_value_gbp: row.market_value_gbp,
            minutes: match mode {
                ScoreMode::General => None,
                ScoreMode::Moneyball => row.minutes,
            },
            borrowed_stats_date: match mode {
                ScoreMode::General => None,
                ScoreMode::Moneyball => source.kept_date.clone(),
            },
            lane_scores,
            age_penalty: if candidates {
                age_penalty(mode, row.age)
            } else {
                0
            },
            lane_allowed,
            under_minimum_minutes,
        })
    };
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let mut rows = statement
        .query(params_from_iter(bind_values.iter()))
        .map_err(|error| error.to_string())?;
    // The many realistic signings are scored on worker threads while SQLite reads on, which
    // returns them in no particular order. The other scopes are few and keep the query's.
    if matches!(scope, PlayerScope::RealisticSignings(_)) {
        return scan_in_parallel(&mut rows, read, score);
    }
    let mut players = Vec::new();
    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        players.push(score(read(row).map_err(|error| error.to_string())?)?);
    }
    Ok(players)
}

struct Lineup {
    /// Each slot's incumbent as an index into the club's players, and where it came from.
    incumbents: Vec<Option<(usize, IncumbentSource)>>,
    unscored_planner_players: Vec<Option<String>>,
}

/// The club's starting XI. A Planner first-string player keeps their slot while the mode
/// scores them there; the open slots are then allocated from the remaining club players.
fn starting_lineup(
    tactic: &PlannerTactic,
    club: &[ScoredPlayer],
    first_string: &[(String, i64)],
) -> Lineup {
    let lane_count = tactic.lanes.len();
    let mut incumbents = vec![None; lane_count];
    let mut unscored_planner_players = vec![None; lane_count];
    let mut reserved_uids = HashSet::new();
    for (lane_id, player_uid) in first_string {
        let lane_index = tactic
            .lanes
            .iter()
            .position(|lane| &lane.lane_id == lane_id);
        let player_index = club.iter().position(|player| player.uid == *player_uid);
        let (Some(lane_index), Some(player_index)) = (lane_index, player_index) else {
            continue;
        };
        if club[player_index].lane_scores[lane_index].is_some() {
            incumbents[lane_index] = Some((player_index, IncumbentSource::Planner));
            reserved_uids.insert(*player_uid);
        } else {
            unscored_planner_players[lane_index] = Some(club[player_index].name.clone());
        }
    }

    let open_lanes = (0..lane_count)
        .filter(|&lane_index| incumbents[lane_index].is_none())
        .collect::<Vec<_>>();
    let candidates = club
        .iter()
        .map(|player| OptimizerCandidate {
            player_uid: player.uid,
            last_known_name: player.name.clone(),
            lane_scores: player
                .lane_scores
                .iter()
                .map(|score| score.map(u16::from))
                .collect(),
        })
        .collect::<Vec<_>>();
    for (lane_index, player_index) in allocate_lanes(&open_lanes, &candidates, &reserved_uids) {
        incumbents[lane_index] = Some((player_index, IncumbentSource::BestFit));
    }

    Lineup {
        incumbents,
        unscored_planner_players,
    }
}

fn lineup_player(player: &ScoredPlayer, lane_index: usize) -> Option<LineupPlayer> {
    Some(LineupPlayer {
        uid: player.uid,
        name: player.name.clone(),
        score: player.lane_scores[lane_index]?,
    })
}

fn cover(club: &[ScoredPlayer], lineup: &Lineup, lane_index: usize) -> Option<LineupPlayer> {
    club.iter()
        .enumerate()
        .filter(|(player_index, _)| {
            !lineup
                .incumbents
                .iter()
                .flatten()
                .any(|(incumbent_index, _)| incumbent_index == player_index)
        })
        .filter_map(|(_, player)| lineup_player(player, lane_index))
        .max_by_key(|player| (player.score, Reverse(player.uid)))
}

/// The points a candidate of `age` loses. Each year over `FULL_SCORE_AGE` costs one more than
/// the last, so 1 at 29, 3 at 30, 6 at 31, 10 at 32, and 15 at 33. General mode takes half of
/// that, rounded up, because its improvements are about a third as large: on one FM27 save the
/// largest was 13 points there and 37 in Moneyball mode. An older player thus ranks level with
/// a younger one only when he improves the slot by that much more, and a large enough
/// improvement still comes through. An unknown age loses nothing. These weights are judgment
/// calls, not Football Manager's numbers.
pub(crate) fn age_penalty(mode: ScoreMode, age: Option<i64>) -> u8 {
    // Fourteen years over already costs more than the highest Moneyball score.
    let years_over = age.map_or(0, |age| age.saturating_sub(FULL_SCORE_AGE).clamp(0, 14));
    let penalty = years_over * (years_over + 1) / 2;
    match mode {
        ScoreMode::General => (penalty + 1) / 2,
        ScoreMode::Moneyball => penalty,
    }
    .min(100) as u8
}

/// The candidate's score in the slot, less his age penalty, and how far it exceeds the
/// incumbent's, or the whole score for an empty slot. `None` when it does not exceed it.
fn improvement(
    player: &ScoredPlayer,
    incumbent_scores: &[Option<u8>],
    lane_index: usize,
) -> Option<(u8, u8)> {
    let score = player.lane_scores[lane_index]?.saturating_sub(player.age_penalty);
    let improvement = score.checked_sub(incumbent_scores[lane_index].unwrap_or(0))?;
    (improvement > 0).then_some((score, improvement))
}

/// The player's best improving slot whose recruitment filters they pass, or `None` when there
/// is none among the considered slots.
fn recommend(
    player: &ScoredPlayer,
    tactic: &PlannerTactic,
    incumbent_scores: &[Option<u8>],
    lane_filter: Option<usize>,
) -> Option<Recommendation> {
    let mut fits = (0..player.lane_scores.len())
        .filter(|&lane_index| lane_filter.map_or(true, |filter| filter == lane_index))
        .filter(|&lane_index| player.lane_allowed[lane_index])
        .filter_map(|lane_index| {
            let (score, improvement) = improvement(player, incumbent_scores, lane_index)?;
            Some((lane_index, score, improvement))
        })
        .collect::<Vec<_>>();
    // The stable sort keeps lane order between equal fits.
    fits.sort_by_key(|&(_, score, improvement)| Reverse((improvement, score)));
    let &(lane_index, score, improvement) = fits.first()?;
    Some(Recommendation {
        uid: player.uid,
        name: player.name.clone(),
        age: player.age,
        club: player.club.clone(),
        club_uid: player.club_uid,
        market_value_gbp: player.market_value_gbp,
        minutes: player.minutes,
        borrowed_stats_date: player.borrowed_stats_date.clone(),
        lane_id: tactic.lanes[lane_index].lane_id.clone(),
        score,
        improvement,
        age_penalty: player.age_penalty,
        other_lanes: fits
            .iter()
            .skip(1)
            .take(MAX_OTHER_LANES)
            .map(|&(lane_index, _, improvement)| OtherLane {
                lane_id: tactic.lanes[lane_index].lane_id.clone(),
                improvement,
            })
            .collect(),
    })
}
