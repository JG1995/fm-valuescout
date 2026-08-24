import type { PlayerBoostResult } from "@/features/player-profile/types/player-boost";
import type { PlayerDetail } from "@/features/player-profile/types/player-detail";

let getPlayerOverride: PlayerDetail | null | undefined;
let playerHiddenInformationRevealed = true;
let playerHiddenInformationMode: PlayerHiddenInformationIpcMockMode = "success";
let playerHiddenInformationCalls: unknown[] = [];
let currentAbilityBoostMode: CurrentAbilityBoostIpcMockMode = "success";
let currentAbilityBoostCalls: unknown[] = [];
let pendingCurrentAbilityBoost: {
  promise: Promise<PlayerBoostResult>;
  resolve: (result: PlayerBoostResult) => void;
} | null = null;
let wonderkidMentalityBoostMode: WonderkidMentalityBoostIpcMockMode = "success";
let wonderkidMentalityBoostCalls: unknown[] = [];
let pendingWonderkidMentalityBoost: {
  promise: Promise<PlayerBoostResult>;
  resolve: (result: PlayerBoostResult) => void;
} | null = null;

export type CurrentAbilityBoostIpcMockMode =
  | "success"
  | "pending"
  | "eligibilityError"
  | "liveValueError"
  | "snapshotSyncError";

export type PlayerHiddenInformationIpcMockMode = "success" | "error";

export type WonderkidMentalityBoostIpcMockMode =
  | "success"
  | "pending"
  | "eligibilityError"
  | "liveValueError"
  | "snapshotSyncError";

export function setGetPlayerOverride(player: PlayerDetail | null | undefined) {
  getPlayerOverride = player;
  if (player !== undefined) {
    playerHiddenInformationRevealed = player?.hiddenInformationRevealed ?? true;
  }
}

export function resetGetPlayerOverride() {
  getPlayerOverride = undefined;
  playerHiddenInformationRevealed = true;
  playerHiddenInformationMode = "success";
  playerHiddenInformationCalls = [];
  currentAbilityBoostMode = "success";
  currentAbilityBoostCalls = [];
  pendingCurrentAbilityBoost = null;
  wonderkidMentalityBoostMode = "success";
  wonderkidMentalityBoostCalls = [];
  pendingWonderkidMentalityBoost = null;
}

export function setPlayerHiddenInformationRevealedIpcMockMode(
  mode: PlayerHiddenInformationIpcMockMode,
) {
  playerHiddenInformationMode = mode;
}

export function getSetPlayerHiddenInformationRevealedIpcMockCalls() {
  return playerHiddenInformationCalls;
}

export function getPlayerHiddenInformationRevealedIpcMock() {
  return playerHiddenInformationRevealed;
}

export function setCurrentAbilityBoostIpcMockMode(
  mode: CurrentAbilityBoostIpcMockMode,
) {
  currentAbilityBoostMode = mode;
  if (mode !== "pending") {
    pendingCurrentAbilityBoost = null;
  }
}

export function getCurrentAbilityBoostIpcMockCalls() {
  return currentAbilityBoostCalls;
}

export function resolvePendingCurrentAbilityBoostIpcMock(
  result = currentAbilityBoostResult(),
) {
  pendingCurrentAbilityBoost?.resolve(result);
  pendingCurrentAbilityBoost = null;
}

export function setWonderkidMentalityBoostIpcMockMode(
  mode: WonderkidMentalityBoostIpcMockMode,
) {
  wonderkidMentalityBoostMode = mode;
  if (mode !== "pending") {
    pendingWonderkidMentalityBoost = null;
  }
}

export function getWonderkidMentalityBoostIpcMockCalls() {
  return wonderkidMentalityBoostCalls;
}

export function resolvePendingWonderkidMentalityBoostIpcMock() {
  const result = wonderkidMentalityBoostResult();
  pendingWonderkidMentalityBoost?.resolve(result);
  pendingWonderkidMentalityBoost = null;
}

