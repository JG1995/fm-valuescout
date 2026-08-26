export type StaffAssignmentScope = "senior" | "reserves" | "youth" | "club";

export type StaffAssignmentTeam = Exclude<StaffAssignmentScope, "club">;

export type StaffAssignmentTarget = {
  scope: StaffAssignmentScope;
  jobId: string;
  jobLabel: string;
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
  classification: "current_staff" | "recruitment";
  score: number;
  coachDiscipline: string | null;
};

export type StaffAssignmentVacancy = {
  kind: "vacancy";
  scope: StaffAssignmentScope;
  scopeDisplayName: string;
  jobId: string;
  jobLabel: string;
  slotNumber: number;
  evidence: StaffAssignmentEvidence;
};

export type StaffAssignmentSlot =
  | StaffAssignmentRecommendation
  | StaffAssignmentVacancy;

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
};
