import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import type { Archetype, MetricWeight } from "$lib/types/archetype";
import type { PlayerScore, ScorablePlayer } from "$lib/scoring/types";
import type { PlayerSeasonData } from "./scouting-store.svelte";

// Import raw imports (used only inside vi.mock() factory — not tracked by vitest)
import { invoke as invokeRaw } from "@tauri-apps/api/core";
import { scoreAllPlayers as scoreAllPlayersRaw } from "$lib/scoring";

// Mock the Tauri invoke function — factory returns the same vi.fn() reference
// so vi.mocked(invokeRaw) in tests tracks the same mock instance
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

// Mock the scoring module — same pattern
vi.mock("$lib/scoring", () => ({
	scoreAllPlayers: vi.fn(),
}));

// Use vi.mocked() for vitest-tracked references in tests
const invoke = vi.mocked(invokeRaw);
const scoreAllPlayers = vi.mocked(scoreAllPlayersRaw);

// Clear mock call history before each test
beforeEach(() => {
	vi.clearAllMocks();
});

// Import store once; use store.reset() for isolation
const { getScoutingStore } = await import("./scouting-store.svelte");

// ─── Test Data Helpers ──────────────────────────────────────────────────────

/** Create a test archetype with metrics array */
function makeArchetype(
	id: number,
	name: string,
	role: string,
	metrics: MetricWeight[],
	is_default = false
): Archetype {
	return {
		id,
		name,
		role,
		metrics,
		is_default,
		created_at: "2026-05-02T00:00:00",
		updated_at: "2026-05-02T00:00:00",
	};
}

/** Create PlayerSeasonData (as returned from Rust) */
function makePlayerData(overrides: Partial<PlayerSeasonData> = {}): PlayerSeasonData {
	return {
		id: 1,
		player_id: 100,
		season_id: 1,
		fm_uid: 12345,
		player_name: "Test Player",
		club: "Test FC",
		age: 25,
		nationality: "England",
		position: "ST",
		minutes: 1800,
		transfer_value_high: 5000000,
		data: { "attacking.goals": 10 },
		...overrides,
	};
}

/** Create ScorablePlayer (internal format) */
function makeScorable(overrides: Partial<ScorablePlayer> = {}): ScorablePlayer {
	return {
		playerId: 100,
		fmUid: 12345,
		name: "Test Player",
		club: "Test FC",
		positions: "ST",
		age: 25,
		transferValueHigh: 5000000,
		data: { "attacking.goals": 10 },
		...overrides,
	};
}

/** Create PlayerScore (returned from scoreAllPlayers) */
function makePlayerScore(overrides: Partial<PlayerScore> = {}): PlayerScore {
	return {
		playerId: 100,
		fmUid: 12345,
		name: "Test Player",
		club: "Test FC",
		positions: "ST",
		age: 25,
		transferValue: 5000000,
		role: "ST",
		rawScore: 75,
		valueAdjustedScore: 75,
		metricPercentiles: {},
		...overrides,
	};
}

// ─── Test Suites ────────────────────────────────────────────────────────────

describe("Scouting Store — toScorable", () => {
	afterEach(() => {
		getScoutingStore().reset();
	});

	it("correctly maps PlayerSeasonData fields to ScorablePlayer", async () => {
		invoke.mockResolvedValue([
			makePlayerData({
				id: 42,
				player_id: 999, // should be ignored per spec
				fm_uid: 99999,
				player_name: "John Doe",
				club: "AC Milan",
				age: 28,
				transfer_value_high: 15000000,
				data: { attacking: { goals: 15 } },
			}),
		]);

		const store = getScoutingStore();

		await store.loadPlayers(1);

		expect(store.players.length).toBe(1);
		const player = store.players[0];
		expect(player.playerId).toBe(42);
		expect(player.fmUid).toBe(99999);
		expect(player.name).toBe("John Doe");
		expect(player.club).toBe("AC Milan");
		expect(player.age).toBe(28);
		expect(player.transferValueHigh).toBe(15000000);
		expect(player.data).toEqual({ attacking: { goals: 15 } });
	});
});

