import { invokeCommand } from "@/lib/tauri-client";
import type { StaffAssignmentOptimization } from "../types/staff-assignment";

export function optimizeStaffAssignments(
  expectedSaveContextToken: string,
  expectedSnapshotContextToken: string,
) {
  return invokeCommand<StaffAssignmentOptimization>(
    "optimize_staff_assignments",
    {
      expectedSaveContextToken,
      expectedSnapshotContextToken,
    },
  );
}
