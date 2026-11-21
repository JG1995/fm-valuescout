use super::assignment_optimizer::{
    allocate_staff_assignments, coach_composition, general_match_edge_bytes,
    preferred_job_classification, recruit_teams, CoachRequirement, PreferredJob,
    StaffAssignmentAction, StaffAssignmentCandidate, StaffAssignmentClassification,
    StaffAssignmentJobSource, StaffAssignmentReplacement, StaffAssignmentScoreSet,
    StaffAssignmentSlot, StaffAssignmentSurplus, Teams,
};
use super::assignment_targets::StaffAssignmentTarget;

fn target(scope: &str, job_id: &str, job_label: &str, slot_count: i64) -> StaffAssignmentTarget {
    StaffAssignmentTarget {
        scope: scope.to_string(),
        job_id: job_id.to_string(),
        job_label: job_label.to_string(),
        section: String::new(),
        max_slot_count: 50,
        slot_count,
    }
}

fn candidate(
    uid: i64,
    preferred_job: &str,
    scores: StaffAssignmentScoreSet,
) -> StaffAssignmentCandidate {
    StaffAssignmentCandidate {
        uid,
        name: Some(format!("Staff {uid}")),
        preferred_job: preferred_job.to_string(),
        job_source: StaffAssignmentJobSource::PreferredJob,
        job: preferred_job_classification(preferred_job),
        classification: StaffAssignmentClassification::Recruitment,
        club: None,
        current_job_id: None,
        current_team: None,
        working_with_youngsters: Some(10),
        scores,
        teams: Teams::ALL,
    }
}

fn assigned_uids(result: &super::assignment_optimizer::StaffAssignmentAllocation) -> Vec<i64> {
    result
        .slots
        .iter()
        .filter_map(|slot| match slot {
            StaffAssignmentSlot::Recommendation(recommendation) => Some(recommendation.uid),
            StaffAssignmentSlot::Vacancy(_) => None,
        })
        .collect()
}

#[test]
fn classifies_only_the_approved_trimmed_ascii_case_insensitive_preferred_jobs() {
    let classifications = [
        ("Manager", PreferredJob::Manager),
        ("Assistant Manager", PreferredJob::AssistantManager),
        ("Coach", PreferredJob::Coach),
        ("Fitness Coach", PreferredJob::FitnessCoach),
        ("Goalkeeping Coach", PreferredJob::GoalkeepingCoach),
        ("Set Piece Coach", PreferredJob::SetPieceCoach),
        (
            "Head Performance Analyst",
            PreferredJob::HeadPerformanceAnalyst,
        ),
        ("Performance Analyst", PreferredJob::PerformanceAnalyst),
        (
            "Head of Youth Development",
            PreferredJob::HeadOfYouthDevelopment,
        ),
        ("Director of Football", PreferredJob::DirectorOfFootball),
        ("Technical Director", PreferredJob::TechnicalDirector),
        ("Loan Manager", PreferredJob::LoanManager),
        ("Scout", PreferredJob::Scout),
        ("Recruitment Analyst", PreferredJob::RecruitmentAnalyst),
        ("Physio", PreferredJob::Physio),
        ("Sports Scientist", PreferredJob::SportsScientist),
    ];

    for (preferred_job, classification) in classifications {
        assert_eq!(
            preferred_job_classification(preferred_job),
            Some(classification)
        );
        assert_eq!(
            preferred_job_classification(&format!("  {}  ", preferred_job.to_ascii_lowercase())),
            Some(classification)
        );
    }

    for unsupported in [
        "Chief Scout",
        "Head Physio",
        "Head of Sports Science",
        "Assistant Coach",
        "Coaches",
        "Chief Scout Assistant",
        "Head Physio Assistant",
        "Head of Sports Science Assistant",
        "Head Scout",
        "Managerial",
        "Coach/Analyst",
        "",
    ] {
        assert_eq!(
            preferred_job_classification(unsupported),
            None,
            "{unsupported}"
        );
    }
}