export function fixturePlayerDetail(
  overrides: Partial<PlayerDetail> = {},
): PlayerDetail {
  return {
    uid: 42,
    name: "Alex Scout",
    age: 25,
    birthYear: 2001,
    birthDayOfYear: 80,
    nationalities: ["England", "Wales"],
    heightCm: 182,
    preferredFoot: "right",
    positions: {
      GK: 0,
      SW: null,
      DL: null,
      DC: null,
      DR: null,
      DM: null,
      ML: null,
      MC: 20,
      MR: null,
      AML: null,
      AMC: null,
      AMR: null,
      ST: null,
      WBL: null,
      WBR: null,
    },
    attributes: { Acceleration: 14 },
    potentialAttributes: { Acceleration: 16 },
    hiddenAttributes: { Consistency: 12 },
    personality: { Ambition: 15 },
    weeklyWageGbp: 50_000,
    contractExpiryYear: 2028,
    contractExpiryDayOfYear: 1,
    transferListed: false,
    loanListed: null,
    notForSale: null,
    setForRelease: null,
    marketValueGbp: 12_500_000,
    reputationCurrent: 5000,
    reputationWorld: 4000,
    club: "Test FC",
    parentClub: null,
    onLoan: false,
    division: "Premier Division",
    teamLevel: "First",
    ca: 140,
    pa: 160,
    hiddenInformationRevealed: playerHiddenInformationRevealed,
    roleScores: [
      {
        roleId: "goalkeeper_ip",
        displayName: "Goalkeeper",
        phase: "in_possession",
        positionTags: ["GK"],
        score: 40,
        potentialScore: 47,
      },
      {
        roleId: "centre_back_ip",
        displayName: "Centre-Back",
        phase: "in_possession",
        positionTags: ["DC"],
        score: 48,
        potentialScore: null,
      },
      {
        roleId: "deep_lying_playmaker_ip",
        displayName: "Deep-Lying Playmaker",
        phase: "in_possession",
        positionTags: ["DM", "MC"],
        score: 82,
        potentialScore: 94,
      },
      {
        roleId: "central_midfielder_ip",
        displayName: "Central Midfielder",
        phase: "in_possession",
        positionTags: ["MC"],
        score: 72,
        potentialScore: 84,
      },
      {
        roleId: "advanced_forward_ip",
        displayName: "Advanced Forward",
        phase: "in_possession",
        positionTags: ["ST"],
        score: 55,
        potentialScore: 67,
      },
    ],
    ...overrides,
  };
}

export function resolveGetPlayerIpcMock(args: unknown): PlayerDetail | null {
  if (getPlayerOverride !== undefined) {
    return getPlayerOverride;
  }

  const uid =
    typeof args === "object" &&
    args !== null &&
    "uid" in args &&
    typeof (args as { uid: unknown }).uid === "number"
      ? (args as { uid: number }).uid
      : null;

  if (uid === 42) {
    return fixturePlayerDetail();
  }

  return null;
}

export function resolveSetPlayerHiddenInformationRevealedIpcMock(
  args: unknown,
): Promise<boolean> {
  playerHiddenInformationCalls = [...playerHiddenInformationCalls, args];
  if (playerHiddenInformationMode === "error") {
    return Promise.reject(new Error("save preference could not be updated"));
  }

  const revealed =
    typeof args === "object" &&
    args !== null &&
    "revealed" in args &&
    typeof (args as { revealed: unknown }).revealed === "boolean"
      ? (args as { revealed: boolean }).revealed
      : null;
  if (revealed === null) {
    return Promise.reject(new Error("Missing revealed state"));
  }

  const player =
    getPlayerOverride === undefined ? fixturePlayerDetail() : getPlayerOverride;
  if (player !== null) {
    playerHiddenInformationRevealed = revealed;
    getPlayerOverride = {
      ...player,
      hiddenInformationRevealed: revealed,
    };
  }
  return Promise.resolve(revealed);
}

function currentAbilityBoostResult(): PlayerBoostResult {
  const player =
    getPlayerOverride === undefined ? fixturePlayerDetail() : getPlayerOverride;
  if (player === null) {
    throw new Error("Player not found");
  }
  if (player.pa === null) {
    throw new Error("Potential ability is unavailable");
  }
  const increment = player.age !== null && player.age <= 20 ? 5 : 10;
  const currentAbility = Math.min(player.ca + increment, player.pa, 200);
  const result: PlayerBoostResult = {
    snapshotId: 1,
    operation: "boost-current-ability",
    previousCurrentAbility: player.ca,
    currentAbility,
    potentialAbility: player.pa,
    previousAmbition: null,
    ambition: null,
    previousProfessionalism: null,
    professionalism: null,
    previousDetermination: null,
    determination: null,
  };
  getPlayerOverride = { ...player, ca: currentAbility };
  return result;
}

