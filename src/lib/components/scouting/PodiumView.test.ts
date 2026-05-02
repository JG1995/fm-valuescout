import { describe, it, expect } from "vitest";
import "@testing-library/svelte/vitest";
import { render, screen } from "@testing-library/svelte";
import type { PlayerScore } from "$lib/scoring";
import PodiumView from "./PodiumView.svelte";

function createMockPlayerScore(overrides: Partial<PlayerScore> = {}): PlayerScore {
	return {
		playerId: 1,
		fmUid: 12345,
		name: "Test Player",
		club: "Test FC",
		positions: "ST",
		age: 25,
		transferValue: 10_000_000,
		role: "ST",
		rawScore: 75.5,
		valueAdjustedScore: 82.3,
		metricPercentiles: {},
		...overrides,
	};
}

describe("PodiumView", () => {
	describe("heading", () => {
		it("renders heading with 'Top 3'", () => {
			const players = [
				createMockPlayerScore({ name: "First" }),
				createMockPlayerScore({ name: "Second" }),
				createMockPlayerScore({ name: "Third" }),
			];

			render(PodiumView, {
				scores: players,
			});

			expect(screen.getByText("Top 3")).toBeTruthy();
		});

		it("renders heading with 'Top 3' regardless of input", () => {
			const players = [
				createMockPlayerScore({ name: "Player A" }),
			];

			render(PodiumView, {
				scores: players,
			});

			expect(screen.getByText("Top 3")).toBeTruthy();
		});

		it("renders heading even with empty array", () => {
			render(PodiumView, {
				scores: [],
			});

			expect(screen.getByText("Top 3")).toBeTruthy();
		});
	});

	describe("podium layout", () => {
		it("renders 3 podium positions when given 3 players", () => {
			const players = [
				createMockPlayerScore({ name: "First" }),
				createMockPlayerScore({ name: "Second" }),
				createMockPlayerScore({ name: "Third" }),
			];

			render(PodiumView, {
				scores: players,
			});

			const podiumPositions = document.querySelectorAll(".podium-position");
			expect(podiumPositions).toHaveLength(3);
		});

		it("shows correct players sorted by value-adjusted score", () => {
			// Players provided in random order, should be sorted by valueAdjustedScore
			const players = [
				createMockPlayerScore({ name: "Third Place", valueAdjustedScore: 75 }),
				createMockPlayerScore({ name: "First Place", valueAdjustedScore: 95 }),
				createMockPlayerScore({ name: "Second Place", valueAdjustedScore: 85 }),
			];

			render(PodiumView, {
				scores: players,
			});

			// All player names should be displayed
			expect(screen.getByText("First Place")).toBeTruthy();
			expect(screen.getByText("Second Place")).toBeTruthy();
			expect(screen.getByText("Third Place")).toBeTruthy();
		});
	});

	describe("handles edge cases", () => {
		it("does not crash when fewer than 3 players provided", () => {
			const players = [
				createMockPlayerScore({ name: "Only Player" }),
			];

			expect(() => {
				render(PodiumView, {
					scores: players,
				});
			}).not.toThrow();
		});

		it("renders 3 podium positions even with fewer players", () => {
			const players = [
				createMockPlayerScore({ name: "Only Player", valueAdjustedScore: 90 }),
			];

			render(PodiumView, {
				scores: players,
			});

			// All 3 positions should exist (empty ones show placeholder)
			const podiumPositions = document.querySelectorAll(".podium-position");
			expect(podiumPositions).toHaveLength(3);
		});

		it("shows placeholder for empty positions", () => {
			const players = [
				createMockPlayerScore({ name: "Top Player", valueAdjustedScore: 95 }),
			];

			render(PodiumView, {
				scores: players,
			});

			// Empty positions should show placeholder text
			const placeholder = document.querySelector(".placeholder-text");
			expect(placeholder).toBeTruthy();
		});

		it("handles empty player names gracefully", () => {
			const players = [
				createMockPlayerScore({ name: "", valueAdjustedScore: 90 }),
				createMockPlayerScore({ name: "", valueAdjustedScore: 80 }),
				createMockPlayerScore({ name: "", valueAdjustedScore: 70 }),
			];

			expect(() => {
				render(PodiumView, {
					scores: players,
				});
			}).not.toThrow();
		});
	});

	describe("score formatting", () => {
		it("formats raw scores to 1 decimal place", () => {
			const players = [
				createMockPlayerScore({ name: "Player One", rawScore: 75.56, valueAdjustedScore: 82.1 }),
				createMockPlayerScore({ name: "Player Two", rawScore: 65.123, valueAdjustedScore: 71.2 }),
				createMockPlayerScore({ name: "Player Three", rawScore: 55, valueAdjustedScore: 60.0 }),
			];

			render(PodiumView, {
				scores: players,
			});

			// Should show scores formatted to 1 decimal
			expect(screen.getByText(/75\.6/)).toBeTruthy();
			expect(screen.getByText(/65\.1/)).toBeTruthy();
			expect(screen.getByText(/55\.0/)).toBeTruthy();
		});

		it("formats value-adjusted scores to 1 decimal place", () => {
			const players = [
				createMockPlayerScore({ name: "P1", rawScore: 70, valueAdjustedScore: 95.87 }),
				createMockPlayerScore({ name: "P2", rawScore: 60, valueAdjustedScore: 82.44 }),
				createMockPlayerScore({ name: "P3", rawScore: 50, valueAdjustedScore: 71.12 }),
			];

			render(PodiumView, {
				scores: players,
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
				createMockPlayerScore({ name: "Gold" }),
				createMockPlayerScore({ name: "Silver" }),
				createMockPlayerScore({ name: "Bronze" }),
			];

			render(PodiumView, {
				scores: players,
			});

			// Check for gold medal class on position badge
			const goldElement = document.querySelector(".medal-gold");
			expect(goldElement).toBeTruthy();
		});

		it("applies silver styling to second place (left)", () => {
			const players = [
				createMockPlayerScore({ name: "Gold" }),
				createMockPlayerScore({ name: "Silver" }),
				createMockPlayerScore({ name: "Bronze" }),
			];

			render(PodiumView, {
				scores: players,
			});

			// Check for silver medal class on position badge
			const silverElement = document.querySelector(".medal-silver");
			expect(silverElement).toBeTruthy();
		});

		it("applies bronze styling to third place (right)", () => {
			const players = [
				createMockPlayerScore({ name: "Gold" }),
				createMockPlayerScore({ name: "Silver" }),
				createMockPlayerScore({ name: "Bronze" }),
			];

			render(PodiumView, {
				scores: players,
			});

			// Check for bronze medal class on position badge
			const bronzeElement = document.querySelector(".medal-bronze");
			expect(bronzeElement).toBeTruthy();
		});
	});

	describe("club display", () => {
		it("shows club when available in player data", () => {
			const players = [
				createMockPlayerScore({
					name: "Star Player",
					club: "Star FC",
					valueAdjustedScore: 95,
				}),
				createMockPlayerScore({
					name: "Another Player",
					club: "Another FC",
					valueAdjustedScore: 85,
				}),
				createMockPlayerScore({
					name: "Third Player",
					club: "Third FC",
					valueAdjustedScore: 75,
				}),
			];

			render(PodiumView, {
				scores: players,
			});

			expect(screen.getByText("Star FC")).toBeTruthy();
			expect(screen.getByText("Another FC")).toBeTruthy();
			expect(screen.getByText("Third FC")).toBeTruthy();
		});

		it("shows '—' when club is null", () => {
			const players = [
				createMockPlayerScore({
					name: "Free Agent",
					club: null,
					valueAdjustedScore: 90,
				}),
			];

			render(PodiumView, {
				scores: players,
			});

			// The first position should show "—" for null club
			expect(screen.getByText("—")).toBeTruthy();
		});
	});

	describe("platform heights", () => {
		it("applies tallest height to first place platform", () => {
			const players = [
				createMockPlayerScore({ name: "First" }),
				createMockPlayerScore({ name: "Second" }),
				createMockPlayerScore({ name: "Third" }),
			];

			render(PodiumView, {
				scores: players,
			});

			const tallestPlatform = document.querySelector(".podium-platform.tallest");
			expect(tallestPlatform).toBeTruthy();
		});

		it("applies medium height to second place platform", () => {
			const players = [
				createMockPlayerScore({ name: "First" }),
				createMockPlayerScore({ name: "Second" }),
				createMockPlayerScore({ name: "Third" }),
			];

			render(PodiumView, {
				scores: players,
			});

			const mediumPlatform = document.querySelector(".podium-platform.medium");
			expect(mediumPlatform).toBeTruthy();
		});

		it("applies short height to third place platform", () => {
			const players = [
				createMockPlayerScore({ name: "First" }),
				createMockPlayerScore({ name: "Second" }),
				createMockPlayerScore({ name: "Third" }),
			];

			render(PodiumView, {
				scores: players,
			});

			const shortPlatform = document.querySelector(".podium-platform.short");
			expect(shortPlatform).toBeTruthy();
		});

		it("renders platform even for empty positions", () => {
			render(PodiumView, {
				scores: [],
			});

			// All 3 positions should have platforms
			const platforms = document.querySelectorAll(".podium-platform");
			expect(platforms).toHaveLength(3);
		});
	});
});
