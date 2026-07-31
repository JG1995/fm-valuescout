import type { PlayerDetail } from "@/features/player-profile/types/player-detail";

let getPlayerOverride: PlayerDetail | null | undefined;

export function setGetPlayerOverride(player: PlayerDetail | null | undefined) {
  getPlayerOverride = player;
}

export function resetGetPlayerOverride() {
  getPlayerOverride = undefined;
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
    nationalities: ["ENG", "WAL"],
    heightCm: 182,
    preferredFoot: "Right",
    positions: { MC: 20 },
    attributes: { Acceleration: 14 },
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
    roleScores: [
      {
        roleId: "central_midfielder_ip",
        displayName: "Central Midfielder",
        phase: "ip",
        positionTags: ["MC"],
        score: 72,
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
