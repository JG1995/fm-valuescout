use std::collections::HashSet;

use super::assignment_targets::StaffAssignmentTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PreferredJob {
    Manager,
    AssistantManager,
    Coach,
    FitnessCoach,
    GoalkeepingCoach,
    SetPieceCoach,
    HeadPerformanceAnalyst,
    PerformanceAnalyst,
    HeadOfYouthDevelopment,
    DirectorOfFootball,
    TechnicalDirector,
    LoanManager,
    Scout,
    RecruitmentAnalyst,
    Physio,
    SportsScientist,
}

const CANONICAL_JOB_IDS: [&str; 17] = [
    "manager",
    "assistant_manager",
    "coaches",
    "set_piece_coach",
    "performance_analyst",
    "physio",
    "sports_scientist",
    "head_of_youth_development",
    "head_performance_analyst",
    "director_of_football",
    "chief_scout",
    "technical_director",
    "scout",
    "recruitment_analyst",
    "loan_manager",
    "head_physio",
    "head_sports_science",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaffAssignmentClassification {
    CurrentStaff,
    Recruitment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CoachDiscipline {
    AttackingTechnical,
    AttackingTactical,
    DefendingTechnical,
    DefendingTactical,
    PossessionTechnical,
    PossessionTactical,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct StaffAssignmentScoreSet {
    pub(super) manager: Option<u8>,
    pub(super) assistant_manager: Option<u8>,
    pub(super) coach_attacking_technical: Option<u8>,
    pub(super) coach_attacking_tactical: Option<u8>,
    pub(super) coach_defending_technical: Option<u8>,
    pub(super) coach_defending_tactical: Option<u8>,
    pub(super) coach_possession_technical: Option<u8>,
    pub(super) coach_possession_tactical: Option<u8>,
    pub(super) coach_fitness: Option<u8>,
    pub(super) coach_goalkeeping: Option<u8>,
    pub(super) set_piece_coach: Option<u8>,
    pub(super) head_performance_analyst: Option<u8>,
    pub(super) performance_analyst: Option<u8>,
    pub(super) head_of_youth_development: Option<u8>,
    pub(super) director_of_football: Option<u8>,
    pub(super) technical_director: Option<u8>,
    pub(super) loan_manager: Option<u8>,
    pub(super) recruitment_analyst: Option<u8>,
    pub(super) scout: Option<u8>,
    pub(super) physio: Option<u8>,
    pub(super) sports_scientist: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentCandidate {
    pub(super) uid: i64,
    pub(super) name: Option<String>,
    pub(super) preferred_job: String,
    pub(super) classification: StaffAssignmentClassification,
    pub(super) scores: StaffAssignmentScoreSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentEvidence {
    pub(super) job_id: String,
    pub(super) joined_candidate_count: usize,
    pub(super) eligible_score_count: usize,
    pub(super) unavailable_score_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentRecommendation {
    pub(super) scope: String,
    pub(super) job_id: String,
    pub(super) job_label: String,
    pub(super) slot_number: i64,
    pub(super) uid: i64,
    pub(super) name: Option<String>,
    pub(super) preferred_job: String,
    pub(super) classification: StaffAssignmentClassification,
    pub(super) score: u8,
    pub(super) coach_discipline: Option<CoachDiscipline>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentVacancy {
    pub(super) scope: String,
    pub(super) job_id: String,
    pub(super) job_label: String,
    pub(super) slot_number: i64,
    pub(super) evidence: StaffAssignmentEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum StaffAssignmentSlot {
    Recommendation(StaffAssignmentRecommendation),
    Vacancy(StaffAssignmentVacancy),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentAllocation {
    pub(super) slots: Vec<StaffAssignmentSlot>,
    pub(super) evidence: Vec<StaffAssignmentEvidence>,
}

struct EligibleCandidate<'a> {
    candidate: &'a StaffAssignmentCandidate,
    score: u8,
    coach_discipline: Option<CoachDiscipline>,
}

#[derive(Default)]
struct CandidateGroup<'a> {
    joined_candidate_count: usize,
    unavailable_score_count: usize,
    eligible: Vec<EligibleCandidate<'a>>,
}

pub(super) fn preferred_job_classification(preferred_job: &str) -> Option<PreferredJob> {
    let preferred_job = preferred_job.trim();
    [
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
    ]
    .into_iter()
    .find_map(|(label, classification)| {
        label
            .eq_ignore_ascii_case(preferred_job)
            .then_some(classification)
    })
}

fn direct_target_job_id(preferred_job: PreferredJob) -> Option<&'static str> {
    match preferred_job {
        PreferredJob::Manager => Some("manager"),
        PreferredJob::AssistantManager => Some("assistant_manager"),
        PreferredJob::Coach => Some("coaches"),
        PreferredJob::SetPieceCoach => Some("set_piece_coach"),
        PreferredJob::HeadPerformanceAnalyst => Some("head_performance_analyst"),
        PreferredJob::PerformanceAnalyst => Some("performance_analyst"),
        PreferredJob::HeadOfYouthDevelopment => Some("head_of_youth_development"),
        PreferredJob::DirectorOfFootball => Some("director_of_football"),
        PreferredJob::TechnicalDirector => Some("technical_director"),
        PreferredJob::LoanManager => Some("loan_manager"),
        PreferredJob::Scout => Some("scout"),
        PreferredJob::Physio => Some("physio"),
        PreferredJob::SportsScientist => Some("sports_scientist"),
        PreferredJob::FitnessCoach
        | PreferredJob::GoalkeepingCoach
        | PreferredJob::RecruitmentAnalyst => None,
    }
}

pub(super) fn allocate_staff_assignments(
    targets: &[StaffAssignmentTarget],
    candidates: &[StaffAssignmentCandidate],
) -> StaffAssignmentAllocation {
    let mut groups = std::array::from_fn::<_, 17, _>(|_| CandidateGroup::default());
    for candidate in candidates {
        let Some(preferred_job) = preferred_job_classification(&candidate.preferred_job) else {
            continue;
        };
        let Some(job_id) = direct_target_job_id(preferred_job) else {
            continue;
        };
        let group = &mut groups[canonical_job_index(job_id).expect("mapped job is canonical")];
        group.joined_candidate_count += 1;
        if let Some((score, coach_discipline)) = score_for_job(&candidate.scores, job_id) {
            group.eligible.push(EligibleCandidate {
                candidate,
                score,
                coach_discipline,
            });
        } else {
            group.unavailable_score_count += 1;
        }
    }

    for group in &mut groups {
        group.eligible.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.candidate.uid.cmp(&right.candidate.uid))
        });
    }

    let evidence = groups
        .iter()
        .enumerate()
        .map(|(index, group)| StaffAssignmentEvidence {
            job_id: CANONICAL_JOB_IDS[index].to_string(),
            joined_candidate_count: group.joined_candidate_count,
            eligible_score_count: group.eligible.len(),
            unavailable_score_count: group.unavailable_score_count,
        })
        .collect::<Vec<_>>();
    let mut ordered_targets = targets.to_vec();
    ordered_targets.sort_by_key(|target| {
        (
            canonical_scope_rank(&target.scope),
            canonical_job_rank(&target.job_id),
        )
    });

    let mut assigned_uids = HashSet::new();
    let mut next_candidate = [0_usize; 17];
    let mut slots = Vec::new();
    for target in ordered_targets {
        if target.job_id == "manager" && !matches!(target.scope.as_str(), "reserves" | "youth") {
            continue;
        }
        let Some(group_index) = canonical_job_index(&target.job_id) else {
            continue;
        };
        for slot_number in 1..=target.slot_count {
            let group = &groups[group_index];
            while next_candidate[group_index] < group.eligible.len()
                && assigned_uids
                    .contains(&group.eligible[next_candidate[group_index]].candidate.uid)
            {
                next_candidate[group_index] += 1;
            }
            if let Some(eligible) = group.eligible.get(next_candidate[group_index]) {
                next_candidate[group_index] += 1;
                assigned_uids.insert(eligible.candidate.uid);
                slots.push(StaffAssignmentSlot::Recommendation(
                    StaffAssignmentRecommendation {
                        scope: target.scope.clone(),
                        job_id: target.job_id.clone(),
                        job_label: target.job_label.clone(),
                        slot_number,
                        uid: eligible.candidate.uid,
                        name: eligible.candidate.name.clone(),
                        preferred_job: eligible.candidate.preferred_job.clone(),
                        classification: eligible.candidate.classification,
                        score: eligible.score,
                        coach_discipline: eligible.coach_discipline,
                    },
                ));
            } else {
                slots.push(StaffAssignmentSlot::Vacancy(StaffAssignmentVacancy {
                    scope: target.scope.clone(),
                    job_id: target.job_id.clone(),
                    job_label: target.job_label.clone(),
                    slot_number,
                    evidence: evidence[group_index].clone(),
                }));
            }
        }
    }

    StaffAssignmentAllocation { slots, evidence }
}

fn canonical_job_index(job_id: &str) -> Option<usize> {
    CANONICAL_JOB_IDS
        .iter()
        .position(|candidate| *candidate == job_id)
}

fn canonical_scope_rank(scope: &str) -> usize {
    match scope {
        "senior" => 0,
        "reserves" => 1,
        "youth" => 2,
        "club" => 3,
        _ => usize::MAX,
    }
}

fn canonical_job_rank(job_id: &str) -> usize {
    canonical_job_index(job_id).unwrap_or(usize::MAX)
}

fn score_for_job(
    scores: &StaffAssignmentScoreSet,
    job_id: &str,
) -> Option<(u8, Option<CoachDiscipline>)> {
    let score = match job_id {
        "manager" => scores.manager,
        "assistant_manager" => scores.assistant_manager,
        "coaches" => return highest_coaching_score(scores),
        "set_piece_coach" => scores.set_piece_coach,
        "head_performance_analyst" => scores.head_performance_analyst,
        "performance_analyst" => scores.performance_analyst,
        "head_of_youth_development" => scores.head_of_youth_development,
        "director_of_football" => scores.director_of_football,
        "technical_director" => scores.technical_director,
        "loan_manager" => scores.loan_manager,
        "chief_scout" | "scout" => scores.scout,
        "head_physio" | "physio" => scores.physio,
        "head_sports_science" | "sports_scientist" => scores.sports_scientist,
        _ => None,
    }?;
    Some((score, None))
}

fn highest_coaching_score(
    scores: &StaffAssignmentScoreSet,
) -> Option<(u8, Option<CoachDiscipline>)> {
    let mut selected = None;
    for (score, discipline) in [
        (
            scores.coach_attacking_technical,
            CoachDiscipline::AttackingTechnical,
        ),
        (
            scores.coach_attacking_tactical,
            CoachDiscipline::AttackingTactical,
        ),
        (
            scores.coach_defending_technical,
            CoachDiscipline::DefendingTechnical,
        ),
        (
            scores.coach_defending_tactical,
            CoachDiscipline::DefendingTactical,
        ),
        (
            scores.coach_possession_technical,
            CoachDiscipline::PossessionTechnical,
        ),
        (
            scores.coach_possession_tactical,
            CoachDiscipline::PossessionTactical,
        ),
    ] {
        if let Some(score) = score {
            if selected.map_or(true, |(best, _)| score > best) {
                selected = Some((score, Some(discipline)));
            }
        }
    }
    selected
}
