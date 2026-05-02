import { describe, it, expect } from "vitest";
import type { Archetype, MetricWeight } from "$lib/types/archetype";
import {
	computePercentile,
	computePlayerScore,
	computeValueAdjustedScore,
} from "./score";

// Mock ParsedPlayer interface for testing (matches FM parser output)
interface TestPlayer {
	/** Index signature required for ParsedPlayer compatibility */
	[key: string]: unknown;
	id: string;
	name: string;
	position: string;
	age: number;
	"attacking.goals_per_90": number;
	"attacking.shots_per_90": number;
	"attacking.assists_per_90": number;
	"defending.tackles_per_90": number;
	"defending.interceptions_per_90": number;
	"fouls_made_per_90": number;
	"passing.passes_per_90": number;
	transfer_value: number;
}

function createMockPlayer(overrides: Partial<TestPlayer> = {}): TestPlayer {
	return {
		id: "player-1",
		name: "Test Player",
		position: "ST",
		age: 25,
		"attacking.goals_per_90": 0.5,
		"attacking.shots_per_90": 3.0,
		"attacking.assists_per_90": 0.2,
		"defending.tackles_per_90": 0.0,
		"defending.interceptions_per_90": 0.0,
		"fouls_made_per_90": 1.5,
		"passing.passes_per_90": 30.0,
		transfer_value: 10000000,
		...overrides,
	};
}

function createMockArchetype(overrides: Partial<Archetype> = {}): Archetype {
	return {
		id: 1,
		name: "Goal Poacher",
		role: "ST",
		metrics: [
			{ metric_key: " // gitleaks:allowattacking.goals_per_90", weight: 0.6, inverted: false },
			{ metric_key: " // gitleaks:allowattacking.shots_per_90", weight: 0.4, inverted: false },
		],
		is_default: true,
		created_at: "2026-05-02T00:00:00",
		updated_at: "2026-05-02T00:00:00",
		...overrides,
	};
}

describe("computePercentile", () => {
	it("returns correct percentile for values in an even-sized array", () => {
		const values = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
		// Value at index 5: countLess = 4 (10, 20, 30, 40), total = 10
		// Percentile = (4 / 9) * 100 = 44.44...
		expect(computePercentile(50, values)).toBeCloseTo(44.44, 1);
	});

	it("handles unsorted input correctly", () => {
		const unsorted = [30, 10, 50, 20, 40];
		// computePercentile should sort internally
		expect(computePercentile(30, unsorted)).toBe(50);
	});

	it("handles duplicate values", () => {
		const values = [10, 20, 20, 20, 30];
		// Values less than 20: 1 (the 10)
		// Total - 1 = 4
		// Percentile: (1/4)*100 = 25
		expect(computePercentile(20, values)).toBe(25);
	});

	it("returns 0 when value equals minimum", () => {
		const values = [10, 20, 30, 40, 50];
		expect(computePercentile(10, values)).toBe(0);
	});

	it("returns 100 when value equals maximum", () => {
		const values = [10, 20, 30, 40, 50];
		expect(computePercentile(50, values)).toBe(100);
	});

	it("returns 50 as fallback for empty array", () => {
		const values: number[] = [];
		expect(computePercentile(42, values)).toBe(50);
	});

	it("returns 50 as fallback for single value array", () => {
		const values = [42];
		expect(computePercentile(42, values)).toBe(50);
	});

	it("returns 50 as fallback when all values are the same", () => {
		const values = [42, 42, 42, 42, 42];
		expect(computePercentile(42, values)).toBe(50);
	});

	it("correctly calculates percentiles in a typical scoring scenario", () => {
		// Player values from worst to best
		const values = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
		// Value at 0.5 is the 50th percentile
		expect(computePercentile(0.5, values)).toBe(50);
		// Value at 0.0 is the minimum (0th percentile)
		expect(computePercentile(0.0, values)).toBe(0);
		// Value at 1.0 is the maximum (100th percentile)
		expect(computePercentile(1.0, values)).toBe(100);
		// Value at 0.9 is near the top
		expect(computePercentile(0.9, values)).toBe(90);
	});
});

