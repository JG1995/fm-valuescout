import { describe, expect, it } from "vitest";
import { parseProfileTab } from "./profile-tab";

describe("parseProfileTab", () => {
  it("accepts attribute tabs and defaults unknown values to technical", () => {
    expect(parseProfileTab("mental")).toBe("mental");
    expect(parseProfileTab("goalkeeping")).toBe("goalkeeping");
    expect(parseProfileTab("personality")).toBe("personality");
    expect(parseProfileTab("overview")).toBe("technical");
    expect(parseProfileTab("nope")).toBe("technical");
    expect(parseProfileTab(undefined)).toBe("technical");
  });
});
