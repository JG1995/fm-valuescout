//! Setting the managed club's players' individual training in FM.
//!
//! A player trains the roles of one tactic slot: the slot the Planner has him in, or, when the
//! save trains by development and he is young enough, his development slot
//! (`development.rs`). He trains the slot's in-possession role at its position and the slot's
//! out-of-possession role. FM27 builds position familiarity only through the in-possession
//! training, so a player who is fully familiar with the slot's in-possession position but not with
//! its out-of-possession position trains an in-possession role at that position instead. Every
//! player's additional focus becomes his Suggested Training as the Squad overview shows it, and a
//! player without one has his focus cleared. Players the club has loaned out train with the
//! borrower and stay out. The bridge sends FM27's own training events and reports what FM holds
//! afterwards. Nothing is stored: snapshots do not hold training. See
//! `.wiki/notes/fm27-training-actions.md`.

use std::collections::{BTreeMap, HashMap};

use rusqlite::{params, params_from_iter, Connection};

use super::depth::{current_snapshot_id, PlannerTeam};
use super::development::{self, same_training, DevelopmentSlots, PlayerDevelopment, TrainingBasis};
use super::fit::{position_familiarity, PositionRule, FULL_FAMILIARITY};
use super::squad::{list_squad_players, SquadSortDir, SquadSortField, MAX_SQUAD_PAGE_LIMIT};
use super::{suggested_training, tactic, teams};
use crate::db::Db;
use crate::features::managed_club::service::{capture_fm_action_context, FmActionContext};
use crate::features::memory_read::service::{
    TrainingChangeItem, TrainingOutcome, TrainingResponse, MAX_TRAINING_CHANGES,
};
use crate::features::player::boost_gate;
use crate::features::scoring::catalog::{all_roles, RoleDefinition, RolePhase};

/// FM27's focus value that clears a player's additional focus.
const NO_FOCUS: i32 = -1;

/// The roles a player trains for a slot, in the app's names and in FM27's IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmTrainingRoles {
    /// The position the in-possession role is trained at, such as `DC` for a right-sided
    /// centre-back.
    pub(super) position: String,
    pub(super) in_possession: String,
    pub(super) out_of_possession: String,
    fm_position: i32,
    fm_in_possession: u64,
    fm_out_of_possession: i32,
}

/// A slot position a player trains to become fully familiar with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LearningPosition {
    pub(super) position: String,
    /// His familiarity there out of 20; 0 when he has none.
    pub(super) familiarity: i64,
    /// Whether it is the slot's out-of-possession position, which he trains in place of the
    /// slot's own in-possession role and position.
    pub(super) out_of_possession: bool,
}

/// Where the Planner has a player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmTrainingSlot {
    pub(super) team: PlannerTeam,
    pub(super) team_name: String,
    pub(super) string_name: String,
    pub(super) lane_id: String,
    /// The slot's placements in and out of possession, such as `AMR` and `MR`.
    pub(super) ip_position: String,
    pub(super) oop_position: String,
    /// The slot position he is not yet fully familiar with, whatever he trains.
    pub(super) learning: Option<LearningPosition>,
}

/// The slot whose roles a player trains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmTrainingTarget {
    pub(super) lane_id: String,
    pub(super) ip_position: String,
    pub(super) oop_position: String,
    /// The slot's own in-possession role. He trains another only while he learns the
    /// out-of-possession position.
    pub(super) in_possession: String,
    pub(super) roles: FmTrainingRoles,
    pub(super) learning: Option<LearningPosition>,
    /// Whether it is his development slot and trains differently from the slot the Planner
    /// has him in, if any.
    pub(super) development: bool,
}

