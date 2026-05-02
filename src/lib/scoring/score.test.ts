import { describe, it, expect } from "vitest";
import { scorePlayer, computeMedianTransferValue, scoreAllPlayers } from "./score";
import type { ScorablePlayer } from "./types";
import type { Archetype } from "$lib/types/archetype";

describe("computeMedianTransferValue", () => {
	it("computes median of non-null values", () => {
		const players: ScorablePlayer[] = [
			{ playerId: 1, fmUid: 1, name: "A", club: null, positions: "ST", age: null, transferValueHigh: 100, data: {} },
			{ playerId: 2, fmUid: 2, name: "B", club: null, positions: "ST", age: null, transferValueHigh: 200, data: {} },
			{ playerId: 3, fmUid: 3, name: "C", club: null, positions: "ST", age: null, transferValueHigh: 300, data: {} },
		];
		expect(computeMedianTransferValue(players)).toBe(200);
	});

	it("returns 1 when all values are null", () => {
		const players: ScorablePlayer[] = [
			{ playerId: 1, fmUid: 1, name: "A", club: null, positions: "ST", age: null, transferValueHigh: null, data: {} },
		];
		expect(computeMedianTransferValue(players)).toBe(1);
	});
});

describe("scorePlayer", () => {
	const archetype: Archetype = {
		id: 1,
		name: "Test Striker",
		role: "ST",
		metrics: [
			{ metric_key: "attacking.goals_per_90", weight: 0.6, inverted: false },
			{ metric_key: "chance_creation.assists_per_90", weight: 0.4, inverted: false }, // gitleaks:allow
		],
		is_default: true,
		created_at: "",
		updated_at: "",
	};

	const allValues = {
		"attacking.goals_per_90": [0.1, 0.2, 0.3, 0.5, 0.8],
		"chance_creation.assists_per_90": [0.1, 0.2, 0.3, 0.4, 0.5],
	};

	it("scores a player with perfect metrics at 100", () => {
		const player: ScorablePlayer = {
			playerId: 1, fmUid: 1, name: "Best", club: "Club A",
			positions: "ST", age: 25, transferValueHigh: 1000000,
			data: { attacking: { goals_per_90: 0.8 }, chance_creation: { assists_per_90: 0.5 } },
		};
		const result = scorePlayer(player, archetype, allValues, 1000000);
		expect(result.rawScore).toBe(100);
		expect(result.valueAdjustedScore).toBe(100);
	});

	it("scores a player with worst metrics at 0", () => {
		const player: ScorablePlayer = {
			playerId: 2, fmUid: 2, name: "Worst", club: null,
			positions: "ST", age: 20, transferValueHigh: 500000,
			data: { attacking: { goals_per_90: 0.1 }, chance_creation: { assists_per_90: 0.1 } },
		};
		const result = scorePlayer(player, archetype, allValues, 500000);
		expect(result.rawScore).toBe(0);
	});

	it("handles missing metrics with 0 percentile", () => {
		const player: ScorablePlayer = {
			playerId: 3, fmUid: 3, name: "No Data", club: null,
			positions: "ST", age: 30, transferValueHigh: null,
			data: {},
		};
		const result = scorePlayer(player, archetype, allValues, 1000000);
		expect(result.rawScore).toBe(0);
	});

	it("handles inverted metrics", () => {
		const invertedArch: Archetype = {
			...archetype,
			metrics: [
				{ metric_key: "discipline.fouls_made_per_90", weight: 1.0, inverted: true },
			],
		};
		const invValues = { "discipline.fouls_made_per_90": [1.0, 2.0, 3.0, 4.0, 5.0] };
		const player: ScorablePlayer = {
			playerId: 4, fmUid: 4, name: "Clean", club: null,
			positions: "ST", age: 25, transferValueHigh: 1000000,
			data: { discipline: { fouls_made_per_90: 1.0 } },
		};
		const result = scorePlayer(player, invertedArch, invValues, 1000000);
		// Lowest fouls (1.0) → percentile 0 → inverted → 100
		expect(result.rawScore).toBe(100);
	});

	it("computes value-adjusted score", () => {
		const player: ScorablePlayer = {
			playerId: 5, fmUid: 5, name: "Value", club: null,
			positions: "ST", age: 25, transferValueHigh: 500000,
			data: { attacking: { goals_per_90: 0.8 }, chance_creation: { assists_per_90: 0.5 } },
		};
		const result = scorePlayer(player, archetype, allValues, 1000000);
		// rawScore = 100, value = 500000, median = 1000000
		// valueAdjusted = 100 / (500000 / 1000000) = 100 / 0.5 = 200
		expect(result.rawScore).toBe(100);
		expect(result.valueAdjustedScore).toBe(200);
	});
});

describe("scoreAllPlayers", () => {
	it("scores all players against an archetype", () => {
		const archetype: Archetype = {
			id: 1,
			name: "Test",
			role: "ST",
			metrics: [
				{ metric_key: "attacking.goals_per_90", weight: 1.0, inverted: false },
			],
			is_default: true,
			created_at: "",
			updated_at: "",
		};

		const players: ScorablePlayer[] = [
			{
				playerId: 1, fmUid: 1, name: "Player1", club: null,
				positions: "ST", age: 25, transferValueHigh: 1000000,
				data: { attacking: { goals_per_90: 0.5 } },
			},
			{
				playerId: 2, fmUid: 2, name: "Player2", club: null,
				positions: "ST", age: 22, transferValueHigh: 500000,
				data: { attacking: { goals_per_90: 0.8 } },
			},
		];

		const results = scoreAllPlayers(players, archetype);
		expect(results).toHaveLength(2);
		expect(results[0].name).toBe("Player1");
		expect(results[1].name).toBe("Player2");
	});
});
