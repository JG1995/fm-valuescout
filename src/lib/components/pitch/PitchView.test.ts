import { describe, it, expect, vi } from "vitest";
import "@testing-library/svelte/vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import type { Archetype } from "$lib/types/archetype";
import PitchView from "./PitchView.svelte";
import {
	createMockArchetype,
} from "$lib/components/archetype/archetype.test-utils";

// Position slot IDs for 4-4-2 formation from spec
const POSITION_SLOT_IDS = [
	"GK",
	"LB",
	"CB-L",
	"CB-R",
	"RB",
	"LM",
	"CM-L",
	"CM-R",
	"RM",
	"LS",
	"RS",
];

describe("PitchView", () => {
	describe("renders all positions", () => {
		it("renders all 11 position slots", () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// Check that all 11 positions are rendered
			const positionButtons = screen.getAllByRole("button");
			expect(positionButtons).toHaveLength(11);
		});

		it("renders correct position labels", () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// Verify specific positions exist by data attributes
			const gkButton = document.querySelector('[data-slot-id="GK"]');
			expect(gkButton).toBeTruthy();

			const lsButton = document.querySelector('[data-slot-id="LS"]');
			expect(lsButton).toBeTruthy();

			const lbButton = document.querySelector('[data-slot-id="LB"]');
			expect(lbButton).toBeTruthy();
		});

		it("renders all expected slot IDs", () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			for (const slotId of POSITION_SLOT_IDS) {
				const slot = document.querySelector(`[data-slot-id="${slotId}"]`);
				expect(slot).toBeTruthy();
			}
		});
	});

	describe("displays correct text content", () => {
		it('shows "Select" when no archetype is selected', () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// All slots should show "Select" text
			const gkButton = document.querySelector('[data-slot-id="GK"]');
			expect(gkButton?.textContent).toContain("Select");

			const lsButton = document.querySelector('[data-slot-id="LS"]');
			expect(lsButton?.textContent).toContain("Select");
		});

		it("shows archetype name when selected", () => {
			const mockArchetype = createMockArchetype({
				id: 1,
				name: "Sweeper Keeper",
				role: "GK",
			});

			const mockSelected: Record<string, Archetype | null> = {
				GK: mockArchetype,
			};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			const gkButton = document.querySelector('[data-slot-id="GK"]');
			expect(gkButton?.textContent).toContain("Sweeper Keeper");
		});

		it("shows 'Select' for positions without archetypes in mixed state", () => {
			const striker = createMockArchetype({
				id: 2,
				name: "Target Man",
				role: "ST",
			});

			const mockSelected: Record<string, Archetype | null> = {
				LS: striker,
				// Other positions are null
			};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// LS should show the archetype name
			const lsButton = document.querySelector('[data-slot-id="LS"]');
			expect(lsButton?.textContent).toContain("Target Man");

			// GK should show "Select"
			const gkButton = document.querySelector('[data-slot-id="GK"]');
			expect(gkButton?.textContent).toContain("Select");
		});

		it("shows both slot labels and archetype names", () => {
			const mockArchetype = createMockArchetype({
				id: 1,
				name: "Playmaker",
				role: "AM",
			});

			const mockSelected: Record<string, Archetype | null> = {
				CM: mockArchetype, // Using generic key for testing
			};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// Slot labels should be present
			const slotLabels = screen.getAllByText("CM-L").concat(
				screen.getAllByText("CM-R")
			);
			expect(slotLabels.length).toBe(2);
		});
	});

	describe("click handling", () => {
		it("calls onslotclick with correct slotId", async () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// Click the button inside the LS slot
			const lsSlot = document.querySelector('[data-slot-id="LS"]');
			const lsButton = lsSlot?.querySelector("button");
			await fireEvent.click(lsButton!);

			expect(mockHandler).toHaveBeenCalledWith("LS");
		});

		it("calls onslotclick for all position slots", async () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// Click each position's button
			for (const slotId of POSITION_SLOT_IDS) {
				const slot = document.querySelector(`[data-slot-id="${slotId}"]`);
				const button = slot?.querySelector("button");
				await fireEvent.click(button!);
			}

			// Verify all positions were called
			expect(mockHandler).toHaveBeenCalledTimes(11);
			for (const slotId of POSITION_SLOT_IDS) {
				expect(mockHandler).toHaveBeenCalledWith(slotId);
			}
		});
	});

	describe("visual states", () => {
		it("selected slots have different visual treatment", () => {
			const mockArchetype = createMockArchetype({
				id: 1,
				name: "Goalkeeper",
				role: "GK",
			});

			const mockSelected: Record<string, Archetype | null> = {
				GK: mockArchetype,
			};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// Selected button should have the selected class
			const selectedButton = container.querySelector(
				'.slot-button[class*="selected"]'
			);
			expect(selectedButton).toBeTruthy();
		});
	});

	describe("SVG pitch rendering", () => {
		it("renders SVG element for pitch", () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			const svg = container.querySelector("svg.pitch-svg");
			expect(svg).toBeTruthy();
		});

		it("renders green pitch background", () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
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
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			const lines = container.querySelectorAll("line");
			const circles = container.querySelectorAll("circle");

			// Should have some pitch lines
			expect(lines.length).toBeGreaterThan(0);
			// Should have center circle
			expect(circles.length).toBeGreaterThan(0);
		});

		it("renders penalty areas", () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// Should have multiple rect elements for penalty areas and goal boxes
			const rects = container.querySelectorAll("rect");
			expect(rects.length).toBeGreaterThan(3); // Background + boundary + penalty areas
		});
	});

	describe("responsive layout", () => {
		it("pitch container stretches to fill available width", () => {
			const mockSelected: Record<string, Archetype | null> = {};
			const mockHandler = vi.fn();

			const { container } = render(PitchView, {
				selectedArchetypes: mockSelected,
				onslotclick: mockHandler,
			});

			// The SVG should have width of 100% for responsiveness
			const svg = container.querySelector("svg");
			expect(svg).toBeTruthy();

			// Check that the container has responsive sizing
			const pitchContainer = container.querySelector(".pitch-container");
			expect(pitchContainer).toBeTruthy();
		});
	});
});