describe("Scouting Store — selectArchetype", () => {
	afterEach(() => {
		getScoutingStore().reset();
	});

	it("with null archetype clears scores", async () => {
		invoke.mockResolvedValue([]);
		scoreAllPlayers.mockReturnValue([]);

		const store = getScoutingStore();

		const archetype = makeArchetype(1, "ST Basic", "ST", [
			{ metric_key: "attacking.goals_per_90", weight: 1.0, inverted: false },
		]);
		store.selectArchetype(archetype);
		expect(store.scores.length).toBeGreaterThanOrEqual(0);

		store.selectArchetype(null);
		expect(store.scores.length).toBe(0);
	});

	it("with archetype and empty players produces empty scores", async () => {
		invoke.mockResolvedValue([]);
		scoreAllPlayers.mockReturnValue([]);

		const store = getScoutingStore();

		const archetype = makeArchetype(1, "ST Basic", "ST", [
			{ metric_key: "attacking.goals_per_90", weight: 1.0, inverted: false },
		]);
		store.selectArchetype(archetype);

		expect(store.scores).toEqual([]);
	});

	it("with archetype and players produces scores", async () => {
		invoke.mockResolvedValue([]);
		const mockScores = [
			makePlayerScore({ playerId: 1, rawScore: 90 }),
			makePlayerScore({ playerId: 2, rawScore: 85 }),
		];
		scoreAllPlayers.mockReturnValue(mockScores);

		const store = getScoutingStore();

		// Load players first
		invoke.mockResolvedValue([
			makePlayerData({ player_id: 1 }),
			makePlayerData({ player_id: 2 }),
		]);
		await store.loadPlayers(1);

		const archetype = makeArchetype(1, "ST Basic", "ST", [
			{ metric_key: "attacking.goals_per_90", weight: 1.0, inverted: false },
		]);
		store.selectArchetype(archetype);

		expect(store.scores.length).toBe(2);
		expect(store.scores[0].rawScore).toBe(90);
	});
});

describe("Scouting Store — loadPlayers", () => {
	afterEach(() => {
		getScoutingStore().reset();
	});

	it("invokes get_players_for_season with correct seasonId", async () => {
		invoke.mockResolvedValue([]);

		const store = getScoutingStore();

		await store.loadPlayers(42);

		expect(invoke).toHaveBeenCalledWith("get_players_for_season", {
			seasonId: 42,
		});
	});

	it("converts response to ScorablePlayer array", async () => {
		invoke.mockResolvedValue([
			makePlayerData({ id: 1, player_id: 10 }),
			makePlayerData({ id: 2, player_id: 20 }),
		]);

		const store = getScoutingStore();

		await store.loadPlayers(1);

		expect(store.players.length).toBe(2);
		expect(store.players[0].playerId).toBe(1);
		expect(store.players[1].playerId).toBe(2);
	});

	it("re-scores if archetype is already selected", async () => {
		// Set up mock BEFORE getting store (module-level state is shared)
		scoreAllPlayers.mockReturnValue([]);

		const store = getScoutingStore();

		// First select archetype
		const archetype = makeArchetype(1, "ST Basic", "ST", [
			{ metric_key: "attacking.goals_per_90", weight: 1.0, inverted: false },
		]);
		store.selectArchetype(archetype);

		// Then load players (should trigger re-scoring)
		invoke.mockResolvedValue([
			makePlayerData({ player_id: 1 }),
			makePlayerData({ player_id: 2 }),
		]);
		await store.loadPlayers(1);

		// scoreAllPlayers called once: selectArchetype skipped (no players), loadPlayers re-scored
		expect(scoreAllPlayers).toHaveBeenCalledTimes(1);
	});
	it("sets error and clears loading when invoke throws", async () => {
		invoke.mockRejectedValue(new Error("Database unavailable"));

		const store = getScoutingStore();

		try {
			await store.loadPlayers(1);
		} catch {
			// Expected
		}
		expect(store.error).toBe("Database unavailable");
		expect(store.loading).toBe(false);
	});

	it("sets error message from thrown Error", async () => {
		invoke.mockRejectedValue(new Error("Season not found"));

		const store = getScoutingStore();

		try {
			await store.loadPlayers(99);
		} catch {
			// Expected
		}
		expect(store.error).toBe("Season not found");
		expect(store.loading).toBe(false);
	});
});