describe("computePlayerScore", () => {
	it("calculates basic weighted average correctly", () => {
		const player = createMockPlayer({
			"attacking.goals_per_90": 0.5,
			"attacking.shots_per_90": 3.0,
		});

		// Percentile data for goals_per_90: player at 50th percentile
		// Percentile data for shots_per_90: player at 50th percentile
		const percentiles = new Map<string, number[]>();
		percentiles.set("attacking.goals_per_90", [0.0, 0.5, 1.0]);
		percentiles.set("attacking.shots_per_90", [0.0, 3.0, 6.0]);

		const archetype = createMockArchetype({
			metrics: [
				{ metric_key: " // gitleaks:allowattacking.goals_per_90", weight: 0.6, inverted: false },
				{ metric_key: " // gitleaks:allowattacking.shots_per_90", weight: 0.4, inverted: false },
			],
		});

		// Score = (50 * 0.6) + (50 * 0.4) = 30 + 20 = 50
		const score = computePlayerScore(player, archetype, percentiles);
		expect(score).toBe(50);
	});

	it("reduces score for inverted metrics when player is better", () => {
		const player = createMockPlayer({
			"fouls_made_per_90": 0.5, // Low fouls (good for inverted metric)
		});

		// Percentile data: player at 10th percentile for fouls (low = 0.5)
		const percentiles = new Map<string, number[]>();
		percentiles.set("fouls_made_per_90", [0.0, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0]);

		const archetype = createMockArchetype({
			metrics: [
				{ metric_key: " // gitleaks:allowfouls_made_per_90", weight: 1.0, inverted: true },
			],
		});

		// Percentile for 0.5 fouls in [0, 0.5, 1, 2, 3, 4, 5] is ~16.67
		// Inverted: 100 - 16.67 = 83.33
		// Score = 83.33 * 1.0 = 83.33
		const score = computePlayerScore(player, archetype, percentiles);
		expect(score).toBeGreaterThan(50);
		expect(score).toBeLessThan(100);
	});

	it("uses 0 percentile for missing metric", () => {
		const player = createMockPlayer({
			"attacking.goals_per_90": 0.5,
			// No attacking.shots_per_90
		});

		const percentiles = new Map<string, number[]>();
		percentiles.set("attacking.goals_per_90", [0.0, 0.5, 1.0]);
		// Note: shots_per_90 is NOT in the percentiles map

		const archetype = createMockArchetype({
			metrics: [
				{ metric_key: " // gitleaks:allowattacking.goals_per_90", weight: 0.6, inverted: false },
				{ metric_key: " // gitleaks:allowattacking.shots_per_90", weight: 0.4, inverted: false },
			],
		});

		// Score = (50 * 0.6) + (0 * 0.4) = 30 + 0 = 30
		const score = computePlayerScore(player, archetype, percentiles);
		expect(score).toBe(30);
	});

	it("handles player value below minimum in percentile range", () => {
		const player = createMockPlayer({
			"attacking.goals_per_90": -1.0, // Below minimum
		});

		const percentiles = new Map<string, number[]>();
		percentiles.set("attacking.goals_per_90", [0.0, 0.5, 1.0]);

		const archetype = createMockArchetype({
			metrics: [
				{ metric_key: " // gitleaks:allowattacking.goals_per_90", weight: 1.0, inverted: false },
			],
		});

		// Score should still compute (value below min returns 0 percentile)
		const score = computePlayerScore(player, archetype, percentiles);
		expect(score).toBe(0);
	});

	it("handles player value above maximum in percentile range", () => {
		const player = createMockPlayer({
			"attacking.goals_per_90": 5.0, // Above maximum
		});

		const percentiles = new Map<string, number[]>();
		percentiles.set("attacking.goals_per_90", [0.0, 0.5, 1.0]);

		const archetype = createMockArchetype({
			metrics: [
				{ metric_key: " // gitleaks:allowattacking.goals_per_90", weight: 1.0, inverted: false },
			],
		});

		// Score should be 100 (value at or above max returns 100 percentile)
		const score = computePlayerScore(player, archetype, percentiles);
		expect(score).toBe(100);
	});
});

describe("computeValueAdjustedScore", () => {
	it("ranks cheaper player higher when scores are equal", () => {
		const rawScore = 50;
		const playerTransferValue = 5000000; // Half the median
		const medianTransferValue = 10000000;

		const adjusted = computeValueAdjustedScore(
			rawScore,
			playerTransferValue,
			medianTransferValue
		);

		// Score = 50 / (5000000 / 10000000) = 50 / 0.5 = 100
		expect(adjusted).toBe(100);
	});

	it("ranks expensive player lower when scores are equal", () => {
		const rawScore = 50;
		const playerTransferValue = 20000000; // Double the median
		const medianTransferValue = 10000000;

		const adjusted = computeValueAdjustedScore(
			rawScore,
			playerTransferValue,
			medianTransferValue
		);

		// Score = 50 / (20000000 / 10000000) = 50 / 2 = 25
		expect(adjusted).toBe(25);
	});

	it("returns raw score when transfer value is zero", () => {
		const rawScore = 75;
		const playerTransferValue = 0;
		const medianTransferValue = 10000000;

		const adjusted = computeValueAdjustedScore(
			rawScore,
			playerTransferValue,
			medianTransferValue
		);

		expect(adjusted).toBe(75);
	});

	it("returns raw score when transfer value is negative", () => {
		const rawScore = 75;
		const playerTransferValue = -5000000;
		const medianTransferValue = 10000000;

		const adjusted = computeValueAdjustedScore(
			rawScore,
			playerTransferValue,
			medianTransferValue
		);

		expect(adjusted).toBe(75);
	});

	it("returns raw score when transfer value equals median", () => {
		const rawScore = 60;
		const playerTransferValue = 10000000;
		const medianTransferValue = 10000000;

		const adjusted = computeValueAdjustedScore(
			rawScore,
			playerTransferValue,
			medianTransferValue
		);

		// Score = 60 / (10000000 / 10000000) = 60 / 1 = 60
		expect(adjusted).toBe(60);
	});

	it("correctly values a cheap high-performer", () => {
		const rawScore = 80;
		const playerTransferValue = 2500000; // Quarter of median
		const medianTransferValue = 10000000;

		const adjusted = computeValueAdjustedScore(
			rawScore,
			playerTransferValue,
			medianTransferValue
		);

		// Score = 80 / (2500000 / 10000000) = 80 / 0.25 = 320
		expect(adjusted).toBe(320);
	});

	it("correctly penalizes an expensive low-performer", () => {
		const rawScore = 20;
		const playerTransferValue = 50000000; // 5x median
		const medianTransferValue = 10000000;

		const adjusted = computeValueAdjustedScore(
			rawScore,
			playerTransferValue,
			medianTransferValue
		);

		// Score = 20 / (50000000 / 10000000) = 20 / 5 = 4
		expect(adjusted).toBe(4);
	});
});
