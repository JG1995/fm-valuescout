import { describe, it, expect } from "vitest";
import type { PlayerScore, ScorablePlayer, PercentileCache } from "./types";

describe("PlayerScore interface shape", () => {
	it("constructs a valid PlayerScore with all required fields", () => {
		const score: PlayerScore = {
			playerId: 1,
			fmUid: 12345,
			name: "Test Player",
			club: "Test FC",
			positions: "ST",
			age: 25,
			transferValue: 10_000_000,
			role: "ST",
			rawScore: 85.5,
			valueAdjustedScore: 92.0,
			metricPercentiles: { "attacking.goals_per_90": 75 },
		};
		expect(score.playerId).toBe(1);
		expect(score.rawScore).toBe(85.5);
		expect(score.valueAdjustedScore).toBe(92.0);
		expect(score.metricPercentiles["attacking.goals_per_90"]).toBe(75);
	});

	it("allows null club, age, and transferValue", () => {
		const score: PlayerScore = {
			playerId: 2,
			fmUid: 99,
			name: "Minimal",
			club: null,
			positions: "GK",
			age: null,
			transferValue: null,
			role: "GK",
			rawScore: 0,
			valueAdjustedScore: 0,
			metricPercentiles: {},
		};
		expect(score.club).toBeNull();
		expect(score.age).toBeNull();
		expect(score.transferValue).toBeNull();
	});
});

describe("ScorablePlayer interface shape", () => {
	it("constructs a valid ScorablePlayer", () => {
		const player: ScorablePlayer = {
			playerId: 1,
			fmUid: 123,
			name: "Player",
			club: null,
			positions: "CB",
			age: 28,
			transferValueHigh: 5_000_000,
			data: { defending: { tackles_per_90: 3.2 } },
		};
		expect(player.data).not.toBeNull();
	});

	it("allows null data", () => {
		const player: ScorablePlayer = {
			playerId: 2,
			fmUid: 456,
			name: "No Data",
			club: null,
			positions: "ST",
			age: null,
			transferValueHigh: null,
			data: null,
		};
		expect(player.data).toBeNull();
	});
});

describe("PercentileCache construction", () => {
	it("constructs with Maps", () => {
		const cache: PercentileCache = {
			metricValues: new Map([["attacking.goals_per_90", [0.1, 0.5, 0.8]]]),
			metricCounts: new Map([["attacking.goals_per_90", 3]]),
		};
		expect(cache.metricValues.get("attacking.goals_per_90")).toHaveLength(3);
		expect(cache.metricCounts.get("attacking.goals_per_90")).toBe(3);
	});
});
