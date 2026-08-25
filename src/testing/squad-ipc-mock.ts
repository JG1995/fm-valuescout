import type {
  SquadPlayer,
  SquadPlayersPage,
} from "@/features/squad/types/squad-player";
import type {
  SquadPlayerBoostProgress,
  SquadPlayerBoostResult,
} from "@/features/squad/types/squad-player-boost";
import type {
  SquadSortDir,
  SquadSortField,
} from "@/features/squad/types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
  isSquadSortDir,
  isSquadSortField,
} from "@/features/squad/types/squad-sort";

let overridePlayers: SquadPlayer[] | null = null;
let lastSquadPlayersArgs: Record<string, unknown> | null = null;
let squadPlayersCallCount = 0;
let squadPlayersPageMode: SquadPlayersPageIpcMockMode = "success";
let pendingSquadPlayersPage: {
  args: unknown;
  promise: Promise<SquadPlayersPage>;
  resolve: (page: SquadPlayersPage) => void;
} | null = null;
let rejectedSecondPage = false;
let rejectedReplacement = false;
let squadCurrentAbilityBoostMode: SquadCurrentAbilityBoostIpcMockMode =
  "success";
let squadCurrentAbilityBoostCalls: unknown[] = [];
let lastSquadCurrentAbilityBoostProgress: SquadPlayerBoostProgress | null =
  null;
let pendingSquadCurrentAbilityBoost: {
  args: unknown;
  promise: Promise<SquadPlayerBoostResult>;
  resolve: (result: SquadPlayerBoostResult) => void;
} | null = null;
let squadWonderkidMentalityBoostMode: SquadWonderkidMentalityBoostIpcMockMode =
  "success";
let squadWonderkidMentalityBoostCalls: unknown[] = [];
let lastSquadWonderkidMentalityBoostProgress: SquadPlayerBoostProgress | null =
  null;
let pendingSquadWonderkidMentalityBoost: {
  args: unknown;
  promise: Promise<SquadPlayerBoostResult>;
  resolve: (result: SquadPlayerBoostResult) => void;
} | null = null;

type SquadBoostIpcArgs = {
  onProgress?: {
    onmessage?: (progress: SquadPlayerBoostProgress) => void;
  };
};

export type SquadCurrentAbilityBoostIpcMockMode =
  | "success"
  | "pending"
  | "pendingEmpty"
  | "recoveryRequired"
  | "error";

export type SquadPlayersPageIpcMockMode =
  | "success"
  | "pendingSecondPage"
  | "rejectSecondPageOnce"
  | "pendingReplacement"
  | "pendingDynamicReplacement"
  | "pendingProjection"
  | "rejectInitial"
  | "rejectReplacementOnce";

export type SquadWonderkidMentalityBoostIpcMockMode =
  | "success"
  | "pending"
  | "pendingEmpty"
  | "recoveryRequired"
  | "error";

export function setSquadPlayersOverride(players: SquadPlayer[] | null) {
  overridePlayers = players;
}

export function resetSquadPlayersOverride() {
  overridePlayers = null;
  lastSquadPlayersArgs = null;
  squadPlayersCallCount = 0;
  squadPlayersPageMode = "success";
  pendingSquadPlayersPage = null;
  rejectedSecondPage = false;
  rejectedReplacement = false;
  squadCurrentAbilityBoostMode = "success";
  squadCurrentAbilityBoostCalls = [];
  lastSquadCurrentAbilityBoostProgress = null;
  pendingSquadCurrentAbilityBoost = null;
  squadWonderkidMentalityBoostMode = "success";
  squadWonderkidMentalityBoostCalls = [];
  lastSquadWonderkidMentalityBoostProgress = null;
  pendingSquadWonderkidMentalityBoost = null;
}

export function getLastSquadPlayersArgs(): Record<string, unknown> | null {
  return lastSquadPlayersArgs;
}

export function getSquadPlayersCallCount(): number {
  return squadPlayersCallCount;
}

export function setSquadPlayersPageIpcMockMode(
  mode: SquadPlayersPageIpcMockMode,
) {
  squadPlayersPageMode = mode;
  pendingSquadPlayersPage = null;
  rejectedSecondPage = false;
  rejectedReplacement = false;
}

