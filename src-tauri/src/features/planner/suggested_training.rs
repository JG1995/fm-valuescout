//! Ranked FM27 individual-training focuses for an assigned tactic lane.
//!
//! Pure ranking engine: no SQL, no I/O, no side effects.

use std::collections::HashMap;

use crate::features::scoring::catalog::{all_roles, RoleDefinition};
use crate::features::scoring::score::score_role_unrounded;

use super::tactic::TacticLane;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FocusCategory {
    Physical,
    Technical,
    Mental,
}

pub(crate) fn category_for_age(age: Option<i64>) -> Option<FocusCategory> {
    match age {
        Some(age) if age < 21 => Some(FocusCategory::Physical),
        Some(21..=24) => Some(FocusCategory::Technical),
        Some(25..=28) => Some(FocusCategory::Mental),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrainingFocus {
    name: &'static str,
    /// FM27's ID for the focus, which Apply in FM sends.
    fm_id: i32,
    attributes: &'static [&'static str],
    category: FocusCategory,
}

/// FM27's outfield additional focuses in its dropdown order (order is the tie-break), each with
/// the attributes FM lists for it as dump keys. See `.wiki/notes/fm27-training-actions.md`.
const OUTFIELD_FOCUSES: &[TrainingFocus] = &[
    TrainingFocus {
        name: "Quickness",
        fm_id: 9,
        attributes: &["Pace", "Acceleration"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "Agility and Balance",
        fm_id: 10,
        attributes: &["Agility", "Balance"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "Resistance",
        fm_id: 11,
        attributes: &["Strength", "JumpingReach"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "Endurance",
        fm_id: 12,
        attributes: &["Stamina", "WorkRate"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "Chance Conversion",
        fm_id: 13,
        attributes: &[
            "Finishing",
            "Heading",
            "LongShots",
            "Technique",
            "Anticipation",
            "Composure",
            "OffTheBall",
        ],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Chance Creation",
        fm_id: 14,
        attributes: &[
            "Crossing",
            "Passing",
            "Technique",
            "Decisions",
            "Flair",
            "Vision",
        ],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Ball Retention",
        fm_id: 15,
        attributes: &[
            "FirstTouch",
            "Concentration",
            "Composure",
            "Decisions",
            "Teamwork",
        ],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "Ball Progression",
        fm_id: 16,
        attributes: &[
            "Dribbling",
            "Passing",
            "Anticipation",
            "Decisions",
            "Teamwork",
            "Vision",
        ],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "Pressing",
        fm_id: 17,
        attributes: &[
            "Tackling",
            "Aggression",
            "Anticipation",
            "Decisions",
            "WorkRate",
        ],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "Containment",
        fm_id: 18,
        attributes: &[
            "Marking",
            "Anticipation",
            "Concentration",
            "Positioning",
            "Teamwork",
        ],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "Defensive Positioning",
        fm_id: 19,
        attributes: &["Concentration", "Positioning", "Decisions"],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "Attacking Movement",
        fm_id: 20,
        attributes: &["OffTheBall", "Anticipation", "Decisions"],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "Final Third",
        fm_id: 21,
        attributes: &["Composure", "Decisions", "Vision"],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "Shooting",
        fm_id: 22,
        attributes: &["Finishing", "LongShots", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Passing",
        fm_id: 23,
        attributes: &["Passing", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Crossing",
        fm_id: 24,
        attributes: &["Crossing", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Ball Control",
        fm_id: 25,
        attributes: &["FirstTouch", "Dribbling", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Tackling",
        fm_id: 26,
        attributes: &["Tackling", "Aggression"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Marking",
        fm_id: 27,
        attributes: &["Marking", "Concentration"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Aerial",
        fm_id: 28,
        attributes: &["Heading", "Bravery"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Free Kick Taking",
        fm_id: 5,
        attributes: &["FreeKicks", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Corner Taking",
        fm_id: 6,
        attributes: &["Corners", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Penalty Taking",
        fm_id: 7,
        attributes: &["PenaltyTaking", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Long Throws",
        fm_id: 8,
        attributes: &["LongThrows"],
        category: FocusCategory::Technical,
    },
];

/// FM27's goalkeeper additional focuses in its dropdown order (order is the tie-break).
const GOALKEEPER_FOCUSES: &[TrainingFocus] = &[
    TrainingFocus {
        name: "Quickness",
        fm_id: 9,
        attributes: &["Pace", "Acceleration"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "Agility and Balance",
        fm_id: 10,
        attributes: &["Agility", "Balance"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "Resistance",
        fm_id: 11,
        attributes: &["Strength", "JumpingReach", "AerialReach"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "Endurance",
        fm_id: 12,
        attributes: &["Stamina", "WorkRate"],
        category: FocusCategory::Physical,
    },
    TrainingFocus {
        name: "GK Shot Stopping",
        fm_id: 29,
        attributes: &["Reflexes", "Anticipation", "Positioning", "Agility"],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "GK Handling",
        fm_id: 30,
        attributes: &["AerialReach", "Handling", "Concentration", "Balance"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "GK Sweeping",
        fm_id: 31,
        attributes: &["OneOnOnes", "Communication", "CommandOfArea", "RushingOut"],
        category: FocusCategory::Mental,
    },
    TrainingFocus {
        name: "GK Distribution",
        fm_id: 32,
        attributes: &[
            "Kicking",
            "Throwing",
            "FirstTouch",
            "Passing",
            "Composure",
            "Vision",
        ],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Free Kick Taking",
        fm_id: 5,
        attributes: &["FreeKicks", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Corner Taking",
        fm_id: 6,
        attributes: &["Corners", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Penalty Taking",
        fm_id: 7,
        attributes: &["PenaltyTaking", "Technique"],
        category: FocusCategory::Technical,
    },
    TrainingFocus {
        name: "Long Throws",
        fm_id: 8,
        attributes: &["LongThrows"],
        category: FocusCategory::Technical,
    },
];

/// FM27's ID for a suggested focus name.
pub(crate) fn fm_focus_id(name: &str) -> Option<i32> {
    OUTFIELD_FOCUSES
        .iter()
        .chain(GOALKEEPER_FOCUSES)
        .find(|focus| focus.name == name)
        .map(|focus| focus.fm_id)
}

/// The goalkeeper focus inventory applies only to the GK Planner lane.
pub fn is_goalkeeper_lane(lane: &TacticLane) -> bool {
    lane.lane_id == "goalkeeper"
}

/// Ranks the applicable inventory for an assigned lane and returns the winning
/// focus name.
///
/// A focus's gain is the unrounded combined-score gain of +1 in each attribute it
/// trains, divided by the number of attributes: FM spreads a focus's extra training
/// across them, so a broad focus earns less per attribute. The best gain stays local
/// to the ranking loop: it is never displayed or exported. Returns `None` when any attribute required
/// by a focus in the selected category is missing/null (no fallback focus),
/// when a lane role is unknown, or when a lane-role required attribute is
/// missing/null. All-zero gains still return the first category focus; ties
/// keep inventory order via a strict `>` comparison.
pub fn suggest_for_lane(
    attributes: &HashMap<String, Option<u8>>,
    lane: &TacticLane,
    category: FocusCategory,
) -> Option<&'static str> {
    let focuses = if is_goalkeeper_lane(lane) {
        GOALKEEPER_FOCUSES
    } else {
        OUTFIELD_FOCUSES
    };

    let focuses = focuses
        .iter()
        .filter(|focus| focus.category == category)
        .collect::<Vec<_>>();
    if focuses.is_empty() {
        return None;
    }
    for focus in &focuses {
        for key in focus.attributes.iter() {
            attributes.get(*key).copied().flatten()?;
        }
    }

    let roles = all_roles();
    let ip_role = roles.iter().find(|role| role.role_id == lane.ip_role_id)?;
    let oop_role = roles.iter().find(|role| role.role_id == lane.oop_role_id)?;
    let baseline = blended_score(attributes, ip_role, oop_role, lane.ip_weight)?;

    let mut best: Option<(&'static str, f64)> = None;
    for focus in &focuses {
        let mut simulated = attributes.clone();
        for key in focus.attributes.iter() {
            if let Some(value) = simulated.get(*key).copied().flatten() {
                if value < 20 {
                    simulated.insert((*key).to_string(), Some(value + 1));
                }
            }
        }
        let simulated_score = blended_score(&simulated, ip_role, oop_role, lane.ip_weight)?;
        let gain = (simulated_score - baseline) / focus.attributes.len() as f64;
        let improves = match &best {
            None => true,
            Some((_, current_gain)) => gain > *current_gain,
        };
        if improves {
            best = Some((focus.name, gain));
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

    use super::{
        category_for_age, fm_focus_id, is_goalkeeper_lane, suggest_for_lane, FocusCategory,
        TrainingFocus, GOALKEEPER_FOCUSES, OUTFIELD_FOCUSES,
    };
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
            custom_metric_keys: Vec::new(),
            recruitment_filters: Vec::new(),
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

    fn inventory(focuses: &[TrainingFocus]) -> Vec<(&'static str, i32, FocusCategory)> {
        focuses
            .iter()
            .map(|focus| (focus.name, focus.fm_id, focus.category))
            .collect()
    }

    #[test]
    fn inventories_follow_fm27_dropdown_order_ids_and_categories() {
        use FocusCategory::{Mental, Physical, Technical};
        let physical = [
            ("Quickness", 9, Physical),
            ("Agility and Balance", 10, Physical),
            ("Resistance", 11, Physical),
            ("Endurance", 12, Physical),
        ];
        let set_pieces = [
            ("Free Kick Taking", 5, Technical),
            ("Corner Taking", 6, Technical),
            ("Penalty Taking", 7, Technical),
            ("Long Throws", 8, Technical),
        ];
        let outfield = [
            ("Chance Conversion", 13, Technical),
            ("Chance Creation", 14, Technical),
            ("Ball Retention", 15, Mental),
            ("Ball Progression", 16, Mental),
            ("Pressing", 17, Mental),
            ("Containment", 18, Mental),
            ("Defensive Positioning", 19, Mental),
            ("Attacking Movement", 20, Mental),
            ("Final Third", 21, Mental),
            ("Shooting", 22, Technical),
            ("Passing", 23, Technical),
            ("Crossing", 24, Technical),
            ("Ball Control", 25, Technical),
            ("Tackling", 26, Technical),
            ("Marking", 27, Technical),
            ("Aerial", 28, Technical),
        ];
        let goalkeeper = [
            ("GK Shot Stopping", 29, Mental),
            ("GK Handling", 30, Technical),
            ("GK Sweeping", 31, Mental),
            ("GK Distribution", 32, Technical),
        ];

        let expected_outfield: Vec<_> = physical
            .iter()
            .chain(&outfield)
            .chain(&set_pieces)
            .copied()
            .collect();
        let expected_goalkeeper: Vec<_> = physical
            .iter()
            .chain(&goalkeeper)
            .chain(&set_pieces)
            .copied()
            .collect();
        assert_eq!(inventory(OUTFIELD_FOCUSES), expected_outfield);
        assert_eq!(inventory(GOALKEEPER_FOCUSES), expected_goalkeeper);
    }

    #[test]
    fn suggested_focus_names_map_to_fm27_ids() {
        assert_eq!(fm_focus_id("Chance Conversion"), Some(13));
        assert_eq!(fm_focus_id("GK Distribution"), Some(32));
        assert_eq!(fm_focus_id("Resistance"), Some(11));
        assert_eq!(fm_focus_id("Strength"), None);
    }

    #[test]
    fn every_mapped_key_is_a_known_dump_key() {
        for focus in OUTFIELD_FOCUSES.iter().chain(GOALKEEPER_FOCUSES.iter()) {
            for key in focus.attributes {
                assert!(
                    DUMP_ATTRIBUTE_KEYS.contains(key),
                    "focus `{}` maps unknown attribute `{key}`",
                    focus.name
                );
            }
        }
    }

    #[test]
    fn ranking_picks_the_largest_unrounded_gain() {
        let suggestion = suggest_for_lane(
            &full_attributes(10),
            &centre_forward_lane(),
            FocusCategory::Mental,
        )
        .expect("suggestion");

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
            suggest_for_lane(&attributes, &oop_only, FocusCategory::Mental),
            Some("Attacking Movement")
        );
        assert_eq!(
            suggest_for_lane(&attributes, &ip_only, FocusCategory::Technical),
            Some("Ball Control")
        );
    }

    #[test]
    fn goalkeeper_lane_uses_the_goalkeeper_inventory() {
        let attributes = full_attributes(10);

        let gk = suggest_for_lane(&attributes, &goalkeeper_lane(), FocusCategory::Mental)
            .expect("gk suggestion");
        assert_eq!(gk, "GK Shot Stopping");

        let outfield = suggest_for_lane(&attributes, &centre_forward_lane(), FocusCategory::Mental)
            .expect("outfield suggestion");
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

        assert_eq!(
            suggest_for_lane(&attributes, &lane, FocusCategory::Mental),
            None
        );
    }

    #[test]
    fn missing_selected_category_attribute_blanks_the_suggestion() {
        let mut attributes = full_attributes(10);
        attributes.insert("Corners".to_string(), None);

        assert_eq!(
            suggest_for_lane(
                &attributes,
                &centre_forward_lane(),
                FocusCategory::Technical
            ),
            None
        );
    }

    #[test]
    fn all_maxed_attributes_return_the_first_focus() {
        let suggestion = suggest_for_lane(
            &full_attributes(20),
            &centre_forward_lane(),
            FocusCategory::Technical,
        )
        .expect("suggestion");

        assert_eq!(suggestion, "Chance Conversion");
    }

    #[test]
    fn ranking_adapts_when_the_best_focus_attributes_are_maxed() {
        let suggestion = suggest_for_lane(
            &full_attributes(10),
            &centre_forward_lane(),
            FocusCategory::Mental,
        )
        .expect("suggestion");
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
        let suggestion = suggest_for_lane(
            &attributes,
            &centre_forward_lane(),
            FocusCategory::Technical,
        )
        .expect("suggestion");
        assert_eq!(suggestion, "Shooting");
    }

    #[test]
    fn age_categories_have_strict_boundaries() {
        assert_eq!(category_for_age(Some(20)), Some(FocusCategory::Physical));
        assert_eq!(category_for_age(Some(21)), Some(FocusCategory::Technical));
        assert_eq!(category_for_age(Some(24)), Some(FocusCategory::Technical));
        assert_eq!(category_for_age(Some(25)), Some(FocusCategory::Mental));
        assert_eq!(category_for_age(Some(28)), Some(FocusCategory::Mental));
        assert_eq!(category_for_age(Some(29)), None);
        assert_eq!(category_for_age(None), None);
    }

    #[test]
    fn missing_selected_category_attribute_returns_none() {
        let mut attributes = full_attributes(10);
        attributes.insert("Acceleration".to_string(), None);

        assert_eq!(
            suggest_for_lane(&attributes, &centre_forward_lane(), FocusCategory::Physical),
            None
        );
    }

    #[test]
    fn unknown_lane_roles_return_none() {
        let lane = lane("centre_forward", "not_a_role_ip", "not_a_role_oop", 0.5);

        assert_eq!(
            suggest_for_lane(&full_attributes(10), &lane, FocusCategory::Mental),
            None
        );
    }
}
