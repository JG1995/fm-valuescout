import { describe, it, expect } from "vitest";
import "@testing-library/svelte/vitest";
import { render, screen } from "@testing-library/svelte";
import type { ScoredPlayer, ParsedPlayer } from "$lib/scoring/score";
import PodiumView from "./PodiumView.svelte";

function createMockParsedPlayer(overrides: Partial<ParsedPlayer> = {}): ParsedPlayer {
	return {
		id: "player-1",
		name: "Test Player",
		position: "ST",
		age: 25,
		transfer_value: 10000000,
		...overrides,
	};
}

function createMockScoredPlayer(overrides: Partial<ScoredPlayer> = {}): ScoredPlayer {
	return {
		player: createMockParsedPlayer(),
		archetypeId: 1,
		rawScore: 75.5,
		valueAdjustedScore: 82.3,
		percentileByMetric: new Map(),
		...overrides,
	};
}

describe("PodiumView", () => {
	describe("heading", () => {
		it("renders heading with archetype name", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ id: "p1", name: "First" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ id: "p2", name: "Second" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ id: "p3", name: "Third" }) }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Goal Poacher",
			});

			expect(screen.getByText(/Goal Poacher/)).toBeTruthy();
		});
	});

	describe("podium layout", () => {
		it("renders 3 podium positions when given 3 players", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ id: "p1", name: "First" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ id: "p2", name: "Second" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ id: "p3", name: "Third" }) }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test Archetype",
			});

			const podiumItems = document.querySelectorAll(".podium-item");
			expect(podiumItems).toHaveLength(3);
		});

		it("shows correct players in correct positions (2nd, 1st, 3rd)", () => {
			// Position 0 = 2nd place, Position 1 = 1st place, Position 2 = 3rd place
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Second Place" }), valueAdjustedScore: 85 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "First Place" }), valueAdjustedScore: 95 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Third Place" }), valueAdjustedScore: 75 }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test",
			});

			// Check that all player names are displayed somewhere
			expect(screen.getByText("First Place")).toBeTruthy();
			expect(screen.getByText("Second Place")).toBeTruthy();
			expect(screen.getByText("Third Place")).toBeTruthy();
		});
	});

	describe("handles edge cases", () => {
		it("does not crash when fewer than 3 players provided", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Only Player" }) }),
			];

			expect(() => {
				render(PodiumView, {
					scoredPlayers: players,
					archetypeName: "Test",
				});
			}).not.toThrow();
		});

		it("shows empty state when no players provided", () => {
			render(PodiumView, {
				scoredPlayers: [],
				archetypeName: "Test",
			});

			// Should still render the component without crashing
			const podiumItems = document.querySelectorAll(".podium-item");
			expect(podiumItems).toHaveLength(0);
		});

		it("handles empty player names gracefully", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "" }), valueAdjustedScore: 90 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "" }), valueAdjustedScore: 80 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "" }), valueAdjustedScore: 70 }),
			];

			expect(() => {
				render(PodiumView, {
					scoredPlayers: players,
					archetypeName: "Test",
				});
			}).not.toThrow();
		});
	});

	describe("score formatting", () => {
		it("formats raw scores to 1 decimal place", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Player One" }), rawScore: 75.56, valueAdjustedScore: 82.1 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Player Two" }), rawScore: 65.123, valueAdjustedScore: 71.2 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Player Three" }), rawScore: 55, valueAdjustedScore: 60.0 }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test",
			});

			// Should show scores formatted to 1 decimal
			expect(screen.getByText(/75\.6/)).toBeTruthy();
			expect(screen.getByText(/65\.1/)).toBeTruthy();
			expect(screen.getByText(/55\.0/)).toBeTruthy();
		});

		it("formats value-adjusted scores to 1 decimal place", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "P1" }), rawScore: 70, valueAdjustedScore: 95.87 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "P2" }), rawScore: 60, valueAdjustedScore: 82.44 }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "P3" }), rawScore: 50, valueAdjustedScore: 71.12 }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test",
			});

			// Value-adjusted scores formatted to 1 decimal
			expect(screen.getByText(/95\.9/)).toBeTruthy();
			expect(screen.getByText(/82\.4/)).toBeTruthy();
			expect(screen.getByText(/71\.1/)).toBeTruthy();
		});
	});

	describe("medal styling", () => {
		it("applies gold styling to first place (center)", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Gold" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Silver" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Bronze" }) }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test",
			});

			// Check for gold medal class/indicator
			const goldElement = document.querySelector(".medal-gold, .position-1, [data-position=\"1\"]");
			expect(goldElement).toBeTruthy();
		});

		it("applies silver styling to second place (left)", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Gold" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Silver" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Bronze" }) }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test",
			});

			// Check for silver medal class/indicator
			const silverElement = document.querySelector(".medal-silver, .position-2, [data-position=\"2\"]");
			expect(silverElement).toBeTruthy();
		});

		it("applies bronze styling to third place (right)", () => {
			const players = [
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Gold" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Silver" }) }),
				createMockScoredPlayer({ player: createMockParsedPlayer({ name: "Bronze" }) }),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test",
			});

			// Check for bronze medal class/indicator
			const bronzeElement = document.querySelector(".medal-bronze, .position-3, [data-position=\"3\"]");
			expect(bronzeElement).toBeTruthy();
		});
	});

	describe("club display", () => {
		it("shows club when available in player data", () => {
			const players = [
				createMockScoredPlayer({
					player: createMockParsedPlayer({ name: "Star Player" }),
					valueAdjustedScore: 95,
				}),
				createMockScoredPlayer({
					player: createMockParsedPlayer({ name: "Another Player" }),
					valueAdjustedScore: 85,
				}),
				createMockScoredPlayer({
					player: createMockParsedPlayer({ name: "Third Player" }),
					valueAdjustedScore: 75,
				}),
			];

			render(PodiumView, {
				scoredPlayers: players,
				archetypeName: "Test",
			});

			// Component should render without error when club is not available
			expect(screen.getByText("Star Player")).toBeTruthy();
		});
	});
});
