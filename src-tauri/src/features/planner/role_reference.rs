use std::collections::{BTreeMap, HashMap};

use rusqlite::{params, Connection};

use crate::features::{
    managed_club::service as managed_club_service,
    scoring::{
        catalog::{all_roles, RolePhase},
        projection::project_attributes,
        score::score_role,
    },
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
    attributes: HashMap<String, Option<u8>>,
    ca: i64,
    pa: i64,
    age: Option<i64>,
    role_scores: HashMap<String, Option<u8>>,
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
    let players = load_players(conn, snapshot_id, &club_name)?;
    let mut assigned = (0..tactic.lanes.len())
        .map(|_| Vec::new())
        .collect::<Vec<Vec<RoleReferencePlayer>>>();
    let mut no_eligible = Vec::new();

    for player in players {
        let projected_attributes = project_attributes(
            &player.attributes,
            player.ca,
            player.pa,
            player.age,
            player
                .positions
                .iter()
                .map(|(position, familiarity)| (position.as_str(), *familiarity)),
        );
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
                score_role(&projected_attributes, role),
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
) -> Result<Vec<PlayerInput>, String> {
    let mut statement = conn
        .prepare(
            "SELECT uid, name, preferred_foot, positions_json, attributes_json, ca, pa, age
             FROM players
             WHERE snapshot_id = ?1 AND current_club = ?2
             ORDER BY uid",
        )
        .map_err(|error| error.to_string())?;
    let mut players = statement
        .query_map(params![snapshot_id, club_name], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, Option<i64>>(7)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .map(|row| {
            let (player_uid, name, preferred_foot, positions_json, attributes_json, ca, pa, age) =
                row.map_err(|error| error.to_string())?;
            let positions = serde_json::from_str(&positions_json).map_err(|error| {
                format!("Invalid positions_json for player {player_uid}: {error}")
            })?;
            let attributes = serde_json::from_str(&attributes_json).map_err(|error| {
                format!("Invalid attributes_json for player {player_uid}: {error}")
            })?;
            Ok(PlayerInput {
                player_uid,
                name,
                preferred_foot,
                positions,
                attributes,
                ca,
                pa,
                age,
                role_scores: HashMap::new(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let player_indices = players
        .iter()
        .enumerate()
        .map(|(index, player)| (player.player_uid, index))
        .collect::<HashMap<_, _>>();
    let mut score_statement = conn
        .prepare(
            "SELECT scores.uid, scores.role_id, scores.score
             FROM player_role_scores scores
             INNER JOIN players p
               ON p.snapshot_id = scores.snapshot_id AND p.uid = scores.uid
             WHERE scores.snapshot_id = ?1 AND p.current_club = ?2",
        )
        .map_err(|error| error.to_string())?;
    let score_rows = score_statement
        .query_map(params![snapshot_id, club_name], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<u8>>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?;
    for row in score_rows {
        let (player_uid, role_id, score) = row.map_err(|error| error.to_string())?;
        if let Some(index) = player_indices.get(&player_uid) {
            players[*index].role_scores.insert(role_id, score);
        }
    }

    Ok(players)
}

fn sort_players(players: &mut [RoleReferencePlayer]) {
    players.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.player_uid.cmp(&right.player_uid))
    });
}
