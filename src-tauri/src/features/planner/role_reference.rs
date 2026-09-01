use std::collections::{BTreeMap, BTreeSet, HashMap};

use rusqlite::{params, Connection};

use crate::features::{
    managed_club::service as managed_club_service,
    player_metrics::compact::{
        player_current_column, player_metrics_join, player_potential_column, PLAYER_METRICS_ALIAS,
    },
    scoring::catalog::{all_roles, RolePhase},
};

use super::{
    depth::current_snapshot_id,
    fit::phase_fit_score,
    tactic::{self, TacticLane},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleReferencePhase {
    InPossession,
    OutOfPossession,
}

impl RoleReferencePhase {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "in_possession" => Ok(Self::InPossession),
            "out_of_possession" => Ok(Self::OutOfPossession),
            _ => Err(format!("Unknown planner role reference phase `{value}`")),
        }
    }

    fn role_phase(self) -> RolePhase {
        match self {
            Self::InPossession => RolePhase::InPossession,
            Self::OutOfPossession => RolePhase::OutOfPossession,
        }
    }

    fn position(self, lane: &TacticLane) -> &str {
        match self {
            Self::InPossession => &lane.ip_position,
            Self::OutOfPossession => &lane.oop_position,
        }
    }

    fn role_id(self, lane: &TacticLane) -> &str {
        match self {
            Self::InPossession => &lane.ip_role_id,
            Self::OutOfPossession => &lane.oop_role_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleReferenceBasis {
    Current,
    Potential,
}

impl RoleReferenceBasis {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "current" => Ok(Self::Current),
            "potential" => Ok(Self::Potential),
            _ => Err(format!(
                "Unknown planner role reference score basis `{value}`"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleReferencePlayer {
    pub player_uid: i64,
    pub name: String,
    pub current_score: Option<u8>,
    pub potential_score: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleReferenceLane {
    pub lane_id: String,
    pub players: Vec<RoleReferencePlayer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleReference {
    pub lanes: Vec<RoleReferenceLane>,
    pub no_eligible: Vec<RoleReferencePlayer>,
}

struct PlayerInput {
    player_uid: i64,
    name: String,
    preferred_foot: String,
    positions: BTreeMap<String, Option<i64>>,
    role_scores: HashMap<String, Option<u8>>,
    potential_role_scores: HashMap<String, Option<u8>>,
}

pub fn get_role_reference(
    conn: &Connection,
    save_id: i64,
    phase: RoleReferencePhase,
    basis: RoleReferenceBasis,
) -> Result<RoleReference, String> {
    let snapshot_id = current_snapshot_id(conn, save_id)?
        .ok_or_else(|| "No current snapshot loaded for this save".to_string())?;
    let club_name = managed_club_service::selected_club(conn, save_id)?
        .ok_or_else(|| "No managed club configured for this save".to_string())?;
    let tactic = tactic::load_tactic(conn, save_id)?;
    let lane_roles = tactic
        .lanes
        .iter()
        .map(|lane| {
            let role_id = phase.role_id(lane);
            let role = all_roles()
                .iter()
                .find(|role| role.role_id == role_id)
                .ok_or_else(|| format!("Unknown tactic lane role `{role_id}`"))?;
            if role.phase != phase.role_phase() {
                return Err(format!(
                    "Tactic lane role `{role_id}` does not belong to the selected phase"
                ));
            }
            Ok((lane, role, phase.position(lane)))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let role_ids = lane_roles
        .iter()
        .map(|(_, role, _)| role.role_id)
        .collect::<BTreeSet<_>>();
    let players = load_players(conn, snapshot_id, &club_name, &role_ids)?;
    let mut assigned = (0..tactic.lanes.len())
        .map(|_| Vec::new())
        .collect::<Vec<Vec<RoleReferencePlayer>>>();
    let mut no_eligible = Vec::new();

    for player in players {
        let mut best: Option<(usize, u8, RoleReferencePlayer)> = None;

        for (lane_index, (lane, role, position)) in lane_roles.iter().enumerate() {
            let current_score = phase_fit_score(
                player.role_scores.get(role.role_id).copied().flatten(),
                &player.preferred_foot,
                &player.positions,
                position,
                &lane.preferred_foot,
                &lane.foot_preference,
            );
            let potential_score = phase_fit_score(
                player
                    .potential_role_scores
                    .get(role.role_id)
                    .copied()
                    .flatten(),
                &player.preferred_foot,
                &player.positions,
                position,
                &lane.preferred_foot,
                &lane.foot_preference,
            );
            let selected_score = match basis {
                RoleReferenceBasis::Current => current_score,
                RoleReferenceBasis::Potential => potential_score,
            };
            let Some(selected_score) = selected_score else {
                continue;
            };

            let candidate = RoleReferencePlayer {
                player_uid: player.player_uid,
                name: player.name.clone(),
                current_score,
                potential_score,
            };
            if best
                .as_ref()
                .map_or(true, |(_, best_score, _)| selected_score > *best_score)
            {
                best = Some((lane_index, selected_score, candidate));
            }
        }

        if let Some((lane_index, _, player)) = best {
            assigned[lane_index].push(player);
        } else {
            no_eligible.push(RoleReferencePlayer {
                player_uid: player.player_uid,
                name: player.name,
                current_score: None,
                potential_score: None,
            });
        }
    }

    assigned
        .iter_mut()
        .for_each(|players| sort_players(players));
    sort_players(&mut no_eligible);

    Ok(RoleReference {
        lanes: tactic
            .lanes
            .into_iter()
            .zip(assigned)
            .map(|(lane, players)| RoleReferenceLane {
                lane_id: lane.lane_id,
                players,
            })
            .collect(),
        no_eligible,
    })
}

fn load_players(
    conn: &Connection,
    snapshot_id: i64,
    club_name: &str,
    role_ids: &BTreeSet<&str>,
) -> Result<Vec<PlayerInput>, String> {
    let current_columns = role_ids
        .iter()
        .map(|role_id| player_current_column(role_id))
        .collect::<Result<Vec<_>, _>>()?;
    let potential_columns = role_ids
        .iter()
        .map(|role_id| player_potential_column(role_id))
        .collect::<Result<Vec<_>, _>>()?;
    let metric_select = current_columns
        .iter()
        .copied()
        .chain(potential_columns.iter().map(String::as_str))
        .map(|column| format!(", {PLAYER_METRICS_ALIAS}.{column}"))
        .collect::<String>();
    let sql = format!(
        "SELECT
             p.uid,
             p.name,
             p.preferred_foot,
             p.positions_json,
             {PLAYER_METRICS_ALIAS}.score_model_version IS NOT NULL{metric_select}
         FROM players p{}
         WHERE p.snapshot_id = ?1 AND p.current_club = ?2
         ORDER BY p.uid",
        player_metrics_join("p", true, true)
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params![snapshot_id, club_name], |row| {
            let has_compact_row = row.get::<_, bool>(4)?;
            let current_scores = (0..role_ids.len())
                .map(|index| row.get::<_, Option<u8>>(index + 5))
                .collect::<Result<Vec<_>, _>>()?;
            let potential_scores = (0..role_ids.len())
                .map(|index| row.get::<_, Option<u8>>(index + 5 + role_ids.len()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                has_compact_row,
                current_scores,
                potential_scores,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    if rows
        .iter()
        .any(|(_, _, _, _, has_compact_row, _, _)| !has_compact_row)
    {
        return Err("Current potential snapshot is incomplete".to_string());
    }
    rows.into_iter()
        .map(
            |(
                player_uid,
                name,
                preferred_foot,
                positions_json,
                _,
                current_scores,
                potential_scores,
            )| {
                let positions = serde_json::from_str(&positions_json).map_err(|error| {
                    format!("Invalid positions_json for player {player_uid}: {error}")
                })?;
                let role_scores = role_ids
                    .iter()
                    .zip(current_scores)
                    .map(|(role_id, score)| ((*role_id).to_string(), score))
                    .collect();
                let potential_role_scores = role_ids
                    .iter()
                    .zip(potential_scores)
                    .map(|(role_id, score)| ((*role_id).to_string(), score))
                    .collect();
                Ok(PlayerInput {
                    player_uid,
                    name,
                    preferred_foot,
                    positions,
                    role_scores,
                    potential_role_scores,
                })
            },
        )
        .collect()
}

fn sort_players(players: &mut [RoleReferencePlayer]) {
    players.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.player_uid.cmp(&right.player_uid))
    });
}
