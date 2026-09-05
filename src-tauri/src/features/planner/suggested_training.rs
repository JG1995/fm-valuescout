//! Ranked FM26 individual-training focuses for an assigned tactic lane.
//!
//! Pure ranking engine: no SQL, no I/O, no side effects.
//!
//! Public API stays unused until the Squad read-model wiring consumes it.

#![allow(dead_code)]

use std::collections::HashMap;

use crate::features::scoring::catalog::{all_roles, RoleDefinition};
use crate::features::scoring::score::score_role_unrounded;

use super::tactic::TacticLane;

/// Outfield focus inventory in exact Linear JAY-46 order (order is the tie-break).
/// Each entry maps a focus name to its simulated dump-key attributes.
pub const OUTFIELD_FOCUSES: &[(&str, &[&str])] = &[
    ("Free Kick Taking", &["Technique", "FreeKicks"]),
    ("Corner Taking", &["Technique", "Corners"]),
    ("Penalty Taking", &["Technique", "PenaltyTaking"]),
    ("Long Throws", &["LongThrows"]),
    ("Quickness", &["Acceleration", "Pace"]),
    ("Agility and Balance", &["Agility", "Balance"]),
    ("Strength", &["JumpingReach", "Strength"]),
    ("Endurance", &["WorkRate", "Stamina"]),
    (
        "Defensive Positioning",
        &["Marking", "Decisions", "Positioning"],
    ),
    (
        "Attacking Movement",
        &["Anticipation", "Decisions", "OffTheBall"],
    ),
    ("Shooting", &["Finishing", "LongShots", "Technique"]),
    ("Passing", &["Passing", "Technique", "Vision"]),
    ("Final Third", &["Composure", "Decisions"]),
    ("Crossing", &["Crossing", "Technique"]),
    ("Ball Control", &["Dribbling", "FirstTouch", "Technique"]),
    ("Aerial", &["Heading", "Bravery"]),
];

/// Goalkeeper focus inventory in exact Linear JAY-46 order (order is the tie-break).
pub const GOALKEEPER_FOCUSES: &[(&str, &[&str])] = &[
    ("Free Kick Taking", &["Technique", "FreeKicks"]),
    ("Corner Taking", &["Technique", "Corners"]),
    ("Penalty Taking", &["Technique", "PenaltyTaking"]),
    ("Long Throws", &["LongThrows"]),
    ("Quickness", &["Acceleration", "Pace"]),
    ("Agility and Balance", &["Agility", "Balance"]),
    ("Strength", &["JumpingReach", "Strength"]),
    ("Endurance", &["WorkRate", "Stamina"]),
    (
        "GK Reactions",
        &["Reflexes", "Anticipation", "Concentration"],
    ),
    (
        "GK Tactical",
        &["Communication", "Decisions", "Positioning"],
    ),
    ("GK Technique", &["Handling", "Composure", "Technique"]),
    ("GK Sweeping", &["CommandOfArea", "OneOnOnes", "RushingOut"]),
    ("GK Distribution (Long)", &["Kicking", "Throwing"]),
    (
        "GK Distribution (Short)",
        &["FirstTouch", "Passing", "Vision"],
    ),
];

/// Chosen focus: the inventory entry name, its ordered mapped attributes,
/// the unrounded blended gain, and the simulated attributes that overlap the
/// lane roles (in focus-mapping order).
#[derive(Debug, Clone, PartialEq)]
pub struct SuggestedFocus {
    pub focus: &'static str,
    pub focus_attributes: Vec<&'static str>,
    pub gain: f64,
    pub contributing_attributes: Vec<&'static str>,
}

/// The goalkeeper focus inventory applies only to the GK Planner lane.
pub fn is_goalkeeper_lane(lane: &TacticLane) -> bool {
    lane.lane_id == "goalkeeper"
}

