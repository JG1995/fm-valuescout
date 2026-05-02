import { describe, it, expect } from "vitest";
import { getSortValue, formatMetricLabel, formatValue } from "./table-helpers";
import type { PlayerScore } from "$lib/scoring/types";

function makeScore(overrides: Partial<PlayerScore> = {}): PlayerScore {
    return {
        playerId: 1,
        fmUid: 12345,
        name: "Test Player",
        club: null,
        positions: "ST",
        role: "ST",
        rawScore: 50,
        valueAdjustedScore: 45,
        age: 25,
        transferValue: 10_000_000,
        metricPercentiles: {
            "attacking.goals_per_90": 75,
            "attacking.assists": 60,
            "defending.tackles": 30,
        },
        ...overrides,
    };
}

describe("getSortValue", () => {
    it("returns rawScore for 'rawScore' key", () => {
        const score = makeScore({ rawScore: 123 });
        expect(getSortValue(score, "rawScore")).toBe(123);
    });

    it("returns valueAdjustedScore for 'valueAdjustedScore' key", () => {
        const score = makeScore({ valueAdjustedScore: 456 });
        expect(getSortValue(score, "valueAdjustedScore")).toBe(456);
    });

    it("returns age for 'age' key", () => {
        const score = makeScore({ age: 28 });
        expect(getSortValue(score, "age")).toBe(28);
    });

    it("returns transferValue for 'transferValue' key", () => {
        const score = makeScore({ transferValue: 25_000_000 });
        expect(getSortValue(score, "transferValue")).toBe(25_000_000);
    });

    it("returns 0 for null age", () => {
        const score = makeScore({ age: null });
        expect(getSortValue(score, "age")).toBe(0);
    });

    it("returns 0 for null transferValue", () => {
        const score = makeScore({ transferValue: null });
        expect(getSortValue(score, "transferValue")).toBe(0);
    });

    it("returns percentile for 'metric.attacking.goals_per_90' key", () => {
        const score = makeScore();
        expect(getSortValue(score, "metric.attacking.goals_per_90")).toBe(75);
    });

    it("returns 0 for unknown metric key", () => {
        const score = makeScore();
        expect(getSortValue(score, "metric.unknown.key")).toBe(0);
    });

    it("returns 0 for unknown sort key", () => {
        const score = makeScore();
        expect(getSortValue(score, "unknownKey")).toBe(0);
    });
});

describe("formatMetricLabel", () => {
    it('converts "attacking.goals_per_90" to "Goals Per 90"', () => {
        expect(formatMetricLabel("attacking.goals_per_90")).toBe("Goals Per 90");
    });

    it('converts "minutes_played" to "Minutes Played"', () => {
        expect(formatMetricLabel("minutes_played")).toBe("Minutes Played");
    });

    it('converts "attacking.crosses" to "Crosses"', () => {
        expect(formatMetricLabel("attacking.crosses")).toBe("Crosses");
    });

    it('converts "a.b_c_d" to "B C D"', () => {
        expect(formatMetricLabel("a.b_c_d")).toBe("B C D");
    });
});

describe("formatValue", () => {
    it('formats 10_000_000 as "10.0M"', () => {
        expect(formatValue(10_000_000)).toBe("10.0M");
    });

    it('formats 15_500_000 as "15.5M"', () => {
        expect(formatValue(15_500_000)).toBe("15.5M");
    });

    it('formats 1_000_000 as "1.0M"', () => {
        expect(formatValue(1_000_000)).toBe("1.0M");
    });

    it('formats 50_000 as "50K"', () => {
        expect(formatValue(50_000)).toBe("50K");
    });

    it('formats 999_999 as "999K"', () => {
        expect(formatValue(999_999)).toBe("999K");
    });

    it('formats null as "—"', () => {
        expect(formatValue(null)).toBe("—");
    });

    it('formats 500 as "500"', () => {
        expect(formatValue(500)).toBe("500");
    });

    it('formats 999 as "999"', () => {
        expect(formatValue(999)).toBe("999");
    });
});
