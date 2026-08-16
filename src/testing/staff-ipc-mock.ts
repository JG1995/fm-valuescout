import type {
  MyStaffBoostProgress,
  MyStaffBoostResult,
} from "@/features/staff/types/my-staff-boost";
import type { StaffBoostResult } from "@/features/staff/types/staff-boost";
import type { StaffDetail } from "@/features/staff/types/staff-detail";
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
import { getPlayerHiddenInformationRevealedIpcMock } from "./player-ipc-mock";
import { resolveGetCurrentSnapshotIpcMock } from "./snapshot-ipc-mock";

let overrideStaff: StaffSummary[] | null = null;
let staffDetailOverride: StaffDetail | null | undefined;
let lastStaffArgs: Record<string, unknown> | null = null;
let staffFamilyConfigured = true;
let staffListMode: StaffListIpcMockMode = "success";
let staffBoostMode: StaffBoostIpcMockMode = "success";
let staffBoostCalls: unknown[] = [];
let pendingStaffBoost: {
  promise: Promise<StaffBoostResult>;
  resolve: (result: StaffBoostResult) => void;
} | null = null;
let myStaffBoostCalls: unknown[] = [];
let myStaffBoostMode: MyStaffBoostIpcMockMode = "pending";
let pendingMyStaffBoost: {
  args: unknown;
  promise: Promise<MyStaffBoostResult>;
  resolve: (result: MyStaffBoostResult) => void;
  reject: (error: Error) => void;
} | null = null;

export type MyStaffBoostIpcMockMode = "pending" | "recoveryRequired" | "error";

function defaultStaff() {
  return [
    fixtureStaff(),
    fixtureStaff({
      uid: 102,
      name: "Jordan Analyst",
      ca: 132,
      club: "Riverside United",
    }),
  ];
}

export type StaffBoostIpcMockMode =
  | "success"
  | "pending"
  | "eligibilityError"
  | "liveValueError"
  | "snapshotSyncError";

export type StaffListIpcMockMode = "success" | "error";

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

export function setStaffDetailOverride(staff: StaffDetail | null | undefined) {
  staffDetailOverride = staff;
}

export function setStaffFamilyConfigured(configured: boolean) {
  staffFamilyConfigured = configured;
}

export function setStaffListIpcMockMode(mode: StaffListIpcMockMode) {
  staffListMode = mode;
}

export function resetStaffIpcMock() {
  overrideStaff = null;
  staffDetailOverride = undefined;
  lastStaffArgs = null;
  staffFamilyConfigured = true;
  staffListMode = "success";
  staffBoostMode = "success";
  staffBoostCalls = [];
  pendingStaffBoost = null;
  myStaffBoostCalls = [];
  myStaffBoostMode = "pending";
  pendingMyStaffBoost = null;
}

export function fixtureStaffDetail(
  overrides: Partial<StaffDetail> = {},
): StaffDetail {
  const attributes = Object.fromEntries(
    [
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
    ].map((key) => [key, key === "Adaptability" ? 16 : 15]),
  );
  return {
    ...fixtureStaff(),
    attributes,
    hiddenInformationRevealed: getPlayerHiddenInformationRevealedIpcMock(),
    roleScores: [
      { roleId: "coach_fitness", displayName: "Coach — Fitness", score: 85 },
      { roleId: "scout", displayName: "Scout", score: 80 },
      { roleId: "physio", displayName: "Physio", score: null },
    ],
    ...overrides,
  };
}

export function resolveGetStaffIpcMock(args: unknown): StaffDetail | null {
  if (staffDetailOverride !== undefined) {
    return staffDetailOverride
      ? {
          ...staffDetailOverride,
          hiddenInformationRevealed:
            getPlayerHiddenInformationRevealedIpcMock(),
        }
      : null;
  }
  const uid =
    typeof args === "object" &&
    args !== null &&
    typeof (args as Record<string, unknown>).uid === "number"
      ? ((args as Record<string, number>).uid ?? 0)
      : 0;
  return uid === 101 ? fixtureStaffDetail() : null;
}

export function setStaffBoostIpcMockMode(mode: StaffBoostIpcMockMode) {
  staffBoostMode = mode;
  if (mode !== "pending") pendingStaffBoost = null;
}

export function getStaffBoostIpcMockCalls() {
  return staffBoostCalls;
}