#[test]
fn six_coach_slots_use_the_exact_general_fitness_and_goalkeeping_pools() {
    let candidates = [
        candidate(
            1,
            "Coach",
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(91),
                coach_attacking_tactical: Some(91),
                coach_defending_technical: Some(91),
                coach_defending_tactical: Some(91),
                coach_possession_technical: Some(91),
                coach_possession_tactical: Some(91),
                ..Default::default()
            },
        ),
        candidate(
            2,
            "Fitness Coach",
            StaffAssignmentScoreSet {
                coach_fitness: Some(88),
                ..Default::default()
            },
        ),
        candidate(
            3,
            "Goalkeeping Coach",
            StaffAssignmentScoreSet {
                coach_goalkeeping: Some(87),
                ..Default::default()
            },
        ),
    ];

    let result =
        allocate_staff_assignments(&[target("senior", "coaches", "Coaches", 6)], &candidates);

    assert_eq!(assigned_uids(&result), [1, 3, 2]);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.coach_requirement == Some(CoachRequirement::AttackingTechnical)
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 3
                && recommendation.coach_requirement == Some(CoachRequirement::Goalkeeping)
    ));
    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 2
                && recommendation.coach_requirement == Some(CoachRequirement::Fitness)
    ));
}

#[test]
fn allocates_by_score_uid_scope_and_slot_order() {
    let candidates = [
        candidate(
            30,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(90),
                ..Default::default()
            },
        ),
        candidate(
            10,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(90),
                ..Default::default()
            },
        ),
        candidate(
            20,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(80),
                ..Default::default()
            },
        ),
    ];
    let targets = [
        target("youth", "assistant_manager", "Assistant Manager", 1),
        target("reserves", "assistant_manager", "Assistant Manager", 1),
        target("senior", "assistant_manager", "Assistant Manager", 1),
    ];

    let result = allocate_staff_assignments(&targets, &candidates);

    assert_eq!(assigned_uids(&result), [10, 30, 20]);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.scope == "senior" && recommendation.slot_number == 1
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.scope == "reserves" && recommendation.slot_number == 1
    ));
    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.scope == "youth" && recommendation.slot_number == 1
    ));
}

#[test]
fn youth_targets_blend_role_scores_with_working_with_youngsters() {
    let high_role_score = StaffAssignmentCandidate {
        working_with_youngsters: Some(2),
        ..candidate(
            1,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(90),
                ..Default::default()
            },
        )
    };
    let high_youth_score = StaffAssignmentCandidate {
        working_with_youngsters: Some(20),
        ..candidate(
            2,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(71),
                ..Default::default()
            },
        )
    };
    let missing_youth_attribute = StaffAssignmentCandidate {
        working_with_youngsters: None,
        ..candidate(
            3,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(100),
                ..Default::default()
            },
        )
    };

    let result = allocate_staff_assignments(
        &[target("youth", "assistant_manager", "Assistant Manager", 1)],
        &[high_role_score, high_youth_score, missing_youth_attribute],
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 2 && recommendation.score == 86
    ));
}

#[test]
fn youth_targets_leave_candidates_without_a_youth_attribute_unavailable() {
    let result = allocate_staff_assignments(
        &[target("youth", "assistant_manager", "Assistant Manager", 1)],
        &[StaffAssignmentCandidate {
            working_with_youngsters: None,
            ..candidate(
                1,
                "Assistant Manager",
                StaffAssignmentScoreSet {
                    assistant_manager: Some(100),
                    ..Default::default()
                },
            )
        }],
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.evidence.eligible_score_count == 0
                && vacancy.evidence.unavailable_score_count == 1
    ));
}

#[test]
fn youth_general_coaches_use_the_blended_score() {
    let high_role_score = StaffAssignmentCandidate {
        working_with_youngsters: Some(2),
        ..candidate(
            1,
            "Coach",
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(90),
                ..Default::default()
            },
        )
    };
    let high_youth_score = StaffAssignmentCandidate {
        working_with_youngsters: Some(20),
        ..candidate(
            2,
            "Coach",
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(70),
                ..Default::default()
            },
        )
    };

    let result = allocate_staff_assignments(
        &[target("youth", "coaches", "Coaches", 1)],
        &[high_role_score, high_youth_score],
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 2 && recommendation.score == 85
    ));
}

#[test]
fn youth_specialist_coaches_use_the_blended_score() {
    let high_role_score = StaffAssignmentCandidate {
        working_with_youngsters: Some(2),
        ..candidate(
            1,
            "Fitness Coach",
            StaffAssignmentScoreSet {
                coach_fitness: Some(90),
                ..Default::default()
            },
        )
    };
    let high_youth_score = StaffAssignmentCandidate {
        working_with_youngsters: Some(20),
        ..candidate(
            2,
            "Fitness Coach",
            StaffAssignmentScoreSet {
                coach_fitness: Some(70),
                ..Default::default()
            },
        )
    };

    let result = allocate_staff_assignments(
        &[target("youth", "coaches", "Coaches", 3)],
        &[high_role_score, high_youth_score],
    );

    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 2
                && recommendation.score == 85
                && recommendation.coach_requirement == Some(CoachRequirement::Fitness)
    ));
}

