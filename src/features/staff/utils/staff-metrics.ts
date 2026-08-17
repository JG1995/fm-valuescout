export type StaffMetricKind = "string" | "integer";
export type StaffMetricAlignment = "left" | "right";
export type StaffMetricOperator = { id: string; label: string };

export type StaffMetric = {
  id: string;
  label: string;
  category: string;
  kind: StaffMetricKind;
  align: StaffMetricAlignment;
  defaultWidth: number;
  sortable: boolean;
  operators: readonly StaffMetricOperator[];
};

const STRING_OPERATORS: readonly StaffMetricOperator[] = [
  { id: "contains", label: "contains" },
  { id: "not_contains", label: "does not contain" },
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
];

const INTEGER_OPERATORS: readonly StaffMetricOperator[] = [
  { id: "gt", label: "greater than" },
  { id: "lt", label: "less than" },
  { id: "eq", label: "equals" },
  { id: "neq", label: "does not equal" },
];

const STAFF_ATTRIBUTE_KEYS = [
  "Attacking",
  "Defending",
  "Fitness",
  "Possession",
  "Technical",
  "Tactical",
  "SetPieces",
  "Determination",
  "ManManagement",
  "Motivating",
  "JudgingPlayerAbility",
  "JudgingPlayerPotential",
  "JudgingStaffAbility",
  "Negotiating",
  "TacticalKnowledge",
  "Physiotherapy",
  "SportsScience",
  "Authority",
  "Adaptability",
  "DataAnalysis",
  "WorkingWithYoungsters",
  "GoalkeepingDistribution",
  "GoalkeepingHandling",
  "GoalkeepingReflexes",
] as const;

const STAFF_ROLES = [
  ["assistant_manager", "Assistant Manager"],
  ["coach_attacking_technical", "Coach — Attacking Technical"],
  ["coach_attacking_tactical", "Coach — Attacking Tactical"],
  ["coach_defending_technical", "Coach — Defending Technical"],
  ["coach_defending_tactical", "Coach — Defending Tactical"],
  ["coach_possession_technical", "Coach — Possession Technical"],
  ["coach_possession_tactical", "Coach — Possession Tactical"],
  ["coach_fitness", "Coach — Fitness"],
  ["coach_goalkeeping", "Coach — Goalkeeping"],
  ["set_piece_coach", "Set Piece Coach"],
  ["loan_manager", "Loan Manager"],
  ["head_of_youth_development", "Head of Youth Development"],
  ["scout", "Scout"],
  ["director_of_football", "Director of Football"],
  ["technical_director", "Technical Director"],
  ["recruitment_analyst", "Recruitment Analyst"],
  ["head_performance_analyst", "Head Performance Analyst"],
  ["performance_analyst", "Performance Analyst"],
  ["physio", "Physio"],
  ["sports_scientist", "Sports Scientist"],
] as const;

function labelFromPascal(key: string): string {
  return key.replaceAll(/([a-z])([A-Z])/g, "$1 $2");
}

function integerMetric(
  id: string,
  label: string,
  category: string,
  defaultWidth = 96,
): StaffMetric {
  return {
    id,
    label,
    category,
    kind: "integer",
    align: "right",
    defaultWidth,
    sortable: true,
    operators: INTEGER_OPERATORS,
  };
}

function stringMetric(
  id: string,
  label: string,
  category: string,
  defaultWidth = 176,
): StaffMetric {
  return {
    id,
    label,
    category,
    kind: "string",
    align: "left",
    defaultWidth,
    sortable: true,
    operators: STRING_OPERATORS,
  };
}

export const STAFF_ROLE_METRICS: readonly StaffMetric[] = STAFF_ROLES.map(
  ([id, label]) =>
    integerMetric(`role.${id}`, label, "current-role-scores", 152),
);

export const STAFF_ATTRIBUTE_METRICS: readonly StaffMetric[] =
  STAFF_ATTRIBUTE_KEYS.map((key) =>
    integerMetric(`attr.${key}`, labelFromPascal(key), "staff-attributes", 112),
  );

const STAFF_BASIC_METRICS: readonly StaffMetric[] = [
  stringMetric("name", "Name", "identity"),
  integerMetric("age", "Age / DOB", "identity", 152),
  integerMetric("birth_year", "Birth year", "identity", 112),
  integerMetric("birth_day_of_year", "Birth day", "identity", 112),
  stringMetric("nationality", "Nation", "identity", 128),
  integerMetric("nation_uid", "Nation ID", "identity", 96),
  stringMetric("gender", "Gender", "identity", 112),
  stringMetric("club", "Club", "club-contract"),
  stringMetric("division", "Division", "club-contract"),
  integerMetric("ca", "CA", "ability-reputation"),
  integerMetric("pa", "PA", "ability-reputation"),
  integerMetric("wage", "Wage", "club-contract", 128),
  integerMetric("contract_year", "Contract expiry", "club-contract", 128),
  integerMetric("contract_day", "Contract day", "club-contract", 112),
  integerMetric("job_id", "Job ID", "identity", 96),
];

export const STAFF_BASIC_METRIC_IDS = STAFF_BASIC_METRICS.map(
  (metric) => metric.id,
);

export const STAFF_METRICS: readonly StaffMetric[] = [
  ...STAFF_BASIC_METRICS,
  ...STAFF_ATTRIBUTE_METRICS,
  ...STAFF_ROLE_METRICS,
];

export const STAFF_SHORTLIST_METRICS: readonly StaffMetric[] = [
  ...STAFF_METRICS,
  stringMetric("preferred_job", "Preferred Job", "shortlist", 160),
  stringMetric("club_job", "Club Job", "shortlist", 160),
  stringMetric(
    "coaching_qualifications",
    "Coaching Qualifications",
    "shortlist",
    184,
  ),
];

export { DEFAULT_STAFF_TABLE_COLUMN_IDS } from "@/utils/staff-table-layout";

export function getStaffMetric(metricId: string): StaffMetric | undefined {
  return STAFF_METRICS.find((metric) => metric.id === metricId);
}

export function getStaffShortlistMetric(
  metricId: string,
): StaffMetric | undefined {
  return STAFF_SHORTLIST_METRICS.find((metric) => metric.id === metricId);
}

export function isStaffMetricId(value: unknown): value is string {
  return (
    typeof value === "string" && getStaffShortlistMetric(value) !== undefined
  );
}

export function defaultDirForStaffSortField(field: string): "asc" | "desc" {
  return [
    "name",
    "age",
    "nationality",
    "club",
    "division",
    "preferred_job",
    "club_job",
    "coaching_qualifications",
  ].includes(field)
    ? "asc"
    : "desc";
}

export function defaultValueForStaffMetric(metricId: string) {
  const metric = getStaffMetric(metricId);
  return metric?.kind === "string"
    ? ({ type: "text", value: "" } as const)
    : ({ type: "integer", value: 0 } as const);
}

export function defaultOperatorForStaffMetric(metricId: string): string {
  return getStaffMetric(metricId)?.operators[0]?.id ?? "contains";
}
