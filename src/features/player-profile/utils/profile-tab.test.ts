import { describe, expect, it } from "vitest";
import {
  defaultProfileTab,
  GOALKEEPER_PROFILE_TABS,
  PROFILE_TABS,
  parseProfileTab,
  profileTabsForPlayer,
} from "./profile-tab";

describe("parseProfileTab", () => {
  it("keeps the four canonical tabs in display order", () => {
    expect(PROFILE_TABS).toEqual([
      "outfield",
      "goalkeeping",
      "hidden",
      "personality",
    ]);
  });

  it("accepts canonical tabs and normalizes legacy visible groups", () => {
    expect(parseProfileTab("outfield")).toBe("outfield");
    expect(parseProfileTab("technical")).toBe("outfield");
    expect(parseProfileTab("mental")).toBe("outfield");
    expect(parseProfileTab("physical")).toBe("outfield");
    expect(parseProfileTab("goalkeeping")).toBe("goalkeeping");
    expect(parseProfileTab("personality")).toBe("personality");
    expect(parseProfileTab("overview")).toBeUndefined();
    expect(parseProfileTab("nope")).toBeUndefined();
    expect(parseProfileTab(undefined)).toBeUndefined();
  });

  it("puts Goalkeeping first only for goalkeeper profiles", () => {
    expect(GOALKEEPER_PROFILE_TABS).toEqual([
      "goalkeeping",
      "outfield",
      "hidden",
      "personality",
    ]);
    expect(profileTabsForPlayer(false)).toBe(PROFILE_TABS);
    expect(profileTabsForPlayer(true)).toBe(GOALKEEPER_PROFILE_TABS);
    expect(defaultProfileTab(false)).toBe("outfield");
    expect(defaultProfileTab(true)).toBe("goalkeeping");
  });
});
