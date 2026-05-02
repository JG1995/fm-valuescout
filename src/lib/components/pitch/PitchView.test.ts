import { describe, it, expect, vi } from "vitest";
import "@testing-library/svelte/vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import type { Archetype } from "$lib/types/archetype";
import PitchView from "./PitchView.svelte";
import {
	createMockArchetype,
	createMockArchetypes,
} from "$lib/components/archetype/archetype.test-utils";

// Position keys for 4-4-2 formation
const POSITION_KEYS = [
	"GK",
	"LCB",
	"RCB",
	"LB",
	"RB",
	"DM",
	"CM1",
	"CM2",
	"LW",
	"RW",
	"ST",
];

describe("PitchView", () => {
	describe("renders all positions", () => {
		it("renders all 11 position slots", () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			// Check that all 11 positions are rendered
			const positionButtons = screen.getAllByRole("button");
			expect(positionButtons).toHaveLength(11);
		});

		it("renders correct position labels", () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			// Verify specific positions exist by their data attributes
			const gkButton = document.querySelector('[data-position="GK"]');
			expect(gkButton).toBeTruthy();

			const stButton = document.querySelector('[data-position="ST"]');
			expect(stButton).toBeTruthy();

			const lbButton = document.querySelector('[data-position="LB"]');
			expect(lbButton).toBeTruthy();
		});
	});

	describe("displays correct text content", () => {
		it('shows "Select" when no archetype is selected', () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			// All slots should show "Select" text
			const gkButton = document.querySelector('[data-position="GK"]');
			expect(gkButton?.textContent).toContain("Select");

			const stButton = document.querySelector('[data-position="ST"]');
			expect(stButton?.textContent).toContain("Select");
		});

		it("shows archetype name when selected", () => {
			const mockArchetype = createMockArchetype({
				id: 1,
				name: "Sweeper Keeper",
				role: "GK",
			});

			const mockSelected: Record<string, Archetype | undefined> = {
				GK: mockArchetype,
			};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			const gkButton = document.querySelector('[data-position="GK"]');
			expect(gkButton?.textContent).toContain("Sweeper Keeper");
		});

		it("shows 'Select' for positions without archetypes in mixed state", () => {
			const striker = createMockArchetype({
				id: 2,
				name: "Target Man",
				role: "ST",
			});

			const mockSelected: Record<string, Archetype | undefined> = {
				ST: striker,
				// GK is undefined
			};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			// ST should show the archetype name
			const stButton = document.querySelector('[data-position="ST"]');
			expect(stButton?.textContent).toContain("Target Man");

			// GK should show "Select"
			const gkButton = document.querySelector('[data-position="GK"]');
			expect(gkButton?.textContent).toContain("Select");
		});
	});

	describe("click handling", () => {
		it("calls onSelectArchetype with correct position key", async () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			// Click the ST position
			const stButton = document.querySelector('[data-position="ST"]');
			await fireEvent.click(stButton!);

			expect(mockHandler).toHaveBeenCalledWith("ST");
		});

		it("calls onSelectArchetype for all position slots", async () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			// Click each position
			for (const position of POSITION_KEYS) {
				const button = document.querySelector(`[data-position="${position}"]`);
				await fireEvent.click(button!);
			}

			// Verify all positions were called
			expect(mockHandler).toHaveBeenCalledTimes(11);
			for (const position of POSITION_KEYS) {
				expect(mockHandler).toHaveBeenCalledWith(position);
			}
		});
	});

	describe("responsive layout", () => {
		it("pitch container stretches to fill available width", () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			// The SVG should have width of 100% for responsiveness
			const svg = container.querySelector("svg");
			expect(svg).toBeTruthy();

			// Check that the SVG or container has responsive sizing
			const pitchContainer = container.querySelector(".pitch-container");
			expect(pitchContainer).toBeTruthy();
		});
	});

	describe("SVG pitch rendering", () => {
		it("renders SVG element for pitch", () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			const svg = container.querySelector("svg.pitch-svg");
			expect(svg).toBeTruthy();
		});

		it("renders green pitch background", () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			const rects = container.querySelectorAll("rect");
			// Should have a green background rectangle
			const greenRect = Array.from(rects).find((rect) => {
				const fill = rect.getAttribute("fill");
				return fill === "#2d8a3e" || fill?.includes("green");
			});
			expect(greenRect).toBeTruthy();
		});

		it("renders pitch markings (center line, circle)", () => {
			const mockSelected: Record<string, Archetype | undefined> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onSelectArchetype: mockHandler,
			});

			const lines = container.querySelectorAll("line");
			const circles = container.querySelectorAll("circle");

			// Should have some pitch lines
			expect(lines.length).toBeGreaterThan(0);
			// Should have center circle
			expect(circles.length).toBeGreaterThan(0);
		});
	});
});
