import { invokeCommand } from "@/lib/tauri-client";
import type { StaffAssignmentTargets } from "../types/staff-assignment";

export function fetchStaffAssignmentTargets(expectedSaveContextToken: string) {
  return invokeCommand<StaffAssignmentTargets>("get_staff_assignment_targets", {
    expectedSaveContextToken,
  });
}