#[test]
fn club_head_of_youth_development_keeps_its_role_score() {
    let high_role_score = StaffAssignmentCandidate {
        working_with_youngsters: Some(2),
        ..candidate(
            1,
            "Head of Youth Development",
            StaffAssignmentScoreSet {
                head_of_youth_development: Some(90),
                ..Default::default()
            },
        )
    };
    let high_youth_attribute = StaffAssignmentCandidate {
        working_with_youngsters: Some(20),
        ..candidate(
            2,
            "Head of Youth Development",
            StaffAssignmentScoreSet {
                head_of_youth_development: Some(70),
                ..Default::default()
            },
        )
    };

    let result = allocate_staff_assignments(
        &[target(
            "club",
            "head_of_youth_development",
            "Head of Youth Development",
            1,
        )],
        &[high_role_score, high_youth_attribute],
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 1 && recommendation.score == 90
    ));
}

#[test]
fn assigns_manager_only_to_the_supported_reserves_and_youth_targets() {
    let candidates = [candidate(
        1,
        "Manager",
        StaffAssignmentScoreSet {
            manager: Some(90),
            ..Default::default()
        },
    )];
    let targets = [
        target("senior", "manager", "Manager", 1),
        target("reserves", "manager", "Manager", 1),
        target("youth", "manager", "Manager", 1),
        target("club", "manager", "Manager", 1),
    ];

    let result = allocate_staff_assignments(&targets, &candidates);

    assert_eq!(result.slots.len(), 2);
    assert_eq!(assigned_uids(&result), [1]);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.scope == "reserves"
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Vacancy(vacancy) if vacancy.scope == "youth"
    ));
}

#[test]
fn reserves_hpa_fallback_before_pa_slots_and_keeps_pa_only_ordinary_eligibility() {
    let candidates = [
        candidate(
            20,
            "Performance Analyst",
            StaffAssignmentScoreSet {
                head_performance_analyst: Some(99),
                performance_analyst: Some(50),
                ..Default::default()
            },
        ),
        candidate(
            10,
            "Performance Analyst",
            StaffAssignmentScoreSet {
                head_performance_analyst: Some(80),
                performance_analyst: Some(98),
                ..Default::default()
            },
        ),
        candidate(
            30,
            "Head Performance Analyst",
            StaffAssignmentScoreSet {
                head_performance_analyst: Some(90),
                performance_analyst: Some(100),
                ..Default::default()
            },
        ),
    ];

    let result = allocate_staff_assignments(
        &[
            target("senior", "performance_analyst", "Performance Analyst", 1),
            target(
                "club",
                "head_performance_analyst",
                "Head Performance Analyst",
                1,
            ),
        ],
        &candidates,
    );

    assert_eq!(assigned_uids(&result), [10, 20]);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.job_id == "performance_analyst"
                && recommendation.uid == 10
                && recommendation.score == 98
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.job_id == "head_performance_analyst"
                && recommendation.uid == 20
                && recommendation.score == 99
    ));
}

#[test]
fn ranks_hpa_leads_by_hpa_score_uid_and_leaves_missing_lead_scores_vacant() {
    let candidates = [
        candidate(
            40,
            "Performance Analyst",
            StaffAssignmentScoreSet {
                performance_analyst: Some(99),
                ..Default::default()
            },
        ),
        candidate(
            30,
            "Head Performance Analyst",
            StaffAssignmentScoreSet {
                head_performance_analyst: Some(80),
                ..Default::default()
            },
        ),
        candidate(
            20,
            "Head Performance Analyst",
            StaffAssignmentScoreSet {
                head_performance_analyst: Some(80),
                ..Default::default()
            },
        ),
    ];

    let result = allocate_staff_assignments(
        &[
            target("senior", "performance_analyst", "Performance Analyst", 2),
            target(
                "club",
                "head_performance_analyst",
                "Head Performance Analyst",
                1,
            ),
        ],
        &candidates,
    );

    assert_eq!(assigned_uids(&result), [40, 20]);
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.job_id == "performance_analyst"
                && vacancy.evidence.joined_candidate_count == 1
                && vacancy.evidence.eligible_score_count == 1
                && vacancy.evidence.unavailable_score_count == 0
    ));
    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.job_id == "head_performance_analyst"
                && recommendation.uid == 20
                && recommendation.score == 80
    ));
    let hpa_evidence = result
        .evidence
        .iter()
        .find(|evidence| evidence.job_id == "head_performance_analyst")
        .expect("HPA evidence");
    assert_eq!(hpa_evidence.joined_candidate_count, 3);
    assert_eq!(hpa_evidence.eligible_score_count, 2);
    assert_eq!(hpa_evidence.unavailable_score_count, 1);
}

