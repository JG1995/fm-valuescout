import { invokeCommand } from "@/lib/tauri-client";

export function setStaffAssignmentSkip(
  expectedSaveContextToken: string,
  uid: number,
  skipped: boolean,
) {
  return invokeCommand<null>("set_staff_assignment_skip", {
    expectedSaveContextToken,
    uid,
    skipped,
  });
}