/// A slot a developing player could grow into.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct FmTrainingDevelopmentOption {
    pub(super) lane_id: String,
    pub(super) ip_position: String,
    pub(super) oop_position: String,
    /// The slot's in-possession role, which names the slot beside its position. He trains
    /// its out-of-possession role too.
    pub(super) in_possession: String,
    pub(super) current_score: u8,
    pub(super) potential_score: u8,
    /// Whether he can play the slot's positions today.
    pub(super) playable: bool,
    /// The share of the slot's players in the save whom his potential outranks; `None` for a
    /// slot the save's position rule does not rank him in.
    pub(super) potential_rank: Option<f64>,
}

/// Where a player young enough to develop is growing, whatever the save trains by.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct FmTrainingDevelopment {
    /// His development slot; `None` when he suits no slot.
    pub(super) lane_id: Option<String>,
    pub(super) suggested_lane_id: Option<String>,
    /// Whether the user chose his development slot.
    pub(super) locked: bool,
    pub(super) options: Vec<FmTrainingDevelopmentOption>,
}

/// One player's training to set in FM.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct FmTrainingChange {
    pub(super) player_uid: i64,
    pub(super) name: String,
    pub(super) age: Option<i64>,
    /// His Planner slot; `None` for a player in no slot.
    pub(super) slot: Option<FmTrainingSlot>,
    /// The slot whose roles he trains; `None` for a player whose roles stay as they are.
    pub(super) target: Option<FmTrainingTarget>,
    /// `None` for a player too old to develop, or of unknown age.
    pub(super) development: Option<FmTrainingDevelopment>,
    /// His Suggested Training; `None` clears his focus.
    pub(super) focus: Option<String>,
    fm_focus: i32,
}

/// A slot of the tactic, as the plan names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmTrainingLane {
    pub(super) lane_id: String,
    pub(super) ip_position: String,
    pub(super) oop_position: String,
    /// The first slot of the tactic that trains the same as this one, which may be itself.
    pub(super) trains_as: String,
}

/// The training to set for the managed club's players, and what it follows.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct FmTrainingPlan {
    pub(super) basis: TrainingBasis,
    /// The oldest age at which a player still counts as developing.
    pub(super) development_max_age: i64,
    /// Whether development slots are suggested only among the positions a player can play.
    pub(super) development_position_familiarity: bool,
    pub(super) lanes: Vec<FmTrainingLane>,
    /// The players in a Planner slot by squad, string, and slot, then the rest by name.
    pub(super) changes: Vec<FmTrainingChange>,
}

/// FM's position mask for a Planner placement's base position.
fn fm_position(placement: &str) -> Option<i32> {
    Some(match tactic::base_position(placement) {
        "GK" => 1,
        "DR" => 4,
        "DL" => 8,
        "DC" => 16,
        "WBR" => 32,
        "WBL" => 64,
        "DM" => 128,
        "MR" => 256,
        "ML" => 512,
        "MC" => 1024,
        "AMR" => 2048,
        "AML" => 4096,
        "AMC" => 8192,
        "ST" => 16384,
        _ => return None,
    })
}