/// Ranks the applicable inventory for an assigned lane and returns the focus
/// with the largest unrounded combined-score gain.
///
/// Returns `None` when any attribute required by any focus in the applicable
/// inventory is missing/null (no fallback focus), when a lane role is
/// unknown, or when a lane-role required attribute is missing/null.
/// All-zero gains still return the first inventory focus; ties keep
/// inventory order via a strict `>` comparison.
pub fn suggest_for_lane(
    attributes: &HashMap<String, Option<u8>>,
    lane: &TacticLane,
) -> Option<SuggestedFocus> {
    let focuses = if is_goalkeeper_lane(lane) {
        GOALKEEPER_FOCUSES
    } else {
        OUTFIELD_FOCUSES
    };

    for (_, keys) in focuses.iter() {
        for key in keys.iter() {
            attributes.get(*key).copied().flatten()?;
        }
    }

    let roles = all_roles();
    let ip_role = roles.iter().find(|role| role.role_id == lane.ip_role_id)?;
    let oop_role = roles.iter().find(|role| role.role_id == lane.oop_role_id)?;
    let baseline = blended_score(attributes, ip_role, oop_role, lane.ip_weight)?;

    let mut best: Option<SuggestedFocus> = None;
    for &(focus, keys) in focuses.iter() {
        let mut simulated = attributes.clone();
        for key in keys.iter() {
            if let Some(value) = simulated.get(*key).copied().flatten() {
                if value < 20 {
                    simulated.insert((*key).to_string(), Some(value + 1));
                }
            }
        }
        let simulated_score = blended_score(&simulated, ip_role, oop_role, lane.ip_weight)?;
        let gain = simulated_score - baseline;
        let improves = match &best {
            None => true,
            Some(current) => gain > current.gain,
        };
        if improves {
            best = Some(SuggestedFocus {
                focus,
                focus_attributes: keys.to_vec(),
                gain,
                contributing_attributes: contributing(keys, attributes, ip_role, oop_role),
            });
        }
    }
    best
}

fn blended_score(
    attributes: &HashMap<String, Option<u8>>,
    ip_role: &RoleDefinition,
    oop_role: &RoleDefinition,
    ip_weight: f64,
) -> Option<f64> {
    let ip = score_role_unrounded(attributes, ip_role)?;
    let oop = score_role_unrounded(attributes, oop_role)?;
    Some(ip * ip_weight + oop * (1.0 - ip_weight))
}