#[test]
fn reserves_scout_physio_and_sports_science_leads_before_ordinary_slots() {
    let candidates = [
        candidate(
            1,
            "Scout",
            StaffAssignmentScoreSet {
                scout: Some(92),
                ..Default::default()
            },
        ),
        candidate(
            2,
            "Scout",
            StaffAssignmentScoreSet {
                scout: Some(81),
                ..Default::default()
            },
        ),
        candidate(
            3,
            "Physio",
            StaffAssignmentScoreSet {
                physio: Some(93),
                ..Default::default()
            },
        ),
        candidate(
            4,
            "Physio",
            StaffAssignmentScoreSet {
                physio: Some(82),
                ..Default::default()
            },
        ),
        candidate(
            5,
            "Sports Scientist",
            StaffAssignmentScoreSet {
                sports_scientist: Some(94),
                ..Default::default()
            },
        ),
        candidate(
            6,
            "Sports Scientist",
            StaffAssignmentScoreSet {
                sports_scientist: Some(83),
                ..Default::default()
            },
        ),
    ];

    let result = allocate_staff_assignments(
        &[
            target("club", "chief_scout", "Chief Scout", 1),
            target("club", "scout", "Scout", 1),
            target("senior", "physio", "Physio", 1),
            target("club", "head_physio", "Head Physio", 1),
            target("reserves", "sports_scientist", "Sports Scientist", 1),
            target("club", "head_sports_science", "Head of Sports Science", 1),
        ],
        &candidates,
    );

    assert_eq!(assigned_uids(&result), [4, 6, 1, 2, 3, 5]);
    assert!(result.slots.iter().all(|slot| matches!(
        slot,
        StaffAssignmentSlot::Recommendation(recommendation)
            if matches!(recommendation.job_id.as_str(),
                "chief_scout" | "scout" | "physio" | "head_physio" | "sports_scientist" | "head_sports_science")
    )));
}

#[test]
fn preserves_classification_enforces_one_duty_and_reports_unavailable_vacancies() {
    let current_staff = StaffAssignmentCandidate {
        classification: StaffAssignmentClassification::CurrentStaff,
        ..candidate(
            7,
            "Scout",
            StaffAssignmentScoreSet {
                scout: Some(99),
                ..Default::default()
            },
        )
    };
    let repeated_uid = candidate(
        7,
        "Scout",
        StaffAssignmentScoreSet {
            scout: Some(99),
            ..Default::default()
        },
    );
    let unavailable = candidate(8, "Assistant Manager", StaffAssignmentScoreSet::default());
    let zero_score = candidate(
        9,
        "Assistant Manager",
        StaffAssignmentScoreSet {
            assistant_manager: Some(0),
            ..Default::default()
        },
    );
    let targets = [
        target("club", "chief_scout", "Chief Scout", 1),
        target("club", "scout", "Scout", 1),
        target("senior", "assistant_manager", "Assistant Manager", 2),
    ];

    let result = allocate_staff_assignments(
        &targets,
        &[current_staff, repeated_uid, unavailable, zero_score],
    );

    assert_eq!(result.evidence.len(), 17);
    assert_eq!(assigned_uids(&result), [9, 7]);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.action == (StaffAssignmentAction::Recruit { club: None })
                && recommendation.score == 0
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.evidence.joined_candidate_count == 2
                && vacancy.evidence.eligible_score_count == 1
                && vacancy.evidence.unavailable_score_count == 1
    ));
    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.job_id == "chief_scout"
                && recommendation.action == StaffAssignmentAction::OnStaff
    ));
    assert!(matches!(
        &result.slots[3],
        StaffAssignmentSlot::Vacancy(vacancy) if vacancy.job_id == "scout"
    ));
}

#[test]
fn allocates_recruitment_analysts_only_from_their_preferred_job_and_persisted_score() {
    let candidates = [
        candidate(
            1,
            "Recruitment Analyst",
            StaffAssignmentScoreSet {
                recruitment_analyst: Some(99),
                ..Default::default()
            },
        ),
        candidate(2, "Recruitment Analyst", StaffAssignmentScoreSet::default()),
        candidate(
            3,
            "Scout",
            StaffAssignmentScoreSet {
                recruitment_analyst: Some(100),
                ..Default::default()
            },
        ),
    ];
    let result = allocate_staff_assignments(
        &[target(
            "club",
            "recruitment_analyst",
            "Recruitment Analyst",
            2,
        )],
        &candidates,
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 1 && recommendation.score == 99
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.job_id == "recruitment_analyst"
                && vacancy.slot_number == 2
                && vacancy.evidence.joined_candidate_count == 2
                && vacancy.evidence.eligible_score_count == 1
                && vacancy.evidence.unavailable_score_count == 1
    ));
}

