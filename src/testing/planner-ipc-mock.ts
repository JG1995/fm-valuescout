import type {
  ClubFamily,
  ClubSourceInput,
} from "@/features/planner/types/club-family";
import type {
  PlannerTactic,
  TacticOptions,
  TacticRoleOption,
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
  roles: [
    tacticRole("goalkeeper_ip", "Goalkeeper", "in_possession", ["GK"]),
    tacticRole(
      "ball_playing_goalkeeper_ip",
      "Ball-Playing Goalkeeper",
      "in_possession",
      ["GK"],
    ),
    tacticRole(
      "line_holding_keeper_oop",
      "Line-Holding Keeper",
      "out_of_possession",
      ["GK"],
    ),
    tacticRole("sweeper_keeper_oop", "Sweeper Keeper", "out_of_possession", [
      "GK",
    ]),
    tacticRole("full_back_ip", "Full-Back", "in_possession", ["DL", "DR"]),
    tacticRole("inside_full_back_ip", "Inside Full-Back", "in_possession", [
      "DL",
      "DR",
    ]),
    tacticRole(
      "holding_full_back_oop",
      "Holding Full-Back",
      "out_of_possession",
      ["DL", "DR"],
    ),
    tacticRole(
      "pressing_full_back_oop",
      "Pressing Full-Back",
      "out_of_possession",
      ["DL", "DR"],
    ),
    tacticRole("centre_back_ip", "Centre-Back", "in_possession", ["DC"]),
    tacticRole(
      "ball_playing_centre_back_ip",
      "Ball-Playing Centre-Back",
      "in_possession",
      ["DC"],
    ),
    tacticRole(
      "covering_centre_back_oop",
      "Covering Centre-Back",
      "out_of_possession",
      ["DC"],
    ),
    tacticRole(
      "stopping_centre_back_oop",
      "Stopping Centre-Back",
      "out_of_possession",
      ["DC"],
    ),
    tacticRole(
      "defensive_midfielder_ip",
      "Defensive Midfielder",
      "in_possession",
      ["DM"],
    ),
    tacticRole(
      "box_to_box_midfielder_ip",
      "Box-to-Box Midfielder",
      "in_possession",
      ["DM", "MC"],
    ),
    tacticRole(
      "deep_lying_playmaker_ip",
      "Deep-Lying Playmaker",
      "in_possession",
      ["DM", "MC"],
    ),
    tacticRole(
      "screening_defensive_midfielder_oop",
      "Screening Defensive Midfielder",
      "out_of_possession",
      ["DM"],
    ),
    tacticRole(
      "pressing_defensive_midfielder_oop",
      "Pressing Defensive Midfielder",
      "out_of_possession",
      ["DM"],
    ),
    tacticRole("central_midfielder_ip", "Central Midfielder", "in_possession", [
      "MC",
    ]),
    tacticRole("advanced_playmaker_ip", "Advanced Playmaker", "in_possession", [
      "MC",
    ]),
    tacticRole(
      "pressing_central_midfielder_oop",
      "Pressing Central Midfielder",
      "out_of_possession",
      ["MC"],
    ),
    tacticRole("wide_midfielder_ip", "Wide Midfielder", "in_possession", [
      "ML",
      "MR",
    ]),
    tacticRole("winger_ip", "Winger", "in_possession", [
      "ML",
      "MR",
      "AML",
      "AMR",
    ]),
    tacticRole(
      "tracking_wide_midfielder_oop",
      "Tracking Wide Midfielder",
      "out_of_possession",
      ["ML", "MR"],
    ),
    tacticRole("inside_winger_ip", "Inside Winger", "in_possession", [
      "ML",
      "MR",
      "AML",
      "AMR",
    ]),
    tacticRole(
      "inside_outlet_winger_oop",
      "Inside Outlet Winger",
      "out_of_possession",
      ["AML", "AMR"],
    ),
    tacticRole("tracking_winger_oop", "Tracking Winger", "out_of_possession", [
      "AML",
      "AMR",
    ]),
    tacticRole("centre_forward_ip", "Centre Forward", "in_possession", ["ST"]),
    tacticRole("deep_lying_forward_ip", "Deep-Lying Forward", "in_possession", [
      "ST",
    ]),
    tacticRole(
      "central_outlet_centre_forward_oop",
      "Central Outlet Centre Forward",
      "out_of_possession",
      ["ST"],
    ),
  ],
};

function tacticRole(
  roleId: string,
  displayName: string,
  phase: TacticRoleOption["phase"],
  positionTags: string[],
): TacticRoleOption {
  return { roleId, displayName, phase, positionTags };
}

let clubFamily: ClubFamily = { ...DEFAULT_CLUB_FAMILY, sources: [] };
let availableClubs: string[] = [];
let tactic: PlannerTactic = cloneTactic(DEFAULT_TACTIC);
let tacticSaveError: string | null = null;

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
  tacticSaveError = null;
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

export function setPlannerTacticSaveError(message: string | null) {
  tacticSaveError = message;
}

export function resolveSavePlannerTacticIpcMock(args: unknown) {
  if (tacticSaveError) {
    throw tacticSaveError;
  }
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
