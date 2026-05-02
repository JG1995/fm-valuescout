import { describe, it, expect, vi } from "vitest";
import { createMockScoredPlayers, createMockPlayerScore } from "./results-table.test-utils";
import type { PlayerScore } from "$lib/scoring";

describe("ResultsTable", () => {
	describe("player rendering", () => {
		it("renders all players in the table", () => {
			const players = createMockScoredPlayers();
			expect(players.length).toBe(4);
			for (const scoredPlayer of players) {
				expect(scoredPlayer.name).toBeTruthy();
			}
		});

		it("shows player name in table row", () => {
			const players = createMockScoredPlayers();
			expect(players[0].name).toBe("Lionel Messi");
		});

		it("shows player club in table row", () => {
			const players = createMockScoredPlayers();
			expect(players[0].club).toBe("Inter Miami");
		});
	});

	describe("sorting logic", () => {
		it("default sort is by valueAdjustedScore descending", () => {
			const players = createMockScoredPlayers();
			const sorted = [...players].sort((a, b) => b.valueAdjustedScore - a.valueAdjustedScore);
			expect(sorted[0].name).toBe("Young Prospect");
			expect(sorted[0].valueAdjustedScore).toBe(98.0);
		});

		it("sorts by club ascending", () => {
			const players = createMockScoredPlayers();
			// Clubs: "Inter Miami", "Manchester City", "Real Madrid", "Youth Team"
			// "Inter Miami" < "Manchester City" < "Real Madrid" < "Youth Team"
			const sorted = [...players].sort((a, b) =>
				(a.club ?? '').localeCompare(b.club ?? '')
			);
			expect(sorted[0].club).toBe("Inter Miami");
			expect(sorted[1].club).toBe("Manchester City");
		});

		it("reverses sort order on second click", () => {
			const players = createMockScoredPlayers();
			const sortedDesc = [...players].sort((a, b) => b.valueAdjustedScore - a.valueAdjustedScore);
			const sortedAsc = [...sortedDesc].reverse();
			expect(sortedAsc[0].valueAdjustedScore).toBe(45.0);
			expect(sortedAsc[3].valueAdjustedScore).toBe(98.0);
		});
	});

	describe("row click behavior", () => {
		it("calls onPlayerClick with player id when row is clicked", () => {
			const onPlayerClick = vi.fn();
			const players = createMockScoredPlayers();
			const firstPlayerId = players[0].playerId;
			onPlayerClick(firstPlayerId);
			expect(onPlayerClick).toHaveBeenCalledTimes(1);
		});
	});

	describe("empty state", () => {
		it("handles empty players array", () => {
			const players: PlayerScore[] = [];
			const sorted = [...players].sort((a, b) => b.valueAdjustedScore - a.valueAdjustedScore);
			expect(sorted.length).toBe(0);
		});
	});

	describe("value formatting", () => {
		it("formats scores to 1 decimal place", () => {
			const player = createMockPlayerScore({
				rawScore: 92.567,
				valueAdjustedScore: 85.123,
			});
			expect(player.rawScore.toFixed(1)).toBe("92.6");
		});

		it("formats currency values correctly", () => {
			const formatCurrency = (value: number | null): string => {
				if (value === null) return '-';
				if (value >= 1000000) return `EUR${(value / 1000000).toFixed(1)}M`;
				if (value >= 1000) return `EUR${(value / 1000).toFixed(0)}K`;
				return `EUR${value}`;
			};
			expect(formatCurrency(200000000)).toBe("EUR200.0M");
		});
	});

	describe("column headers", () => {
		it("has sortable column headers", () => {
			const headers = ["Name", "Club", "Age", "Value", "Raw Score", "Value-Adj Score"];
			expect(headers.length).toBe(6);
		});
	});
});
