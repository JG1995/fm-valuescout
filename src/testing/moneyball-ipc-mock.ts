import type { MoneyballProfile } from "@/features/moneyball/types/moneyball-profile";

let profileOverride: MoneyballProfile | null | undefined;
let pendingProfile: {
  promise: Promise<MoneyballProfile | null>;
  resolve: (profile: MoneyballProfile | null) => void;
} | null = null;

export function setPlayerMoneyballOverride(
  profile: MoneyballProfile | null | undefined,
) {
  profileOverride = profile;
}

export function resetPlayerMoneyballOverride() {
  profileOverride = undefined;
  pendingProfile = null;
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
    askingPriceKind: "single",
    askingPriceLowerEur: 12_000_000,
    askingPriceUpperEur: null,
    starts: 18,
    substituteAppearances: 3,
    minutes: 1500,
    statistics: { goals: 10, goals_per_90: 0.6 },
    percentiles: { goals: 83, goals_per_90: 75 },
    ...overrides,
  };
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
