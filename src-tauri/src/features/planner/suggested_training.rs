//! Ranked FM26 individual-training focuses for an assigned tactic lane.
//!
//! Pure ranking engine: no SQL, no I/O, no side effects.

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

/// The goalkeeper focus inventory applies only to the GK Planner lane.
pub fn is_goalkeeper_lane(lane: &TacticLane) -> bool {
    lane.lane_id == "goalkeeper"
}

/// Ranks the applicable inventory for an assigned lane and returns the winning
/// focus name.
///
/// The best unrounded combined-score gain stays local to the ranking loop: it
/// is never displayed or exported. Returns `None` when any attribute required
/// by any focus in the applicable inventory is missing/null (no fallback
/// focus), when a lane role is unknown, or when a lane-role required attribute
/// is missing/null. All-zero gains still return the first inventory focus;
/// ties keep inventory order via a strict `>` comparison.
pub fn suggest_for_lane(
    attributes: &HashMap<String, Option<u8>>,
    lane: &TacticLane,
) -> Option<&'static str> {
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

    let mut best: Option<(&'static str, f64)> = None;
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
            Some((_, current_gain)) => gain > *current_gain,
        };
        if improves {
            best = Some((focus, gain));
        }
    }
    best.map(|(focus, _)| focus)
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
        let expected_outfield: &[(&str, &[&str])] = &[
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
        assert_eq!(OUTFIELD_FOCUSES, expected_outfield);

        let expected_goalkeeper: &[(&str, &[&str])] = &[
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
        assert_eq!(GOALKEEPER_FOCUSES, expected_goalkeeper);
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

        assert_eq!(suggestion, "Attacking Movement");
    }

    #[test]
    fn lane_weight_selects_between_oop_and_ip_winners() {
        let attributes = full_attributes(10);
        let oop_only = lane(
            "centre_forward",
            "centre_forward_ip",
            "central_outlet_centre_forward_oop",
            0.0,
        );
        let ip_only = lane(
            "centre_forward",
            "centre_forward_ip",
            "central_outlet_centre_forward_oop",
            1.0,
        );

        assert_eq!(
            suggest_for_lane(&attributes, &oop_only),
            Some("Attacking Movement")
        );
        assert_eq!(
            suggest_for_lane(&attributes, &ip_only),
            Some("Ball Control")
        );
    }

    #[test]
    fn goalkeeper_lane_uses_the_goalkeeper_inventory() {
        let attributes = full_attributes(10);

        let gk = suggest_for_lane(&attributes, &goalkeeper_lane()).expect("gk suggestion");
        assert_eq!(gk, "GK Reactions");

        let outfield =
            suggest_for_lane(&attributes, &centre_forward_lane()).expect("outfield suggestion");
        assert_eq!(outfield, "Attacking Movement");

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
    fn all_maxed_attributes_return_the_first_focus() {
        let suggestion =
            suggest_for_lane(&full_attributes(20), &centre_forward_lane()).expect("suggestion");

        assert_eq!(suggestion, "Free Kick Taking");
    }

    #[test]
    fn ranking_adapts_when_the_best_focus_attributes_are_maxed() {
        let suggestion =
            suggest_for_lane(&full_attributes(10), &centre_forward_lane()).expect("suggestion");
        assert_eq!(suggestion, "Attacking Movement");

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
        assert_eq!(suggestion, "Shooting");
    }

    #[test]
    fn unknown_lane_roles_return_none() {
        let lane = lane("centre_forward", "not_a_role_ip", "not_a_role_oop", 0.5);

        assert_eq!(suggest_for_lane(&full_attributes(10), &lane), None);
    }
}
