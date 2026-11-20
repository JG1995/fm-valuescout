//! Setting the managed club's players' individual training in FM.
//!
//! A player in a Planner slot trains the slot's in-possession role at its position and the slot's
//! out-of-possession role. Every player's additional focus becomes his Suggested Training as the
//! Squad overview shows it, and a player without one has his focus cleared. Players the club has
//! loaned out train with the borrower and stay out. The bridge sends FM27's own training events
//! and reports what FM holds afterwards. Nothing is stored: snapshots do not hold training. See
//! `.wiki/notes/fm27-training-actions.md`.

use std::collections::HashMap;

use rusqlite::{params, Connection};

use super::squad::{list_squad_players, SquadSortDir, SquadSortField, MAX_SQUAD_PAGE_LIMIT};
use super::{suggested_training, tactic};
use crate::db::Db;
use crate::features::managed_club::service::{capture_fm_action_context, FmActionContext};
use crate::features::memory_read::service::{
    TrainingChangeItem, TrainingOutcome, TrainingResponse, MAX_TRAINING_CHANGES,
};
use crate::features::player::boost_gate;
use crate::features::scoring::catalog::all_roles;

/// FM27's focus value that clears a player's additional focus.
const NO_FOCUS: i32 = -1;

/// The roles a Planner slot trains, in the app's names and in FM27's IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmTrainingRoles {
    /// The slot's in-possession position, such as `DC` for a right-sided centre-back.
    pub(super) position: String,
    pub(super) in_possession: String,
    pub(super) out_of_possession: String,
    fm_position: i32,
    fm_in_possession: u64,
    fm_out_of_possession: i32,
}

/// One player's training to set in FM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmTrainingChange {
    pub(super) player_uid: i64,
    pub(super) name: String,
    /// The Planner slot's roles; `None` for a player in no slot, whose roles stay as they are.
    pub(super) roles: Option<FmTrainingRoles>,
    /// His Suggested Training; `None` clears his focus.
    pub(super) focus: Option<String>,
    fm_focus: i32,
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

fn slot_roles(lane: &tactic::TacticLane) -> Result<FmTrainingRoles, String> {
    let roles = all_roles();
    let name = |role_id: &str| {
        roles
            .iter()
            .find(|role| role.role_id == role_id)
            .map(|role| role.display_name.to_string())
            .ok_or_else(|| format!("Unknown role `{role_id}`"))
    };
    let unknown = || format!("FM has no training for the `{}` slot's roles", lane.lane_id);
    Ok(FmTrainingRoles {
        position: tactic::base_position(&lane.ip_position).to_string(),
        in_possession: name(&lane.ip_role_id)?,
        out_of_possession: name(&lane.oop_role_id)?,
        fm_position: fm_position(&lane.ip_position).ok_or_else(unknown)?,
        fm_in_possession: fm_in_possession_role(&lane.ip_role_id).ok_or_else(unknown)?,
        fm_out_of_possession: fm_out_of_possession_role(&lane.oop_role_id).ok_or_else(unknown)?,
    })
}

/// The training to set for each of the managed club's players who is not loaned out, in name
/// order.
pub(super) fn plan_fm_training(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<FmTrainingChange>, String> {
    let has_tactic: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM planner_tactic_lanes WHERE save_id = ?1)",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let slots = if has_tactic {
        let tactic = tactic::load_tactic(conn, save_id)?;
        let mut statement = conn
            .prepare("SELECT player_uid, lane_id FROM planner_assignments WHERE save_id = ?1")
            .map_err(|error| error.to_string())?;
        let assignments = statement
            .query_map(params![save_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        assignments
            .into_iter()
            .map(|(player_uid, lane_id)| {
                let lane = tactic
                    .lanes
                    .iter()
                    .find(|lane| lane.lane_id == lane_id)
                    .ok_or_else(|| format!("Unknown tactic lane `{lane_id}`"))?;
                Ok((player_uid, slot_roles(lane)?))
            })
            .collect::<Result<HashMap<_, _>, String>>()?
    } else {
        HashMap::new()
    };

    // The Squad overview's own read, so the focus is the one its column shows.
    let mut changes = Vec::new();
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
        for player in page.players.into_iter().filter(|player| !player.loaned_out) {
            let fm_focus = match player.suggested_training.as_deref() {
                Some(focus) => suggested_training::fm_focus_id(focus)
                    .ok_or_else(|| format!("FM27 has no `{focus}` focus"))?,
                None => NO_FOCUS,
            };
            changes.push(FmTrainingChange {
                player_uid: player.uid,
                name: player.name,
                roles: slots.get(&player.uid).cloned(),
                focus: player.suggested_training,
                fm_focus,
            });
        }
        if last_page {
            return Ok(changes);
        }
    }
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
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let context = capture_training_context(&conn)?;
        let changes = plan_fm_training(&conn, context.save_id)?
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
            Ok(TrainingChangeItem {
                player_uid: u32::try_from(planned.player_uid)
                    .map_err(|_| format!("{} has an invalid player identity", planned.name))?,
                ip_position: planned.roles.as_ref().map(|roles| roles.fm_position),
                ip_role: planned.roles.as_ref().map(|roles| roles.fm_in_possession),
                oop_role: planned
                    .roles
                    .as_ref()
                    .map(|roles| roles.fm_out_of_possession),
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
    if let Some(roles) = &planned.roles {
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
    use super::{fm_in_possession_role, fm_out_of_possession_role, fm_position};
    use crate::features::scoring::catalog::{all_roles, RolePhase};

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