#[test]
fn general_matching_keeps_residual_edges_compact_at_the_supported_candidate_bound() {
    assert!(
        general_match_edge_bytes() <= 64,
        "General matching stores two residual edges for each of up to 50 × 10,000 candidate pairs"
    );
}

#[test]
fn coach_composition_repeats_the_exact_jay_44_boundaries() {
    for (count, general, fitness, goalkeeping) in [
        (0, 0, 0, 0),
        (1, 1, 0, 0),
        (2, 1, 0, 1),
        (3, 1, 1, 1),
        (6, 4, 1, 1),
        (8, 6, 1, 1),
        (9, 6, 1, 2),
        (10, 6, 2, 2),
        (16, 12, 2, 2),
        (17, 12, 2, 3),
        (21, 15, 3, 3),
    ] {
        assert_eq!(
            coach_composition(count),
            super::assignment_optimizer::CoachComposition {
                general,
                fitness,
                goalkeeping,
            },
            "count {count}",
        );
    }
}

#[test]
fn general_matching_maximizes_filled_requirements_before_total_score() {
    let result = allocate_staff_assignments(
        &[target("senior", "coaches", "Coaches", 4)],
        &[
            candidate(
                10,
                "Coach",
                StaffAssignmentScoreSet {
                    coach_attacking_technical: Some(100),
                    coach_attacking_tactical: Some(0),
                    ..Default::default()
                },
            ),
            candidate(
                20,
                "Coach",
                StaffAssignmentScoreSet {
                    coach_attacking_technical: Some(99),
                    ..Default::default()
                },
            ),
        ],
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 20
                && recommendation.score == 99
                && recommendation.coach_requirement == Some(CoachRequirement::AttackingTechnical)
    ));
    assert!(matches!(
        &result.slots[3],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 10
                && recommendation.score == 0
                && recommendation.coach_requirement == Some(CoachRequirement::AttackingTactical)
    ));
}

#[test]
fn general_matching_selects_partial_disciplines_from_the_complete_cycle() {
    let result = allocate_staff_assignments(
        &[target("senior", "coaches", "Coaches", 1)],
        &[candidate(
            10,
            "Coach",
            StaffAssignmentScoreSet {
                coach_possession_tactical: Some(99),
                ..Default::default()
            },
        )],
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 10
                && recommendation.coach_requirement == Some(CoachRequirement::PossessionTactical)
    ));
}

#[test]
fn general_matching_orders_repeated_requirements_and_vacancies_by_cycle_discipline_and_uid() {
    let result = allocate_staff_assignments(
        &[target("senior", "coaches", "Coaches", 16)],
        &[
            candidate(
                20,
                "Coach",
                StaffAssignmentScoreSet {
                    coach_attacking_technical: Some(80),
                    ..Default::default()
                },
            ),
            candidate(
                10,
                "Coach",
                StaffAssignmentScoreSet {
                    coach_attacking_technical: Some(80),
                    ..Default::default()
                },
            ),
        ],
    );

    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 10
                && recommendation.coach_requirement == Some(CoachRequirement::AttackingTechnical)
    ));
    assert_eq!(
        result
            .slots
            .iter()
            .filter_map(|slot| {
                let requirement = match slot {
                    StaffAssignmentSlot::Recommendation(recommendation) => {
                        recommendation.coach_requirement
                    }
                    StaffAssignmentSlot::Vacancy(vacancy) => vacancy.coach_requirement,
                };
                (!matches!(
                    requirement,
                    Some(CoachRequirement::Fitness | CoachRequirement::Goalkeeping)
                ))
                .then_some(requirement)
            })
            .take(6)
            .collect::<Vec<_>>(),
        [
            Some(CoachRequirement::AttackingTechnical),
            Some(CoachRequirement::AttackingTactical),
            Some(CoachRequirement::DefendingTechnical),
            Some(CoachRequirement::DefendingTactical),
            Some(CoachRequirement::PossessionTechnical),
            Some(CoachRequirement::PossessionTactical),
        ],
    );
    assert!(matches!(
        &result.slots[3],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.coach_requirement == Some(CoachRequirement::AttackingTactical)
                && vacancy.evidence.joined_candidate_count == 2
                && vacancy.evidence.eligible_score_count == 0
                && vacancy.evidence.unavailable_score_count == 2
    ));
    assert!(matches!(
        &result.slots[10],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 20
                && recommendation.coach_requirement == Some(CoachRequirement::AttackingTechnical)
    ));
}

