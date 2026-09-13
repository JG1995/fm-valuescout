import { describe, expect, it } from "vitest";
import type { StaffAssignmentContext } from "../types/staff-assignment";
import { staffKeys } from "./staff-keys";
import { staffSearchQueryOptions } from "./staff-query-options";

const saveOne: StaffAssignmentContext = {
  saveId: 1,
  saveContextToken: "one",
  snapshotId: 2,
  snapshotContextToken: "snap-one",
};

describe("staffKeys.list", () => {
  it("keeps mounted save/snapshot context in the list cache identity", () => {
    const saveTwo: StaffAssignmentContext = {
      ...saveOne,
      saveContextToken: "two",
    };

    expect(
      staffKeys.list(
        "search",
        0,
        1,
        "ca",
        "desc",
        [],
        "and",
        [],
        undefined,
        false,
        true,
        saveOne,
      ),
    ).not.toEqual(
      staffKeys.list(
        "search",
        0,
        1,
        "ca",
        "desc",
        [],
        "and",
        [],
        undefined,
        false,
        true,
        saveTwo,
      ),
    );
    expect(
      staffKeys.list("search", 0, 1, "ca", "desc", [], "and", []),
    ).not.toEqual(
      staffKeys.list(
        "search",
        0,
        1,
        "ca",
        "desc",
        [],
        "and",
        [],
        undefined,
        false,
        true,
        saveOne,
      ),
    );
  });

  it("keeps the exact options key without changing IPC arguments", () => {
    const options = staffSearchQueryOptions(
      0,
      1,
      "ca",
      "desc",
      [],
      "and",
      [],
      true,
      undefined,
      false,
      saveOne,
    );

    expect(options.queryKey).toEqual(
      staffKeys.list(
        "search",
        0,
        1,
        "ca",
        "desc",
        [],
        "and",
        [],
        undefined,
        false,
        true,
        saveOne,
      ),
    );
  });
});
