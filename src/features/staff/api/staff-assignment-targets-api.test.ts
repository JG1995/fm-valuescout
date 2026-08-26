import { describe, expect, it, vi } from "vitest";
import { invokeCommand } from "@/lib/tauri-client";
import { fetchStaffAssignmentTargets } from "./fetch-staff-assignment-targets";
import { saveStaffAssignmentTargets } from "./save-staff-assignment-targets";
import { staffAssignmentTargetsQueryOptions } from "./staff-assignment-targets-query-options";
import { staffKeys } from "./staff-keys";

vi.mock("@/lib/tauri-client", () => ({ invokeCommand: vi.fn() }));

const context = {
  saveId: 1,
  saveContextToken: "save-token-a",
  snapshotId: 1,
  snapshotContextToken: "snapshot-token-a",
};

const targets = [{ scope: "senior" as const, jobId: "coaches", slotCount: 50 }];

describe("staff assignment targets API", () => {
  it("uses typed target commands and token-separated keys", async () => {
    const invoke = vi.mocked(invokeCommand);
    invoke.mockResolvedValue(undefined);

    await fetchStaffAssignmentTargets(context.saveContextToken);
    await saveStaffAssignmentTargets(context.saveContextToken, targets);

    expect(invoke).toHaveBeenNthCalledWith(1, "get_staff_assignment_targets", {
      expectedSaveContextToken: "save-token-a",
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "save_staff_assignment_targets", {
      expectedSaveContextToken: "save-token-a",
      targets,
    });
    expect(staffKeys.assignmentTargets("context-a")).not.toEqual(
      staffKeys.assignmentTargets("context-b"),
    );
    expect(
      staffAssignmentTargetsQueryOptions(context, "context-a").queryKey,
    ).toEqual(staffKeys.assignmentTargets("context-a"));
  });
});
