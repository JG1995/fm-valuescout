import { invokeCommand } from "@/lib/tauri-client";
import type {
  StaffAssignmentTargetInput,
  StaffAssignmentTargets,
} from "../types/staff-assignment";

export function saveStaffAssignmentTargets(
  expectedSaveContextToken: string,
  targets: StaffAssignmentTargetInput[],
) {
  return invokeCommand<StaffAssignmentTargets>(
    "save_staff_assignment_targets",
    {
      expectedSaveContextToken,
      targets,
    },
  );
}
