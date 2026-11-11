//! Recommended signings: the managed club's starting XI per tactic slot, and the shortlisted
//! players who would improve it.

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, HashSet};

use rusqlite::{params_from_iter, types::Value, Connection};

use crate::features::moneyball::role_catalog::definition_for_attribute_role;
use crate::features::planner::fit::{
    custom_metrics_lane_score, moneyball_roles_lane_score, tactic_adjusted_score,
};
use crate::features::planner::optimizer::{allocate_lanes, OptimizerCandidate};
use crate::features::planner::tactic::{
    base_position, default_tactic, read_tactic, PlannerTactic, TacticLane,
};
use crate::features::player_metrics::compact::{
    assert_read_models_complete, player_current_column, player_metrics_join, PLAYER_METRICS_ALIAS,
};

use super::{club_snapshot_id, MANAGED_CLUB_NAME_SQL};

/// The most ranked recommendations one read returns: as many as the dashboard shows at once.
pub(crate) const MAX_RECOMMENDATIONS: usize = 12;
const MAX_OTHER_LANES: usize = 2;

/// Which scores compare a shortlisted player with the club's player in a slot.
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
    /// The shortlisted player who would improve the slot most. Slots answer independently, so
    /// one player can be the best signing for several.
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
    pub(crate) lane_id: String,
    pub(crate) score: u8,
    /// Points above the slot's incumbent, or the whole score when the slot has none.
    pub(crate) improvement: u8,
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
    /// Shortlisted players of the current snapshot outside the managed club.
    pub(crate) shortlisted_players: usize,
    pub(crate) total_recommendations: usize,
    pub(crate) recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SigningsRequest<'a> {
    pub(crate) mode: ScoreMode,
    /// One of the save's shortlists, or every shortlist when absent.
    pub(crate) shortlist_id: Option<i64>,
    /// Restricts the ranked recommendations to one slot. Each slot's best signing is unaffected.
    pub(crate) lane_id: Option<&'a str>,
}

struct ScoredPlayer {
    uid: i64,
    name: String,
    age: Option<i64>,
    club: Option<String>,
    club_uid: Option<i64>,
    market_value_gbp: Option<i64>,
    minutes: Option<i64>,
    /// The player's score in each tactic slot, in lane order.
    lane_scores: Vec<Option<u8>>,
}

#[derive(Clone, Copy)]
enum PlayerScope {
    Club,
    /// Players on the given shortlist, or on any of the save's, outside the managed club.
    Shortlisted(Option<i64>),
}

