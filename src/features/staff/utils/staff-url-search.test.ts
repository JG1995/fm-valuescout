import { describe, expect, it } from "vitest";
import {
  parseShortlistOnly,
  parseStaffSort,
  parseStaffSortDir,
  parseStaffView,
  staffFiltersForUrl,
} from "./staff-url-search";

describe("staff URL state", () => {
  it("defaults invalid workspace and sort state without accepting arbitrary fields", () => {
    expect(parseStaffView("my-staff")).toBe("my-staff");
    expect(parseStaffView("profiles")).toBe("search");
    expect(parseStaffSort("role.coach_fitness")).toBe("role.coach_fitness");
    expect(parseStaffSort("role.not_real")).toBe("ca");
    expect(parseStaffSortDir("asc")).toBe("asc");
    expect(parseStaffSortDir("sideways")).toBe("desc");
  });

  it("parses the shortlist toggle with strict invalid-to-off", () => {
    expect(parseShortlistOnly(true)).toBe(true);
    expect(parseShortlistOnly("true")).toBe(true);
    expect(parseShortlistOnly(false)).toBe(false);
    expect(parseShortlistOnly(undefined)).toBe(false);
    expect(parseShortlistOnly("yes")).toBe(false);
    expect(parseShortlistOnly("false")).toBe(false);
    expect(parseShortlistOnly(1)).toBe(false);
  });

  it("serializes only the bounded, complete filter shape", () => {
    expect(
      staffFiltersForUrl([
        {
          id: "r1",
          field: "role.scout",
          op: "gt",
          value: { type: "integer", value: 70 },
        },
      ]),
    ).toEqual([{ id: "r1", field: "role.scout", op: "gt", value: 70 }]);
  });
});