export function resolvePendingStaffBoostIpcMock(
  result = staffBoostResult(101),
) {
  pendingStaffBoost?.resolve(result);
  pendingStaffBoost = null;
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
  if (staffListMode === "error") {
    throw new Error("staff list failed");
  }
  lastStaffArgs =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  if (!resolveGetCurrentSnapshotIpcMock()) {
    return { state: "no_current_snapshot", staff: [], total: 0 };
  }
  const parsed = parseArgs(args);
  const source = overrideStaff ?? defaultStaff();
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

function staffBoostResult(uid: number): StaffBoostResult {
  const source = overrideStaff ?? defaultStaff();
  const staff = source.find((candidate) => candidate.uid === uid);
  if (!staff) throw new Error("Staff member not found");
  const target = Math.min(staff.ca + 10, staff.pa, 200);
  overrideStaff = source.map((candidate) =>
    candidate.uid === uid ? { ...candidate, ca: target } : candidate,
  );
  return {
    snapshotId: 1,
    operation: "boost-staff-current-ability",
    previousCurrentAbility: staff.ca,
    currentAbility: target,
    potentialAbility: staff.pa,
  };
}

export function resolveBoostStaffCurrentAbilityIpcMock(
  args: unknown,
): Promise<StaffBoostResult> {
  staffBoostCalls = [...staffBoostCalls, args];
  const uid =
    typeof args === "object" &&
    args !== null &&
    typeof (args as Record<string, unknown>).uid === "number"
      ? ((args as Record<string, number>).uid ?? 0)
      : 0;
  if (staffBoostMode === "pending") {
    if (!pendingStaffBoost) {
      let resolve!: (result: StaffBoostResult) => void;
      const promise = new Promise<StaffBoostResult>((next) => {
        resolve = next;
      });
      pendingStaffBoost = { promise, resolve };
    }
    return pendingStaffBoost.promise;
  }
  if (staffBoostMode === "eligibilityError") {
    return Promise.reject({
      phase: "eligibility",
      kind: "currentAbilityAtLimit",
      message: "current ability has reached its potential limit",
    });
  }
  if (staffBoostMode === "liveValueError") {
    return Promise.reject({
      phase: "liveValue",
      message: "staff values changed in FM; Load Data again",
    });
  }
  if (staffBoostMode === "snapshotSyncError") {
    return Promise.reject({
      phase: "snapshotSync",
      message: "FM may have changed. Load Data again.",
    });
  }
  return Promise.resolve(staffBoostResult(uid));
}

export function getMyStaffBoostIpcMockCalls() {
  return myStaffBoostCalls;
}

export function setMyStaffBoostIpcMockMode(mode: MyStaffBoostIpcMockMode) {
  myStaffBoostMode = mode;
  pendingMyStaffBoost = null;
}

export function resolvePendingMyStaffBoostIpcMock() {
  const pending = pendingMyStaffBoost;
  if (!pending) return;
  const source = overrideStaff ?? defaultStaff();
  const eligible = source.filter(
    (staff) => staff.ca < staff.pa && staff.ca < 200,
  );
  const skipped = source.length - eligible.length;
  sendMyStaffBoostProgress(pending.args, {
    processed: source.length,
    total: source.length,
    updated: eligible.length,
    skipped,
    failed: 0,
  });
  overrideStaff = source.map((staff) => ({
    ...staff,
    ca: staff.ca < staff.pa ? Math.min(staff.ca + 10, staff.pa, 200) : staff.ca,
  }));
  pending.resolve({
    updated: eligible.length,
    skipped,
    failed: 0,
    recoveryRequired: false,
    recoveryMessage: null,
  });
  pendingMyStaffBoost = null;
}

export function rejectPendingMyStaffBoostIpcMock(error: Error) {
  const pending = pendingMyStaffBoost;
  if (!pending) return;
  pending.reject(error);
  pendingMyStaffBoost = null;
}

export function sendPendingMyStaffBoostProgressIpcMock(
  progress: MyStaffBoostProgress = {
    processed: 1,
    total: 2,
    updated: 1,
    skipped: 0,
    failed: 0,
  },
) {
  if (pendingMyStaffBoost)
    sendMyStaffBoostProgress(pendingMyStaffBoost.args, progress);
}

export function resolveBoostMyStaffCurrentAbilityIpcMock(
  args: unknown,
): Promise<MyStaffBoostResult> {
  myStaffBoostCalls = [...myStaffBoostCalls, args];
  if (myStaffBoostMode === "recoveryRequired") {
    return Promise.resolve({
      updated: 1,
      skipped: 0,
      failed: 0,
      recoveryRequired: true,
      recoveryMessage: "FM may have changed before verification.",
    });
  }
  if (myStaffBoostMode === "error") {
    return Promise.reject({
      phase: "bridge",
      kind: "unavailable",
      message: "Bridge is unavailable.",
    });
  }
  if (!pendingMyStaffBoost) {
    let resolve!: (result: MyStaffBoostResult) => void;
    let reject!: (error: Error) => void;
    const promise = new Promise<MyStaffBoostResult>((next, fail) => {
      resolve = next;
      reject = fail;
    });
    pendingMyStaffBoost = { args, promise, resolve, reject };
    const total = (overrideStaff ?? defaultStaff()).length;
    sendMyStaffBoostProgress(args, {
      processed: 0,
      total,
      updated: 0,
      skipped: 0,
      failed: 0,
    });
  }
  return pendingMyStaffBoost.promise;
}

function sendMyStaffBoostProgress(
  args: unknown,
  progress: MyStaffBoostProgress,
) {
  const channel =
    typeof args === "object" && args !== null
      ? (
          args as {
            onProgress?: { onmessage?: (value: MyStaffBoostProgress) => void };
          }
        ).onProgress
      : undefined;
  channel?.onmessage?.(progress);
}
