use std::collections::{HashSet, VecDeque};

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

const GENERAL_REQUIREMENTS: [CoachRequirement; 6] = [
    CoachRequirement::AttackingTechnical,
    CoachRequirement::AttackingTactical,
    CoachRequirement::DefendingTechnical,
    CoachRequirement::DefendingTactical,
    CoachRequirement::PossessionTechnical,
    CoachRequirement::PossessionTactical,
];
const MAX_GENERAL_COACH_SLOTS: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CoachRequirement {
    AttackingTechnical,
    AttackingTactical,
    DefendingTechnical,
    DefendingTactical,
    PossessionTechnical,
    PossessionTactical,
    Fitness,
    Goalkeeping,
}

impl CoachRequirement {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::AttackingTechnical => "attacking_technical",
            Self::AttackingTactical => "attacking_tactical",
            Self::DefendingTechnical => "defending_technical",
            Self::DefendingTactical => "defending_tactical",
            Self::PossessionTechnical => "possession_technical",
            Self::PossessionTactical => "possession_tactical",
            Self::Fitness => "fitness",
            Self::Goalkeeping => "goalkeeping",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CoachComposition {
    pub(super) general: usize,
    pub(super) fitness: usize,
    pub(super) goalkeeping: usize,
}

pub(super) fn coach_composition(count: usize) -> CoachComposition {
    coach_slot_types(count).into_iter().fold(
        CoachComposition {
            general: 0,
            fitness: 0,
            goalkeeping: 0,
        },
        |mut composition, requirement| {
            match requirement {
                CoachSlotType::General => composition.general += 1,
                CoachSlotType::Fitness => composition.fitness += 1,
                CoachSlotType::Goalkeeping => composition.goalkeeping += 1,
            }
            composition
        },
    )
}

#[derive(Clone, Copy)]
enum CoachSlotType {
    General,
    Fitness,
    Goalkeeping,
}

fn coach_slot_types(count: usize) -> Vec<CoachSlotType> {
    let mut requirements = Vec::with_capacity(count);
    requirements.extend(
        [
            CoachSlotType::General,
            CoachSlotType::Goalkeeping,
            CoachSlotType::Fitness,
            CoachSlotType::General,
            CoachSlotType::General,
            CoachSlotType::General,
            CoachSlotType::General,
            CoachSlotType::General,
        ]
        .into_iter()
        .take(count.min(8)),
    );
    let mut remaining = count.saturating_sub(8);
    while remaining > 0 {
        requirements.push(CoachSlotType::Goalkeeping);
        remaining -= 1;
        if remaining == 0 {
            break;
        }
        requirements.push(CoachSlotType::Fitness);
        remaining -= 1;
        for _ in 0..remaining.min(6) {
            requirements.push(CoachSlotType::General);
        }
        remaining = remaining.saturating_sub(6);
    }
    requirements
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaffAssignmentClassification {
    CurrentStaff,
    Recruitment,
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
    pub(super) coach_requirement: Option<CoachRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentVacancy {
    pub(super) scope: String,
    pub(super) job_id: String,
    pub(super) job_label: String,
    pub(super) slot_number: i64,
    pub(super) coach_requirement: Option<CoachRequirement>,
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
}

#[derive(Default)]
struct CandidateGroup<'a> {
    joined_candidate_count: usize,
    unavailable_score_count: usize,
    eligible: Vec<EligibleCandidate<'a>>,
}

impl<'a> CandidateGroup<'a> {
    fn add_candidate(&mut self, candidate: &'a StaffAssignmentCandidate, score: Option<u8>) {
        self.joined_candidate_count += 1;
        if let Some(score) = score {
            self.eligible.push(EligibleCandidate { candidate, score });
        } else {
            self.unavailable_score_count += 1;
        }
    }

    fn evidence(&self, job_id: &str) -> StaffAssignmentEvidence {
        StaffAssignmentEvidence {
            job_id: job_id.to_string(),
            joined_candidate_count: self.joined_candidate_count,
            eligible_score_count: self.eligible.len(),
            unavailable_score_count: self.unavailable_score_count,
        }
    }

    fn sort(&mut self) {
        self.eligible.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.candidate.uid.cmp(&right.candidate.uid))
        });
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
    let mut general_coaches = Vec::new();
    let mut fitness_coaches = CandidateGroup::default();
    let mut goalkeeping_coaches = CandidateGroup::default();
    for candidate in candidates {
        let Some(preferred_job) = preferred_job_classification(&candidate.preferred_job) else {
            continue;
        };
        match preferred_job {
            PreferredJob::Manager => add_candidate_to_group(&mut groups, candidate, "manager"),
            PreferredJob::AssistantManager => {
                add_candidate_to_group(&mut groups, candidate, "assistant_manager")
            }
            PreferredJob::Coach => general_coaches.push(candidate),
            PreferredJob::FitnessCoach => {
                fitness_coaches.add_candidate(candidate, candidate.scores.coach_fitness)
            }
            PreferredJob::GoalkeepingCoach => {
                goalkeeping_coaches.add_candidate(candidate, candidate.scores.coach_goalkeeping)
            }
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
        }
    }

