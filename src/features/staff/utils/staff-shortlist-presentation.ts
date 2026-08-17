const BASIC_COLUMNS = [
  "name",
  "age",
  "nationality",
  "club",
  "ca",
  "pa",
  "preferred_job",
  "club_job",
  "coaching_qualifications",
];

const PREFERRED_JOB_SCORE_FIELDS: Record<string, string> = {
  "Assistant Manager": "role.assistant_manager",
  "Director of Football": "role.director_of_football",
  "Fitness Coach": "role.coach_fitness",
  "Goalkeeping Coach": "role.coach_goalkeeping",
  "Head of Performance Analysis": "role.head_performance_analyst",
  "Head of Youth Development": "role.head_of_youth_development",
  "Loan Manager": "role.loan_manager",
  "Performance Analyst": "role.performance_analyst",
  Physio: "role.physio",
  "Recruitment Analyst": "role.recruitment_analyst",
  Scout: "role.scout",
  "Set Piece Coach": "role.set_piece_coach",
  "Sports Scientist": "role.sports_scientist",
  "Technical Director": "role.technical_director",
};

const COACH_SCORE_FIELDS = [
  "role.coach_attacking_technical",
  "role.coach_attacking_tactical",
  "role.coach_defending_technical",
  "role.coach_defending_tactical",
  "role.coach_possession_technical",
  "role.coach_possession_tactical",
];

export function staffShortlistPresentation(preferredJob?: string) {
  if (!preferredJob) return undefined;
  if (preferredJob === "Coach") {
    return { columnIds: [...BASIC_COLUMNS, ...COACH_SCORE_FIELDS] };
  }
  const scoreField = PREFERRED_JOB_SCORE_FIELDS[preferredJob];
  if (!scoreField) {
    return { columnIds: BASIC_COLUMNS, sort: "ca", dir: "desc" as const };
  }
  return {
    columnIds: [...BASIC_COLUMNS, scoreField],
    sort: scoreField,
    dir: "desc" as const,
  };
}
