import type { StaffAssignmentScope } from "./staff-assignment";

/** The level of an FM team a staff member works with. */
export type StaffFmTeamLevel = "senior" | "reserve" | "youth";

export const STAFF_FM_TEAM_LEVEL_NAMES: Record<StaffFmTeamLevel, string> = {
  senior: "First team",
  reserve: "Reserves",
  youth: "Youth",
};

/**
 * `sack` ends a contract, `change` sacks and hires the person again in another
 * job or team, and `hire` brings someone in through FM's editor.
 */
export type StaffFmChangeKind = "sack" | "change" | "hire";

export type StaffFmChange = {
  uid: number;
  name: string | null;
  kind: StaffFmChangeKind;
  /** The job they hold now, for club staff whose job is known. */
  currentJob: string | null;
  currentTeamName: string | null;
  /** Their club now, for a hire; null when unemployed. */
  currentClub: string | null;
  weeklyWageGbp: number | null;
  /** The slot a hire or change fills. */
  target: {
    scope: StaffAssignmentScope;
    scopeDisplayName: string;
    jobLabel: string;
    level: StaffFmTeamLevel;
  } | null;
};

export type StaffFmChanges = {
  changes: StaffFmChange[];
  /** Why the changes cannot be applied now; null when they can. */
  unavailableReason: string | null;
};

export type StaffFmChangeResult = {
  done: {
    uid: number;
    name: string | null;
    kind: StaffFmChangeKind;
    /** FM's name for the job they hold afterwards; null after a sack. */
    jobLabel: string | null;
    teamLevel: StaffFmTeamLevel | null;
    weeklyWageGbp: number | null;
  }[];
  failed: {
    uid: number;
    name: string | null;
    kind: StaffFmChangeKind;
    reason: string;
  }[];
  /** The error that stopped the changes early; FM may have made some first. */
  error: string | null;
};
