import { describe, expect, it } from "vitest";
import { PROFILE_TABS, parseProfileTab } from "./profile-tab";

describe("parseProfileTab", () => {
  it("keeps the four canonical tabs in display order", () => {
    expect(PROFILE_TABS).toEqual([
      "outfield",
      "goalkeeping",
      "hidden",
      "personality",
    ]);
  });

  it("accepts the four canonical tabs and normalizes legacy visible groups", () => {
    expect(parseProfileTab("outfield")).toBe("outfield");
    expect(parseProfileTab("technical")).toBe("outfield");
    expect(parseProfileTab("mental")).toBe("outfield");
    expect(parseProfileTab("physical")).toBe("outfield");
    expect(parseProfileTab("goalkeeping")).toBe("goalkeeping");
    expect(parseProfileTab("personality")).toBe("personality");
    expect(parseProfileTab("overview")).toBe("outfield");
    expect(parseProfileTab("nope")).toBe("outfield");
    expect(parseProfileTab(undefined)).toBe("outfield");
  });
});