export function resolvePendingSquadPlayersPageIpcMock() {
  const pending = pendingSquadPlayersPage;
  if (!pending) {
    return;
  }
  pendingSquadPlayersPage = null;
  squadPlayersPageMode = "success";
  pending.resolve(squadPlayersPage(pending.args));
}

export function setSquadCurrentAbilityBoostIpcMockMode(
  mode: SquadCurrentAbilityBoostIpcMockMode,
) {
  squadCurrentAbilityBoostMode = mode;
  if (mode !== "pending" && mode !== "pendingEmpty") {
    pendingSquadCurrentAbilityBoost = null;
  }
}

export function getSquadCurrentAbilityBoostIpcMockCalls() {
  return squadCurrentAbilityBoostCalls;
}

export function getLastSquadCurrentAbilityBoostProgress() {
  return lastSquadCurrentAbilityBoostProgress;
}

export function resolvePendingSquadCurrentAbilityBoostIpcMock(
  result?: SquadPlayerBoostResult,
) {
  const pending = pendingSquadCurrentAbilityBoost;
  if (!pending) {
    return;
  }
  const total = squadCurrentAbilityBoostMode === "pendingEmpty" ? 0 : 2;
  sendSquadCurrentAbilityBoostProgress(pending.args, {
    processed: total,
    total,
    updated: total,
    skipped: 0,
    failed: 0,
  });
  pending.resolve(result ?? squadCurrentAbilityBoostResult(total));
  pendingSquadCurrentAbilityBoost = null;
}

export function sendPendingSquadCurrentAbilityBoostProgressIpcMock(
  progress: SquadPlayerBoostProgress = {
    processed: 1,
    total: 2,
    updated: 1,
    skipped: 0,
    failed: 0,
  },
) {
  if (pendingSquadCurrentAbilityBoost) {
    sendSquadCurrentAbilityBoostProgress(
      pendingSquadCurrentAbilityBoost.args,
      progress,
    );
  }
}

function squadCurrentAbilityBoostResult(total = 2): SquadPlayerBoostResult {
  return {
    updated: total,
    skipped: 0,
    failed: 0,
    recoveryRequired: false,
    recoveryMessage: null,
  };
}

export function resolveSquadCurrentAbilityBoostIpcMock(
  args: unknown,
): Promise<SquadPlayerBoostResult> {
  squadCurrentAbilityBoostCalls = [...squadCurrentAbilityBoostCalls, args];

  if (
    squadCurrentAbilityBoostMode === "pending" ||
    squadCurrentAbilityBoostMode === "pendingEmpty"
  ) {
    const total = squadCurrentAbilityBoostMode === "pendingEmpty" ? 0 : 2;
    if (!pendingSquadCurrentAbilityBoost) {
      let resolve!: (result: SquadPlayerBoostResult) => void;
      const promise = new Promise<SquadPlayerBoostResult>((next) => {
        resolve = next;
      });
      pendingSquadCurrentAbilityBoost = { args, promise, resolve };
      sendSquadCurrentAbilityBoostProgress(args, {
        processed: 0,
        total,
        updated: 0,
        skipped: 0,
        failed: 0,
      });
    }
    return pendingSquadCurrentAbilityBoost.promise;
  }

  if (squadCurrentAbilityBoostMode === "recoveryRequired") {
    return Promise.resolve({
      updated: 1,
      skipped: 2,
      failed: 1,
      recoveryRequired: true,
      recoveryMessage: "FM may have changed before the result was verified.",
    });
  }

  if (squadCurrentAbilityBoostMode === "error") {
    return Promise.reject({
      phase: "eligibility",
      kind: "managedClubRequired",
      message:
        "Choose your managed club in Settings before boosting the squad.",
    });
  }

  return Promise.resolve(squadCurrentAbilityBoostResult());
}

export function setSquadWonderkidMentalityBoostIpcMockMode(
  mode: SquadWonderkidMentalityBoostIpcMockMode,
) {
  squadWonderkidMentalityBoostMode = mode;
  if (mode !== "pending" && mode !== "pendingEmpty") {
    pendingSquadWonderkidMentalityBoost = null;
  }
}

