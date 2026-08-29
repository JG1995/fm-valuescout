use super::assignment_optimizer::{
    allocate_staff_assignments, preferred_job_classification, CoachDiscipline, PreferredJob,
    StaffAssignmentCandidate, StaffAssignmentClassification, StaffAssignmentScoreSet,
    StaffAssignmentSlot,
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
fn coach_uses_the_highest_available_score_and_recorded_tie_order() {
    let candidates = [
        candidate(
            1,
            "Coach",
            StaffAssignmentScoreSet {
                coach_attacking_technical: Some(91),
                coach_attacking_tactical: Some(91),
                coach_defending_tactical: Some(90),
                ..Default::default()
            },
        ),
        candidate(
            2,
            "Coach",
            StaffAssignmentScoreSet {
                coach_possession_tactical: Some(99),
                ..Default::default()
            },
        ),
        candidate(3, "Coach", StaffAssignmentScoreSet::default()),
    ];

    let result =
        allocate_staff_assignments(&[target("senior", "coaches", "Coaches", 3)], &candidates);

    assert_eq!(result.evidence[2].joined_candidate_count, 3);
    assert_eq!(result.evidence[2].eligible_score_count, 2);
    assert_eq!(result.evidence[2].unavailable_score_count, 1);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 2
                && recommendation.score == 99
                && recommendation.coach_discipline == Some(CoachDiscipline::PossessionTactical)
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.uid == 1
                && recommendation.score == 91
                && recommendation.coach_discipline == Some(CoachDiscipline::AttackingTechnical)
    ));
    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.evidence.eligible_score_count == 2
                && vacancy.evidence.unavailable_score_count == 1
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
