import type { ManagedClubStatus } from "@/features/managed-club/types/managed-club";
import type {
  PlannerDepth,
  PlannerSlotCandidate,
} from "@/features/planner/types/depth";
import type {
  PlannerRoleReference,
  PlannerRoleReferencePhase,
  PlannerRoleReferenceScoreBasis,
} from "@/features/planner/types/role-reference";
import type {
  PlannerTactic,
  TacticOptions,
  TacticRoleOption,
} from "@/features/planner/types/tactic";
import { PLANNER_TEAMS, type PlannerTeam } from "@/features/planner/types/team";
import type { PlannerTeamRemovalImpact } from "@/features/planner/types/team-removal-impact";
import { resolveListSavesIpcMock } from "@/testing/snapshot-ipc-mock";

const DEFAULT_MANAGED_CLUB: ManagedClubStatus = {
  clubName: null,
  status: "unconfigured",
  unclassifiedPlayerCount: 0,
};

const DEFAULT_TACTIC: PlannerTactic = {
  lanes: [
    ["goalkeeper", "GK", "goalkeeper_ip", "GK", "line_holding_keeper_oop"],
    ["left_back", "DL", "full_back_ip", "DL", "holding_full_back_oop"],
    [
      "left_centre_back",
      "DCR",
      "centre_back_ip",
      "DCR",
      "covering_centre_back_oop",
    ],
    [
      "right_centre_back",
      "DCL",
      "centre_back_ip",
      "DCL",
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
      "MCR",
      "central_midfielder_ip",
      "MCR",
      "pressing_central_midfielder_oop",
    ],
    [
      "right_central_midfielder",
      "MCL",
      "central_midfielder_ip",
      "MCL",
      "pressing_central_midfielder_oop",
    ],
    ["left_winger", "AML", "winger_ip", "ML", "tracking_wide_midfielder_oop"],
    ["right_winger", "AMR", "winger_ip", "MR", "tracking_wide_midfielder_oop"],
    [
      "centre_forward",
      "STC",
      "centre_forward_ip",
      "STC",
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
    "DCR",
    "DC",
    "DCL",
    "DR",
    "WBR",
    "WBL",
    "DMCR",
    "DM",
    "DMCL",
    "MCR",
    "MC",
    "MCL",
    "ML",
    "MR",
    "AML",
    "AMCR",
    "AMC",
    "AMCL",
    "AMR",
    "STCR",
    "STC",
    "STCL",
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

let managedClub: ManagedClubStatus = { ...DEFAULT_MANAGED_CLUB };
let availableClubs: string[] = [];
let managedClubOptionsError: Error | null = null;
let managedClubSaveCalls = 0;
let onManagedClubSaveCall: (() => void) | undefined;
let managedClubSavePending = false;
let pendingManagedClubSave: {
  result: ManagedClubStatus;
  resolve: (result: ManagedClubStatus) => void;
} | null = null;
const tacticStore = new Map<string, PlannerTactic>([
  [`1:save-token-1`, cloneTactic(DEFAULT_TACTIC)],
]);
let tacticReadCalls: Array<{ saveId: number; contextToken: string }> = [];
let tacticOptionsReadCalls: Array<{ saveId: number; contextToken: string }> =
  [];
let tacticSaveCalls: Array<{ saveId: number; contextToken: string }> = [];
let depth: PlannerDepth = buildDefaultDepth();
let depthError: string | null = null;
let roleReference: PlannerRoleReference = buildDefaultRoleReference();
let roleReferenceError: string | null = null;
let roleReferenceCalls: Array<{
  phase: PlannerRoleReferencePhase;
  scoreBasis: PlannerRoleReferenceScoreBasis;
}> = [];
let depthFetchCount = 0;
let tacticSaveError: string | null = null;
let slotCandidates: PlannerSlotCandidate[] = [];
let assignmentError: string | null = null;
let addStringError: string | null = null;
let addStringPending = false;
let addStringCalls = 0;
let clearAllError: string | null = null;
let clearAllPending = false;
let clearAllCalls = 0;
let slotCandidateFetchCount = 0;
let optimizeDepth: PlannerDepth | null = null;
let optimizeError: string | null = null;
let optimizePending = false;
let optimizeCalls = 0;
let optimizeBases: string[] = [];
let teamSaveError: string | null = null;
let teamSavePending = false;
let teamRemovalImpactOverride: PlannerTeamRemovalImpact[] | null = null;
let teamRemovalImpactPending = false;
let pendingTeamRemovalImpact: (() => void) | null = null;
let teamSaveCalls: Array<{
  teams: Array<{ team: PlannerTeam; displayName: string }>;
  confirmPopulatedRemoval: boolean;
}> = [];
let pendingPlannerTeamSave: {
  teams: Array<{ team: PlannerTeam; displayName: string }>;
  confirmPopulatedRemoval: boolean;
  resolve: (depth: PlannerDepth) => void;
} | null = null;

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
      displayName: team.displayName,
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

function cloneRoleReference(value: PlannerRoleReference): PlannerRoleReference {
  return {
    lanes: value.lanes.map((lane) => ({
      laneId: lane.laneId,
      players: lane.players.map((player) => ({ ...player })),
    })),
    noEligible: value.noEligible.map((player) => ({ ...player })),
  };
}

function buildDefaultRoleReference(): PlannerRoleReference {
  return {
    lanes: DEFAULT_TACTIC.lanes.map((lane) => ({
      laneId: lane.laneId,
      players:
        lane.laneId === "goalkeeper"
          ? [
              {
                playerUid: 42,
                name: "Alex Scout",
                currentScore: 80,
                potentialScore: 90,
              },
            ]
          : [],
    })),
    noEligible: [],
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
  const displayNames = {
    senior: "Senior",
    reserves: "Reserves",
    youth: "Youth",
  } as const;
  return {
    tactic: cloneTactic(DEFAULT_TACTIC),
    teams: ["senior", "reserves", "youth"].map((team, index) => ({
      team: team as PlannerDepth["teams"][number]["team"],
      displayName: displayNames[team as keyof typeof displayNames],
      strings: [{ id: index + 1, stringOrder: 0, assignments: [] }],
    })),
  };
}

function plannerContextKey(saveId: number, contextToken: string) {
  return `${saveId}:${contextToken}`;
}

function validatePlannerTacticContext(args: unknown): {
  saveId: number;
  contextToken: string;
} {
  if (
    typeof args !== "object" ||
    args === null ||
    !("saveId" in args) ||
    !("contextToken" in args) ||
    typeof (args as { saveId: unknown }).saveId !== "number" ||
    typeof (args as { contextToken: unknown }).contextToken !== "string"
  ) {
    throw new Error("Invalid planner tactic request");
  }
  return {
    saveId: (args as { saveId: number }).saveId,
    contextToken: (args as { contextToken: string }).contextToken,
  };
}

function ensurePlannerSaveContext(saveId: number, contextToken: string) {
  const save = resolveListSavesIpcMock().find(
    (candidate) => candidate.id === saveId,
  );
  if (!save) {
    throw `Save ${saveId} not found`;
  }
  if (save.contextToken !== contextToken) {
    throw "Save changed or no longer exists";
  }
}

export function resetPlannerIpcMock() {
  managedClub = { ...DEFAULT_MANAGED_CLUB };
  availableClubs = [];
  managedClubOptionsError = null;
  managedClubSaveCalls = 0;
  onManagedClubSaveCall = undefined;
  managedClubSavePending = false;
  pendingManagedClubSave = null;
  tacticStore.clear();
  tacticStore.set(`1:save-token-1`, cloneTactic(DEFAULT_TACTIC));
  tacticReadCalls = [];
  tacticOptionsReadCalls = [];
  tacticSaveCalls = [];
  depth = buildDefaultDepth();
  depthError = null;
  roleReference = buildDefaultRoleReference();
  roleReferenceError = null;
  roleReferenceCalls = [];
  depthFetchCount = 0;
  tacticSaveError = null;
  slotCandidates = [];
  assignmentError = null;
  addStringError = null;
  addStringPending = false;
  addStringCalls = 0;
  clearAllError = null;
  clearAllPending = false;
  clearAllCalls = 0;
  slotCandidateFetchCount = 0;
  optimizeDepth = null;
  optimizeError = null;
  optimizePending = false;
  optimizeCalls = 0;
  optimizeBases = [];
  teamSaveError = null;
  teamSavePending = false;
  pendingPlannerTeamSave = null;
  teamRemovalImpactOverride = null;
  teamRemovalImpactPending = false;
  pendingTeamRemovalImpact = null;
  teamSaveCalls = [];
}

export function setPlannerAvailableClubs(clubs: string[]) {
  availableClubs = [...clubs];
}

export function setManagedClubOptionsError(message: string | null) {
  managedClubOptionsError = message ? new Error(message) : null;
}

export function setManagedClubIpcMock(status: ManagedClubStatus) {
  managedClub = { ...status };
}

export function setManagedClubSavePending(pending: boolean) {
  managedClubSavePending = pending;
}

export function resolvePendingManagedClubSave() {
  const pending = pendingManagedClubSave;
  if (!pending) {
    throw new Error("No managed-club save is pending");
  }
  pendingManagedClubSave = null;
  pending.resolve(pending.result);
}

export function resolveManagedClubIpcMock(): ManagedClubStatus {
  return { ...managedClub };
}

export function getManagedClubSaveCalls() {
  return managedClubSaveCalls;
}

export function observeManagedClubSaveCall(observer: (() => void) | undefined) {
  onManagedClubSaveCall = observer;
}

export function resolveManagedClubOptionsIpcMock() {
  if (managedClubOptionsError) {
    throw managedClubOptionsError;
  }
  return [...availableClubs];
}

export function resolvePlannerTacticIpcMock(args?: unknown) {
  if (args !== undefined) {
    const { saveId, contextToken } = validatePlannerTacticContext(args);
    tacticReadCalls.push({ saveId, contextToken });
    ensurePlannerSaveContext(saveId, contextToken);
    const key = plannerContextKey(saveId, contextToken);
    const stored = tacticStore.get(key);
    if (stored) {
      return cloneTactic(stored);
    }
    return cloneTactic(DEFAULT_TACTIC);
  }
  return cloneTactic(tacticStore.get(`1:save-token-1`) ?? DEFAULT_TACTIC);
}

export function getPlannerTacticIpcMockCalls() {
  return tacticReadCalls.map((call) => ({ ...call }));
}

export function getPlannerTacticOptionsIpcMockCalls() {
  return tacticOptionsReadCalls.map((call) => ({ ...call }));
}

export function getPlannerTacticSaveIpcMockCalls() {
  return tacticSaveCalls.map((call) => ({ ...call }));
}

export function setPlannerTacticIpcMock(value: PlannerTactic) {
  tacticStore.set(`1:save-token-1`, cloneTactic(value));
}

export function setPlannerTacticForContext(
  context: { saveId: number; contextToken: string },
  value: PlannerTactic,
) {
  tacticStore.set(
    plannerContextKey(context.saveId, context.contextToken),
    cloneTactic(value),
  );
}

export function setPlannerRoleReference(value: PlannerRoleReference) {
  roleReference = cloneRoleReference(value);
}

export function setPlannerRoleReferenceError(message: string | null) {
  roleReferenceError = message;
}

export function getPlannerRoleReferenceCalls() {
  return roleReferenceCalls.map((call) => ({ ...call }));
}

export function resolvePlannerRoleReferenceIpcMock(args: unknown) {
  if (
    typeof args !== "object" ||
    args === null ||
    !("phase" in args) ||
    !("scoreBasis" in args) ||
    (args.phase !== "in_possession" && args.phase !== "out_of_possession") ||
    (args.scoreBasis !== "current" && args.scoreBasis !== "potential")
  ) {
    throw new Error("Invalid planner role reference request");
  }
  roleReferenceCalls.push({
    phase: args.phase,
    scoreBasis: args.scoreBasis,
  });
  if (roleReferenceError) {
    throw roleReferenceError;
  }
  return cloneRoleReference(roleReference);
}

export function resolvePlannerTacticOptionsIpcMock(args?: unknown) {
  if (args !== undefined) {
    const { saveId, contextToken } = validatePlannerTacticContext(args);
    tacticOptionsReadCalls.push({ saveId, contextToken });
    ensurePlannerSaveContext(saveId, contextToken);
  }
  return {
    placements: [...DEFAULT_TACTIC_OPTIONS.placements],
    roles: [...DEFAULT_TACTIC_OPTIONS.roles],
  };
}

export function resolvePlannerDepthIpcMock(): PlannerDepth {
  depthFetchCount += 1;
  if (depthError) throw new Error(depthError);
  return cloneDepth(depth);
}

export function getPlannerDepthIpcMockCalls() {
  return depthFetchCount;
}

export function setPlannerDepthIpcMock(value: PlannerDepth) {
  depth = cloneDepth(value);
}

export function setPlannerDepthError(message: string | null) {
  depthError = message;
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

export function setPlannerClearAllError(message: string | null) {
  clearAllError = message;
}

export function setPlannerClearAllPending(value: boolean) {
  clearAllPending = value;
}

export function getPlannerClearAllIpcMockCalls() {
  return clearAllCalls;
}

export function getPlannerSlotCandidateFetchCount() {
  return slotCandidateFetchCount;
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

export function setPlannerTeamSaveError(message: string | null) {
  teamSaveError = message;
}

export function setPlannerTeamSavePending(value: boolean) {
  teamSavePending = value;
}

export function setPlannerTeamRemovalImpacts(
  impacts: PlannerTeamRemovalImpact[] | null,
) {
  teamRemovalImpactOverride = impacts;
}

export function setPlannerTeamRemovalImpactPending(value: boolean) {
  teamRemovalImpactPending = value;
}

export function resolvePendingPlannerTeamRemovalImpact() {
  if (!pendingTeamRemovalImpact) {
    throw new Error("No planner team removal impact request is pending");
  }
  const resolve = pendingTeamRemovalImpact;
  pendingTeamRemovalImpact = null;
  teamRemovalImpactPending = false;
  resolve();
}

export function getPlannerTeamSaveIpcMockCalls() {
  return teamSaveCalls.map((call) => ({
    teams: call.teams.map((team) => ({ ...team })),
    confirmPopulatedRemoval: call.confirmPopulatedRemoval,
  }));
}

export function getPlannerOptimizeIpcMockCalls() {
  return optimizeCalls;
}

export function getPlannerOptimizeIpcMockBases() {
  return [...optimizeBases];
}

export function resolvePlannerSlotCandidatesIpcMock(args: unknown) {
  slotCandidateFetchCount += 1;
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
    throw new Error("Invalid planner slot");
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
    throw new Error("Invalid planner assignment");
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
    throw new Error("Planner string not found");
  }
  if (target.assignments.some((assignment) => assignment.laneId === laneId)) {
    throw new Error("Planner cell is already occupied");
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
    potentialCombinedScore: null,
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
    throw new Error("Planner string not found");
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
    throw new Error("Invalid planner team");
  }
  const team = depth.teams.find((candidate) => candidate.team === args.team);
  if (!team) {
    throw new Error("Planner team not found");
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
    throw new Error("Invalid planner string");
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
    throw new Error("Planner string not found");
  }
  if (team.strings.length <= 1) {
    throw `The ${team.team} team must keep at least one string`;
  }
  if (plannerString.assignments.length > 0 && !args.confirmPopulated) {
    throw new Error("Removing a populated string requires confirmation");
  }
  team.strings = team.strings
    .filter((candidate) => candidate.id !== args.stringId)
    .map((candidate, index) => ({ ...candidate, stringOrder: index }));
  return cloneDepth(depth);
}

export function resolveClearPlannerDepthIpcMock(args: unknown) {
  clearAllCalls += 1;
  if (
    typeof args !== "object" ||
    args === null ||
    !("confirmed" in args) ||
    typeof args.confirmed !== "boolean"
  ) {
    throw new Error("Invalid planner clear request");
  }
  if (!args.confirmed) {
    throw new Error("Clearing all squads requires confirmation");
  }
  if (clearAllError) {
    throw clearAllError;
  }
  if (clearAllPending) {
    return new Promise<PlannerDepth>(() => {});
  }
  depth.teams = depth.teams.map((team) => ({
    ...team,
    strings: team.strings.map((plannerString) => ({
      ...plannerString,
      assignments: [],
    })),
  }));
  return cloneDepth(depth);
}

export function resolveOptimizePlannerDepthIpcMock(args: unknown) {
  optimizeCalls += 1;
  if (
    typeof args !== "object" ||
    args === null ||
    !("scoreBasis" in args) ||
    (args.scoreBasis !== "current" && args.scoreBasis !== "potential")
  ) {
    throw new Error("Optimizer requires a valid score basis");
  }
  optimizeBases.push(args.scoreBasis);
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

export function resolvePlannerTeamRemovalImpactsIpcMock(args: unknown) {
  if (
    typeof args !== "object" ||
    args === null ||
    !("teams" in args) ||
    !Array.isArray(args.teams)
  ) {
    throw new Error("Invalid planner team settings");
  }
  const impacts = teamRemovalImpactOverride
    ? teamRemovalImpactOverride.map((impact) => ({
        ...impact,
        staffingTargets: [...impact.staffingTargets],
      }))
    : removalImpactsForTeams({ teams: args.teams });
  if (teamRemovalImpactPending) {
    return new Promise<PlannerTeamRemovalImpact[]>((resolve) => {
      pendingTeamRemovalImpact = () => resolve(impacts);
    });
  }
  return impacts;
}

function removalImpactsForTeams(args: { teams: unknown[] }) {
  const included = new Set(
    args.teams
      .filter(
        (team): team is { team: PlannerTeam } =>
          typeof team === "object" &&
          team !== null &&
          "team" in team &&
          typeof team.team === "string" &&
          PLANNER_TEAMS.includes(team.team as PlannerTeam),
      )
      .map((team) => team.team),
  );
  return depth.teams
    .filter((team) => !included.has(team.team))
    .map((team) => ({
      team: team.team,
      displayName: team.displayName,
      assignmentCount: team.strings.reduce(
        (count, plannerString) => count + plannerString.assignments.length,
        0,
      ),
      staffingTargets: [],
    }));
}

export function resolveSavePlannerTeamsIpcMock(args: unknown) {
  if (
    typeof args !== "object" ||
    args === null ||
    !("teams" in args) ||
    !("confirmPopulatedRemoval" in args) ||
    !Array.isArray(args.teams) ||
    typeof args.confirmPopulatedRemoval !== "boolean"
  ) {
    throw new Error("Invalid planner team settings");
  }
  const teams = args.teams as Array<{
    team: unknown;
    displayName: unknown;
  }>;
  if (teams.length < 1 || teams.length > PLANNER_TEAMS.length) {
    throw new Error("Planner configuration must contain one to three teams");
  }
  const inputs = teams.map((team) => {
    if (
      typeof team.team !== "string" ||
      !PLANNER_TEAMS.includes(team.team as PlannerTeam) ||
      typeof team.displayName !== "string"
    ) {
      throw new Error("Invalid planner team settings");
    }
    return {
      team: team.team as PlannerTeam,
      displayName: team.displayName.trim(),
    };
  });
  teamSaveCalls.push({
    teams: inputs,
    confirmPopulatedRemoval: args.confirmPopulatedRemoval,
  });
  if (teamSaveError) {
    throw teamSaveError;
  }
  if (teamSavePending) {
    return new Promise<PlannerDepth>((resolve) => {
      pendingPlannerTeamSave = {
        teams: inputs,
        confirmPopulatedRemoval: args.confirmPopulatedRemoval as boolean,
        resolve,
      };
    });
  }

  return applyPlannerTeamSave(inputs, args.confirmPopulatedRemoval);
}

export function resolvePendingPlannerTeamSaveIpcMock() {
  const pending = pendingPlannerTeamSave;
  if (!pending) {
    return;
  }
  pendingPlannerTeamSave = null;
  pending.resolve(
    applyPlannerTeamSave(pending.teams, pending.confirmPopulatedRemoval),
  );
}

function applyPlannerTeamSave(
  inputs: Array<{ team: PlannerTeam; displayName: string }>,
  confirmPopulatedRemoval: boolean,
) {
  const removedPopulatedTeams = depth.teams.filter(
    (team) =>
      !inputs.some((input) => input.team === team.team) &&
      team.strings.some(
        (plannerString) => plannerString.assignments.length > 0,
      ),
  );
  if (removedPopulatedTeams.length > 0 && !confirmPopulatedRemoval) {
    throw `Removing populated planner teams requires confirmation: ${removedPopulatedTeams
      .map((team) => team.displayName)
      .join(", ")}`;
  }

  let nextStringId =
    Math.max(
      0,
      ...depth.teams.flatMap((candidate) =>
        candidate.strings.map((plannerString) => plannerString.id),
      ),
    ) + 1;
  const nextTeams = PLANNER_TEAMS.filter((team) =>
    inputs.some((input) => input.team === team),
  ).map((team) => {
    const current = depth.teams.find((candidate) => candidate.team === team);
    const input = inputs.find((candidate) => candidate.team === team);
    if (current) {
      return {
        ...current,
        displayName: input?.displayName ?? current.displayName,
      };
    }
    return {
      team,
      displayName: input?.displayName ?? team,
      strings: [{ id: nextStringId++, stringOrder: 0, assignments: [] }],
    };
  });
  depth.teams = nextTeams;
  return cloneDepth(depth);
}

export function setPlannerTacticSaveError(message: string | null) {
  tacticSaveError = message;
}

export function resolveSavePlannerTacticIpcMock(args: unknown) {
  if (tacticSaveError) {
    throw tacticSaveError;
  }
  const record = args as {
    saveId?: unknown;
    contextToken?: unknown;
    tactic?: PlannerTactic;
  };
  const { saveId, contextToken } = validatePlannerTacticContext(args);
  tacticSaveCalls.push({ saveId, contextToken });
  ensurePlannerSaveContext(saveId, contextToken);
  if (!record.tactic) {
    throw new Error("Tactic is required");
  }
  const key = plannerContextKey(saveId, contextToken);
  const next = cloneTactic(record.tactic);
  tacticStore.set(key, next);
  return cloneTactic(next);
}

export function resolveSetManagedClubIpcMock(
  args: unknown,
): ManagedClubStatus | Promise<ManagedClubStatus> {
  managedClubSaveCalls += 1;
  onManagedClubSaveCall?.();
  const clubName = (args as { clubName?: unknown }).clubName;
  if (typeof clubName !== "string" || !clubName.trim()) {
    throw new Error("Managed club must not be empty");
  }
  const normalized = clubName.trim();
  const result: ManagedClubStatus = {
    clubName: normalized,
    status: availableClubs.includes(normalized) ? "available" : "missing",
    unclassifiedPlayerCount: 0,
  };
  if (managedClubSavePending) {
    return new Promise<ManagedClubStatus>((resolve) => {
      pendingManagedClubSave = { result, resolve };
    });
  }
  managedClub = result;
  return resolveManagedClubIpcMock();
}

// Existing route tests use this setup helper to seed the selected club. The
// attached-source input is intentionally ignored because membership is now
// derived from the latest snapshot.
export function resolveSavePlannerClubFamilyIpcMock(args: unknown) {
  const primaryClub = (args as { primaryClub?: unknown }).primaryClub;
  return resolveSetManagedClubIpcMock({ clubName: primaryClub });
}