/// FM27's in-possession role value (`FM.UI.InPossessionRole`) for a catalog role.
fn fm_in_possession_role(role_id: &str) -> Option<u64> {
    Some(match role_id {
        "goalkeeper_ip" => 1,
        "ball_playing_goalkeeper_ip" => 4096,
        "no_nonsense_goalkeeper_ip" => 9_007_199_254_740_992,
        "centre_back_ip" => 2,
        "ball_playing_centre_back_ip" => 16_777_216,
        "no_nonsense_centre_back_ip" => 536_870_912,
        "wide_centre_back_ip" => 2_251_799_813_685_248,
        "advanced_centre_back_ip" => 16_384,
        "overlapping_centre_back_ip" => 18_014_398_509_481_984,
        "full_back_ip" => 4,
        "inside_full_back_ip" => 4_503_599_627_370_496,
        "inside_wing_back_ip" => 17_592_186_044_416,
        "playmaking_wing_back_ip" => 36_028_797_018_963_968,
        "wing_back_ip" => 8,
        "advanced_wing_back_ip" => 274_877_906_944,
        "defensive_midfielder_ip" => 16,
        "box_to_box_midfielder_ip" => 65_536,
        "box_to_box_playmaker_ip" => 70_368_744_177_664,
        "deep_lying_playmaker_ip" => 32_768,
        "half_back_ip" => 34_359_738_368,
        "central_midfielder_ip" => 32,
        "advanced_playmaker_ip" => 131_072,
        "midfield_playmaker_ip" => 144_115_188_075_855_872,
        "wide_central_midfielder_ip" => 281_474_976_710_656,
        "wide_midfielder_ip" => 64,
        "inside_winger_ip" => 562_949_953_421_312,
        "playmaking_winger_ip" => 8_796_093_022_208,
        "winger_ip" => 128,
        "attacking_midfielder_ip" => 512,
        "channel_midfielder_ip" => 72_057_594_037_927_936,
        "free_role_ip" => 4_294_967_296,
        "second_striker_ip" => 2_199_023_255_552,
        "wide_forward_ip" => 35_184_372_088_832,
        "inside_forward_ip" => 134_217_728,
        "centre_forward_ip" => 2048,
        "channel_forward_ip" => 2_147_483_648,
        "deep_lying_forward_ip" => 1024,
        "false_nine_ip" => 1_099_511_627_776,
        "poacher_ip" => 524_288,
        "target_forward_ip" => 262_144,
        _ => return None,
    })
}

/// FM27's out-of-possession training role for a catalog role, as its training dropdown lists it.
fn fm_out_of_possession_role(role_id: &str) -> Option<i32> {
    Some(match role_id {
        "goalkeeper_oop" => -31,
        "sweeper_keeper_oop" => -30,
        "line_holding_keeper_oop" => -29,
        "centre_back_oop" => -28,
        "stopping_centre_back_oop" => -27,
        "covering_centre_back_oop" => -26,
        "wide_centre_back_oop" => -25,
        "stopping_wide_centre_back_oop" => -24,
        "covering_wide_centre_back_oop" => -23,
        "full_back_oop" => -22,
        "pressing_full_back_oop" => -21,
        "holding_full_back_oop" => -20,
        "wing_back_oop" => -19,
        "pressing_wing_back_oop" => -18,
        "holding_wing_back_oop" => -17,
        "defensive_midfielder_oop" => -16,
        "dropping_defensive_midfielder_oop" => -15,
        "pressing_defensive_midfielder_oop" => -14,
        "screening_defensive_midfielder_oop" => -13,
        "wide_covering_defensive_midfielder_oop" => -12,
        "central_midfielder_oop" => -11,
        "pressing_central_midfielder_oop" => -10,
        "screening_central_midfielder_oop" => -9,
        "wide_covering_central_midfielder_oop" => -8,
        "attacking_midfielder_oop" => -7,
        "tracking_attacking_midfielder_oop" => -6,
        "central_outlet_attacking_midfielder_oop" => -5,
        "splitting_outlet_attacking_midfielder_oop" => -4,
        "wide_midfielder_oop" => -3,
        "tracking_wide_midfielder_oop" => -2,
        "wide_outlet_wide_midfielder_oop" => -1,
        "winger_oop" => 0,
        "tracking_winger_oop" => 1,
        "inside_outlet_winger_oop" => 2,
        "wide_outlet_winger_oop" => 3,
        "centre_forward_oop" => 4,
        "tracking_centre_forward_oop" => 5,
        "central_outlet_centre_forward_oop" => 6,
        "splitting_outlet_centre_forward_oop" => 7,
        _ => return None,
    })
}

fn catalog_role(role_id: &str) -> Result<&'static RoleDefinition, String> {
    all_roles()
        .iter()
        .find(|role| role.role_id == role_id)
        .ok_or_else(|| format!("Unknown role `{role_id}`"))
}