#[test]
fn coach_vacancy_evidence_uses_only_its_exact_preferred_job_and_score() {
    let result = allocate_staff_assignments(
        &[target("senior", "coaches", "Coaches", 8)],
        &[
            candidate(
                1,
                "Fitness Coach",
                StaffAssignmentScoreSet {
                    coach_goalkeeping: Some(99),
                    ..Default::default()
                },
            ),
            candidate(2, "Fitness Coach", StaffAssignmentScoreSet::default()),
            candidate(
                3,
                "Goalkeeping Coach",
                StaffAssignmentScoreSet {
                    coach_fitness: Some(99),
                    ..Default::default()
                },
            ),
        ],
    );

    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.coach_requirement == Some(CoachRequirement::Goalkeeping)
                && vacancy.evidence.joined_candidate_count == 1
                && vacancy.evidence.eligible_score_count == 0
                && vacancy.evidence.unavailable_score_count == 1
    ));
    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.coach_requirement == Some(CoachRequirement::Fitness)
                && vacancy.evidence.joined_candidate_count == 2
                && vacancy.evidence.eligible_score_count == 0
                && vacancy.evidence.unavailable_score_count == 2
    ));
}

#[test]
fn coach_allocation_keeps_senior_priority_and_global_uid_uniqueness() {
    let result = allocate_staff_assignments(
        &[
            target("reserves", "coaches", "Coaches", 1),
            target("senior", "coaches", "Coaches", 1),
        ],
        &[candidate(
            1,
            "Coach",
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(90),
                ..Default::default()
            },
        )],
    );

    assert_eq!(assigned_uids(&result), [1]);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation) if recommendation.scope == "senior"
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Vacancy(vacancy) if vacancy.scope == "reserves"
    ));
}

#[test]
fn retains_slot_order_through_the_fifty_slot_boundary() {
    let candidates = (51..=100)
        .rev()
        .map(|uid| {
            candidate(
                uid,
                "Physio",
                StaffAssignmentScoreSet {
                    physio: Some(75),
                    ..Default::default()
                },
            )
        })
        .collect::<Vec<_>>();

    let result =
        allocate_staff_assignments(&[target("senior", "physio", "Physio", 50)], &candidates);

    assert_eq!(result.slots.len(), 50);
    assert_eq!(assigned_uids(&result).first(), Some(&51));
    assert_eq!(assigned_uids(&result).last(), Some(&100));
    assert!(result.slots.iter().enumerate().all(|(index, slot)| {
        matches!(slot, StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.slot_number == i64::try_from(index + 1).expect("slot number"))
    }));
}

#[test]
fn lists_club_staff_who_lost_every_slot_they_could_fill() {
    let club_staff =
        |uid: i64, preferred_job: &str, current_job_id, scores| StaffAssignmentCandidate {
            classification: StaffAssignmentClassification::CurrentStaff,
            current_job_id: Some(current_job_id),
            ..candidate(uid, preferred_job, scores)
        };
    let candidates = [
        club_staff(
            1,
            "Assistant Manager",
            20,
            StaffAssignmentScoreSet {
                assistant_manager: Some(80),
                ..Default::default()
            },
        ),
        club_staff(
            2,
            "Assistant Manager",
            20,
            StaffAssignmentScoreSet {
                assistant_manager: Some(60),
                ..Default::default()
            },
        ),
        // A recruit is never on the club's list.
        candidate(
            3,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(50),
                ..Default::default()
            },
        ),
        // No physio slot is set, so the physio stays.
        club_staff(
            4,
            "Physio",
            12,
            StaffAssignmentScoreSet {
                physio: Some(90),
                ..Default::default()
            },
        ),
        // An unrated assistant was never compared.
        club_staff(
            5,
            "Assistant Manager",
            20,
            StaffAssignmentScoreSet::default(),
        ),
        // One coaching slot is a general one, so a goalkeeping coach has nothing to lose.
        club_staff(
            6,
            "Goalkeeping Coach",
            34,
            StaffAssignmentScoreSet {
                coach_goalkeeping: Some(70),
                ..Default::default()
            },
        ),
        club_staff(
            7,
            "Coach",
            2,
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(75),
                ..Default::default()
            },
        ),
        club_staff(
            8,
            "Coach",
            2,
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(65),
                ..Default::default()
            },
        ),
    ];
    let targets = [
        target("senior", "assistant_manager", "Assistant Manager", 1),
        target("senior", "coaches", "Coaches", 1),
    ];

    let result = allocate_staff_assignments(&targets, &candidates);

    assert_eq!(assigned_uids(&result), [1, 7]);
    assert_eq!(
        result.surplus,
        [
            StaffAssignmentSurplus {
                uid: 2,
                name: Some("Staff 2".to_string()),
                current_job: Some("Assistant Manager"),
                current_team: None,
                replaced_by: None,
            },
            StaffAssignmentSurplus {
                uid: 8,
                name: Some("Staff 8".to_string()),
                current_job: Some("Coach"),
                current_team: None,
                replaced_by: None,
            },
        ]
    );
}

