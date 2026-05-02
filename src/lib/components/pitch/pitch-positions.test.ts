import { describe, it, expect } from "vitest";
import {
	PITCH_POSITIONS,
	type PitchPosition,
} from "./pitch-positions";

describe("PITCH_POSITIONS", () => {
	it("exports an array of positions", () => {
		expect(Array.isArray(PITCH_POSITIONS)).toBe(true);
	});

	it("contains exactly 11 positions for a 4-4-2 formation", () => {
		expect(PITCH_POSITIONS).toHaveLength(11);
	});

	it("has unique slotId for each position", () => {
		const slotIds = PITCH_POSITIONS.map((p) => p.slotId);
		const uniqueSlotIds = new Set(slotIds);
		expect(uniqueSlotIds.size).toBe(11);
	});

	it("contains a goalkeeper position", () => {
		const gk = PITCH_POSITIONS.find((p) => p.slotId === "GK");
		expect(gk).toBeDefined();
		expect(gk?.role).toBe("GK");
	});

	it("contains 4 defender positions (CB-L, CB-R, LB, RB)", () => {
		const defenders = PITCH_POSITIONS.filter((p) => p.role === "D");
		expect(defenders).toHaveLength(4);
		const slotIds = defenders.map((p) => p.slotId).sort();
		expect(slotIds).toEqual(["CB-L", "CB-R", "LB", "RB"]);
	});

	it("contains 2 midfield positions (CM-L, CM-R)", () => {
		const midfielders = PITCH_POSITIONS.filter((p) => p.role === "M");
		expect(midfielders).toHaveLength(2);
		const slotIds = midfielders.map((p) => p.slotId).sort();
		expect(slotIds).toEqual(["CM-L", "CM-R"]);
	});

	it("contains 2 attacking midfield/wing positions (LM, RM)", () => {
		const wingers = PITCH_POSITIONS.filter((p) => p.role === "AM");
		expect(wingers).toHaveLength(2);
		const slotIds = wingers.map((p) => p.slotId).sort();
		expect(slotIds).toEqual(["LM", "RM"]);
	});

	it("contains 2 striker positions (LS, RS)", () => {
		const strikers = PITCH_POSITIONS.filter((p) => p.role === "ST");
		expect(strikers).toHaveLength(2);
		const slotIds = strikers.map((p) => p.slotId).sort();
		expect(slotIds).toEqual(["LS", "RS"]);
	});

	it("each position has x/y coordinates within 0-100 range", () => {
		for (const pos of PITCH_POSITIONS) {
			expect(pos.x).toBeGreaterThanOrEqual(0);
			expect(pos.x).toBeLessThanOrEqual(100);
			expect(pos.y).toBeGreaterThanOrEqual(0);
			expect(pos.y).toBeLessThanOrEqual(100);
		}
	});

	it("positions have correct label values", () => {
		for (const pos of PITCH_POSITIONS) {
			expect(typeof pos.label).toBe("string");
			expect(pos.label.length).toBeGreaterThan(0);
		}
	});

	it("GK is positioned at bottom center (y around 82)", () => {
		const gk = PITCH_POSITIONS.find((p) => p.slotId === "GK");
		expect(gk?.x).toBeCloseTo(50, 0);
		expect(gk?.y).toBeGreaterThan(75);
	});

	it("strikers are positioned at top of pitch (low y values)", () => {
		const strikers = PITCH_POSITIONS.filter((p) => p.role === "ST");
		for (const striker of strikers) {
			expect(striker.y).toBeLessThan(20);
		}
	});

	it("midfielders are in the middle band (y around 32)", () => {
		const midfielders = PITCH_POSITIONS.filter(
			(p) => p.role === "M" || p.role === "AM"
		);
		for (const mid of midfielders) {
			expect(mid.y).toBeGreaterThan(25);
			expect(mid.y).toBeLessThan(40);
		}
	});

	it("defenders are near the bottom half (y around 58)", () => {
		const defenders = PITCH_POSITIONS.filter((p) => p.role === "D");
		for (const def of defenders) {
			expect(def.y).toBeGreaterThan(50);
			expect(def.y).toBeLessThan(65);
		}
	});

	it("wing positions (LB, RB, LM, RM) are at the flanks (low/high x)", () => {
		const wings = PITCH_POSITIONS.filter(
			(p) => p.slotId === "LB" || p.slotId === "RB" || p.slotId === "LM" || p.slotId === "RM"
		);
		// Flanks are either low (left) or high (right)
		for (const wing of wings) {
			const isFlank = wing.x < 25 || wing.x > 75;
			expect(isFlank).toBe(true);
		}
	});

	it("center positions (GK, CM-L, CM-R) are near x=50", () => {
		const centers = PITCH_POSITIONS.filter(
			(p) => p.slotId === "GK" || p.slotId === "CM-L" || p.slotId === "CM-R"
		);
		for (const center of centers) {
			expect(center.x).toBeGreaterThan(35);
			expect(center.x).toBeLessThan(65);
		}
	});
});

describe("PitchPosition interface", () => {
	it("has all required fields for each position", () => {
		const requiredFields: (keyof PitchPosition)[] = [
			"slotId",
			"label",
			"role",
			"x",
			"y",
		];
		for (const pos of PITCH_POSITIONS) {
			for (const field of requiredFields) {
				expect(pos).toHaveProperty(field);
			}
		}
	});

	it("slotId values match expected set", () => {
		const expectedSlotIds = [
			"LS",
			"RS",
			"LM",
			"CM-L",
			"CM-R",
			"RM",
			"LB",
			"CB-L",
			"CB-R",
			"RB",
			"GK",
		];
		const actualSlotIds = PITCH_POSITIONS.map((p) => p.slotId).sort();
		expect(actualSlotIds).toEqual(expectedSlotIds.sort());
	});
});
