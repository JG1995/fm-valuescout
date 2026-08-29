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
fn uses_the_three_approved_ordinary_scores_without_cross_job_substitution() {
    let candidates = [
        candidate(
            1,
            "Scout",
            StaffAssignmentScoreSet {
                scout: Some(71),
                ..Default::default()
            },
        ),
        candidate(
            2,
            "Physio",
            StaffAssignmentScoreSet {
                physio: Some(82),
                ..Default::default()
            },
        ),
        candidate(
            3,
            "Sports Scientist",
            StaffAssignmentScoreSet {
                sports_scientist: Some(93),
                ..Default::default()
            },
        ),
    ];

    let result = allocate_staff_assignments(
        &[
            target("club", "scout", "Scout", 1),
            target("senior", "physio", "Physio", 1),
            target("reserves", "sports_scientist", "Sports Scientist", 1),
        ],
        &candidates,
    );

    assert_eq!(assigned_uids(&result), [2, 3, 1]);
    assert!(matches!(
        &result.slots[0],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.score == 82 && recommendation.job_id == "physio"
    ));
    assert!(matches!(
        &result.slots[1],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.score == 93 && recommendation.job_id == "sports_scientist"
    ));
    assert!(matches!(
        &result.slots[2],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.score == 71 && recommendation.job_id == "scout"
    ));
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
        StaffAssignmentSlot::Vacancy(vacancy) if vacancy.job_id == "chief_scout"
    ));
    assert!(matches!(
        &result.slots[3],
        StaffAssignmentSlot::Recommendation(recommendation)
            if recommendation.classification == StaffAssignmentClassification::CurrentStaff
    ));
}

#[test]
fn recruitment_analyst_slots_remain_vacant_until_their_allocation_phase() {
    let candidates = [candidate(
        1,
        "Recruitment Analyst",
        StaffAssignmentScoreSet {
            recruitment_analyst: Some(99),
            ..Default::default()
        },
    )];
    let result = allocate_staff_assignments(
        &[target(
            "club",
            "recruitment_analyst",
            "Recruitment Analyst",
            2,
        )],
        &candidates,
    );

    assert_eq!(result.slots.len(), 2);
    assert!(result.slots.iter().enumerate().all(|(index, slot)| {
        matches!(slot, StaffAssignmentSlot::Vacancy(vacancy)
            if vacancy.job_id == "recruitment_analyst"
                && vacancy.slot_number == i64::try_from(index + 1).expect("slot number")
                && vacancy.evidence.joined_candidate_count == 0
                && vacancy.evidence.eligible_score_count == 0
                && vacancy.evidence.unavailable_score_count == 0)
    }));
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
