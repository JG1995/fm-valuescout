use super::assignment_optimizer::{
    allocate_staff_assignments, coach_composition, general_match_edge_bytes,
    preferred_job_classification, CoachRequirement, PreferredJob, StaffAssignmentCandidate,
    StaffAssignmentClassification, StaffAssignmentScoreSet, StaffAssignmentSlot,
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
        classification: StaffAssignmentClassification::Recruitment,
        scores,
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
            if recommendation.classification == StaffAssignmentClassification::Recruitment
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
                && recommendation.classification == StaffAssignmentClassification::CurrentStaff
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
