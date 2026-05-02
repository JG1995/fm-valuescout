import { describe, it, expect } from "vitest";
import { getMetricValue } from "./metric-accessor";

describe("getMetricValue", () => {
	const player = {
		attacking: { goals_per_90: 0.5, xg_per_90: 0.3 },
		chance_creation: { assists_per_90: 0.2, pass_completion_rate: 0.85 },
		defending: { tackles_per_90: 2.1 },
		movement: { dribbles_per_90: 1.5 },
	};

	it("extracts top-level metric", () => {
		expect(getMetricValue(player, "attacking.goals_per_90")).toBe(0.5);
	});

	it("extracts nested metric", () => {
		expect(getMetricValue(player, "chance_creation.pass_completion_rate")).toBe(0.85);
	});

	it("returns null for missing category", () => {
		expect(getMetricValue(player, "goalkeeping.saves_per_90")).toBeNull();
	});

	it("returns null for missing field in category", () => {
		expect(getMetricValue(player, "attacking.shots_per_90")).toBeNull();
	});

	it("returns null for null data", () => {
		expect(getMetricValue(null, "attacking.goals_per_90")).toBeNull();
	});

	it("returns null for malformed key", () => {
		expect(getMetricValue(player, "invalid")).toBeNull();
	});

	it("returns null for deeply nested key", () => {
		expect(getMetricValue(player, "a.b.c")).toBeNull();
	});
});
