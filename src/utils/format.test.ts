import { describe, expect, it } from "vitest";
import {
  formatAbsoluteUtc,
  formatCount,
  formatDaysRemaining,
  formatMissable,
  formatMoney,
  formatPlayerDob,
  formatPreferredFoot,
  formatRelativeAge,
  formatShortDate,
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

describe("formatPreferredFoot", () => {
  it("title-cases dump values and treats blanks as missing", () => {
    expect(formatPreferredFoot("right")).toBe("Right");
    expect(formatPreferredFoot("LEFT")).toBe("Left");
    expect(formatPreferredFoot("either")).toBe("Either");
    expect(formatPreferredFoot("")).toBe("—");
    expect(formatPreferredFoot(null)).toBe("—");
  });
});

describe("formatMoney", () => {
  it.each([
    [750, "€750"],
    [900_000, "€900k"],
    [12_500_000, "€12.5M"],
    [120_000_000, "€120M"],
  ])("renders %s as %s", (value, expected) => {
    expect(formatMoney(value)).toBe(expected);
  });
});

describe("formatPlayerDob", () => {
  it("formats day-of-year with age", () => {
    // 2001-03-21 is day-of-year 80 in a non-leap year.
    expect(formatPlayerDob(2001, 80, 25)).toBe("21/03/2001 (25)");
  });

  it("omits age when unknown", () => {
    expect(formatPlayerDob(2001, 80, null)).toBe("21/03/2001");
  });
});

describe("formatShortDate", () => {
  it("renders a canonical date with a short month", () => {
    expect(formatShortDate("2027-06-30")).toBe("30 Jun 2027");
  });

  it("returns a value that is not a real calendar date unchanged", () => {
    expect(formatShortDate("2027-02-30")).toBe("2027-02-30");
    expect(formatShortDate("next summer")).toBe("next summer");
  });
});

describe("formatDaysRemaining", () => {
  it.each([
    [-3, "expired"],
    [0, "today"],
    [1, "in 1 day"],
    [60, "in 60 days"],
    [61, "in 2 months"],
    [320, "in 10 months"],
  ])("renders %i days as %s", (days, expected) => {
    expect(formatDaysRemaining(days)).toBe(expected);
  });
});