fn club_assistant(uid: i64, team: Option<&'static str>, score: u8) -> StaffAssignmentCandidate {
    StaffAssignmentCandidate {
        classification: StaffAssignmentClassification::CurrentStaff,
        current_job_id: Some(20),
        current_team: team,
        ..candidate(
            uid,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(score),
                ..Default::default()
            },
        )
    }
}

fn recommendations(
    result: &super::assignment_optimizer::StaffAssignmentAllocation,
) -> Vec<(
    i64,
    String,
    StaffAssignmentAction,
    Option<StaffAssignmentReplacement>,
)> {
    result
        .slots
        .iter()
        .filter_map(|slot| match slot {
            StaffAssignmentSlot::Recommendation(recommendation) => Some((
                recommendation.uid,
                recommendation.scope.clone(),
                recommendation.action.clone(),
                recommendation.replaces.clone(),
            )),
            StaffAssignmentSlot::Vacancy(_) => None,
        })
        .collect()
}

#[test]
fn keeps_staff_on_their_own_team_and_names_whom_each_newcomer_replaces() {
    let candidates = [
        club_assistant(1, Some("reserves"), 90),
        club_assistant(2, Some("senior"), 50),
        candidate(
            3,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(70),
                ..Default::default()
            },
        ),
        club_assistant(4, Some("youth"), 60),
    ];
    let targets = [
        target("senior", "assistant_manager", "Assistant Manager", 1),
        target("reserves", "assistant_manager", "Assistant Manager", 1),
        target("youth", "assistant_manager", "Assistant Manager", 1),
    ];

    let result = allocate_staff_assignments(&targets, &candidates);

    let replacement = |uid: i64, moves_to: Option<(&str, &str)>| {
        Some(StaffAssignmentReplacement {
            uid,
            name: Some(format!("Staff {uid}")),
            moves_to: moves_to.map(|(scope, job)| (scope.to_string(), job.to_string())),
        })
    };
    assert_eq!(
        recommendations(&result),
        [
            // The reserve team's assistant moves up and takes the sacked senior assistant's job.
            (
                1,
                "senior".to_string(),
                StaffAssignmentAction::MoveTeam {
                    current_team: "reserves"
                },
                replacement(2, None),
            ),
            // The recruit takes the job the reserve team's assistant leaves.
            (
                3,
                "reserves".to_string(),
                StaffAssignmentAction::Recruit { club: None },
                replacement(1, Some(("senior", "Assistant Manager"))),
            ),
            (4, "youth".to_string(), StaffAssignmentAction::Keep, None),
        ]
    );
    assert_eq!(
        result.surplus,
        [StaffAssignmentSurplus {
            uid: 2,
            name: Some("Staff 2".to_string()),
            current_job: Some("Assistant Manager"),
            current_team: Some("senior"),
            replaced_by: Some((1, Some("Staff 1".to_string()))),
        }]
    );
}

#[test]
fn staff_whose_team_is_unknown_keep_their_job_on_any_team() {
    let result = allocate_staff_assignments(
        &[target(
            "reserves",
            "assistant_manager",
            "Assistant Manager",
            1,
        )],
        &[club_assistant(1, None, 80)],
    );

    assert_eq!(
        recommendations(&result),
        [(1, "reserves".to_string(), StaffAssignmentAction::Keep, None)]
    );
}

fn teams(senior: bool, reserves: bool, youth: bool) -> Teams {
    Teams {
        senior,
        reserves,
        youth,
    }
}

