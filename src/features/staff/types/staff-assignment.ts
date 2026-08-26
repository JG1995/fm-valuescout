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
