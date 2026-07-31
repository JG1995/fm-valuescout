import { describe, expect, it } from "vitest";
import { parseProfileTab } from "./profile-tab";

describe("parseProfileTab", () => {
  it("accepts known tabs and defaults unknown values to overview", () => {
    expect(parseProfileTab("attributes")).toBe("attributes");
    expect(parseProfileTab("roles")).toBe("roles");
    expect(parseProfileTab("overview")).toBe("overview");
    expect(parseProfileTab("nope")).toBe("overview");
    expect(parseProfileTab(undefined)).toBe("overview");
  });
});
