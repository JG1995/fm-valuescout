import type {
  ClubFamily,
  ClubSourceInput,
} from "@/features/planner/types/club-family";
import type {
  PlannerDepth,
  PlannerSlotCandidate,
} from "@/features/planner/types/depth";
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
    ipWeight: 0.5,
    importanceRank: null,
    preferredFoot: "any",
    footPreference: "preferred",
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
let clubFamilySaveCalls = 0;
let tactic: PlannerTactic = cloneTactic(DEFAULT_TACTIC);
let depth: PlannerDepth = buildDefaultDepth();
let depthFetchCount = 0;
let tacticSaveError: string | null = null;
let slotCandidates: PlannerSlotCandidate[] = [];
let assignmentError: string | null = null;
let addStringError: string | null = null;
let addStringPending = false;
let addStringCalls = 0;
let clearTeamError: string | null = null;
let clearTeamPending = false;
let clearTeamCalls = 0;
let optimizeDepth: PlannerDepth | null = null;
let optimizeError: string | null = null;
let optimizePending = false;
let optimizeCalls = 0;

function cloneTactic(value: PlannerTactic): PlannerTactic {
  return {
    lanes: value.lanes.map((lane) => ({ ...lane })),
  };
}

function cloneDepth(value: PlannerDepth): PlannerDepth {
  return {
    tactic: cloneTactic(value.tactic),
    teams: value.teams.map((team) => ({
      team: team.team,
      strings: team.strings.map((plannerString) => ({
        id: plannerString.id,
        stringOrder: plannerString.stringOrder,
        assignments: plannerString.assignments.map((assignment) => ({
          ...assignment,
        })),
      })),
    })),
  };
}

function cloneSlotCandidates(value: PlannerSlotCandidate[]) {
  return value.map((candidate) => ({
    ...candidate,
    assignmentLocation: candidate.assignmentLocation
      ? { ...candidate.assignmentLocation }
      : null,
  }));
}

function buildDefaultDepth(): PlannerDepth {
  return {
    tactic: cloneTactic(DEFAULT_TACTIC),
    teams: ["senior", "reserves", "youth"].map((team, index) => ({
      team: team as PlannerDepth["teams"][number]["team"],
      strings: [{ id: index + 1, stringOrder: 0, assignments: [] }],
    })),
  };
}

