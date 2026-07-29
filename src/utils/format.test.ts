import { describe, expect, it } from "vitest";
import {
  formatAbsoluteUtc,
  formatCount,
  formatMissable,
  formatRelativeAge,
} from "@/utils/format";

const now = Date.parse("2026-07-29T20:00:00.000Z");

describe("formatRelativeAge", () => {
  it.each([
    ["2026-07-29T19:59:30.000Z", "just now"],
    ["2026-07-29T19:56:00.000Z", "4 min ago"],
    ["2026-07-29T19:00:00.000Z", "1 hour ago"],
    ["2026-07-29T14:00:00.000Z", "6 hours ago"],
    ["2026-07-28T19:00:00.000Z", "yesterday"],
    ["2026-07-26T19:00:00.000Z", "3 days ago"],
  ])("renders %s as %s", (isoUtc, expected) => {
    expect(formatRelativeAge(isoUtc, now)).toBe(expected);
  });

  it("does not claim an age it cannot compute", () => {
    expect(formatRelativeAge("not a date", now)).toBe("unknown");
  });
});

describe("formatAbsoluteUtc", () => {
  it("renders a minute-precision UTC stamp", () => {
    expect(formatAbsoluteUtc("2026-07-29T20:14:37.000Z")).toBe(
      "2026-07-29 20:14 UTC",
    );
  });

  it("passes an unparseable value through untouched", () => {
    expect(formatAbsoluteUtc("whenever")).toBe("whenever");
  });
});

describe("formatCount and formatMissable", () => {
  it("separates thousands", () => {
    expect(formatCount(1247)).toBe("1,247");
  });

  it("distinguishes absent data from zero", () => {
    expect(formatMissable(null)).toBe("—");
    expect(formatMissable("")).toBe("—");
    expect(formatMissable(0)).toBe(0);
  });
});