export function getSquadWonderkidMentalityBoostIpcMockCalls() {
  return squadWonderkidMentalityBoostCalls;
}

export function getLastSquadWonderkidMentalityBoostProgress() {
  return lastSquadWonderkidMentalityBoostProgress;
}

export function resolvePendingSquadWonderkidMentalityBoostIpcMock(
  result?: SquadPlayerBoostResult,
) {
  const pending = pendingSquadWonderkidMentalityBoost;
  if (!pending) {
    return;
  }
  const total = squadWonderkidMentalityBoostMode === "pendingEmpty" ? 0 : 2;
  sendSquadWonderkidMentalityBoostProgress(pending.args, {
    processed: total,
    total,
    updated: total,
    skipped: 0,
    failed: 0,
  });
  pending.resolve(result ?? squadWonderkidMentalityBoostResult(total));
  pendingSquadWonderkidMentalityBoost = null;
}

export function sendPendingSquadWonderkidMentalityBoostProgressIpcMock(
  progress: SquadPlayerBoostProgress = {
    processed: 1,
    total: 2,
    updated: 1,
    skipped: 0,
    failed: 0,
  },
) {
  if (pendingSquadWonderkidMentalityBoost) {
    sendSquadWonderkidMentalityBoostProgress(
      pendingSquadWonderkidMentalityBoost.args,
      progress,
    );
  }
}

function squadWonderkidMentalityBoostResult(total = 2): SquadPlayerBoostResult {
  return {
    updated: total,
    skipped: 0,
    failed: 0,
    recoveryRequired: false,
    recoveryMessage: null,
  };
}

export function resolveSquadWonderkidMentalityBoostIpcMock(
  args: unknown,
): Promise<SquadPlayerBoostResult> {
  squadWonderkidMentalityBoostCalls = [
    ...squadWonderkidMentalityBoostCalls,
    args,
  ];

  if (
    squadWonderkidMentalityBoostMode === "pending" ||
    squadWonderkidMentalityBoostMode === "pendingEmpty"
  ) {
    const total = squadWonderkidMentalityBoostMode === "pendingEmpty" ? 0 : 2;
    if (!pendingSquadWonderkidMentalityBoost) {
      let resolve!: (result: SquadPlayerBoostResult) => void;
      const promise = new Promise<SquadPlayerBoostResult>((next) => {
        resolve = next;
      });
      pendingSquadWonderkidMentalityBoost = { args, promise, resolve };
      sendSquadWonderkidMentalityBoostProgress(args, {
        processed: 0,
        total,
        updated: 0,
        skipped: 0,
        failed: 0,
      });
    }
    return pendingSquadWonderkidMentalityBoost.promise;
  }

  if (squadWonderkidMentalityBoostMode === "recoveryRequired") {
    return Promise.resolve({
      updated: 1,
      skipped: 2,
      failed: 1,
      recoveryRequired: true,
      recoveryMessage: "FM may have changed before the result was verified.",
    });
  }

  if (squadWonderkidMentalityBoostMode === "error") {
    return Promise.reject({
      phase: "eligibility",
      kind: "managedClubRequired",
      message:
        "Choose your managed club in Settings before boosting the squad.",
    });
  }

  return Promise.resolve(squadWonderkidMentalityBoostResult());
}

function sendSquadBoostProgress(
  args: unknown,
  progress: SquadPlayerBoostProgress,
) {
  if (typeof args !== "object" || args === null) {
    return;
  }
  (args as SquadBoostIpcArgs).onProgress?.onmessage?.(progress);
}

function sendSquadCurrentAbilityBoostProgress(
  args: unknown,
  progress: SquadPlayerBoostProgress,
) {
  lastSquadCurrentAbilityBoostProgress = progress;
  sendSquadBoostProgress(args, progress);
}

function sendSquadWonderkidMentalityBoostProgress(
  args: unknown,
  progress: SquadPlayerBoostProgress,
) {
  lastSquadWonderkidMentalityBoostProgress = progress;
  sendSquadBoostProgress(args, progress);
}