/// Each attribute's share of a role's score: three quarters across the primary attributes and a
/// quarter across the secondary ones, as `score_role` blends them.
fn attribute_weights(role: &RoleDefinition) -> BTreeMap<&'static str, f64> {
    let primary_share = if role.secondary.is_empty() { 1.0 } else { 0.75 };
    let mut weights = BTreeMap::new();
    for key in role.primary {
        *weights.entry(*key).or_insert(0.0) += primary_share / role.primary.len() as f64;
    }
    for key in role.secondary {
        *weights.entry(*key).or_insert(0.0) += 0.25 / role.secondary.len() as f64;
    }
    weights
}

/// The in-possession role to train at `position` in place of `role`: `role` itself where the
/// catalog has it there, otherwise the role there whose weighted attributes are most like its
/// own, so the training stays close to what the slot asks for. The earlier catalog role wins a
/// tie.
fn closest_in_possession_role(
    role: &'static RoleDefinition,
    position: &str,
) -> Option<&'static RoleDefinition> {
    let position = tactic::base_position(position);
    if role.position_tags.contains(&position) {
        return Some(role);
    }
    let length = |weights: &BTreeMap<&str, f64>| {
        weights
            .values()
            .map(|weight| weight * weight)
            .sum::<f64>()
            .sqrt()
    };
    let wanted = attribute_weights(role);
    let mut closest: Option<(&'static RoleDefinition, f64)> = None;
    for candidate in all_roles().iter().filter(|candidate| {
        candidate.phase == RolePhase::InPossession && candidate.position_tags.contains(&position)
    }) {
        let weights = attribute_weights(candidate);
        let shared = wanted
            .iter()
            .map(|(key, weight)| weight * weights.get(key).copied().unwrap_or(0.0))
            .sum::<f64>();
        let likeness = shared / (length(&wanted) * length(&weights));
        if !closest.is_some_and(|(_, best)| likeness <= best) {
            closest = Some((candidate, likeness));
        }
    }
    closest.map(|(role, _)| role)
}

/// The slot position a player is not yet fully familiar with and trains: the in-possession
/// position first, then the out-of-possession position.
fn learning_position(
    lane: &tactic::TacticLane,
    positions: &BTreeMap<String, Option<i64>>,
) -> Option<LearningPosition> {
    [(&lane.ip_position, false), (&lane.oop_position, true)]
        .into_iter()
        .find_map(|(position, out_of_possession)| {
            let familiarity = position_familiarity(positions, position).unwrap_or(0);
            (familiarity < FULL_FAMILIARITY).then(|| LearningPosition {
                position: tactic::base_position(position).to_string(),
                familiarity,
                out_of_possession,
            })
        })
}

/// The roles a player in `lane` trains. FM27 builds familiarity only at the position of the
/// in-possession training, so a player learning the out-of-possession position trains an
/// in-possession role there.
fn slot_roles(
    lane: &tactic::TacticLane,
    learning: Option<&LearningPosition>,
) -> Result<FmTrainingRoles, String> {
    let unknown = || format!("FM has no training for the `{}` slot's roles", lane.lane_id);
    let slot_role = catalog_role(&lane.ip_role_id)?;
    let (position, in_possession) = match learning {
        Some(learning) if learning.out_of_possession => (
            learning.position.as_str(),
            closest_in_possession_role(slot_role, &learning.position).ok_or_else(unknown)?,
        ),
        _ => (tactic::base_position(&lane.ip_position), slot_role),
    };
    Ok(FmTrainingRoles {
        position: position.to_string(),
        in_possession: in_possession.display_name.to_string(),
        out_of_possession: catalog_role(&lane.oop_role_id)?.display_name.to_string(),
        fm_position: fm_position(position).ok_or_else(unknown)?,
        fm_in_possession: fm_in_possession_role(in_possession.role_id).ok_or_else(unknown)?,
        fm_out_of_possession: fm_out_of_possession_role(&lane.oop_role_id).ok_or_else(unknown)?,
    })
}