    for group in &mut groups {
        group.sort();
    }
    fitness_coaches.sort();
    goalkeeping_coaches.sort();

    let evidence = groups
        .iter()
        .enumerate()
        .map(|(index, group)| group.evidence(CANONICAL_JOB_IDS[index]))
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
    allocate_coaches_slots(
        &configured_slots,
        &mut slots,
        &general_coaches,
        &fitness_coaches,
        &goalkeeping_coaches,
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
        recommendation(configured_slot, eligible.candidate, eligible.score, None)
    } else {
        vacancy(configured_slot, None, evidence.clone())
    }
}

fn allocate_coaches_slots(
    configured_slots: &[ConfiguredSlot],
    slots: &mut [Option<StaffAssignmentSlot>],
    general_coaches: &[&StaffAssignmentCandidate],
    fitness_coaches: &CandidateGroup<'_>,
    goalkeeping_coaches: &CandidateGroup<'_>,
    assigned_uids: &mut HashSet<i64>,
) {
    for scope in ["senior", "reserves", "youth"] {
        let coach_slots = configured_slots
            .iter()
            .enumerate()
            .filter_map(|(index, configured_slot)| {
                (configured_slot.target.scope == scope
                    && configured_slot.target.job_id == "coaches")
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        if coach_slots.is_empty() {
            continue;
        }
        let composition = coach_composition(coach_slots.len());
        let mut general =
            allocate_general_requirements(composition.general, general_coaches, assigned_uids)
                .into_iter();
        let mut next_fitness = 0;
        let mut next_goalkeeping = 0;
        for (slot_index, requirement) in coach_slots
            .iter()
            .copied()
            .zip(coach_slot_types(coach_slots.len()))
        {
            let configured_slot = &configured_slots[slot_index];
            slots[slot_index] = Some(match requirement {
                CoachSlotType::General => {
                    let assignment = match general.next() {
                        Some(assignment) => assignment,
                        None => unreachable!("General count matches composition"),
                    };
                    let evidence = general_evidence(general_coaches, assignment.requirement);
                    match assignment.candidate {
                        Some((candidate, score)) => {
                            assigned_uids.insert(candidate.uid);
                            recommendation(
                                configured_slot,
                                candidate,
                                score,
                                Some(assignment.requirement),
                            )
                        }
                        None => vacancy(configured_slot, Some(assignment.requirement), evidence),
                    }
                }
                CoachSlotType::Fitness => allocate_coach_requirement_slot(
                    configured_slot,
                    CoachRequirement::Fitness,
                    fitness_coaches,
                    &mut next_fitness,
                    assigned_uids,
                ),
                CoachSlotType::Goalkeeping => allocate_coach_requirement_slot(
                    configured_slot,
                    CoachRequirement::Goalkeeping,
                    goalkeeping_coaches,
                    &mut next_goalkeeping,
                    assigned_uids,
                ),
            });
        }
    }
}

fn allocate_coach_requirement_slot(
    configured_slot: &ConfiguredSlot,
    requirement: CoachRequirement,
    group: &CandidateGroup<'_>,
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
        recommendation(
            configured_slot,
            eligible.candidate,
            eligible.score,
            Some(requirement),
        )
    } else {
        vacancy(
            configured_slot,
            Some(requirement),
            group.evidence("coaches"),
        )
    }
}

fn recommendation(
    configured_slot: &ConfiguredSlot,
    candidate: &StaffAssignmentCandidate,
    score: u8,
    coach_requirement: Option<CoachRequirement>,
) -> StaffAssignmentSlot {
    StaffAssignmentSlot::Recommendation(StaffAssignmentRecommendation {
        scope: configured_slot.target.scope.clone(),
        job_id: configured_slot.target.job_id.clone(),
        job_label: configured_slot.target.job_label.clone(),
        slot_number: configured_slot.slot_number,
        uid: candidate.uid,
        name: candidate.name.clone(),
        preferred_job: candidate.preferred_job.clone(),
        classification: candidate.classification,
        score,
        coach_requirement,
    })
}

fn vacancy(
    configured_slot: &ConfiguredSlot,
    coach_requirement: Option<CoachRequirement>,
    evidence: StaffAssignmentEvidence,
) -> StaffAssignmentSlot {
    StaffAssignmentSlot::Vacancy(StaffAssignmentVacancy {
        scope: configured_slot.target.scope.clone(),
        job_id: configured_slot.target.job_id.clone(),
        job_label: configured_slot.target.job_label.clone(),
        slot_number: configured_slot.slot_number,
        coach_requirement,
        evidence,
    })
}

struct GeneralAssignment<'a> {
    requirement: CoachRequirement,
    candidate: Option<(&'a StaffAssignmentCandidate, u8)>,
}

fn allocate_general_requirements<'a>(
    count: usize,
    candidates: &[&'a StaffAssignmentCandidate],
    assigned_uids: &HashSet<i64>,
) -> Vec<GeneralAssignment<'a>> {
    let candidates = candidates
        .iter()
        .copied()
        .filter(|candidate| !assigned_uids.contains(&candidate.uid))
        .collect::<Vec<_>>();
    let matching = match_general_requirements(count, &candidates);
    matching
        .requirements
        .into_iter()
        .zip(matching.assignments)
        .map(|(requirement, candidate)| GeneralAssignment {
            requirement,
            candidate,
        })
        .collect()
}

