import type { StaffFilterRuleIpc } from "@/features/staff/types/staff-filter-rule";
import {
  DEFAULT_STAFF_SORT_DIR,
  DEFAULT_STAFF_SORT_FIELD,
  isStaffSortDir,
  isStaffSortField,
} from "@/features/staff/types/staff-sort";
import type {
  StaffPage,
  StaffSummary,
} from "@/features/staff/types/staff-summary";
import { resolveGetCurrentSnapshotIpcMock } from "./snapshot-ipc-mock";

let overrideStaff: StaffSummary[] | null = null;
let lastStaffArgs: Record<string, unknown> | null = null;
let staffFamilyConfigured = true;

const ROLE_IDS = [
  "assistant_manager",
  "coach_attacking_technical",
  "coach_attacking_tactical",
  "coach_defending_technical",
  "coach_defending_tactical",
  "coach_possession_technical",
  "coach_possession_tactical",
  "coach_fitness",
  "coach_goalkeeping",
  "set_piece_coach",
  "loan_manager",
  "head_of_youth_development",
  "scout",
  "director_of_football",
  "technical_director",
  "recruitment_analyst",
  "head_performance_analyst",
  "performance_analyst",
  "physio",
  "sports_scientist",
] as const;

export function fixtureStaff(
  overrides: Partial<StaffSummary> = {},
): StaffSummary {
  const dynamicValues: Record<string, number> = {};
  for (const role of ROLE_IDS) {
    dynamicValues[`role.${role}`] = 72;
  }
  dynamicValues["attr.Authority"] = 15;
  dynamicValues["attr.Adaptability"] = 16;
  return {
    uid: 101,
    name: "Alex Coach",
    age: 44,
    birthYear: 1982,
    birthDayOfYear: 120,
    nationalities: ["Denmark"],
    nationUid: null,
    gender: "male",
    club: "Metro FC",
    division: "Premier Division",
    ca: 145,
    pa: 160,
    jobId: 1,
    weeklyWageGbp: 15_000,
    contractExpiryYear: 2028,
    contractExpiryDayOfYear: 220,
    dynamicValues,
    ...overrides,
  };
}

export function setStaffOverride(staff: StaffSummary[] | null) {
  overrideStaff = staff;
}

export function setStaffFamilyConfigured(configured: boolean) {
  staffFamilyConfigured = configured;
}

export function resetStaffIpcMock() {
  overrideStaff = null;
  lastStaffArgs = null;
  staffFamilyConfigured = true;
}

export function getLastStaffArgs() {
  return lastStaffArgs;
}

function parseArgs(args: unknown) {
  const record =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  return {
    sortBy: isStaffSortField(record.sortBy)
      ? record.sortBy
      : DEFAULT_STAFF_SORT_FIELD,
    sortDir: isStaffSortDir(record.sortDir)
      ? record.sortDir
      : DEFAULT_STAFF_SORT_DIR,
    offset: typeof record.offset === "number" ? Math.max(0, record.offset) : 0,
    limit:
      typeof record.limit === "number"
        ? Math.min(200, Math.max(1, record.limit))
        : 50,
    requestedFields: Array.isArray(record.requestedFields)
      ? record.requestedFields.filter(
          (field): field is string => typeof field === "string",
        )
      : [],
    filters: Array.isArray(record.filters)
      ? record.filters.filter(isFilterRule)
      : [],
    combine: record.filterCombine === "or" ? ("or" as const) : ("and" as const),
  };
}

function isFilterRule(value: unknown): value is StaffFilterRuleIpc {
  if (typeof value !== "object" || value === null) return false;
  const record = value as Record<string, unknown>;
  return (
    typeof record.field === "string" &&
    typeof record.op === "string" &&
    (typeof record.value === "string" || typeof record.value === "number")
  );
}

function fieldValue(
  staff: StaffSummary,
  field: string,
): string | number | null {
  switch (field) {
    case "name":
      return staff.name;
    case "age":
      return staff.age;
    case "birth_year":
      return staff.birthYear;
    case "birth_day_of_year":
      return staff.birthDayOfYear;
    case "nationality":
      return staff.nationalities.join(", ");
    case "nation_uid":
      return staff.nationUid;
    case "gender":
      return staff.gender;
    case "club":
      return staff.club;
    case "division":
      return staff.division;
    case "ca":
      return staff.ca;
    case "pa":
      return staff.pa;
    case "wage":
      return staff.weeklyWageGbp;
    case "contract_year":
      return staff.contractExpiryYear;
    case "contract_day":
      return staff.contractExpiryDayOfYear;
    case "job_id":
      return staff.jobId;
    default:
      return staff.dynamicValues?.[field] ?? null;
  }
}

function matches(staff: StaffSummary, rule: StaffFilterRuleIpc) {
  const raw = fieldValue(staff, rule.field);
  if (raw === null) return false;
  if (typeof rule.value === "string") {
    const text = String(raw).toLowerCase();
    const target = rule.value.toLowerCase();
    if (rule.op === "contains") return text.includes(target);
    if (rule.op === "not_contains") return !text.includes(target);
    if (rule.op === "is") return text === target;
    if (rule.op === "is_not") return text !== target;
    return false;
  }
  const value = Number(raw);
  if (!Number.isFinite(value)) return false;
  if (rule.op === "gt") return value > rule.value;
  if (rule.op === "lt") return value < rule.value;
  if (rule.op === "eq") return value === rule.value;
  if (rule.op === "neq") return value !== rule.value;
  return false;
}

function sortStaff(
  staff: StaffSummary[],
  sortBy: string,
  sortDir: "asc" | "desc",
) {
  return [...staff].sort((left, right) => {
    const a = fieldValue(left, sortBy);
    const b = fieldValue(right, sortBy);
    const cmp =
      typeof a === "number" && typeof b === "number"
        ? a - b
        : String(a ?? "").localeCompare(String(b ?? ""));
    return (sortDir === "asc" ? cmp : -cmp) || left.uid - right.uid;
  });
}

export function resolveSearchStaffIpcMock(args: unknown): StaffPage {
  lastStaffArgs =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  if (!resolveGetCurrentSnapshotIpcMock()) {
    return { state: "no_current_snapshot", staff: [], total: 0 };
  }
  const parsed = parseArgs(args);
  const source = overrideStaff ?? [
    fixtureStaff(),
    fixtureStaff({
      uid: 102,
      name: "Jordan Analyst",
      ca: 132,
      club: "Riverside United",
    }),
  ];
  const filtered =
    parsed.filters.length === 0
      ? source
      : source.filter((staff) => {
          const results = parsed.filters.map((rule) => matches(staff, rule));
          return parsed.combine === "and"
            ? results.every(Boolean)
            : results.some(Boolean);
        });
  const sorted = sortStaff(filtered, parsed.sortBy, parsed.sortDir);
  const page = sorted
    .slice(parsed.offset, parsed.offset + parsed.limit)
    .map((staff) => ({
      ...staff,
      dynamicValues: Object.fromEntries(
        parsed.requestedFields.map((field) => [
          field,
          staff.dynamicValues?.[field] ?? null,
        ]),
      ),
    }));
  return { state: "ready", staff: page, total: sorted.length };
}

export function resolveListMyStaffIpcMock(args: unknown): StaffPage {
  if (!resolveGetCurrentSnapshotIpcMock()) {
    return { state: "no_current_snapshot", staff: [], total: 0 };
  }
  if (!staffFamilyConfigured) {
    return { state: "no_club_family", staff: [], total: 0 };
  }
  return resolveSearchStaffIpcMock(args);
}
