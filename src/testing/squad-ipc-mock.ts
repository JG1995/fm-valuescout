import type {
  SquadPlayer,
  SquadPlayersPage,
} from "@/features/squad/types/squad-player";
import type { SquadPlayerBoostResult } from "@/features/squad/types/squad-player-boost";
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
let squadCurrentAbilityBoostMode: SquadCurrentAbilityBoostIpcMockMode =
  "success";
let squadCurrentAbilityBoostCalls: unknown[] = [];
let pendingSquadCurrentAbilityBoost: {
  promise: Promise<SquadPlayerBoostResult>;
  resolve: (result: SquadPlayerBoostResult) => void;
} | null = null;
let squadWonderkidMentalityBoostMode: SquadWonderkidMentalityBoostIpcMockMode =
  "success";
let squadWonderkidMentalityBoostCalls: unknown[] = [];
let pendingSquadWonderkidMentalityBoost: {
  promise: Promise<SquadPlayerBoostResult>;
  resolve: (result: SquadPlayerBoostResult) => void;
} | null = null;

export type SquadCurrentAbilityBoostIpcMockMode =
  | "success"
  | "pending"
  | "recoveryRequired"
  | "error";

export type SquadWonderkidMentalityBoostIpcMockMode =
  | "success"
  | "pending"
  | "recoveryRequired"
  | "error";

export function setSquadPlayersOverride(players: SquadPlayer[] | null) {
  overridePlayers = players;
}

export function resetSquadPlayersOverride() {
  overridePlayers = null;
  lastSquadPlayersArgs = null;
  squadCurrentAbilityBoostMode = "success";
  squadCurrentAbilityBoostCalls = [];
  pendingSquadCurrentAbilityBoost = null;
  squadWonderkidMentalityBoostMode = "success";
  squadWonderkidMentalityBoostCalls = [];
  pendingSquadWonderkidMentalityBoost = null;
}

export function getLastSquadPlayersArgs(): Record<string, unknown> | null {
  return lastSquadPlayersArgs;
}

export function setSquadCurrentAbilityBoostIpcMockMode(
  mode: SquadCurrentAbilityBoostIpcMockMode,
) {
  squadCurrentAbilityBoostMode = mode;
  if (mode !== "pending") {
    pendingSquadCurrentAbilityBoost = null;
  }
}

export function getSquadCurrentAbilityBoostIpcMockCalls() {
  return squadCurrentAbilityBoostCalls;
}

export function resolvePendingSquadCurrentAbilityBoostIpcMock(
  result = squadCurrentAbilityBoostResult(),
) {
  pendingSquadCurrentAbilityBoost?.resolve(result);
  pendingSquadCurrentAbilityBoost = null;
}

function squadCurrentAbilityBoostResult(): SquadPlayerBoostResult {
  return {
    updated: 2,
    skipped: 1,
    failed: 0,
    recoveryRequired: false,
    recoveryMessage: null,
  };
}

export function resolveSquadCurrentAbilityBoostIpcMock(
  args: unknown,
): Promise<SquadPlayerBoostResult> {
  squadCurrentAbilityBoostCalls = [...squadCurrentAbilityBoostCalls, args];

  if (squadCurrentAbilityBoostMode === "pending") {
    if (!pendingSquadCurrentAbilityBoost) {
      let resolve!: (result: SquadPlayerBoostResult) => void;
      const promise = new Promise<SquadPlayerBoostResult>((next) => {
        resolve = next;
      });
      pendingSquadCurrentAbilityBoost = { promise, resolve };
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
      kind: "clubFamilyRequired",
      message:
        "Set up your club family in Dashboard before boosting the squad.",
    });
  }

  return Promise.resolve(squadCurrentAbilityBoostResult());
}

export function setSquadWonderkidMentalityBoostIpcMockMode(
  mode: SquadWonderkidMentalityBoostIpcMockMode,
) {
  squadWonderkidMentalityBoostMode = mode;
  if (mode !== "pending") {
    pendingSquadWonderkidMentalityBoost = null;
  }
}

export function getSquadWonderkidMentalityBoostIpcMockCalls() {
  return squadWonderkidMentalityBoostCalls;
}

export function resolvePendingSquadWonderkidMentalityBoostIpcMock(
  result = squadWonderkidMentalityBoostResult(),
) {
  pendingSquadWonderkidMentalityBoost?.resolve(result);
  pendingSquadWonderkidMentalityBoost = null;
}

function squadWonderkidMentalityBoostResult(): SquadPlayerBoostResult {
  return {
    updated: 2,
    skipped: 1,
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

  if (squadWonderkidMentalityBoostMode === "pending") {
    if (!pendingSquadWonderkidMentalityBoost) {
      let resolve!: (result: SquadPlayerBoostResult) => void;
      const promise = new Promise<SquadPlayerBoostResult>((next) => {
        resolve = next;
      });
      pendingSquadWonderkidMentalityBoost = { promise, resolve };
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
      kind: "clubFamilyRequired",
      message:
        "Set up your club family in Dashboard before boosting the squad.",
    });
  }

  return Promise.resolve(squadWonderkidMentalityBoostResult());
}

function parsePaging(args: unknown): {
  offset: number;
  limit: number;
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
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
  return { offset, limit, sortBy, sortDir };
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

export function resolveSquadPlayersIpcMock(args: unknown): SquadPlayersPage {
  lastSquadPlayersArgs =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  const { offset, limit, sortBy, sortDir } = parsePaging(args);
  const players = [...(overridePlayers ?? [])].sort((a, b) =>
    comparePlayers(a, b, sortBy, sortDir),
  );
  return {
    players: players.slice(offset, offset + limit),
    total: players.length,
  };
}
