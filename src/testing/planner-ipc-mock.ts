import type {
  ClubFamily,
  ClubSourceInput,
} from "@/features/planner/types/club-family";

const DEFAULT_CLUB_FAMILY: ClubFamily = {
  primaryClub: null,
  sources: [],
};

let clubFamily: ClubFamily = { ...DEFAULT_CLUB_FAMILY, sources: [] };
let availableClubs: string[] = [];

export function resetPlannerIpcMock() {
  clubFamily = { ...DEFAULT_CLUB_FAMILY, sources: [] };
  availableClubs = [];
}

export function setPlannerAvailableClubs(clubs: string[]) {
  availableClubs = [...clubs];
}

export function resolvePlannerClubFamilyIpcMock(): ClubFamily {
  return {
    ...clubFamily,
    sources: clubFamily.sources.map((source) => ({ ...source })),
  };
}

export function resolvePlannerClubsIpcMock() {
  return [...availableClubs];
}

export function resolveSavePlannerClubFamilyIpcMock(args: unknown): ClubFamily {
  const record = args as {
    primaryClub?: unknown;
    sources?: unknown;
  };
  const primaryClub =
    typeof record.primaryClub === "string" ? record.primaryClub.trim() : "";
  if (!primaryClub) {
    throw "Primary club must not be empty";
  }

  const sources = Array.isArray(record.sources)
    ? (record.sources as Array<ClubSourceInput & { id?: number }>).map(
        (source, index) => ({
          id: source.id ?? -(index + 1),
          team: source.team,
          clubName: source.clubName,
          teamLevel: source.teamLevel,
          isPrimary: false,
        }),
      )
    : [];
  clubFamily = {
    primaryClub,
    sources: [
      {
        id: 1,
        team: "senior",
        clubName: primaryClub,
        teamLevel: "senior",
        isPrimary: true,
      },
      {
        id: 2,
        team: "reserves",
        clubName: primaryClub,
        teamLevel: "reserve",
        isPrimary: true,
      },
      {
        id: 3,
        team: "youth",
        clubName: primaryClub,
        teamLevel: "youth",
        isPrimary: true,
      },
      ...sources,
    ],
  };
  return resolvePlannerClubFamilyIpcMock();
}