/// Focus-mapped attributes actually simulated (below 20) that also appear in
/// either lane role's primary/secondary bands, in focus-mapping order.
fn contributing(
    keys: &[&'static str],
    attributes: &HashMap<String, Option<u8>>,
    ip_role: &RoleDefinition,
    oop_role: &RoleDefinition,
) -> Vec<&'static str> {
    keys.iter()
        .filter(|key| {
            let simulated = attributes
                .get(**key)
                .copied()
                .flatten()
                .is_some_and(|value| value < 20);
            simulated
                && (ip_role.primary.contains(key)
                    || ip_role.secondary.contains(key)
                    || oop_role.primary.contains(key)
                    || oop_role.secondary.contains(key))
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS;

    use super::{is_goalkeeper_lane, suggest_for_lane, GOALKEEPER_FOCUSES, OUTFIELD_FOCUSES};
    use crate::features::planner::tactic::TacticLane;

    fn lane(lane_id: &str, ip_role_id: &str, oop_role_id: &str, ip_weight: f64) -> TacticLane {
        TacticLane {
            lane_id: lane_id.to_string(),
            ip_weight,
            importance_rank: None,
            preferred_foot: "any".to_string(),
            foot_preference: "preferred".to_string(),
            ip_position: "STC".to_string(),
            ip_role_id: ip_role_id.to_string(),
            oop_position: "STC".to_string(),
            oop_role_id: oop_role_id.to_string(),
        }
    }

    fn centre_forward_lane() -> TacticLane {
        lane(
            "centre_forward",
            "centre_forward_ip",
            "central_outlet_centre_forward_oop",
            0.5,
        )
    }

    fn goalkeeper_lane() -> TacticLane {
        TacticLane {
            ip_position: "GK".to_string(),
            oop_position: "GK".to_string(),
            ..lane(
                "goalkeeper",
                "goalkeeper_ip",
                "line_holding_keeper_oop",
                0.5,
            )
        }
    }

    fn full_attributes(value: u8) -> HashMap<String, Option<u8>> {
        DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), Some(value)))
            .collect()
    }

    #[test]
    fn outfield_and_goalkeeper_inventories_match_linear_order_exactly() {
        let outfield_names = OUTFIELD_FOCUSES
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        assert_eq!(
            outfield_names,
            vec![
                "Free Kick Taking",
                "Corner Taking",
                "Penalty Taking",
                "Long Throws",
                "Quickness",
                "Agility and Balance",
                "Strength",
                "Endurance",
                "Defensive Positioning",
                "Attacking Movement",
                "Shooting",
                "Passing",
                "Final Third",
                "Crossing",
                "Ball Control",
                "Aerial",
            ]
        );

        let goalkeeper_names = GOALKEEPER_FOCUSES
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        assert_eq!(
            goalkeeper_names,
            vec![
                "Free Kick Taking",
                "Corner Taking",
                "Penalty Taking",
                "Long Throws",
                "Quickness",
                "Agility and Balance",
                "Strength",
                "Endurance",
                "GK Reactions",
                "GK Tactical",
                "GK Technique",
                "GK Sweeping",
                "GK Distribution (Long)",
                "GK Distribution (Short)",
            ]
        );
    }

    #[test]
    fn every_mapped_key_is_a_known_dump_key() {
        for (focus, keys) in OUTFIELD_FOCUSES.iter().chain(GOALKEEPER_FOCUSES.iter()) {
            for key in *keys {
                assert!(
                    DUMP_ATTRIBUTE_KEYS.contains(key),
                    "focus `{focus}` maps unknown attribute `{key}`"
                );
            }
        }
    }

    #[test]
    fn ranking_picks_the_largest_unrounded_gain() {
        let suggestion =
            suggest_for_lane(&full_attributes(10), &centre_forward_lane()).expect("suggestion");

        assert_eq!(suggestion.focus, "Attacking Movement");
        assert_eq!(
            suggestion.focus_attributes,
            vec!["Anticipation", "Decisions", "OffTheBall"]
        );
        assert!((suggestion.gain - 2.890625).abs() < 1e-9);
    }

    #[test]
    fn goalkeeper_lane_uses_the_goalkeeper_inventory() {
        let attributes = full_attributes(10);

        let gk = suggest_for_lane(&attributes, &goalkeeper_lane()).expect("gk suggestion");
        assert_eq!(gk.focus, "GK Reactions");
        assert_eq!(
            gk.focus_attributes,
            vec!["Reflexes", "Anticipation", "Concentration"]
        );

        let outfield =
            suggest_for_lane(&attributes, &centre_forward_lane()).expect("outfield suggestion");
        assert_eq!(outfield.focus, "Attacking Movement");

        assert!(is_goalkeeper_lane(&goalkeeper_lane()));
        assert!(!is_goalkeeper_lane(&centre_forward_lane()));
    }

    #[test]
    fn missing_lane_role_attribute_returns_none() {
        let mut attributes = full_attributes(10);
        attributes.insert("Tackling".to_string(), None);
        let lane = lane(
            "left_centre_back",
            "centre_back_ip",
            "covering_centre_back_oop",
            0.5,
        );

        assert_eq!(suggest_for_lane(&attributes, &lane), None);
    }

    #[test]
    fn missing_any_inventory_attribute_blanks_the_whole_suggestion() {
        let mut attributes = full_attributes(10);
        attributes.insert("Corners".to_string(), None);

        assert_eq!(suggest_for_lane(&attributes, &centre_forward_lane()), None);
    }

    #[test]
    fn all_maxed_attributes_return_the_first_focus_with_zero_gain() {
        let suggestion =
            suggest_for_lane(&full_attributes(20), &centre_forward_lane()).expect("suggestion");

        assert_eq!(suggestion.focus, "Free Kick Taking");
        assert_eq!(suggestion.focus_attributes, vec!["Technique", "FreeKicks"]);
        assert_eq!(suggestion.gain, 0.0);
        assert!(suggestion.contributing_attributes.is_empty());
    }

    #[test]
    fn contributing_attributes_are_the_simulated_lane_role_overlap() {
        let suggestion =
            suggest_for_lane(&full_attributes(10), &centre_forward_lane()).expect("suggestion");

        assert_eq!(
            suggestion.contributing_attributes,
            vec!["Anticipation", "Decisions", "OffTheBall"]
        );

        let mut attributes = full_attributes(10);
        for key in [
            "Anticipation",
            "Decisions",
            "OffTheBall",
            "Dribbling",
            "FirstTouch",
        ] {
            attributes.insert(key.to_string(), Some(20));
        }
        let suggestion = suggest_for_lane(&attributes, &centre_forward_lane()).expect("suggestion");
        assert_eq!(suggestion.focus, "Shooting");
        assert_eq!(
            suggestion.focus_attributes,
            vec!["Finishing", "LongShots", "Technique"]
        );
        assert_eq!(
            suggestion.contributing_attributes,
            vec!["Finishing", "Technique"]
        );
        assert!((suggestion.gain - 0.46875).abs() < 1e-9);
    }

    #[test]
    fn unknown_lane_roles_return_none() {
        let lane = lane("centre_forward", "not_a_role_ip", "not_a_role_oop", 0.5);

        assert_eq!(suggest_for_lane(&full_attributes(10), &lane), None);
    }
}
