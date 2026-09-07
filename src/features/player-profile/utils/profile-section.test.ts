import { describe, expect, it } from "vitest";
import {
  isStandardProfileSection,
  PROFILE_SECTIONS,
  parseProfileSection,
  resolveProfileSection,
} from "./profile-section";

describe("parseProfileSection", () => {
  it("keeps the four canonical sections in display order", () => {
    expect(PROFILE_SECTIONS).toEqual([
      "overview",
      "attributes",
      "role-fit",
      "moneyball",
    ]);
  });

  it("accepts canonical sections and rejects unknown values", () => {
    expect(parseProfileSection("overview")).toBe("overview");
    expect(parseProfileSection("attributes")).toBe("attributes");
    expect(parseProfileSection("role-fit")).toBe("role-fit");
    expect(parseProfileSection("moneyball")).toBe("moneyball");
    expect(parseProfileSection("general")).toBeUndefined();
    expect(parseProfileSection("not-a-section")).toBeUndefined();
    expect(parseProfileSection(undefined)).toBeUndefined();
  });
});

describe("resolveProfileSection", () => {
  it("lets a valid canonical section win over legacy view and tab state", () => {
    expect(
      resolveProfileSection({
        section: "role-fit",
        view: "moneyball",
        tab: "hidden",
        defaultAnalysisView: "moneyball",
      }),
    ).toBe("role-fit");
    expect(
      resolveProfileSection({
        section: "overview",
        view: "general",
        tab: "hidden",
        defaultAnalysisView: "moneyball",
      }),
    ).toBe("overview");
  });

  it("sends an explicit Moneyball view to Moneyball", () => {
    expect(
      resolveProfileSection({
        section: undefined,
        view: "moneyball",
        tab: "hidden",
        defaultAnalysisView: "general",
      }),
    ).toBe("moneyball");
  });

  it("sends General with an explicit valid attribute tab to Attributes", () => {
    expect(
      resolveProfileSection({
        section: undefined,
        view: "general",
        tab: "hidden",
        defaultAnalysisView: "moneyball",
      }),
    ).toBe("attributes");
  });

  it("sends an ordinary General view to Overview", () => {
    expect(
      resolveProfileSection({
        section: undefined,
        view: "general",
        tab: undefined,
        defaultAnalysisView: "moneyball",
      }),
    ).toBe("overview");
  });

  it("falls through an invalid view to the saved default under both defaults", () => {
    expect(
      resolveProfileSection({
        section: undefined,
        view: undefined,
        tab: "hidden",
        defaultAnalysisView: "general",
      }),
    ).toBe("overview");
    expect(
      resolveProfileSection({
        section: undefined,
        view: undefined,
        tab: "hidden",
        defaultAnalysisView: "moneyball",
      }),
    ).toBe("moneyball");
  });

  it("uses the saved default when neither section nor view is present", () => {
    expect(
      resolveProfileSection({
        section: undefined,
        view: undefined,
        tab: undefined,
        defaultAnalysisView: "general",
      }),
    ).toBe("overview");
    expect(
      resolveProfileSection({
        section: undefined,
        view: undefined,
        tab: undefined,
        defaultAnalysisView: "moneyball",
      }),
    ).toBe("moneyball");
  });

  it("marks only Moneyball as outside the standard section family", () => {
    expect(isStandardProfileSection("overview")).toBe(true);
    expect(isStandardProfileSection("attributes")).toBe(true);
    expect(isStandardProfileSection("role-fit")).toBe(true);
    expect(isStandardProfileSection("moneyball")).toBe(false);
  });
});
