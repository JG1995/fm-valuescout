use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffRoleDefinition {
    pub role_id: &'static str,
    pub display_name: &'static str,
    pub attributes: &'static [&'static str],
}

const STAFF_ROLES: &[StaffRoleDefinition] = &[
    StaffRoleDefinition {
        role_id: "assistant_manager",
        display_name: "Assistant Manager",
        attributes: &[
            "ManManagement",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "manager",
        display_name: "Manager",
        attributes: &[
            "Motivating",
            "ManManagement",
            "JudgingPlayerAbility",
            "JudgingPlayerPotential",
            "TacticalKnowledge",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_attacking_technical",
        display_name: "Coach — Attacking Technical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Attacking",
            "Technical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_attacking_tactical",
        display_name: "Coach — Attacking Tactical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Attacking",
            "Tactical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_defending_technical",
        display_name: "Coach — Defending Technical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Defending",
            "Technical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_defending_tactical",
        display_name: "Coach — Defending Tactical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Defending",
            "Tactical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_possession_technical",
        display_name: "Coach — Possession Technical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Possession",
            "Technical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_possession_tactical",
        display_name: "Coach — Possession Tactical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Possession",
            "Tactical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_fitness",
        display_name: "Coach — Fitness",
        attributes: &["Authority", "Determination", "Motivating", "Fitness"],
    },
    StaffRoleDefinition {
        role_id: "coach_goalkeeping",
        display_name: "Coach — Goalkeeping",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "GoalkeepingDistribution",
            "GoalkeepingHandling",
            "GoalkeepingReflexes",
        ],
    },
    StaffRoleDefinition {
        role_id: "set_piece_coach",
        display_name: "Set Piece Coach",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "SetPieces",
            "TacticalKnowledge",
        ],
    },
    StaffRoleDefinition {
        role_id: "loan_manager",
        display_name: "Loan Manager",
        attributes: &[
            "ManManagement",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "head_of_youth_development",
        display_name: "Head of Youth Development",
        attributes: &[
            "WorkingWithYoungsters",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "scout",
        display_name: "Scout",
        attributes: &[
            "Adaptability",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "director_of_football",
        display_name: "Director of Football",
        attributes: &[
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
            "Negotiating",
        ],
    },
    StaffRoleDefinition {
        role_id: "technical_director",
        display_name: "Technical Director",
        attributes: &["JudgingStaffAbility", "Negotiating"],
    },
    StaffRoleDefinition {
        role_id: "recruitment_analyst",
        display_name: "Recruitment Analyst",
        attributes: &["DataAnalysis", "JudgingPlayerAbility"],
    },
    StaffRoleDefinition {
        role_id: "head_performance_analyst",
        display_name: "Head Performance Analyst",
        attributes: &[
            "DataAnalysis",
            "Determination",
            "JudgingPlayerAbility",
            "TacticalKnowledge",
        ],
    },
    StaffRoleDefinition {
        role_id: "performance_analyst",
        display_name: "Performance Analyst",
        attributes: &["DataAnalysis", "TacticalKnowledge"],
    },
    StaffRoleDefinition {
        role_id: "physio",
        display_name: "Physio",
        attributes: &["Physiotherapy"],
    },
    StaffRoleDefinition {
        role_id: "sports_scientist",
        display_name: "Sports Scientist",
        attributes: &["SportsScience"],
    },
];

pub fn all_staff_roles() -> &'static [StaffRoleDefinition] {
    STAFF_ROLES
}

pub fn score_staff_role(
    attributes: &HashMap<String, Option<u8>>,
    role: &StaffRoleDefinition,
) -> Option<u8> {
    if role.attributes.is_empty() {
        return None;
    }
    let sum = role.attributes.iter().try_fold(0u32, |sum, key| {
        let value = attributes.get(*key).copied().flatten()?;
        (1..=20).contains(&value).then_some(sum + u32::from(value))
    })?;
    let mean = f64::from(sum) / role.attributes.len() as f64;
    Some((mean * 5.0).round() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(pairs: &[(&str, Option<u8>)]) -> HashMap<String, Option<u8>> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), *value))
            .collect()
    }

    fn role(role_id: &str) -> &'static StaffRoleDefinition {
        all_staff_roles()
            .iter()
            .find(|role| role.role_id == role_id)
            .expect("known staff role")
    }

    #[test]
    fn catalog_has_twenty_one_unique_stable_ids_and_exact_special_formulas() {
        let roles = all_staff_roles();
        let unique_ids = roles
            .iter()
            .map(|role| role.role_id)
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(roles.len(), 21);
        assert_eq!(unique_ids.len(), 21);
        let expected = [
            (
                "assistant_manager",
                &[
                    "ManManagement",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "manager",
                &[
                    "Motivating",
                    "ManManagement",
                    "JudgingPlayerAbility",
                    "JudgingPlayerPotential",
                    "TacticalKnowledge",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_attacking_technical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Attacking",
                    "Technical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_attacking_tactical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Attacking",
                    "Tactical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_defending_technical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Defending",
                    "Technical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_defending_tactical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Defending",
                    "Tactical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_possession_technical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Possession",
                    "Technical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_possession_tactical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Possession",
                    "Tactical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_fitness",
                &["Authority", "Determination", "Motivating", "Fitness"][..].as_ref(),
            ),
            (
                "coach_goalkeeping",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "GoalkeepingDistribution",
                    "GoalkeepingHandling",
                    "GoalkeepingReflexes",
                ][..]
                    .as_ref(),
            ),
            (
                "set_piece_coach",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "SetPieces",
                    "TacticalKnowledge",
                ][..]
                    .as_ref(),
            ),
            (
                "loan_manager",
                &[
                    "ManManagement",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "head_of_youth_development",
                &[
                    "WorkingWithYoungsters",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "scout",
                &[
                    "Adaptability",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "director_of_football",
                &[
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                    "Negotiating",
                ][..]
                    .as_ref(),
            ),
            (
                "technical_director",
                &["JudgingStaffAbility", "Negotiating"][..].as_ref(),
            ),
            (
                "recruitment_analyst",
                &["DataAnalysis", "JudgingPlayerAbility"][..].as_ref(),
            ),
            (
                "head_performance_analyst",
                &[
                    "DataAnalysis",
                    "Determination",
                    "JudgingPlayerAbility",
                    "TacticalKnowledge",
                ][..]
                    .as_ref(),
            ),
            (
                "performance_analyst",
                &["DataAnalysis", "TacticalKnowledge"][..].as_ref(),
            ),
            ("physio", &["Physiotherapy"][..].as_ref()),
            ("sports_scientist", &["SportsScience"][..].as_ref()),
        ];
        for (role_id, attributes) in expected {
            assert_eq!(
                role(role_id).attributes,
                *attributes,
                "formula for {role_id}"
            );
        }
    }

    #[test]
    fn scores_single_and_multi_attribute_roles_on_a_hundred_point_scale() {
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", Some(17))]), role("physio")),
            Some(85)
        );
        assert_eq!(
            score_staff_role(
                &attrs(&[
                    ("ManManagement", Some(10)),
                    ("JudgingPlayerPotential", Some(11)),
                    ("JudgingPlayerAbility", Some(12)),
                ]),
                role("assistant_manager"),
            ),
            Some(55)
        );
        assert_eq!(
            score_staff_role(
                &attrs(&[
                    ("Motivating", Some(10)),
                    ("ManManagement", Some(11)),
                    ("JudgingPlayerAbility", Some(12)),
                    ("JudgingPlayerPotential", Some(13)),
                    ("TacticalKnowledge", Some(14)),
                ]),
                role("manager"),
            ),
            Some(60)
        );
    }

    #[test]
    fn rounds_the_mean_after_scaling() {
        assert_eq!(
            score_staff_role(
                &attrs(&[("DataAnalysis", Some(10)), ("TacticalKnowledge", Some(11))]),
                role("performance_analyst"),
            ),
            Some(53)
        );
    }

    #[test]
    fn rejects_missing_null_and_out_of_range_inputs() {
        assert_eq!(score_staff_role(&HashMap::new(), role("physio")), None);
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", None)]), role("physio")),
            None
        );
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", Some(0))]), role("physio")),
            None
        );
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", Some(21))]), role("physio")),
            None
        );
    }
}
