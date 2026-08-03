use std::collections::{HashMap, HashSet, VecDeque};

use rusqlite::{params, Connection};

use crate::features::scoring::combine::combine_role_scores;

use super::depth::{
    current_snapshot_id, ensure_depth, get_depth, insert_assignment, AssignmentProvenance,
    PlannerDepth, PlannerTeam, PLANNER_TEAMS,
};
use super::tactic::{PlannerTactic, TacticLane, TACTIC_LANE_COUNT};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OptimizerCandidate {
    pub(super) player_uid: i64,
    pub(super) last_known_name: String,
    pub(super) lane_scores: Vec<Option<u8>>,
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
    debug_assert_eq!(graph.send_max_flow(source, sink, lane_count), lane_count);

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

pub(super) fn optimize_depth(conn: &Connection, save_id: i64) -> Result<PlannerDepth, String> {
    let tactic = ensure_depth(conn, save_id)?;
    let snapshot_id = current_snapshot_id(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
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

    let strings = load_ordered_strings(&tx, save_id)?;
    for team in PLANNER_TEAMS {
        let candidates = load_optimizer_candidates(&tx, save_id, snapshot_id, team, &tactic)?;
        for planner_string in strings
            .iter()
            .filter(|planner_string| planner_string.team == team)
        {
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
            let available_candidates = candidates
                .iter()
                .filter(|candidate| !reserved_uids.contains(&candidate.player_uid))
                .cloned()
                .collect::<Vec<_>>();
            let matches = match_lanes(&lane_indices, &available_candidates);

            for (lane_index, candidate_index) in lane_indices.into_iter().zip(matches) {
                let Some(candidate_index) = candidate_index else {
                    continue;
                };
                let candidate = &available_candidates[candidate_index];
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
    }

    tx.commit().map_err(|error| error.to_string())?;
    get_depth(conn, save_id)
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

fn load_optimizer_candidates(
    tx: &rusqlite::Transaction<'_>,
    save_id: i64,
    snapshot_id: i64,
    team: PlannerTeam,
    tactic: &PlannerTactic,
) -> Result<Vec<OptimizerCandidate>, String> {
    let mut score_statement = tx
        .prepare(
            "SELECT scores.uid, scores.role_id, scores.score
             FROM players player
             CROSS JOIN player_role_scores scores
             WHERE player.snapshot_id = ?1
               AND EXISTS(
                   SELECT 1
                   FROM planner_club_sources source
                   WHERE source.save_id = ?2
                     AND source.team = ?3
                     AND source.club_name = player.current_club
               )
               AND scores.snapshot_id = player.snapshot_id
               AND scores.uid = player.uid",
        )
        .map_err(|error| error.to_string())?;
    let role_scores = score_statement
        .query_map(params![snapshot_id, save_id, team.as_str()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<u8>>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?
        .into_iter()
        .fold(
            HashMap::<i64, HashMap<String, Option<u8>>>::new(),
            |mut scores, row| {
                scores.entry(row.0).or_default().insert(row.1, row.2);
                scores
            },
        );

    let mut player_statement = tx
        .prepare(
            "SELECT p.uid, p.name, p.age, p.positions_json
             FROM players p
             WHERE p.snapshot_id = ?1
               AND EXISTS(
                   SELECT 1
                   FROM planner_club_sources source
                   WHERE source.save_id = ?2
                     AND source.team = ?3
                     AND source.club_name = p.current_club
               )
             ORDER BY p.uid",
        )
        .map_err(|error| error.to_string())?;
    let players = player_statement
        .query_map(params![snapshot_id, save_id, team.as_str()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    players
        .into_iter()
        .filter(|(_, _, age, _)| is_age_eligible(team, *age))
        .map(|(player_uid, last_known_name, _, positions_json)| {
            let positions =
                serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&positions_json)
                    .map_err(|error| error.to_string())?;
            let lane_scores = tactic
                .lanes
                .iter()
                .map(|lane| {
                    is_suitable_for_lane(&positions, lane)
                        .then(|| {
                            let player_scores = role_scores.get(&player_uid)?;
                            combine_role_scores(
                                player_scores
                                    .get(lane.ip_role_id.as_str())
                                    .copied()
                                    .flatten(),
                                player_scores
                                    .get(lane.oop_role_id.as_str())
                                    .copied()
                                    .flatten(),
                                lane.ip_weight,
                            )
                        })
                        .flatten()
                })
                .collect::<Vec<_>>();
            Ok(OptimizerCandidate {
                player_uid,
                last_known_name,
                lane_scores,
            })
        })
        .filter_map(|candidate| match candidate {
            Ok(candidate) if candidate.lane_scores.iter().any(Option::is_some) => {
                Some(Ok(candidate))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn is_age_eligible(team: PlannerTeam, age: Option<i64>) -> bool {
    match team {
        PlannerTeam::Senior => true,
        PlannerTeam::Reserves => age.is_some_and(|age| age <= 23),
        PlannerTeam::Youth => age.is_some_and(|age| age <= 18),
    }
}

fn is_suitable_for_lane(
    positions: &serde_json::Map<String, serde_json::Value>,
    lane: &TacticLane,
) -> bool {
    let has_suitability = |position: &str| {
        positions
            .get(position)
            .and_then(serde_json::Value::as_i64)
            .is_some_and(|suitability| suitability >= 15)
    };
    has_suitability(&lane.ip_position)
        && (lane.ip_position == lane.oop_position || has_suitability(&lane.oop_position))
}
