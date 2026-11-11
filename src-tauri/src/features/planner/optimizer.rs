use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use rusqlite::{params, Connection};

use crate::features::{
    player_metrics::compact::{
        player_current_column, player_metrics_join, player_potential_column, PLAYER_METRICS_ALIAS,
    },
    scoring::combine::combine_role_scores,
};

use super::depth::{
    ensure_depth, insert_assignment, load_depth, preflight_depth_snapshot, AssignmentProvenance,
    PlannerDepth, PlannerTeam,
};
use super::fit::lane_fit_score;
use super::outlook::{expected_score, slot_value, Development};
use super::tactic::{base_position, PlannerTactic, TACTIC_LANE_COUNT};
use super::teams;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OptimizerCandidate {
    pub(crate) player_uid: i64,
    pub(crate) last_known_name: String,
    /// The player's value in each tactic lane, on any scale where higher is better. `None`
    /// where he cannot play the lane.
    pub(crate) lane_scores: Vec<Option<u16>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct MatchObjective {
    total_score: i64,
    filled_lanes: i64,
    uid_tie_break: [i64; TACTIC_LANE_COUNT],
}

impl MatchObjective {
    fn add(self, other: Self) -> Self {
        Self {
            total_score: self.total_score + other.total_score,
            filled_lanes: self.filled_lanes + other.filled_lanes,
            uid_tie_break: std::array::from_fn(|index| {
                self.uid_tie_break[index] + other.uid_tie_break[index]
            }),
        }
    }

    fn negated(self) -> Self {
        Self {
            total_score: -self.total_score,
            filled_lanes: -self.filled_lanes,
            uid_tie_break: std::array::from_fn(|index| -self.uid_tie_break[index]),
        }
    }
}

#[derive(Debug, Clone)]
struct MatchEdge {
    to: usize,
    reverse: usize,
    capacity: usize,
    objective: MatchObjective,
}

#[derive(Debug)]
struct MatchGraph {
    edges: Vec<Vec<MatchEdge>>,
}

impl MatchGraph {
    fn new(node_count: usize) -> Self {
        Self {
            edges: vec![Vec::new(); node_count],
        }
    }

    fn add_edge(&mut self, from: usize, to: usize, objective: MatchObjective) -> usize {
        let edge_index = self.edges[from].len();
        let reverse_index = self.edges[to].len();
        self.edges[from].push(MatchEdge {
            to,
            reverse: reverse_index,
            capacity: 1,
            objective,
        });
        self.edges[to].push(MatchEdge {
            to: from,
            reverse: edge_index,
            capacity: 0,
            objective: objective.negated(),
        });
        edge_index
    }

    fn send_max_flow(&mut self, source: usize, sink: usize, target_flow: usize) -> usize {
        let mut flow = 0;

        while flow < target_flow {
            let mut best = vec![None; self.edges.len()];
            let mut previous = vec![None; self.edges.len()];
            let mut queued = vec![false; self.edges.len()];
            let mut queue = VecDeque::from([source]);
            best[source] = Some(MatchObjective::default());
            queued[source] = true;

            while let Some(node) = queue.pop_front() {
                queued[node] = false;
                let Some(node_objective) = best[node] else {
                    continue;
                };

                for (edge_index, edge) in self.edges[node].iter().enumerate() {
                    if edge.capacity == 0 {
                        continue;
                    }
                    let objective = node_objective.add(edge.objective);
                    if best[edge.to].is_some_and(|existing| existing >= objective) {
                        continue;
                    }
                    best[edge.to] = Some(objective);
                    previous[edge.to] = Some((node, edge_index));
                    if !queued[edge.to] {
                        queue.push_back(edge.to);
                        queued[edge.to] = true;
                    }
                }
            }

            if best[sink].is_none() {
                break;
            }

            let mut node = sink;
            while node != source {
                let (from, edge_index) = previous[node].expect("path reaches source");
                let reverse = self.edges[from][edge_index].reverse;
                self.edges[from][edge_index].capacity -= 1;
                self.edges[node][reverse].capacity += 1;
                node = from;
            }
            flow += 1;
        }

        flow
    }
}

pub(super) fn match_lanes(
    lane_indices: &[usize],
    candidates: &[OptimizerCandidate],
) -> Vec<Option<usize>> {
    let lane_count = lane_indices.len();
    let source = 0;
    let lane_start = source + 1;
    let player_start = lane_start + lane_count;
    let blank_start = player_start + candidates.len();
    let sink = blank_start + lane_count;
    let mut graph = MatchGraph::new(sink + 1);
    let mut candidate_ranks = vec![0_i64; candidates.len()];
    let mut candidate_indices = (0..candidates.len()).collect::<Vec<_>>();
    candidate_indices.sort_by_key(|&index| candidates[index].player_uid);
    for (rank, candidate_index) in candidate_indices.into_iter().enumerate() {
        candidate_ranks[candidate_index] = rank as i64 + 1;
    }
    let blank_rank = candidates.len() as i64 + 1;
    let mut candidate_edges = vec![Vec::new(); lane_count];

    for (local_lane_index, &lane_index) in lane_indices.iter().enumerate() {
        let lane_node = lane_start + local_lane_index;
        graph.add_edge(source, lane_node, MatchObjective::default());
        graph.add_edge(
            lane_node,
            blank_start + local_lane_index,
            MatchObjective {
                uid_tie_break: std::array::from_fn(|index| {
                    i64::from(index == lane_index) * -blank_rank
                }),
                ..MatchObjective::default()
            },
        );
        graph.add_edge(
            blank_start + local_lane_index,
            sink,
            MatchObjective::default(),
        );

        for (candidate_index, candidate) in candidates.iter().enumerate() {
            let Some(score) = candidate.lane_scores.get(lane_index).copied().flatten() else {
                continue;
            };
            let edge_index = graph.add_edge(
                lane_node,
                player_start + candidate_index,
                MatchObjective {
                    total_score: i64::from(score),
                    filled_lanes: 1,
                    uid_tie_break: std::array::from_fn(|index| {
                        i64::from(index == lane_index) * -candidate_ranks[candidate_index]
                    }),
                },
            );
            candidate_edges[local_lane_index].push((candidate_index, edge_index));
        }
    }

    for candidate_index in 0..candidates.len() {
        graph.add_edge(
            player_start + candidate_index,
            sink,
            MatchObjective::default(),
        );
    }
    let flow = graph.send_max_flow(source, sink, lane_count);
    debug_assert_eq!(flow, lane_count);

    candidate_edges
        .into_iter()
        .enumerate()
        .map(|(local_lane_index, edges)| {
            let lane_node = lane_start + local_lane_index;
            edges.into_iter().find_map(|(candidate_index, edge_index)| {
                (graph.edges[lane_node][edge_index].capacity == 0).then_some(candidate_index)
            })
        })
        .collect()
}

/// Fills `lane_indices` from the candidates outside `reserved_uids`. Lanes with an importance
/// rank take their best remaining candidate in rank order; the other lanes then share an exact
/// maximum-score match. Returns each filled lane's index with its candidate's index, in the
/// order filled.
pub(crate) fn allocate_lanes(
    tactic: &PlannerTactic,
    lane_indices: &[usize],
    candidates: &[OptimizerCandidate],
    reserved_uids: &HashSet<i64>,
) -> Vec<(usize, usize)> {
    let mut taken_uids = reserved_uids.clone();
    let mut allocation = Vec::with_capacity(lane_indices.len());

    let mut ranked_lane_indices = lane_indices
        .iter()
        .copied()
        .filter(|&index| tactic.lanes[index].importance_rank.is_some())
        .collect::<Vec<_>>();
    ranked_lane_indices.sort_by_key(|&index| tactic.lanes[index].importance_rank);
    for lane_index in ranked_lane_indices {
        let best = candidates
            .iter()
            .enumerate()
            .filter(|(_, candidate)| !taken_uids.contains(&candidate.player_uid))
            .filter_map(|(candidate_index, candidate)| {
                candidate
                    .lane_scores
                    .get(lane_index)
                    .copied()
                    .flatten()
                    .map(|score| (candidate_index, candidate.player_uid, score))
            })
            .max_by_key(|&(_, player_uid, score)| (score, Reverse(player_uid)));
        let Some((candidate_index, player_uid, _)) = best else {
            continue;
        };
        allocation.push((lane_index, candidate_index));
        taken_uids.insert(player_uid);
    }

    let unranked_lane_indices = lane_indices
        .iter()
        .copied()
        .filter(|&index| tactic.lanes[index].importance_rank.is_none())
        .collect::<Vec<_>>();
    let available_indices = (0..candidates.len())
        .filter(|&index| !taken_uids.contains(&candidates[index].player_uid))
        .collect::<Vec<_>>();
    let available_candidates = available_indices
        .iter()
        .map(|&index| candidates[index].clone())
        .collect::<Vec<_>>();
    let matches = match_lanes(&unranked_lane_indices, &available_candidates);
    allocation.extend(
        unranked_lane_indices
            .into_iter()
            .zip(matches)
            .filter_map(|(lane_index, matched)| Some((lane_index, available_indices[matched?]))),
    );
    allocation
}

/// Refills every lane the user has not assigned by hand. Each team values a player by his
/// score now and where his development is heading ([`slot_value`]), and each player is used
/// once, in the strings' [`fill_order`].
pub(super) fn optimize_depth(conn: &Connection, save_id: i64) -> Result<PlannerDepth, String> {
    let snapshot_id = preflight_depth_snapshot(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    let tactic = ensure_depth(conn, save_id)?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let available_teams = teams::load_team_settings_from_tx(&tx, save_id)?;
    let manual_assignments = load_manual_assignments(&tx, save_id)?;
    let mut reserved_uids = manual_assignments
        .iter()
        .map(|assignment| assignment.player_uid)
        .collect::<HashSet<_>>();
    let mut manual_lanes = HashMap::<i64, HashSet<String>>::new();
    for assignment in manual_assignments {
        manual_lanes
            .entry(assignment.string_id)
            .or_default()
            .insert(assignment.lane_id);
    }

    tx.execute(
        "DELETE FROM planner_assignments WHERE save_id = ?1 AND provenance = 'optimizer'",
        params![save_id],
    )
    .map_err(|error| error.to_string())?;

    let players = load_club_players(&tx, save_id, snapshot_id, &tactic)?;
    let candidates_by_team = available_teams
        .iter()
        .map(|setting| (setting.team, team_candidates(setting, &tactic, &players)))
        .collect::<Vec<_>>();
    for planner_string in fill_order(load_ordered_strings(&tx, save_id)?) {
        let Some((_, candidates)) = candidates_by_team
            .iter()
            .find(|(team, _)| *team == planner_string.team)
        else {
            continue;
        };
        let occupied_lanes = manual_lanes.get(&planner_string.id);
        let lane_indices = tactic
            .lanes
            .iter()
            .enumerate()
            .filter_map(|(index, lane)| {
                (!occupied_lanes.is_some_and(|lanes| lanes.contains(&lane.lane_id)))
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        for (lane_index, candidate_index) in
            allocate_lanes(&tactic, &lane_indices, candidates, &reserved_uids)
        {
            let candidate = &candidates[candidate_index];
            insert_assignment(
                &tx,
                save_id,
                planner_string.id,
                &tactic.lanes[lane_index].lane_id,
                candidate.player_uid,
                &candidate.last_known_name,
                AssignmentProvenance::Optimizer,
            )?;
            reserved_uids.insert(candidate.player_uid);
        }
    }

    tx.commit().map_err(|error| error.to_string())?;
    load_depth(conn, save_id, Some(snapshot_id))
}

struct ManualAssignment {
    string_id: i64,
    lane_id: String,
    player_uid: i64,
}

struct OrderedPlannerString {
    id: i64,
    team: PlannerTeam,
}

fn load_manual_assignments(
    tx: &rusqlite::Transaction<'_>,
    save_id: i64,
) -> Result<Vec<ManualAssignment>, String> {
    let mut statement = tx
        .prepare(
            "SELECT string_id, lane_id, player_uid
             FROM planner_assignments
             WHERE save_id = ?1 AND provenance = 'manual'",
        )
        .map_err(|error| error.to_string())?;
    let assignments = statement
        .query_map(params![save_id], |row| {
            Ok(ManualAssignment {
                string_id: row.get(0)?,
                lane_id: row.get(1)?,
                player_uid: row.get(2)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(assignments)
}

/// The save's strings by team, then by their order within the team.
fn load_ordered_strings(
    tx: &rusqlite::Transaction<'_>,
    save_id: i64,
) -> Result<Vec<OrderedPlannerString>, String> {
    let mut statement = tx
        .prepare(
            "SELECT id, team
             FROM planner_strings
             WHERE save_id = ?1
             ORDER BY CASE team
                 WHEN 'senior' THEN 0
                 WHEN 'reserves' THEN 1
                 WHEN 'youth' THEN 2
             END, string_order",
        )
        .map_err(|error| error.to_string())?;
    let strings = statement
        .query_map(params![save_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    strings
        .into_iter()
        .map(|(id, team)| {
            Ok(OrderedPlannerString {
                id,
                team: PlannerTeam::parse(&team)?,
            })
        })
        .collect()
}

/// The order strings fill in, from strings sorted within each team. The senior first and
/// second strings come first: rotation gives both of them regular minutes. After them a
/// player starts for a lower team before he sits deeper in any squad, so each further rank
/// fills Reserves, then Youth, then the next senior string.
fn fill_order(strings: Vec<OrderedPlannerString>) -> Vec<OrderedPlannerString> {
    let (mut senior, mut reserves, mut youth) = (0, 0, 0);
    let mut keyed = strings
        .into_iter()
        .map(|planner_string| {
            let rank = match planner_string.team {
                PlannerTeam::Senior => &mut senior,
                PlannerTeam::Reserves => &mut reserves,
                PlannerTeam::Youth => &mut youth,
            };
            let key = fill_key(planner_string.team, *rank);
            *rank += 1;
            (key, planner_string)
        })
        .collect::<Vec<_>>();
    keyed.sort_by_key(|(key, _)| *key);
    keyed
        .into_iter()
        .map(|(_, planner_string)| planner_string)
        .collect()
}

/// A string's place in the fill order, from its 0-based rank within its team.
pub(super) fn fill_key(team: PlannerTeam, rank: usize) -> (usize, usize) {
    match team {
        PlannerTeam::Senior if rank < 2 => (0, rank),
        PlannerTeam::Senior => (rank, 2),
        PlannerTeam::Reserves => (rank + 1, 0),
        PlannerTeam::Youth => (rank + 1, 1),
    }
}

struct ClubPlayer {
    uid: i64,
    name: String,
    development: Development,
    /// Each tactic lane's current and potential fit scores, where the player suits the lane.
    lane_scores: Vec<Option<(u8, u8)>>,
}

/// A managed-club player whom no squad holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UnassignedPlayer {
    pub(super) player_uid: i64,
    pub(super) name: String,
    pub(super) age: Option<i64>,
    /// The lane he fits best now, or `None` when he suits no lane of the tactic.
    pub(super) best_lane: Option<UnassignedPlayerLane>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UnassignedPlayerLane {
    pub(super) lane_id: String,
    /// His fit score in the lane now, and the one he is expected to reach.
    pub(super) current_score: u8,
    pub(super) expected_score: u8,
}

/// The managed club's players in no squad, best current fit first: who is surplus to the plan.
pub(super) fn unassigned_players(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<UnassignedPlayer>, String> {
    let snapshot_id = preflight_depth_snapshot(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    let tactic = ensure_depth(conn, save_id)?;
    let assigned_uids = conn
        .prepare("SELECT player_uid FROM planner_assignments WHERE save_id = ?1")
        .map_err(|error| error.to_string())?
        .query_map(params![save_id], |row| row.get::<_, i64>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(|error| error.to_string())?;

    let mut players = load_club_players(conn, save_id, snapshot_id, &tactic)?
        .into_iter()
        .filter(|player| !assigned_uids.contains(&player.uid))
        .map(|player| {
            let best_lane = player
                .lane_scores
                .iter()
                .enumerate()
                .filter_map(|(lane_index, scores)| Some((lane_index, (*scores)?)))
                .max_by_key(|&(lane_index, (current, _))| (current, Reverse(lane_index)))
                .map(|(lane_index, (current, potential))| UnassignedPlayerLane {
                    lane_id: tactic.lanes[lane_index].lane_id.clone(),
                    current_score: current,
                    expected_score: expected_score(current, potential, &player.development).round()
                        as u8,
                });
            UnassignedPlayer {
                player_uid: player.uid,
                name: player.name,
                age: player.development.age,
                best_lane,
            }
        })
        .collect::<Vec<_>>();
    players.sort_by(|left, right| {
        let score =
            |player: &UnassignedPlayer| player.best_lane.as_ref().map(|lane| lane.current_score);
        score(right)
            .cmp(&score(left))
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.player_uid.cmp(&right.player_uid))
    });
    Ok(players)
}

/// The managed club's players with their fit scores for every lane of the tactic.
fn load_club_players(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    tactic: &PlannerTactic,
) -> Result<Vec<ClubPlayer>, String> {
    // Four identity columns, then the four development columns, then the lane scores.
    const DEVELOPMENT_COLUMN: usize = 4;
    const SCORE_COLUMN: usize = 8;
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
        "SELECT p.uid, p.name, p.preferred_foot, p.positions_json, {}{metric_select}
         FROM players p{}
         WHERE p.snapshot_id = ?1
           AND p.current_club = (
               SELECT club_name FROM managed_club_settings WHERE save_id = ?2
           )
         ORDER BY p.uid",
        Development::columns("p"),
        player_metrics_join("p", true, true)
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let players = statement
        .query_map(params![snapshot_id, save_id], |row| {
            let scores = (0..metric_columns.len())
                .map(|index| row.get::<_, Option<u8>>(SCORE_COLUMN + index))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                Development::from_row(row, DEVELOPMENT_COLUMN)?,
                scores,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    players
        .into_iter()
        .map(
            |(uid, name, preferred_foot, positions_json, development, scores)| {
                let positions =
                    serde_json::from_str::<BTreeMap<String, Option<i64>>>(&positions_json)
                        .map_err(|error| error.to_string())?;
                let lane_scores = tactic
                    .lanes
                    .iter()
                    .zip(scores.chunks_exact(SCORES_PER_LANE))
                    .map(|(lane, scores)| {
                        let fit = |ip_score, oop_score| {
                            lane_fit_score(
                                combine_role_scores(ip_score, oop_score, lane.ip_weight),
                                &preferred_foot,
                                &positions,
                                lane,
                            )
                        };
                        let current = fit(scores[0], scores[1])?;
                        // A lane without a potential score promises no growth.
                        Some((current, fit(scores[2], scores[3]).unwrap_or(current)))
                    })
                    .collect();
                Ok(ClubPlayer {
                    uid,
                    name,
                    development,
                    lane_scores,
                })
            },
        )
        .collect()
}

/// The players within the team's age limit who suit at least one lane, each valued for the team.
fn team_candidates(
    setting: &teams::PlannerTeamSetting,
    tactic: &PlannerTactic,
    players: &[ClubPlayer],
) -> Vec<OptimizerCandidate> {
    let team = setting.team;
    players
        .iter()
        .filter(|player| setting.takes_age(player.development.age))
        .filter(|player| player.lane_scores.iter().any(Option::is_some))
        .map(|player| OptimizerCandidate {
            player_uid: player.uid,
            last_known_name: player.name.clone(),
            lane_scores: player
                .lane_scores
                .iter()
                .zip(&tactic.lanes)
                .map(|(scores, lane)| {
                    let (current, potential) = (*scores)?;
                    Some(slot_value(
                        team,
                        current,
                        potential,
                        &player.development,
                        base_position(&lane.ip_position) == "GK",
                    ))
                })
                .collect(),
        })
        .collect()
}
