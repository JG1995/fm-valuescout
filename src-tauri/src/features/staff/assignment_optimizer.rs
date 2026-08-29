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

impl<'a> CandidateGroup<'a> {
    fn add_candidate(
        &mut self,
        candidate: &'a StaffAssignmentCandidate,
        score: Option<(u8, Option<CoachDiscipline>)>,
    ) {
        self.joined_candidate_count += 1;
        if let Some((score, coach_discipline)) = score {
            self.eligible.push(EligibleCandidate {
                candidate,
                score,
                coach_discipline,
            });
        } else {
            self.unavailable_score_count += 1;
        }
    }
}

struct ConfiguredSlot {
    target: StaffAssignmentTarget,
    slot_number: i64,
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

fn add_candidate_to_group<'a>(
    groups: &mut [CandidateGroup<'a>; 17],
    candidate: &'a StaffAssignmentCandidate,
    job_id: &str,
) {
    let group_index = canonical_job_index(job_id).expect("mapped job is canonical");
    groups[group_index].add_candidate(candidate, score_for_job(&candidate.scores, job_id));
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
        match preferred_job {
            PreferredJob::Manager => add_candidate_to_group(&mut groups, candidate, "manager"),
            PreferredJob::AssistantManager => {
                add_candidate_to_group(&mut groups, candidate, "assistant_manager")
            }
            PreferredJob::Coach => add_candidate_to_group(&mut groups, candidate, "coaches"),
            PreferredJob::SetPieceCoach => {
                add_candidate_to_group(&mut groups, candidate, "set_piece_coach")
            }
            PreferredJob::HeadPerformanceAnalyst => {
                add_candidate_to_group(&mut groups, candidate, "head_performance_analyst")
            }
            PreferredJob::PerformanceAnalyst => {
                add_candidate_to_group(&mut groups, candidate, "head_performance_analyst");
                add_candidate_to_group(&mut groups, candidate, "performance_analyst");
            }
            PreferredJob::HeadOfYouthDevelopment => {
                add_candidate_to_group(&mut groups, candidate, "head_of_youth_development")
            }
            PreferredJob::DirectorOfFootball => {
                add_candidate_to_group(&mut groups, candidate, "director_of_football")
            }
            PreferredJob::TechnicalDirector => {
                add_candidate_to_group(&mut groups, candidate, "technical_director")
            }
            PreferredJob::LoanManager => {
                add_candidate_to_group(&mut groups, candidate, "loan_manager")
            }
            PreferredJob::Scout => {
                add_candidate_to_group(&mut groups, candidate, "chief_scout");
                add_candidate_to_group(&mut groups, candidate, "scout");
            }
            PreferredJob::RecruitmentAnalyst => {
                add_candidate_to_group(&mut groups, candidate, "recruitment_analyst")
            }
            PreferredJob::Physio => {
                add_candidate_to_group(&mut groups, candidate, "head_physio");
                add_candidate_to_group(&mut groups, candidate, "physio");
            }
            PreferredJob::SportsScientist => {
                add_candidate_to_group(&mut groups, candidate, "head_sports_science");
                add_candidate_to_group(&mut groups, candidate, "sports_scientist");
            }
            PreferredJob::FitnessCoach | PreferredJob::GoalkeepingCoach => {}
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
    let configured_slots = ordered_targets
        .into_iter()
        .filter(|target| {
            !(target.job_id == "manager" && !matches!(target.scope.as_str(), "reserves" | "youth"))
                && canonical_job_index(&target.job_id).is_some()
        })
        .flat_map(|target| {
            (1..=target.slot_count).map(move |slot_number| ConfiguredSlot {
                target: target.clone(),
                slot_number,
            })
        })
        .collect::<Vec<_>>();

    let mut assigned_uids = HashSet::new();
    let mut next_candidate = [0_usize; 17];
    let mut slots = std::iter::repeat_with(|| None)
        .take(configured_slots.len())
        .collect::<Vec<Option<StaffAssignmentSlot>>>();
    for job_id in [
        "head_performance_analyst",
        "chief_scout",
        "head_physio",
        "head_sports_science",
    ] {
        allocate_job_slots(
            job_id,
            &configured_slots,
            &mut slots,
            &groups,
            &evidence,
            &mut next_candidate,
            &mut assigned_uids,
        );
    }
    allocate_ordinary_slots(
        &configured_slots,
        &mut slots,
        &groups,
        &evidence,
        &mut next_candidate,
        &mut assigned_uids,
    );
    for (index, configured_slot) in configured_slots.iter().enumerate() {
        if slots[index].is_some()
            || configured_slot.target.job_id == "coaches"
            || is_lead_job(&configured_slot.target.job_id)
            || is_ordinary_job(&configured_slot.target.job_id)
        {
            continue;
        }
        let group_index = canonical_job_index(&configured_slot.target.job_id)
            .expect("configured target is canonical");
        slots[index] = Some(allocate_slot(
            configured_slot,
            &groups[group_index],
            &evidence[group_index],
            &mut next_candidate[group_index],
            &mut assigned_uids,
        ));
    }
    allocate_job_slots(
        "coaches",
        &configured_slots,
        &mut slots,
        &groups,
        &evidence,
        &mut next_candidate,
        &mut assigned_uids,
    );

    StaffAssignmentAllocation {
        slots: slots
            .into_iter()
            .map(|slot| slot.expect("every configured slot is allocated"))
            .collect(),
        evidence,
    }
}

fn is_lead_job(job_id: &str) -> bool {
    matches!(
        job_id,
        "head_performance_analyst" | "chief_scout" | "head_physio" | "head_sports_science"
    )
}

fn is_ordinary_job(job_id: &str) -> bool {
    matches!(
        job_id,
        "performance_analyst" | "scout" | "physio" | "sports_scientist"
    )
}

fn allocate_ordinary_slots(
    configured_slots: &[ConfiguredSlot],
    slots: &mut [Option<StaffAssignmentSlot>],
    groups: &[CandidateGroup<'_>; 17],
    evidence: &[StaffAssignmentEvidence],
    next_candidate: &mut [usize; 17],
    assigned_uids: &mut HashSet<i64>,
) {
    for (slot, configured_slot) in slots.iter_mut().zip(configured_slots) {
        if !is_ordinary_job(&configured_slot.target.job_id) {
            continue;
        }
        let group_index = canonical_job_index(&configured_slot.target.job_id)
            .expect("ordinary target is canonical");
        *slot = Some(allocate_slot(
            configured_slot,
            &groups[group_index],
            &evidence[group_index],
            &mut next_candidate[group_index],
            assigned_uids,
        ));
    }
}

fn allocate_job_slots(
    job_id: &str,
    configured_slots: &[ConfiguredSlot],
    slots: &mut [Option<StaffAssignmentSlot>],
    groups: &[CandidateGroup<'_>; 17],
    evidence: &[StaffAssignmentEvidence],
    next_candidate: &mut [usize; 17],
    assigned_uids: &mut HashSet<i64>,
) {
    let group_index = canonical_job_index(job_id).expect("allocated job is canonical");
    for (slot, configured_slot) in slots.iter_mut().zip(configured_slots) {
        if configured_slot.target.job_id != job_id {
            continue;
        }
        *slot = Some(allocate_slot(
            configured_slot,
            &groups[group_index],
            &evidence[group_index],
            &mut next_candidate[group_index],
            assigned_uids,
        ));
    }
}

fn allocate_slot(
    configured_slot: &ConfiguredSlot,
    group: &CandidateGroup<'_>,
    evidence: &StaffAssignmentEvidence,
    next_candidate: &mut usize,
    assigned_uids: &mut HashSet<i64>,
) -> StaffAssignmentSlot {
    while *next_candidate < group.eligible.len()
        && assigned_uids.contains(&group.eligible[*next_candidate].candidate.uid)
    {
        *next_candidate += 1;
    }
    if let Some(eligible) = group.eligible.get(*next_candidate) {
        *next_candidate += 1;
        assigned_uids.insert(eligible.candidate.uid);
        StaffAssignmentSlot::Recommendation(StaffAssignmentRecommendation {
            scope: configured_slot.target.scope.clone(),
            job_id: configured_slot.target.job_id.clone(),
            job_label: configured_slot.target.job_label.clone(),
            slot_number: configured_slot.slot_number,
            uid: eligible.candidate.uid,
            name: eligible.candidate.name.clone(),
            preferred_job: eligible.candidate.preferred_job.clone(),
            classification: eligible.candidate.classification,
            score: eligible.score,
            coach_discipline: eligible.coach_discipline,
        })
    } else {
        StaffAssignmentSlot::Vacancy(StaffAssignmentVacancy {
            scope: configured_slot.target.scope.clone(),
            job_id: configured_slot.target.job_id.clone(),
            job_label: configured_slot.target.job_label.clone(),
            slot_number: configured_slot.slot_number,
            evidence: evidence.clone(),
        })
    }
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
        "recruitment_analyst" => scores.recruitment_analyst,
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
