import { describe, it, expect } from "vitest";

describe("Scoring barrel export", () => {
	it("exports getMetricValue from metric-accessor", async () => {
		const mod = await import("./index");
		expect(typeof mod.getMetricValue).toBe("function");
	});

	it("exports computePercentile and buildPercentileCache from percentiles", async () => {
		const mod = await import("./index");
		expect(typeof mod.computePercentile).toBe("function");
		expect(typeof mod.buildPercentileCache).toBe("function");
	});

	it("exports scoring functions from score", async () => {
		const mod = await import("./index");
		expect(typeof mod.scorePlayer).toBe("function");
		expect(typeof mod.scoreAllPlayers).toBe("function");
		expect(typeof mod.computeMedianTransferValue).toBe("function");
	});
});
