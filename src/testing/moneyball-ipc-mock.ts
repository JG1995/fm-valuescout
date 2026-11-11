import type { SeasonStatsOutcome } from "@/features/memory-read/types/load-data";
import type {
  MoneyballLeagueAdjustment,
  MoneyballProfile,
} from "@/features/moneyball/types/moneyball-profile";
import { setMoneyballStatsGameDateIpcMock } from "./snapshot-ipc-mock";

let profileOverride: MoneyballProfile | null | undefined;
let pendingProfile: {
  promise: Promise<MoneyballProfile | null>;
  resolve: (profile: MoneyballProfile | null) => void;
} | null = null;

type RefreshSeasonStatsMockResult = SeasonStatsOutcome | "loadDataRunning";

let refreshResult: RefreshSeasonStatsMockResult = {
  status: "notLinked",
  shortlists: { status: "notRead" },
  staffShortlist: { status: "notRead" },
  elapsedMs: 0,
};
let refreshArgs: unknown;
let pendingRefresh: (() => void) | null = null;

const DEFAULT_LEAGUE_ADJUSTMENT: MoneyballLeagueAdjustment = {
  enabled: true,
  referenceDivision: "Premier Division",
};
let leagueAdjustment = DEFAULT_LEAGUE_ADJUSTMENT;
let leagueAdjustmentChanges: boolean[] = [];

export function setMoneyballLeagueAdjustmentIpcMock(
  adjustment: MoneyballLeagueAdjustment,
) {
  leagueAdjustment = adjustment;
}

/** Each value `set_moneyball_league_adjustment` was called with, in order. */
export function getMoneyballLeagueAdjustmentChanges() {
  return leagueAdjustmentChanges;
}

export function resolveGetMoneyballLeagueAdjustmentIpcMock() {
  return leagueAdjustment;
}

export function resolveSetMoneyballLeagueAdjustmentIpcMock(args: unknown) {
  const enabled = (args as { enabled: boolean }).enabled;
  leagueAdjustmentChanges.push(enabled);
  leagueAdjustment = { ...leagueAdjustment, enabled };
  return leagueAdjustment;
}

export function setRefreshSeasonStatsIpcMockResult(
  result: RefreshSeasonStatsMockResult,
) {
  refreshResult = result;
}

/** Holds the next refresh until `resolvePendingRefreshSeasonStats` runs. */
export function setRefreshSeasonStatsIpcMockPending() {
  pendingRefresh = () => undefined;
}

export function resolvePendingRefreshSeasonStats() {
  pendingRefresh?.();
  pendingRefresh = null;
}

export function getLastRefreshSeasonStatsIpcArgs() {
  return refreshArgs;
}

/** Mirrors the Rust command: an attached read sets the current snapshot's as-of date. */
export async function resolveRefreshSeasonStatsIpcMock(
  args: unknown,
): Promise<SeasonStatsOutcome> {
  refreshArgs = args;
  if (pendingRefresh) {
    await new Promise<void>((resolve) => {
      pendingRefresh = resolve;
    });
  }
  if (refreshResult === "loadDataRunning") {
    throw {
      kind: "loadDataRunning",
      message: "Load Data is running. Refresh stats after it finishes.",
    };
  }
  if (refreshResult.status === "attached") {
    setMoneyballStatsGameDateIpcMock(refreshResult.gameDate);
  }
  return refreshResult;
}

export function setPlayerMoneyballOverride(
  profile: MoneyballProfile | null | undefined,
) {
  profileOverride = profile;
}

export function resetPlayerMoneyballOverride() {
  profileOverride = undefined;
  pendingProfile = null;
  refreshResult = {
    status: "notLinked",
    shortlists: { status: "notRead" },
    staffShortlist: { status: "notRead" },
    elapsedMs: 0,
  };
  refreshArgs = undefined;
  pendingRefresh = null;
  leagueAdjustment = DEFAULT_LEAGUE_ADJUSTMENT;
  leagueAdjustmentChanges = [];
}

export function setPlayerMoneyballPending() {
  let resolve: ((profile: MoneyballProfile | null) => void) | null = null;
  const promise = new Promise<MoneyballProfile | null>((nextResolve) => {
    resolve = nextResolve;
  });
  if (!resolve) throw new Error("Moneyball mock promise did not initialize");
  pendingProfile = { promise, resolve };
}

export function resolvePendingPlayerMoneyball(
  profile: MoneyballProfile | null,
) {
  pendingProfile?.resolve(profile);
  pendingProfile = null;
}

export function fixturePlayerMoneyball(
  overrides: Partial<Extract<MoneyballProfile, { state: "ready" }>> = {},
): Extract<MoneyballProfile, { state: "ready" }> {
  return {
    state: "ready",
    starts: 18,
    substituteAppearances: 3,
    minutes: 1500,
    statistics: { goals: 10, goals_per_90: 0.6 },
    percentiles: { goals: 83, goals_per_90: 75 },
    roleCatalogVersion: 1,
    roleScores: [],
    comparisonBasis: {
      kind: "available",
      naturalPositions: ["AMR"],
      comparisonPlayerCount: 24,
    },
    leagueAdjustment: null,
    ...overrides,
  };
}

export function fixturePlayerMoneyballWithoutNaturalPosition(
  overrides: Partial<Extract<MoneyballProfile, { state: "ready" }>> = {},
): Extract<MoneyballProfile, { state: "ready" }> {
  return fixturePlayerMoneyball({
    percentiles: null,
    roleCatalogVersion: null,
    roleScores: null,
    comparisonBasis: { kind: "unavailableNoNaturalPosition" },
    ...overrides,
  });
}

export function resolveGetPlayerMoneyballIpcMock(
  args: unknown,
): MoneyballProfile | null | Promise<MoneyballProfile | null> {
  const uid =
    typeof args === "object" &&
    args !== null &&
    "uid" in args &&
    typeof (args as { uid: unknown }).uid === "number"
      ? (args as { uid: number }).uid
      : null;
  if (uid !== 42) return null;
  if (pendingProfile) return pendingProfile.promise;
  return profileOverride === undefined ? { state: "noData" } : profileOverride;
}
