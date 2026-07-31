import type {
  ClubFamily,
  ClubSourceInput,
} from "@/features/planner/types/club-family";
import type {
  PlannerTactic,
  TacticOptions,
} from "@/features/planner/types/tactic";

const DEFAULT_CLUB_FAMILY: ClubFamily = {
  primaryClub: null,
  sources: [],
};

const DEFAULT_TACTIC: PlannerTactic = {
  ipWeight: 0.5,
  lanes: [
    ["goalkeeper", "GK", "goalkeeper_ip", "GK", "line_holding_keeper_oop"],
    ["left_back", "DL", "full_back_ip", "DL", "holding_full_back_oop"],
    [
      "left_centre_back",
      "DC",
      "centre_back_ip",
      "DC",
      "covering_centre_back_oop",
    ],
    [
      "right_centre_back",
      "DC",
      "centre_back_ip",
      "DC",
      "covering_centre_back_oop",
    ],
    ["right_back", "DR", "full_back_ip", "DR", "holding_full_back_oop"],
    [
      "defensive_midfielder",
      "DM",
      "defensive_midfielder_ip",
      "DM",
      "screening_defensive_midfielder_oop",
    ],
    [
      "left_central_midfielder",
      "MC",
      "central_midfielder_ip",
      "MC",
      "pressing_central_midfielder_oop",
    ],
    [
      "right_central_midfielder",
      "MC",
      "central_midfielder_ip",
      "MC",
      "pressing_central_midfielder_oop",
    ],
    ["left_winger", "AML", "winger_ip", "ML", "tracking_wide_midfielder_oop"],
    ["right_winger", "AMR", "winger_ip", "MR", "tracking_wide_midfielder_oop"],
    [
      "centre_forward",
      "ST",
      "centre_forward_ip",
      "ST",
      "central_outlet_centre_forward_oop",
    ],
  ].map(([laneId, ipPosition, ipRoleId, oopPosition, oopRoleId]) => ({
    laneId,
    ipPosition,
    ipRoleId,
    oopPosition,
    oopRoleId,
  })),
};

const DEFAULT_TACTIC_OPTIONS: TacticOptions = {
  placements: [
    "GK",
    "DL",
    "DC",
    "DR",
    "DM",
    "MC",
    "ML",
    "MR",
    "AML",
    "AMR",
    "ST",
  ],
  roles: [],
};

let clubFamily: ClubFamily = { ...DEFAULT_CLUB_FAMILY, sources: [] };
let availableClubs: string[] = [];
let tactic: PlannerTactic = cloneTactic(DEFAULT_TACTIC);

function cloneTactic(value: PlannerTactic): PlannerTactic {
  return {
    ipWeight: value.ipWeight,
    lanes: value.lanes.map((lane) => ({ ...lane })),
  };
}

export function resetPlannerIpcMock() {
  clubFamily = { ...DEFAULT_CLUB_FAMILY, sources: [] };
  availableClubs = [];
  tactic = cloneTactic(DEFAULT_TACTIC);
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

export function resolvePlannerTacticIpcMock() {
  return cloneTactic(tactic);
}

export function resolvePlannerTacticOptionsIpcMock() {
  return {
    placements: [...DEFAULT_TACTIC_OPTIONS.placements],
    roles: [...DEFAULT_TACTIC_OPTIONS.roles],
  };
}

export function resolveSavePlannerTacticIpcMock(args: unknown) {
  const record = args as { tactic?: PlannerTactic };
  if (!record.tactic) {
    throw "Tactic is required";
  }
  tactic = cloneTactic(record.tactic);
  return resolvePlannerTacticIpcMock();
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
