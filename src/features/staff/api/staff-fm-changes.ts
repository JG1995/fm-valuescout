import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { StaffAssignmentContext } from "../types/staff-assignment";
import type {
  StaffFmChangeResult,
  StaffFmChanges,
} from "../types/staff-fm-changes";
import { staffKeys } from "./staff-keys";

/**
 * The sacks, job changes, and hires that put the optimizer's result into FM.
 * FM and the bridge change outside the app, so every read is fresh.
 */
export function staffFmChangesQueryOptions(
  context: StaffAssignmentContext,
  contextKey: string,
) {
  return queryOptions({
    queryKey: staffKeys.fmChanges(contextKey),
    queryFn: () =>
      invokeCommand<StaffFmChanges>("get_staff_fm_changes", {
        expectedSaveContextToken: context.saveContextToken,
        expectedSnapshotContextToken: context.snapshotContextToken,
      }),
    staleTime: 0,
  });
}

/** Makes the chosen changes in FM; Rust re-plans and ignores any other UID. */
export function applyStaffFmChanges(
  context: StaffAssignmentContext,
  staffUids: number[],
) {
  return invokeCommand<StaffFmChangeResult>("apply_staff_fm_changes", {
    expectedSaveContextToken: context.saveContextToken,
    expectedSnapshotContextToken: context.snapshotContextToken,
    staffUids,
  });
}