/// A player's place in the Planner.
struct Assignment<'a> {
    /// His squad, string, and slot, each in the Planner's order.
    order: (usize, i64, usize),
    team: PlannerTeam,
    team_name: String,
    string_name: String,
    lane: &'a tactic::TacticLane,
}

fn load_assignments<'a>(
    conn: &Connection,
    save_id: i64,
    tactic: &'a tactic::PlannerTactic,
) -> Result<HashMap<i64, Assignment<'a>>, String> {
    let teams = teams::get_team_settings(conn, save_id)?;
    let mut statement = conn
        .prepare(
            "SELECT assignment.player_uid, assignment.lane_id, planner_string.team,
                    planner_string.string_order, planner_string.display_name
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
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    rows.into_iter()
        .map(|(player_uid, lane_id, team, string_order, string_name)| {
            let team = PlannerTeam::parse(&team)?;
            let (team_order, setting) = teams
                .iter()
                .enumerate()
                .find(|(_, setting)| setting.team == team)
                .ok_or_else(|| format!("The {} team is not in this save", team.as_str()))?;
            let (lane_order, lane) = tactic
                .lanes
                .iter()
                .enumerate()
                .find(|(_, lane)| lane.lane_id == lane_id)
                .ok_or_else(|| format!("Unknown tactic lane `{lane_id}`"))?;
            Ok((
                player_uid,
                Assignment {
                    order: (team_order, string_order, lane_order),
                    team,
                    team_name: setting.display_name.clone(),
                    string_name,
                    lane,
                },
            ))
        })
        .collect()
}

/// The position familiarity of each of `player_uids` in the save's current snapshot.
fn load_positions(
    conn: &Connection,
    save_id: i64,
    player_uids: &[i64],
) -> Result<HashMap<i64, BTreeMap<String, Option<i64>>>, String> {
    let Some(snapshot_id) = current_snapshot_id(conn, save_id)? else {
        return Ok(HashMap::new());
    };
    if player_uids.is_empty() {
        return Ok(HashMap::new());
    }
    let sql = format!(
        "SELECT uid, positions_json FROM players WHERE snapshot_id = ? AND uid IN ({})",
        vec!["?"; player_uids.len()].join(", ")
    );
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params_from_iter(std::iter::once(&snapshot_id).chain(player_uids)),
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    rows.into_iter()
        .map(|(uid, positions_json)| {
            let positions = serde_json::from_str(&positions_json)
                .map_err(|error| format!("Invalid positions for player {uid}: {error}"))?;
            Ok((uid, positions))
        })
        .collect()
}

fn development_of(
    tactic: &tactic::PlannerTactic,
    development: &PlayerDevelopment,
) -> Result<FmTrainingDevelopment, String> {
    let lane_id = |lane_index: usize| tactic.lanes[lane_index].lane_id.clone();
    Ok(FmTrainingDevelopment {
        lane_id: development.chosen.map(lane_id),
        suggested_lane_id: development.suggested.map(lane_id),
        locked: development.locked,
        options: development
            .options
            .iter()
            .map(|option| {
                let lane = &tactic.lanes[option.lane_index];
                Ok(FmTrainingDevelopmentOption {
                    lane_id: lane.lane_id.clone(),
                    ip_position: lane.ip_position.clone(),
                    oop_position: lane.oop_position.clone(),
                    in_possession: catalog_role(&lane.ip_role_id)?.display_name.to_string(),
                    current_score: option.current,
                    potential_score: option.potential,
                    playable: option.playable,
                    potential_rank: option.potential_rank,
                })
            })
            .collect::<Result<_, String>>()?,
    })
}