#[test]
fn each_team_with_a_shortlist_recruits_from_it_and_the_rest_from_no_teams_list() {
    let none = teams(false, false, false);
    let senior = teams(true, false, false);
    let senior_and_youth = teams(true, false, true);

    for (case, listed_by, with_shortlist, expected) in [
        ("no team shortlists", none, none, Teams::ALL),
        ("on the senior list", senior, senior, senior),
        ("on no team's list", none, senior, teams(false, true, true)),
        (
            "on two lists",
            senior_and_youth,
            senior_and_youth,
            senior_and_youth,
        ),
        ("on the senior list only", senior, senior_and_youth, senior),
        ("on no list of three", none, Teams::ALL, none),
    ] {
        assert_eq!(recruit_teams(listed_by, with_shortlist), expected, "{case}");
    }
}

#[test]
fn recruits_fill_only_the_teams_they_may_join_and_any_club_job() {
    let assistant = |uid, score, teams| StaffAssignmentCandidate {
        teams,
        ..candidate(
            uid,
            "Assistant Manager",
            StaffAssignmentScoreSet {
                assistant_manager: Some(score),
                ..Default::default()
            },
        )
    };
    let general_coach = |uid, score, teams| StaffAssignmentCandidate {
        teams,
        ..candidate(
            uid,
            "Coach",
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(score),
                ..Default::default()
            },
        )
    };
    let youth_only = teams(false, false, true);
    let candidates = [
        assistant(1, 90, youth_only),
        assistant(2, 80, teams(true, false, false)),
        assistant(3, 70, teams(false, true, true)),
        general_coach(4, 95, youth_only),
        general_coach(5, 60, teams(true, false, false)),
        StaffAssignmentCandidate {
            teams: youth_only,
            ..candidate(
                6,
                "Director of Football",
                StaffAssignmentScoreSet {
                    director_of_football: Some(50),
                    ..Default::default()
                },
            )
        },
    ];

    let result = allocate_staff_assignments(
        &[
            target("senior", "assistant_manager", "Assistant Manager", 1),
            target("senior", "coaches", "Coaches", 1),
            target("reserves", "assistant_manager", "Assistant Manager", 1),
            target("youth", "assistant_manager", "Assistant Manager", 1),
            target("club", "director_of_football", "Director of Football", 1),
        ],
        &candidates,
    );

    let placed = result
        .slots
        .iter()
        .map(|slot| match slot {
            StaffAssignmentSlot::Recommendation(recommendation) => (
                recommendation.scope.as_str(),
                recommendation.job_id.as_str(),
                Some(recommendation.uid),
            ),
            StaffAssignmentSlot::Vacancy(vacancy) => {
                (vacancy.scope.as_str(), vacancy.job_id.as_str(), None)
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        placed,
        [
            ("senior", "assistant_manager", Some(2)),
            ("senior", "coaches", Some(5)),
            ("reserves", "assistant_manager", Some(3)),
            ("youth", "assistant_manager", Some(1)),
            ("club", "director_of_football", Some(6)),
        ]
    );
}

#[test]
fn a_vacancy_counts_only_the_people_who_may_join_its_team() {
    let physio = |uid, teams| StaffAssignmentCandidate {
        teams,
        ..candidate(
            uid,
            "Physio",
            StaffAssignmentScoreSet {
                physio: Some(70),
                ..Default::default()
            },
        )
    };
    let fitness_coach = StaffAssignmentCandidate {
        teams: teams(false, false, true),
        ..candidate(
            3,
            "Fitness Coach",
            StaffAssignmentScoreSet {
                coach_fitness: Some(70),
                ..Default::default()
            },
        )
    };
    let general_coach = StaffAssignmentCandidate {
        teams: teams(false, false, true),
        ..candidate(
            4,
            "Coach",
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(70),
                ..Default::default()
            },
        )
    };
    let youth_only = teams(false, false, true);

    let result = allocate_staff_assignments(
        &[
            target("senior", "physio", "Physio", 1),
            target("senior", "coaches", "Coaches", 3),
        ],
        &[
            physio(1, youth_only),
            physio(2, youth_only),
            fitness_coach,
            general_coach,
        ],
    );

    let vacancy_counts = result
        .slots
        .iter()
        .map(|slot| match slot {
            StaffAssignmentSlot::Vacancy(vacancy) => (
                vacancy.job_id.as_str(),
                vacancy.evidence.joined_candidate_count,
                vacancy.evidence.eligible_score_count,
            ),
            StaffAssignmentSlot::Recommendation(recommendation) => {
                panic!(
                    "{} was recruited to a team they may not join",
                    recommendation.uid
                )
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        vacancy_counts,
        [
            ("coaches", 0, 0),
            ("coaches", 0, 0),
            ("coaches", 0, 0),
            ("physio", 0, 0),
        ]
    );
}