function parsePaging(args: unknown): {
  offset: number;
  limit: number;
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
  requestedFields: string[];
} {
  const record =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  const offset =
    typeof record.offset === "number" ? Math.max(0, record.offset) : 0;
  const limit =
    typeof record.limit === "number"
      ? Math.min(200, Math.max(1, record.limit))
      : 50;
  const sortBy = isSquadSortField(record.sortBy)
    ? record.sortBy
    : DEFAULT_SQUAD_SORT_FIELD;
  const sortDir = isSquadSortDir(record.sortDir)
    ? record.sortDir
    : DEFAULT_SQUAD_SORT_DIR;
  const requestedFields = Array.isArray(record.requestedFields)
    ? record.requestedFields.filter(
        (field): field is string => typeof field === "string",
      )
    : [];
  return { offset, limit, sortBy, sortDir, requestedFields };
}

function compareNullableString(
  a: string | null | undefined,
  b: string | null | undefined,
): number {
  return (a ?? "").localeCompare(b ?? "", "en", { sensitivity: "base" });
}

function comparePlayers(
  a: SquadPlayer,
  b: SquadPlayer,
  sortBy: SquadSortField,
  sortDir: SquadSortDir,
): number {
  let cmp = 0;
  switch (sortBy) {
    case "name":
      cmp = compareNullableString(a.name, b.name);
      break;
    case "age":
      cmp = (a.age ?? -1) - (b.age ?? -1);
      break;
    case "nationality":
      cmp = compareNullableString(
        a.nationalities.join(", "),
        b.nationalities.join(", "),
      );
      break;
    case "club":
      cmp = compareNullableString(a.club, b.club);
      break;
    case "division":
      cmp = compareNullableString(a.division, b.division);
      break;
    case "ca":
      cmp = a.ca - b.ca;
      break;
    case "pa":
      cmp = a.pa - b.pa;
      break;
    case "value":
      cmp = (a.marketValueGbp ?? -1) - (b.marketValueGbp ?? -1);
      break;
  }
  if (cmp === 0) {
    return a.uid - b.uid;
  }
  return sortDir === "asc" ? cmp : -cmp;
}

function squadPlayersPage(args: unknown): SquadPlayersPage {
  const { offset, limit, sortBy, sortDir } = parsePaging(args);
  const players = [...(overridePlayers ?? [])].sort((a, b) =>
    comparePlayers(a, b, sortBy, sortDir),
  );
  return {
    players: players.slice(offset, offset + limit),
    total: players.length,
  };
}

export function resolveSquadPlayersIpcMock(
  args: unknown,
): SquadPlayersPage | Promise<SquadPlayersPage> {
  squadPlayersCallCount += 1;
  lastSquadPlayersArgs =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  const { offset, requestedFields, sortBy } = parsePaging(args);

  if (
    (offset >= 50 && squadPlayersPageMode === "pendingSecondPage") ||
    (offset === 0 &&
      sortBy === "name" &&
      squadPlayersPageMode === "pendingReplacement") ||
    (offset === 0 &&
      sortBy === "attr.Acceleration" &&
      squadPlayersPageMode === "pendingDynamicReplacement") ||
    (offset === 0 &&
      requestedFields.length > 0 &&
      squadPlayersPageMode === "pendingProjection")
  ) {
    if (!pendingSquadPlayersPage) {
      let resolve!: (page: SquadPlayersPage) => void;
      const promise = new Promise<SquadPlayersPage>((next) => {
        resolve = next;
      });
      pendingSquadPlayersPage = { args, promise, resolve };
    }
    return pendingSquadPlayersPage.promise;
  }

  if (offset === 0 && squadPlayersPageMode === "rejectInitial") {
    return Promise.reject(new Error("Could not load squad."));
  }

  if (
    offset >= 50 &&
    squadPlayersPageMode === "rejectSecondPageOnce" &&
    !rejectedSecondPage
  ) {
    rejectedSecondPage = true;
    return Promise.reject(new Error("Could not load the next squad page."));
  }

  if (
    offset === 0 &&
    sortBy === "name" &&
    squadPlayersPageMode === "rejectReplacementOnce" &&
    !rejectedReplacement
  ) {
    rejectedReplacement = true;
    return Promise.reject(new Error("Could not sort squad."));
  }

  return squadPlayersPage(args);
}