/// The save's starting XI and recommended signings, or `None` without a current snapshot or a
/// configured managed club. It only reads, so a save without stored lanes uses the default
/// tactic in memory.
pub(crate) fn get_signings(
    conn: &Connection,
    save_id: i64,
    request: SigningsRequest<'_>,
) -> Result<Option<Signings>, String> {
    let Some(snapshot_id) = club_snapshot_id(conn, save_id)? else {
        return Ok(None);
    };
    let tactic = read_tactic(conn, save_id)?;
    let lane_filter = request
        .lane_id
        .map(|lane_id| {
            tactic
                .lanes
                .iter()
                .position(|lane| lane.lane_id == lane_id)
                .ok_or_else(|| format!("unknown tactic lane: {lane_id}"))
        })
        .transpose()?;
    let mode = request.mode;
    let scores_available = match mode {
        ScoreMode::General => {
            assert_read_models_complete(conn, snapshot_id, true, false)?;
            true
        }
        ScoreMode::Moneyball => snapshot_has_percentiles(conn, snapshot_id)?,
    };

    let club = load_players(conn, save_id, snapshot_id, PlayerScope::Club, &tactic, mode)?;
    let lineup = starting_lineup(&tactic, &club, &first_string_assignments(conn, save_id)?);
    let shortlisted = load_players(
        conn,
        save_id,
        snapshot_id,
        PlayerScope::Shortlisted(request.shortlist_id),
        &tactic,
        mode,
    )?;

    let incumbent_scores = lineup
        .incumbents
        .iter()
        .enumerate()
        .map(|(lane_index, incumbent)| {
            incumbent.and_then(|(player_index, _)| club[player_index].lane_scores[lane_index])
        })
        .collect::<Vec<_>>();
    let mut recommendations = shortlisted
        .iter()
        .filter_map(|player| recommend(player, &tactic, &incumbent_scores, lane_filter))
        .collect::<Vec<_>>();
    recommendations.sort_by(by_improvement);
    let total_recommendations = recommendations.len();
    recommendations.truncate(MAX_RECOMMENDATIONS);

    let slots = tactic
        .lanes
        .iter()
        .enumerate()
        .map(|(lane_index, lane)| LineupSlot {
            lane: lane.clone(),
            score_basis: score_basis(lane, mode),
            incumbent: lineup.incumbents[lane_index].and_then(|(player_index, source)| {
                lineup_player(&club[player_index], lane_index).map(|player| (player, source))
            }),
            cover: cover(&club, &lineup, lane_index),
            unscored_planner_player: lineup.unscored_planner_players[lane_index].clone(),
            best_signing: shortlisted
                .iter()
                .filter_map(|player| {
                    recommend(player, &tactic, &incumbent_scores, Some(lane_index))
                })
                .min_by(by_improvement),
        })
        .collect();

    Ok(Some(Signings {
        tactic_is_default: tactic == default_tactic(),
        scores_available,
        slots,
        shortlisted_players: shortlisted.len(),
        total_recommendations,
        recommendations,
    }))
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

fn load_players(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    scope: PlayerScope,
    tactic: &PlannerTactic,
    mode: ScoreMode,
) -> Result<Vec<ScoredPlayer>, String> {
    let (score_columns, score_join) = match mode {
        ScoreMode::General => {
            let mut columns = String::new();
            for lane in &tactic.lanes {
                for role_id in [&lane.ip_role_id, &lane.oop_role_id] {
                    columns.push_str(&format!(
                        ", {PLAYER_METRICS_ALIAS}.{}",
                        player_current_column(role_id)?
                    ));
                }
            }
            (columns, player_metrics_join("p", true, false))
        }
        ScoreMode::Moneyball => (
            ", moneyball.minutes, moneyball.percentiles_json".to_string(),
            " LEFT JOIN player_moneyball_stats moneyball
                ON moneyball.snapshot_id = p.snapshot_id AND moneyball.player_uid = p.uid"
                .to_string(),
        ),
    };
    let mut bind_values = vec![Value::Integer(snapshot_id), Value::Integer(save_id)];
    let scope_sql = match scope {
        PlayerScope::Club => format!("p.current_club = {MANAGED_CLUB_NAME_SQL}"),
        PlayerScope::Shortlisted(shortlist_id) => {
            bind_values.push(shortlist_id.map_or(Value::Null, Value::Integer));
            format!(
                "p.uid IN (
                     SELECT entry.player_uid
                     FROM player_shortlist_entries entry
                     INNER JOIN player_shortlists shortlist ON shortlist.id = entry.shortlist_id
                     WHERE shortlist.save_id = ?2 AND (?3 IS NULL OR shortlist.id = ?3)
                 )
                 AND p.current_club IS NOT {MANAGED_CLUB_NAME_SQL}
                 AND NOT (p.on_loan IS 1 AND p.parent_club IS {MANAGED_CLUB_NAME_SQL})"
            )
        }
    };
    let sql = format!(
        "SELECT p.uid, p.name, p.age, p.current_club, p.current_club_uid, p.market_value_gbp,
                p.preferred_foot, p.positions_json{score_columns}
         FROM players p{score_join}
         WHERE p.snapshot_id = ?1 AND {scope_sql}
         ORDER BY p.uid"
    );
    let lane_count = tactic.lanes.len();
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let mut rows = statement
        .query(params_from_iter(bind_values.iter()))
        .map_err(|error| error.to_string())?;
    let mut players = Vec::new();
    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let read = |error: rusqlite::Error| error.to_string();
        let foot: String = row.get(6).map_err(read)?;
        let positions = serde_json::from_str::<BTreeMap<String, Option<i64>>>(
            &row.get::<_, String>(7).map_err(read)?,
        )
        .map_err(|error| error.to_string())?;
        let (minutes, lane_scores) = match mode {
            ScoreMode::General => {
                let mut scores = Vec::with_capacity(lane_count);
                for (lane_index, lane) in tactic.lanes.iter().enumerate() {
                    scores.push(tactic_adjusted_score(
                        row.get(8 + lane_index * 2).map_err(read)?,
                        row.get(9 + lane_index * 2).map_err(read)?,
                        lane.ip_weight,
                        &foot,
                        &positions,
                        lane,
                    ));
                }
                (None, scores)
            }
            ScoreMode::Moneyball => {
                let percentiles = row
                    .get::<_, Option<String>>(9)
                    .map_err(read)?
                    .map(|json| serde_json::from_str::<BTreeMap<String, Option<u8>>>(&json))
                    .transpose()
                    .map_err(|error| error.to_string())?;
                let scores = tactic
                    .lanes
                    .iter()
                    .map(|lane| {
                        let percentiles = percentiles.as_ref()?;
                        match score_basis(lane, mode) {
                            ScoreBasis::MoneyballCustom => {
                                custom_metrics_lane_score(percentiles, &foot, &positions, lane)
                            }
                            ScoreBasis::MoneyballRoles => {
                                moneyball_roles_lane_score(percentiles, &foot, &positions, lane)
                            }
                            ScoreBasis::Attributes | ScoreBasis::Unavailable => None,
                        }
                    })
                    .collect();
                (row.get(8).map_err(read)?, scores)
            }
        };
        players.push(ScoredPlayer {
            uid: row.get(0).map_err(read)?,
            name: row.get(1).map_err(read)?,
            age: row.get(2).map_err(read)?,
            club: row.get(3).map_err(read)?,
            club_uid: row.get(4).map_err(read)?,
            market_value_gbp: row.get(5).map_err(read)?,
            minutes,
            lane_scores,
        });
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
            lane_scores: player.lane_scores.clone(),
        })
        .collect::<Vec<_>>();
    for (lane_index, player_index) in
        allocate_lanes(tactic, &open_lanes, &candidates, &reserved_uids)
    {
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

/// The player's best improving slot, or `None` when they improve none of the considered slots.
fn recommend(
    player: &ScoredPlayer,
    tactic: &PlannerTactic,
    incumbent_scores: &[Option<u8>],
    lane_filter: Option<usize>,
) -> Option<Recommendation> {
    let mut fits = player
        .lane_scores
        .iter()
        .enumerate()
        .filter(|(lane_index, _)| lane_filter.map_or(true, |filter| filter == *lane_index))
        .filter_map(|(lane_index, score)| {
            let score = (*score)?;
            let improvement = score.checked_sub(incumbent_scores[lane_index].unwrap_or(0))?;
            (improvement > 0).then_some((lane_index, score, improvement))
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
        lane_id: tactic.lanes[lane_index].lane_id.clone(),
        score,
        improvement,
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