fn general_evidence(
    candidates: &[&StaffAssignmentCandidate],
    requirement: CoachRequirement,
) -> StaffAssignmentEvidence {
    let eligible_score_count = candidates
        .iter()
        .filter(|candidate| score_for_requirement(&candidate.scores, requirement).is_some())
        .count();
    StaffAssignmentEvidence {
        job_id: "coaches".to_string(),
        joined_candidate_count: candidates.len(),
        eligible_score_count,
        unavailable_score_count: candidates.len() - eligible_score_count,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GeneralMatchObjective {
    filled_nodes: i16,
    total_score: i16,
    full_uid_tie_break: [i16; MAX_GENERAL_COACH_SLOTS],
    partial_selected_tie_break: [i16; GENERAL_REQUIREMENTS.len()],
    partial_uid_tie_break: [i16; GENERAL_REQUIREMENTS.len()],
}

impl Default for GeneralMatchObjective {
    fn default() -> Self {
        Self {
            filled_nodes: 0,
            total_score: 0,
            full_uid_tie_break: [0; MAX_GENERAL_COACH_SLOTS],
            partial_selected_tie_break: [0; GENERAL_REQUIREMENTS.len()],
            partial_uid_tie_break: [0; GENERAL_REQUIREMENTS.len()],
        }
    }
}

impl GeneralMatchObjective {
    fn add(mut self, contribution: GeneralMatchContribution) -> Self {
        self.filled_nodes += contribution.filled_nodes;
        self.total_score += contribution.total_score;
        match contribution.tie {
            GeneralMatchTie::None => {}
            GeneralMatchTie::Full {
                requirement_index,
                uid_delta,
            } => self.full_uid_tie_break[usize::from(requirement_index)] += uid_delta,
            GeneralMatchTie::Partial {
                requirement_index,
                selected_delta,
                uid_delta,
            } => {
                let index = usize::from(requirement_index);
                self.partial_selected_tie_break[index] += i16::from(selected_delta);
                self.partial_uid_tie_break[index] += uid_delta;
            }
        }
        self
    }

    fn is_better_than(self, current: Self) -> bool {
        let primary =
            (self.filled_nodes, self.total_score).cmp(&(current.filled_nodes, current.total_score));
        if !primary.is_eq() {
            return primary.is_gt();
        }
        let full = self.full_uid_tie_break.cmp(&current.full_uid_tie_break);
        if !full.is_eq() {
            return full.is_gt();
        }
        for requirement_index in 0..GENERAL_REQUIREMENTS.len() {
            let partial = (
                self.partial_selected_tie_break[requirement_index],
                self.partial_uid_tie_break[requirement_index],
            )
                .cmp(&(
                    current.partial_selected_tie_break[requirement_index],
                    current.partial_uid_tie_break[requirement_index],
                ));
            if !partial.is_eq() {
                return partial.is_gt();
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy)]
enum GeneralMatchTie {
    None,
    Full {
        requirement_index: u8,
        uid_delta: i16,
    },
    Partial {
        requirement_index: u8,
        selected_delta: i8,
        uid_delta: i16,
    },
}

impl GeneralMatchTie {
    fn negated(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Full {
                requirement_index,
                uid_delta,
            } => Self::Full {
                requirement_index,
                uid_delta: -uid_delta,
            },
            Self::Partial {
                requirement_index,
                selected_delta,
                uid_delta,
            } => Self::Partial {
                requirement_index,
                selected_delta: -selected_delta,
                uid_delta: -uid_delta,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GeneralMatchContribution {
    filled_nodes: i16,
    total_score: i16,
    tie: GeneralMatchTie,
}

impl GeneralMatchContribution {
    const NONE: Self = Self {
        filled_nodes: 0,
        total_score: 0,
        tie: GeneralMatchTie::None,
    };

    fn candidate(score: u8, tie: GeneralMatchTie) -> Self {
        Self {
            filled_nodes: 1,
            total_score: i16::from(score),
            tie,
        }
    }

    fn vacancy(tie: GeneralMatchTie) -> Self {
        Self { tie, ..Self::NONE }
    }

    fn negated(self) -> Self {
        Self {
            filled_nodes: -self.filled_nodes,
            total_score: -self.total_score,
            tie: self.tie.negated(),
        }
    }
}

#[derive(Debug, Clone)]
struct GeneralMatchEdge {
    to: usize,
    reverse: usize,
    capacity: usize,
    contribution: GeneralMatchContribution,
}

#[cfg(test)]
pub(super) fn general_match_edge_bytes() -> usize {
    std::mem::size_of::<GeneralMatchEdge>()
}

#[derive(Debug)]
struct GeneralMatchGraph {
    edges: Vec<Vec<GeneralMatchEdge>>,
}

impl GeneralMatchGraph {
    fn new(node_count: usize) -> Self {
        Self {
            edges: vec![Vec::new(); node_count],
        }
    }

    fn add_edge(
        &mut self,
        from: usize,
        to: usize,
        capacity: usize,
        contribution: GeneralMatchContribution,
    ) {
        let edge_index = self.edges[from].len();
        let reverse_index = self.edges[to].len();
        self.edges[from].push(GeneralMatchEdge {
            to,
            reverse: reverse_index,
            capacity,
            contribution,
        });
        self.edges[to].push(GeneralMatchEdge {
            to: from,
            reverse: edge_index,
            capacity: 0,
            contribution: contribution.negated(),
        });
    }

    fn send_max_flow(&mut self, source: usize, sink: usize, target_flow: usize) -> usize {
        let mut flow = 0;
        while flow < target_flow {
            let mut best = vec![None; self.edges.len()];
            let mut previous = vec![None; self.edges.len()];
            let mut queued = vec![false; self.edges.len()];
            let mut queue = VecDeque::from([source]);
            best[source] = Some(GeneralMatchObjective::default());
            queued[source] = true;
            while let Some(node) = queue.pop_front() {
                queued[node] = false;
                let Some(node_objective) = best[node] else {
                    continue;
                };
                for (edge_index, edge) in self.edges[node].iter().enumerate() {
                    if edge.capacity == 0 {
                        continue;
                    }
                    let objective = node_objective.add(edge.contribution);
                    if best[edge.to].is_some_and(|current| !objective.is_better_than(current)) {
                        continue;
                    }
                    best[edge.to] = Some(objective);
                    previous[edge.to] = Some((node, edge_index));
                    if !queued[edge.to] {
                        queue.push_back(edge.to);
                        queued[edge.to] = true;
                    }
                }
            }
            let Some(_) = best[sink] else {
                break;
            };
            let mut node = sink;
            while node != source {
                let (from, edge_index) = previous[node].expect("path reaches source");
                let reverse = self.edges[from][edge_index].reverse;
                self.edges[from][edge_index].capacity -= 1;
                self.edges[node][reverse].capacity += 1;
                node = from;
            }
            flow += 1;
        }
        flow
    }
}

struct GeneralMatch<'a> {
    requirements: Vec<CoachRequirement>,
    assignments: Vec<Option<(&'a StaffAssignmentCandidate, u8)>>,
}

fn match_general_requirements<'a>(
    count: usize,
    candidates: &[&'a StaffAssignmentCandidate],
) -> GeneralMatch<'a> {
    assert!(count <= MAX_GENERAL_COACH_SLOTS);
    let full_requirement_count = count - count % GENERAL_REQUIREMENTS.len();
    let partial_count = count % GENERAL_REQUIREMENTS.len();
    let source = 0;
    let full_requirement_start = source + 1;
    let partial_gate = full_requirement_start + full_requirement_count;
    let partial_requirement_start = partial_gate + 1;
    let candidate_start = partial_requirement_start + GENERAL_REQUIREMENTS.len();
    let sink = candidate_start + candidates.len();
    let mut graph = GeneralMatchGraph::new(sink + 1);
    let mut candidate_ranks = vec![0_i16; candidates.len()];
    let mut candidate_indices = (0..candidates.len()).collect::<Vec<_>>();
    candidate_indices.sort_by_key(|&index| candidates[index].uid);
    for (rank, candidate_index) in candidate_indices.into_iter().enumerate() {
        candidate_ranks[candidate_index] = i16::try_from(rank + 1).expect("candidate rank");
    }
    let vacancy_rank = i16::try_from(candidates.len() + 1).expect("supported candidate bound");

    for requirement_index in 0..full_requirement_count {
        let requirement = GENERAL_REQUIREMENTS[requirement_index % GENERAL_REQUIREMENTS.len()];
        let requirement_node = full_requirement_start + requirement_index;
        let requirement_index = u8::try_from(requirement_index).expect("general requirement index");
        graph.add_edge(source, requirement_node, 1, GeneralMatchContribution::NONE);
        graph.add_edge(
            requirement_node,
            sink,
            1,
            GeneralMatchContribution::vacancy(GeneralMatchTie::Full {
                requirement_index,
                uid_delta: -vacancy_rank,
            }),
        );
        for (candidate_index, candidate) in candidates.iter().enumerate() {
            let Some(score) = score_for_requirement(&candidate.scores, requirement) else {
                continue;
            };
            graph.add_edge(
                requirement_node,
                candidate_start + candidate_index,
                1,
                GeneralMatchContribution::candidate(
                    score,
                    GeneralMatchTie::Full {
                        requirement_index,
                        uid_delta: -candidate_ranks[candidate_index],
                    },
                ),
            );
        }
    }

    graph.add_edge(
        source,
        partial_gate,
        partial_count,
        GeneralMatchContribution::NONE,
    );
    for (requirement_index, requirement) in GENERAL_REQUIREMENTS.iter().copied().enumerate() {
        let requirement_node = partial_requirement_start + requirement_index;
        let requirement_index = u8::try_from(requirement_index).expect("partial requirement index");
        graph.add_edge(
            partial_gate,
            requirement_node,
            1,
            GeneralMatchContribution::NONE,
        );
        graph.add_edge(
            requirement_node,
            sink,
            1,
            GeneralMatchContribution::vacancy(GeneralMatchTie::Partial {
                requirement_index,
                selected_delta: 1,
                uid_delta: -vacancy_rank,
            }),
        );
        for (candidate_index, candidate) in candidates.iter().enumerate() {
            let Some(score) = score_for_requirement(&candidate.scores, requirement) else {
                continue;
            };
            graph.add_edge(
                requirement_node,
                candidate_start + candidate_index,
                1,
                GeneralMatchContribution::candidate(
                    score,
                    GeneralMatchTie::Partial {
                        requirement_index,
                        selected_delta: 1,
                        uid_delta: -candidate_ranks[candidate_index],
                    },
                ),
            );
        }
    }
    for candidate_index in 0..candidates.len() {
        graph.add_edge(
            candidate_start + candidate_index,
            sink,
            1,
            GeneralMatchContribution::NONE,
        );
    }

    debug_assert_eq!(graph.send_max_flow(source, sink, count), count);

    let candidate_for_requirement = |requirement_node: usize, requirement: CoachRequirement| {
        graph.edges[requirement_node].iter().find_map(|edge| {
            (edge.to >= candidate_start
                && edge.to < candidate_start + candidates.len()
                && edge.capacity == 0)
                .then(|| {
                    let candidate = candidates[edge.to - candidate_start];
                    (
                        candidate,
                        score_for_requirement(&candidate.scores, requirement).expect("edge score"),
                    )
                })
        })
    };
    let mut requirements = Vec::with_capacity(count);
    let mut assignments = Vec::with_capacity(count);
    for requirement_index in 0..full_requirement_count {
        let requirement = GENERAL_REQUIREMENTS[requirement_index % GENERAL_REQUIREMENTS.len()];
        requirements.push(requirement);
        assignments.push(candidate_for_requirement(
            full_requirement_start + requirement_index,
            requirement,
        ));
    }
    for (requirement_index, requirement) in GENERAL_REQUIREMENTS.iter().copied().enumerate() {
        let requirement_node = partial_requirement_start + requirement_index;
        let selected = graph.edges[partial_gate]
            .iter()
            .any(|edge| edge.to == requirement_node && edge.capacity == 0);
        if selected {
            requirements.push(requirement);
            assignments.push(candidate_for_requirement(requirement_node, requirement));
        }
    }

    GeneralMatch {
        requirements,
        assignments,
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

fn score_for_job(scores: &StaffAssignmentScoreSet, job_id: &str) -> Option<u8> {
    match job_id {
        "manager" => scores.manager,
        "assistant_manager" => scores.assistant_manager,
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
    }
}

fn score_for_requirement(
    scores: &StaffAssignmentScoreSet,
    requirement: CoachRequirement,
) -> Option<u8> {
    match requirement {
        CoachRequirement::AttackingTechnical => scores.coach_attacking_technical,
        CoachRequirement::AttackingTactical => scores.coach_attacking_tactical,
        CoachRequirement::DefendingTechnical => scores.coach_defending_technical,
        CoachRequirement::DefendingTactical => scores.coach_defending_tactical,
        CoachRequirement::PossessionTechnical => scores.coach_possession_technical,
        CoachRequirement::PossessionTactical => scores.coach_possession_tactical,
        CoachRequirement::Fitness => scores.coach_fitness,
        CoachRequirement::Goalkeeping => scores.coach_goalkeeping,
    }
}
