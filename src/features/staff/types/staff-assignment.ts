export type StaffAssignmentScope = "senior" | "reserves" | "youth" | "club";

export type StaffAssignmentTeam = Exclude<StaffAssignmentScope, "club">;

export type StaffAssignmentSection = "coaching" | "recruitment" | "medical";

export type StaffAssignmentTarget = {
  scope: StaffAssignmentScope;
  jobId: string;
  jobLabel: string;
  section: StaffAssignmentSection;
  maxSlotCount: number;
  slotCount: number;
};

export type StaffAssignmentTargetInput = Pick<
  StaffAssignmentTarget,
  "scope" | "jobId" | "slotCount"
>;

export type StaffAssignmentTargetTeam = {
  team: StaffAssignmentTeam;
  displayName: string;
};

export type StaffAssignmentTargets = {
  teams: StaffAssignmentTargetTeam[];
  targets: StaffAssignmentTarget[];
};

export type StaffAssignmentContext = {
  saveId: number;
  saveContextToken: string;
  snapshotId: number;
  snapshotContextToken: string;
};

export type StaffAssignmentEvidence = {
  jobId: string;
  joinedCandidateCount: number;
  eligibleScoreCount: number;
  unavailableScoreCount: number;
};

export type CoachRequirement =
  | "attacking_technical"
  | "attacking_tactical"
  | "defending_technical"
  | "defending_tactical"
  | "possession_technical"
  | "possession_tactical"
  | "fitness"
  | "goalkeeping";

/** What the club has to do to fill a slot with its recommended person. */
export type StaffAssignmentAction =
  | { kind: "keep" }
  /** Same job, another team. */
  | { kind: "move_team"; currentTeam: StaffAssignmentTeam }
  /** `currentTeam` is null for a club-wide job or when the team is unknown. */
  | {
      kind: "change_job";
      currentJob: string;
      currentTeam: StaffAssignmentTeam | null;
    }
  | { kind: "on_staff" }
  /** `club` is null when the person is unemployed. */
  | { kind: "recruit"; club: string | null };

export type StaffAssignmentRecommendation = {
  kind: "recommendation";
  scope: StaffAssignmentScope;
  scopeDisplayName: string;
  jobId: string;
  jobLabel: string;
  slotNumber: number;
  uid: number;
  name: string | null;
  preferredJob: string;
  jobSource: "preferred_job" | "current_job";
  action: StaffAssignmentAction;
  score: number;
  coachRequirement: CoachRequirement | null;
  /** The club staff member leaving the job and team this person takes, when one does. */
  replaces: StaffAssignmentReplacement | null;
};

/** `movesTo` is the slot they move to, or null when they are sacked. */
export type StaffAssignmentReplacement = {
  uid: number;
  name: string | null;
  movesTo: { scope: StaffAssignmentScope; job: string } | null;
};

export type StaffAssignmentVacancy = {
  kind: "vacancy";
  scope: StaffAssignmentScope;
  scopeDisplayName: string;
  jobId: string;
  jobLabel: string;
  slotNumber: number;
  coachRequirement: CoachRequirement | null;
  evidence: StaffAssignmentEvidence;
};

export type StaffAssignmentSlot =
  | StaffAssignmentRecommendation
  | StaffAssignmentVacancy;

export type StaffAssignmentSkip = {
  uid: number;
  name: string | null;
};

/** Club staff who lost every configured slot they were scored for. */
export type StaffAssignmentSurplus = {
  uid: number;
  name: string | null;
  /** The job they hold now, when FM's job type is mapped. */
  currentJob: string | null;
  /** Their team, when the job has one and the team is known. */
  currentTeam: StaffAssignmentTeam | null;
  /** The recommendation that takes their job and team, when one does. */
  replacedBy: StaffAssignmentSkip | null;
};

export type StaffAssignmentOptimizationState =
  | "stale_context"
  | "no_current_snapshot"
  | "no_managed_club"
  | "no_shortlist"
  | "ready";

export type StaffAssignmentOptimization = {
  state: StaffAssignmentOptimizationState;
  saveId: number;
  saveContextToken: string;
  snapshotId: number | null;
  snapshotContextToken: string | null;
  joinedCandidateCount: number;
  configuredSlotCount: number;
  unsupportedPreferredJobCount: number;
  slots: StaffAssignmentSlot[];
  evidence: StaffAssignmentEvidence[];
  skipped: StaffAssignmentSkip[];
  surplus: StaffAssignmentSurplus[];
  /** Each enabled Planner team's display name. */
  teamNames: Partial<Record<StaffAssignmentTeam, string>>;
};