export function resolveBoostCurrentAbilityIpcMock(
  args: unknown,
): Promise<PlayerBoostResult> {
  currentAbilityBoostCalls = [...currentAbilityBoostCalls, args];

  if (currentAbilityBoostMode === "pending") {
    if (!pendingCurrentAbilityBoost) {
      let resolve!: (result: PlayerBoostResult) => void;
      const promise = new Promise<PlayerBoostResult>((next) => {
        resolve = next;
      });
      pendingCurrentAbilityBoost = { promise, resolve };
    }
    return pendingCurrentAbilityBoost.promise;
  }

  if (currentAbilityBoostMode === "eligibilityError") {
    return Promise.reject({
      phase: "eligibility",
      kind: "unknownAge",
      message: "player age is unknown",
    });
  }
  if (currentAbilityBoostMode === "liveValueError") {
    return Promise.reject({
      phase: "liveValue",
      message: "player values changed in FM; Load Data again",
    });
  }
  if (currentAbilityBoostMode === "snapshotSyncError") {
    return Promise.reject({
      phase: "snapshotSync",
      message: "FM may have changed. Load Data again.",
    });
  }

  return Promise.resolve(currentAbilityBoostResult());
}

function wonderkidMentalityTarget(value: number | null | undefined) {
  return typeof value === "number" && value >= 1 && value <= 10
    ? value + 10
    : (value ?? null);
}

function wonderkidMentalityBoostResult(): PlayerBoostResult {
  const player =
    getPlayerOverride === undefined ? fixturePlayerDetail() : getPlayerOverride;
  if (player === null) {
    throw new Error("Player not found");
  }

  const previousAmbition = player.personality.Ambition ?? null;
  const previousProfessionalism = player.personality.Professionalism ?? null;
  const previousDetermination = player.attributes.Determination ?? null;
  const ambition = wonderkidMentalityTarget(previousAmbition);
  const professionalism = wonderkidMentalityTarget(previousProfessionalism);
  const determination = wonderkidMentalityTarget(previousDetermination);
  const result: PlayerBoostResult = {
    snapshotId: 1,
    operation: "wonderkid-mentality",
    previousCurrentAbility: null,
    currentAbility: null,
    potentialAbility: null,
    previousAmbition,
    ambition,
    previousProfessionalism,
    professionalism,
    previousDetermination,
    determination,
  };
  getPlayerOverride = {
    ...player,
    attributes: {
      ...player.attributes,
      Determination: determination,
    },
    personality: {
      ...player.personality,
      Ambition: ambition,
      Professionalism: professionalism,
    },
  };
  return result;
}

export function resolveBoostWonderkidMentalityIpcMock(
  args: unknown,
): Promise<PlayerBoostResult> {
  wonderkidMentalityBoostCalls = [...wonderkidMentalityBoostCalls, args];

  if (wonderkidMentalityBoostMode === "pending") {
    if (!pendingWonderkidMentalityBoost) {
      let resolve!: (result: PlayerBoostResult) => void;
      const promise = new Promise<PlayerBoostResult>((next) => {
        resolve = next;
      });
      pendingWonderkidMentalityBoost = { promise, resolve };
    }
    return pendingWonderkidMentalityBoost.promise;
  }

  if (wonderkidMentalityBoostMode === "eligibilityError") {
    return Promise.reject({
      phase: "eligibility",
      kind: "noEligibleMentality",
      message: "no known mentality attribute is 10 or lower",
    });
  }
  if (wonderkidMentalityBoostMode === "liveValueError") {
    return Promise.reject({
      phase: "liveValue",
      message: "player values changed in FM; Load Data again",
    });
  }
  if (wonderkidMentalityBoostMode === "snapshotSyncError") {
    return Promise.reject({
      phase: "snapshotSync",
      message: "FM may have changed. Load Data again.",
    });
  }

  return Promise.resolve(wonderkidMentalityBoostResult());
}
