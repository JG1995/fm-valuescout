import { describe, it, expect } from "vitest";
import { computePercentile, buildPercentileCache } from "./percentiles";
import type { ScorablePlayer } from "./types";

describe("computePercentile", () => {
	it("returns 0 for empty values", () => {
		expect(computePercentile(5, [])).toBe(0);
	});

	it("returns 100 for highest value", () => {
		expect(computePercentile(10, [1, 5, 10])).toBe(100);
	});

	it("returns 0 for lowest value", () => {
		expect(computePercentile(1, [1, 5, 10])).toBe(0);
	});

	it("returns 50 for median in 3-element set", () => {
		expect(computePercentile(5, [1, 5, 10])).toBe(50);
	});

	it("handles duplicate values", () => {
		// All same value → percentile should be between 0 and 100
		const result = computePercentile(5, [5, 5, 5]);
		expect(result).toBeGreaterThan(0);
		expect(result).toBeLessThanOrEqual(100);
	});

	it("returns 0 for value not in list", () => {
		expect(computePercentile(0, [1, 5, 10])).toBe(0);
	});
});

describe("buildPercentileCache", () => {
	const players: ScorablePlayer[] = [
		{
			playerId: 1, fmUid: 100, name: "A", club: null,
			positions: "ST", age: 25, transferValueHigh: 1000000,
			data: { attacking: { goals_per_90: 0.3 } },
		},
		{
			playerId: 2, fmUid: 200, name: "B", club: null,
			positions: "ST", age: 27, transferValueHigh: 2000000,
			data: { attacking: { goals_per_90: 0.6 } },
		},
		{
			playerId: 3, fmUid: 300, name: "C", club: null,
			positions: "ST", age: 22, transferValueHigh: null,
			data: null, // Missing data
		},
	];

	it("builds cache for specified metrics", () => {
		const cache = buildPercentileCache(players, ["attacking.goals_per_90"]);
		const values = cache.metricValues.get("attacking.goals_per_90")!;
		expect(values).toHaveLength(2); // Only 2 non-null values
		expect(values).toEqual([0.3, 0.6]);
	});

	it("handles all-null metric", () => {
		const cache = buildPercentileCache(players, ["chance_creation.assists_per_90"]);
		const values = cache.metricValues.get("chance_creation.assists_per_90")!;
		expect(values).toHaveLength(0);
	});
});
