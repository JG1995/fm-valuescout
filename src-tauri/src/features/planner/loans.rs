//! Which players outside the senior squad would gain most from a loan.
//!
//! A loan helps a player who still has growing to do, is close enough to the senior squad to
//! need senior football, and would play little at the club. Every weight here is a judgment
//! call, not a number from Football Manager. Tune them here.

use std::collections::HashMap;

use rusqlite::{params, Connection};

use super::depth::{ensure_depth, preflight_depth_snapshot, PlannerTeam};
use super::optimizer::load_club_players;
use super::outlook::expected_score;

/// The ages a development loan suits: old enough for senior football, young enough to grow.
const LOAN_AGES: std::ops::RangeInclusive<i64> = 18..=23;

/// The expected growth, in score points, that makes the full case for a loan.
const FULL_GROWTH: f64 = 12.0;

/// How far behind the weakest senior player in his slot a player can be, in score points,
/// before senior football elsewhere stops mattering more than the club's own teams.
const FAR_GAP: f64 = 20.0;

/// The benefit from which the Planner marks a player as a loan candidate.
const RECOMMENDED_BENEFIT: u8 = 50;

/// Where the Planner holds a player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LoanCandidatePlace {
    pub(super) team: PlannerTeam,
    pub(super) string_order: i64,
    pub(super) string_name: String,
}

/// A player old enough and young enough for a development loan, outside the senior squad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LoanCandidate {
    pub(super) player_uid: i64,
    pub(super) name: String,
    pub(super) age: i64,
    /// The string that holds him, or `None` when no squad does.
    pub(super) place: Option<LoanCandidatePlace>,
    /// The slot he is judged in: the one that holds him, or the one he fits best now.
    pub(super) lane_id: String,
    /// His fit score in that slot now, and the one he is expected to reach.
    pub(super) current_score: u8,
    pub(super) expected_score: u8,
    /// How much a loan would help him, from 0 to 100.
    pub(super) benefit: u8,
    pub(super) recommended: bool,
}

/// How much a loan would help a player, from 0 to 100.
///
/// `growth` is the score points he is expected to gain, without which a loan develops
/// nothing. The rest weighs two things equally: how close he is to the senior squad
/// (`senior_gap`, the points he trails its weakest player in his slot by, or `None` when the
/// squad has nobody there), and how little he would play at the club (`string_order`, the
/// string that holds him, or `None` when no squad does).
pub(super) fn loan_benefit(growth: f64, senior_gap: Option<f64>, string_order: Option<i64>) -> u8 {
    let growth = (growth / FULL_GROWTH).clamp(0.0, 1.0);
    let closeness = senior_gap.map_or(1.0, |gap| 1.0 - (gap / FAR_GAP).clamp(0.0, 1.0));
    let blocked = match string_order {
        Some(0) => 0.5,
        Some(1) => 0.75,
        _ => 1.0,
    };
    (100.0 * growth * (0.5 * closeness + 0.5 * blocked)).round() as u8
}

/// The managed club's possible loan players, most to gain first.
pub(super) fn loan_candidates(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<LoanCandidate>, String> {
    let snapshot_id = preflight_depth_snapshot(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    let tactic = ensure_depth(conn, save_id)?;
    let players = load_club_players(conn, save_id, snapshot_id, &tactic)?;
    let places = load_places(conn, save_id)?;
    let lane_index = |lane_id: &str| tactic.lanes.iter().position(|lane| lane.lane_id == lane_id);

    // The weakest senior player in each slot is the one a candidate would have to displace.
    let mut senior_floors = vec![None::<u8>; tactic.lanes.len()];
    for player in &players {
        let Some((place, lane_id)) = places.get(&player.uid) else {
            continue;
        };
        if place.team != PlannerTeam::Senior {
            continue;
        }
        let Some(lane_index) = lane_index(lane_id) else {
            continue;
        };
        if let Some((current, _)) = player.lane_scores[lane_index] {
            let floor = &mut senior_floors[lane_index];
            *floor = Some(floor.map_or(current, |floor| floor.min(current)));
        }
    }

    let mut candidates = players
        .iter()
        .filter_map(|player| {
            let age = player
                .development
                .age
                .filter(|age| LOAN_AGES.contains(age))?;
            let place = places.get(&player.uid);
            if place.is_some_and(|(place, _)| place.team == PlannerTeam::Senior) {
                return None;
            }
            let lane_index = place
                .and_then(|(_, lane_id)| lane_index(lane_id))
                .filter(|&lane_index| player.lane_scores[lane_index].is_some())
                .or_else(|| player.best_lane())?;
            let (current, potential) = player.lane_scores[lane_index]?;
            let expected = expected_score(current, potential, &player.development);
            let benefit = loan_benefit(
                expected - f64::from(current),
                senior_floors[lane_index].map(|floor| f64::from(floor) - f64::from(current)),
                place.map(|(place, _)| place.string_order),
            );
            Some(LoanCandidate {
                player_uid: player.uid,
                name: player.name.clone(),
                age,
                place: place.map(|(place, _)| place.clone()),
                lane_id: tactic.lanes[lane_index].lane_id.clone(),
                current_score: current,
                expected_score: expected.round() as u8,
                benefit,
                recommended: benefit >= RECOMMENDED_BENEFIT,
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .benefit
            .cmp(&left.benefit)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.player_uid.cmp(&right.player_uid))
    });
    Ok(candidates)
}

/// Each assigned player's string and lane.
fn load_places(
    conn: &Connection,
    save_id: i64,
) -> Result<HashMap<i64, (LoanCandidatePlace, String)>, String> {
    let mut statement = conn
        .prepare(
            "SELECT assignment.player_uid,
                    planner_string.team,
                    planner_string.string_order,
                    planner_string.display_name,
                    assignment.lane_id
             FROM planner_assignments assignment
             INNER JOIN planner_strings planner_string
               ON planner_string.id = assignment.string_id
             WHERE assignment.save_id = ?1",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![save_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    rows.into_iter()
        .map(|(player_uid, team, string_order, string_name, lane_id)| {
            Ok((
                player_uid,
                (
                    LoanCandidatePlace {
                        team: PlannerTeam::parse(&team)?,
                        string_order,
                        string_name,
                    },
                    lane_id,
                ),
            ))
        })
        .collect()
}