/// The training to set for each of the managed club's players who is not loaned out, and what
/// the save's training follows.
pub(super) fn plan_fm_training(conn: &Connection, save_id: i64) -> Result<FmTrainingPlan, String> {
    let settings = development::load_settings(conn, save_id)?;
    let has_tactic: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM planner_tactic_lanes WHERE save_id = ?1)",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let tactic = if has_tactic {
        Some(tactic::load_tactic(conn, save_id)?)
    } else {
        None
    };
    let assignments = match &tactic {
        Some(tactic) => load_assignments(conn, save_id, tactic)?,
        None => HashMap::new(),
    };
    let slots = match (&tactic, current_snapshot_id(conn, save_id)?) {
        (Some(tactic), Some(snapshot_id)) => {
            development::development_slots(conn, save_id, snapshot_id, tactic, &settings)?
        }
        _ => DevelopmentSlots::default(),
    };

    // The Squad overview's own read, so the focus is the one its column shows.
    let mut players = Vec::new();
    let mut offset = 0;
    loop {
        let page = list_squad_players(
            conn,
            save_id,
            offset,
            MAX_SQUAD_PAGE_LIMIT,
            SquadSortField::Name,
            SquadSortDir::Asc,
            &[],
        )?;
        offset += page.players.len();
        let last_page = page.players.is_empty() || offset as i64 >= page.total;
        players.extend(page.players.into_iter().filter(|player| !player.loaned_out));
        if last_page {
            break;
        }
    }

    // The lane each player trains: his development slot when the save trains by it and it
    // trains differently from his Planner slot, otherwise his Planner slot.
    let trained_lane = |player_uid: i64| {
        let planner_lane = assignments
            .get(&player_uid)
            .map(|assignment| assignment.lane);
        let tactic = tactic.as_ref()?;
        let development_lane = (settings.training_basis == TrainingBasis::Development)
            .then(|| slots.lane_of(player_uid))
            .flatten()
            .map(|lane_index| &tactic.lanes[lane_index])
            .filter(|lane| !planner_lane.is_some_and(|planner| same_training(planner, lane)));
        match development_lane {
            Some(lane) => Some((lane, true)),
            None => planner_lane.map(|lane| (lane, false)),
        }
    };
    let positioned_uids = players
        .iter()
        .map(|player| player.uid)
        .filter(|uid| trained_lane(*uid).is_some())
        .collect::<Vec<_>>();
    let positions = load_positions(conn, save_id, &positioned_uids)?;
    let no_positions = BTreeMap::new();

    let mut changes = players
        .into_iter()
        .map(|player| {
            let fm_focus = match player.suggested_training.as_deref() {
                Some(focus) => suggested_training::fm_focus_id(focus)
                    .ok_or_else(|| format!("FM27 has no `{focus}` focus"))?,
                None => NO_FOCUS,
            };
            let positions = positions.get(&player.uid).unwrap_or(&no_positions);
            let assignment = assignments.get(&player.uid);
            let slot = assignment.map(|assignment| FmTrainingSlot {
                team: assignment.team,
                team_name: assignment.team_name.clone(),
                string_name: assignment.string_name.clone(),
                lane_id: assignment.lane.lane_id.clone(),
                ip_position: assignment.lane.ip_position.clone(),
                oop_position: assignment.lane.oop_position.clone(),
                learning: learning_position(assignment.lane, positions),
            });
            let target = trained_lane(player.uid)
                .map(|(lane, development)| {
                    let learning = learning_position(lane, positions);
                    Ok::<_, String>(FmTrainingTarget {
                        lane_id: lane.lane_id.clone(),
                        ip_position: lane.ip_position.clone(),
                        oop_position: lane.oop_position.clone(),
                        in_possession: catalog_role(&lane.ip_role_id)?.display_name.to_string(),
                        roles: slot_roles(lane, learning.as_ref())?,
                        learning,
                        development,
                    })
                })
                .transpose()?;
            let development = tactic
                .as_ref()
                .zip(slots.of(player.uid))
                .map(|(tactic, development)| development_of(tactic, development))
                .transpose()?;
            Ok((
                assignment.map(|assignment| assignment.order),
                FmTrainingChange {
                    player_uid: player.uid,
                    name: player.name,
                    age: player.age,
                    slot,
                    target,
                    development,
                    focus: player.suggested_training,
                    fm_focus,
                },
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    // The sort is stable, so the players in no slot keep the read's name order.
    changes.sort_by_key(|(order, _)| (order.is_none(), *order));

    let lanes = tactic.iter().flat_map(|tactic| {
        tactic.lanes.iter().map(|lane| FmTrainingLane {
            lane_id: lane.lane_id.clone(),
            ip_position: lane.ip_position.clone(),
            oop_position: lane.oop_position.clone(),
            trains_as: tactic
                .lanes
                .iter()
                .find(|other| same_training(other, lane))
                .map_or_else(|| lane.lane_id.clone(), |other| other.lane_id.clone()),
        })
    });
    Ok(FmTrainingPlan {
        basis: settings.training_basis,
        development_max_age: settings.max_age,
        development_position_familiarity: settings.position_rule == PositionRule::Familiarity,
        lanes: lanes.collect(),
        changes: changes.into_iter().map(|(_, change)| change).collect(),
    })
}

pub(super) const LOAD_DATA_FIRST: &str =
    "Load Data with FM open on this save before setting training";

pub(super) fn capture_training_context(conn: &Connection) -> Result<FmActionContext, String> {
    capture_fm_action_context(conn, LOAD_DATA_FIRST)
}

/// A player whose training FM did not fully take, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmTrainingFailure {
    pub(super) player_uid: i64,
    pub(super) name: String,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FmTrainingReport {
    /// Players whose training FM now holds as planned.
    pub(super) applied: Vec<i64>,
    pub(super) failed: Vec<FmTrainingFailure>,
    /// The error that stopped the request early, when one did.
    pub(super) error: Option<String>,
}

/// Sets the training of the chosen players, those of the current plan whose UIDs `player_uids`
/// lists, and reports what FM took.
pub(super) fn apply_fm_training<F>(
    db: &Db,
    player_uids: &[i64],
    request: F,
) -> Result<FmTrainingReport, String>
where
    F: FnOnce(&str, u32, &[TrainingChangeItem]) -> Result<TrainingResponse, String>,
{
    let _guard = boost_gate::acquire_boost_gate()?;
    let (context, changes) = {
        // A reader, which keeps the save's rankings between requests.
        let conn = db.read()?;
        let context = capture_training_context(&conn)?;
        let changes = plan_fm_training(&conn, context.save_id)?
            .changes
            .into_iter()
            .filter(|planned| player_uids.contains(&planned.player_uid))
            .collect::<Vec<_>>();
        (context, changes)
    };
    if changes.is_empty() {
        return Err("None of the chosen players is in the squad".to_string());
    }
    if changes.len() > MAX_TRAINING_CHANGES {
        return Err(format!(
            "Set training for at most {MAX_TRAINING_CHANGES} players at a time"
        ));
    }
    let items = changes
        .iter()
        .map(|planned| {
            let roles = planned.target.as_ref().map(|target| &target.roles);
            Ok(TrainingChangeItem {
                player_uid: u32::try_from(planned.player_uid)
                    .map_err(|_| format!("{} has an invalid player identity", planned.name))?,
                ip_position: roles.map(|roles| roles.fm_position),
                ip_role: roles.map(|roles| roles.fm_in_possession),
                oop_role: roles.map(|roles| roles.fm_out_of_possession),
                focus: Some(planned.fm_focus),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let response = request(&context.source_request_id, context.club_uid, &items)?;
    if response.players.len() != changes.len() {
        return Err(
            "The bridge reported a different number of training changes than were sent".to_string(),
        );
    }

    let mut report = FmTrainingReport {
        error: response.error,
        ..FmTrainingReport::default()
    };
    for (planned, outcome) in changes.iter().zip(response.players) {
        if outcome.outcome == "applied" {
            report.applied.push(planned.player_uid);
        } else {
            report.failed.push(FmTrainingFailure {
                player_uid: planned.player_uid,
                name: planned.name.clone(),
                reason: failure_reason(planned, &outcome),
            });
        }
    }
    Ok(report)
}

fn failure_reason(planned: &FmTrainingChange, outcome: &TrainingOutcome) -> String {
    match outcome.outcome.as_str() {
        "not-found" => return "FM could not find him; Load Data and try again".to_string(),
        "not-at-club" => return "He is not in one of the club's own squads in FM".to_string(),
        "not-confirmed" => {}
        _ => return "FM did not report an outcome".to_string(),
    }
    let part = |what: &str, held: &Option<String>| match held {
        Some(held) => format!("the {what} (FM shows {held})"),
        None => format!("the {what}"),
    };
    let mut parts = Vec::new();
    if let Some(roles) = planned.target.as_ref().map(|target| &target.roles) {
        if outcome.ip_position != Some(roles.fm_position)
            || outcome.ip_role != Some(roles.fm_in_possession)
        {
            parts.push(part("in-possession role", &outcome.ip_role_name));
        }
        if outcome.oop_role != Some(roles.fm_out_of_possession) {
            parts.push(part("out-of-possession role", &outcome.oop_role_name));
        }
    }
    if outcome.focus != Some(planned.fm_focus) {
        parts.push(part("focus", &outcome.focus_name));
    }
    if parts.is_empty() {
        "FM did not confirm the training".to_string()
    } else {
        format!("FM did not take {}", parts.join(" or "))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        closest_in_possession_role, fm_in_possession_role, fm_out_of_possession_role, fm_position,
    };
    use crate::features::scoring::catalog::{all_roles, RolePhase};

    #[test]
    fn every_in_possession_role_has_a_stand_in_at_every_position() {
        let positions = all_roles()
            .iter()
            .flat_map(|role| role.position_tags.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        for role in all_roles()
            .iter()
            .filter(|role| role.phase == RolePhase::InPossession)
        {
            for position in &positions {
                let stand_in = closest_in_possession_role(role, position)
                    .unwrap_or_else(|| panic!("no in-possession role at {position}"));
                assert_eq!(stand_in.phase, RolePhase::InPossession);
                assert!(stand_in.position_tags.contains(position));
                if role.position_tags.contains(position) {
                    assert_eq!(
                        stand_in.role_id, role.role_id,
                        "a role stands in for itself"
                    );
                }
            }
        }
    }

    #[test]
    fn every_catalog_role_has_fm_training_at_each_of_its_positions() {
        for role in all_roles() {
            match role.phase {
                RolePhase::InPossession => assert!(
                    fm_in_possession_role(role.role_id).is_some(),
                    "{} has no FM in-possession value",
                    role.role_id
                ),
                RolePhase::OutOfPossession => assert!(
                    fm_out_of_possession_role(role.role_id).is_some(),
                    "{} has no FM out-of-possession value",
                    role.role_id
                ),
            }
            for position in role.position_tags {
                assert!(
                    fm_position(position).is_some(),
                    "{position} has no FM position"
                );
            }
        }
        for placement in [
            "DCR", "DCL", "DMCR", "DMCL", "MCR", "MCL", "AMCR", "AMCL", "STC", "STCR", "STCL",
        ] {
            assert!(
                fm_position(placement).is_some(),
                "{placement} has no FM position"
            );
        }
    }

    #[test]
    fn in_possession_values_are_single_distinct_bits() {
        let values = all_roles()
            .iter()
            .filter_map(|role| fm_in_possession_role(role.role_id))
            .collect::<Vec<_>>();
        assert!(values.iter().all(|value| value.count_ones() == 1));
        let distinct = values.iter().collect::<std::collections::HashSet<_>>();
        assert_eq!(distinct.len(), values.len());
    }
}