export function resetPlannerIpcMock() {
  clubFamily = { ...DEFAULT_CLUB_FAMILY, sources: [] };
  availableClubs = [];
  clubFamilySaveCalls = 0;
  tactic = cloneTactic(DEFAULT_TACTIC);
  depth = buildDefaultDepth();
  depthFetchCount = 0;
  tacticSaveError = null;
  slotCandidates = [];
  assignmentError = null;
  addStringError = null;
  addStringPending = false;
  addStringCalls = 0;
  clearTeamError = null;
  clearTeamPending = false;
  clearTeamCalls = 0;
  optimizeDepth = null;
  optimizeError = null;
  optimizePending = false;
  optimizeCalls = 0;
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

export function getPlannerClubFamilySaveCalls() {
  return clubFamilySaveCalls;
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

export function resolvePlannerDepthIpcMock(): PlannerDepth {
  depthFetchCount += 1;
  return cloneDepth(depth);
}

export function getPlannerDepthIpcMockCalls() {
  return depthFetchCount;
}

export function setPlannerDepthIpcMock(value: PlannerDepth) {
  depth = cloneDepth(value);
}

export function setPlannerSlotCandidates(value: PlannerSlotCandidate[]) {
  slotCandidates = cloneSlotCandidates(value);
}

export function setPlannerAssignmentError(message: string | null) {
  assignmentError = message;
}

export function setPlannerAddStringError(message: string | null) {
  addStringError = message;
}

export function setPlannerAddStringPending(value: boolean) {
  addStringPending = value;
}

export function getPlannerAddStringIpcMockCalls() {
  return addStringCalls;
}

export function setPlannerClearTeamError(message: string | null) {
  clearTeamError = message;
}

export function setPlannerClearTeamPending(value: boolean) {
  clearTeamPending = value;
}

export function getPlannerClearTeamIpcMockCalls() {
  return clearTeamCalls;
}

export function setPlannerOptimizeDepth(value: PlannerDepth | null) {
  optimizeDepth = value ? cloneDepth(value) : null;
}

export function setPlannerOptimizeError(message: string | null) {
  optimizeError = message;
}

export function setPlannerOptimizePending(value: boolean) {
  optimizePending = value;
}

export function getPlannerOptimizeIpcMockCalls() {
  return optimizeCalls;
}

export function resolvePlannerSlotCandidatesIpcMock(args: unknown) {
  const search =
    typeof args === "object" &&
    args !== null &&
    "search" in args &&
    typeof args.search === "string"
      ? args.search.trim().toLowerCase()
      : "";
  return cloneSlotCandidates(
    slotCandidates
      .filter((candidate) => candidate.name.toLowerCase().includes(search))
      .map((candidate) => ({
        ...candidate,
        assignmentLocation: assignmentLocation(candidate.playerUid),
      })),
  );
}

function assignmentLocation(playerUid: number) {
  for (const team of depth.teams) {
    for (const plannerString of team.strings) {
      const assignment = plannerString.assignments.find(
        (candidate) => candidate.playerUid === playerUid,
      );
      if (assignment) {
        return {
          team: team.team,
          stringId: plannerString.id,
          stringOrder: plannerString.stringOrder,
          laneId: assignment.laneId,
        };
      }
    }
  }
  return null;
}

type PlannerAssignmentIpcArgs = {
  stringId: number;
  laneId: string;
  playerUid: number;
};

type PlannerSlotIpcArgs = {
  stringId: number;
  laneId: string;
};

function plannerSlotArgs(args: unknown): PlannerSlotIpcArgs {
  if (
    typeof args !== "object" ||
    args === null ||
    !("stringId" in args) ||
    !("laneId" in args) ||
    typeof args.stringId !== "number" ||
    typeof args.laneId !== "string"
  ) {
    throw "Invalid planner slot";
  }
  return { stringId: args.stringId, laneId: args.laneId };
}

function plannerAssignmentArgs(args: unknown): PlannerAssignmentIpcArgs {
  if (
    typeof args !== "object" ||
    args === null ||
    !("stringId" in args) ||
    !("laneId" in args) ||
    !("playerUid" in args) ||
    typeof args.stringId !== "number" ||
    typeof args.laneId !== "string" ||
    typeof args.playerUid !== "number"
  ) {
    throw "Invalid planner assignment";
  }
  return {
    stringId: args.stringId,
    laneId: args.laneId,
    playerUid: args.playerUid,
  };
}

function resolvePlannerAssignmentIpcMock(args: unknown, move: boolean) {
  if (assignmentError) {
    throw assignmentError;
  }
  const { stringId, laneId, playerUid } = plannerAssignmentArgs(args);
  const target = depth.teams
    .flatMap((team) => team.strings)
    .find((plannerString) => plannerString.id === stringId);
  if (!target) {
    throw "Planner string not found";
  }
  if (target.assignments.some((assignment) => assignment.laneId === laneId)) {
    throw "Planner cell is already occupied";
  }
  const existing = depth.teams
    .flatMap((team) => team.strings)
    .flatMap((plannerString) => plannerString.assignments)
    .find((assignment) => assignment.playerUid === playerUid);
  if (existing && !move) {
    throw `Player ${playerUid} is already assigned`;
  }
  if (!existing && move) {
    throw `Player ${playerUid} is not assigned`;
  }
  if (existing) {
    for (const team of depth.teams) {
      for (const plannerString of team.strings) {
        plannerString.assignments = plannerString.assignments.filter(
          (assignment) => assignment.playerUid !== playerUid,
        );
      }
    }
  }
  const candidate = slotCandidates.find(
    (slotCandidate) => slotCandidate.playerUid === playerUid,
  );
  target.assignments.push({
    id: -playerUid,
    laneId,
    playerUid,
    lastKnownName: candidate?.name ?? `Player ${playerUid}`,
    currentName: candidate?.name ?? `Player ${playerUid}`,
    state: "resolved",
    combinedScore: candidate?.combinedScore ?? null,
  });
  return cloneDepth(depth);
}

export function resolveAssignPlannerPlayerIpcMock(args: unknown) {
  return resolvePlannerAssignmentIpcMock(args, false);
}

export function resolveMovePlannerPlayerIpcMock(args: unknown) {
  return resolvePlannerAssignmentIpcMock(args, true);
}

export function resolveClearPlannerAssignmentIpcMock(args: unknown) {
  if (assignmentError) {
    throw assignmentError;
  }
  const { stringId, laneId } = plannerSlotArgs(args);
  const target = depth.teams
    .flatMap((team) => team.strings)
    .find((plannerString) => plannerString.id === stringId);
  if (!target) {
    throw "Planner string not found";
  }
  target.assignments = target.assignments.filter(
    (assignment) => assignment.laneId !== laneId,
  );
  return cloneDepth(depth);
}

export function resolveAddPlannerStringIpcMock(args: unknown) {
  addStringCalls += 1;
  if (addStringError) {
    throw addStringError;
  }
  if (addStringPending) {
    return new Promise<PlannerDepth>(() => {});
  }
  if (
    typeof args !== "object" ||
    args === null ||
    !("team" in args) ||
    (args.team !== "senior" &&
      args.team !== "reserves" &&
      args.team !== "youth")
  ) {
    throw "Invalid planner team";
  }
  const team = depth.teams.find((candidate) => candidate.team === args.team);
  if (!team) {
    throw "Planner team not found";
  }
  const id =
    Math.max(
      ...depth.teams.flatMap((candidate) =>
        candidate.strings.map((plannerString) => plannerString.id),
      ),
    ) + 1;
  team.strings.push({ id, stringOrder: team.strings.length, assignments: [] });
  return cloneDepth(depth);
}

export function resolveRemovePlannerStringIpcMock(args: unknown) {
  if (
    typeof args !== "object" ||
    args === null ||
    !("stringId" in args) ||
    !("confirmPopulated" in args) ||
    typeof args.stringId !== "number" ||
    typeof args.confirmPopulated !== "boolean"
  ) {
    throw "Invalid planner string";
  }
  if (assignmentError) {
    throw assignmentError;
  }
  const team = depth.teams.find((candidate) =>
    candidate.strings.some(
      (plannerString) => plannerString.id === args.stringId,
    ),
  );
  const plannerString = team?.strings.find(
    (candidate) => candidate.id === args.stringId,
  );
  if (!team || !plannerString) {
    throw "Planner string not found";
  }
  if (team.strings.length <= 1) {
    throw `The ${team.team} team must keep at least one string`;
  }
  if (plannerString.assignments.length > 0 && !args.confirmPopulated) {
    throw "Removing a populated string requires confirmation";
  }
  team.strings = team.strings
    .filter((candidate) => candidate.id !== args.stringId)
    .map((candidate, index) => ({ ...candidate, stringOrder: index }));
  return cloneDepth(depth);
}

export function resolveClearPlannerTeamIpcMock(args: unknown) {
  clearTeamCalls += 1;
  if (
    typeof args !== "object" ||
    args === null ||
    !("team" in args) ||
    !("confirmed" in args) ||
    (args.team !== "senior" &&
      args.team !== "reserves" &&
      args.team !== "youth") ||
    typeof args.confirmed !== "boolean"
  ) {
    throw "Invalid planner team";
  }
  if (!args.confirmed) {
    throw "Clearing a squad requires confirmation";
  }
  if (clearTeamError) {
    throw clearTeamError;
  }
  if (clearTeamPending) {
    return new Promise<PlannerDepth>(() => {});
  }
  const team = depth.teams.find((candidate) => candidate.team === args.team);
  if (!team) {
    throw "Planner team not found";
  }
  team.strings = team.strings.map((plannerString) => ({
    ...plannerString,
    assignments: [],
  }));
  return cloneDepth(depth);
}

export function resolveOptimizePlannerDepthIpcMock(args: unknown) {
  optimizeCalls += 1;
  if (
    args !== undefined &&
    args !== null &&
    (typeof args !== "object" || Object.keys(args).length > 0)
  ) {
    throw "Optimizer does not accept arguments";
  }
  if (optimizeError) {
    throw optimizeError;
  }
  if (optimizePending) {
    return new Promise<PlannerDepth>(() => {});
  }
  if (optimizeDepth) {
    depth = cloneDepth(optimizeDepth);
  }
  return cloneDepth(depth);
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
  clubFamilySaveCalls += 1;
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
