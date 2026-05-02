import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Archetype, MetricWeight } from "$lib/types/archetype";

// Mock the API module
vi.mock("$lib/api/archetypes", () => ({
	createArchetype: vi.fn(),
	listArchetypesByRole: vi.fn(),
	listAllArchetypes: vi.fn(),
	getArchetype: vi.fn(),
	updateArchetype: vi.fn(),
	deleteArchetype: vi.fn(),
}));

// We need to import the store after mocking
import * as api from "$lib/api/archetypes";
import { getArchetypeStore } from "$lib/stores/archetype-store.svelte";

function createMockArchetype(overrides: Partial<Archetype> = {}): Archetype {
	return {
		id: 1,
		name: "Test Archetype",
		role: "ST",
		metrics: [
			{ metric_key: " // gitleaks:allowattacking.goals_per_90", weight: 0.6, inverted: false },
			{ metric_key: " // gitleaks:allowattacking.shots_per_90", weight: 0.4, inverted: false },
		],
		is_default: false,
		created_at: "2026-05-02T00:00:00",
		updated_at: "2026-05-02T00:00:00",
		...overrides,
	};
}

describe("ArchetypeStore", () => {
	let store: ReturnType<typeof getArchetypeStore>;

	beforeEach(() => {
		vi.clearAllMocks();
		store = getArchetypeStore();
		store.reset();
	});

	describe("loadAll", () => {
		it("loads all archetypes from the API", async () => {
			const mockArchetypes = [
				createMockArchetype({ id: 1, name: "ST Arch 1", role: "ST" }),
				createMockArchetype({ id: 2, name: "ST Arch 2", role: "ST" }),
			];
			vi.mocked(api.listAllArchetypes).mockResolvedValue(mockArchetypes);

			await store.loadAll();

			expect(api.listAllArchetypes).toHaveBeenCalledOnce();
			expect(store.archetypes).toEqual(mockArchetypes);
		});

		it("handles API errors gracefully", async () => {
			vi.mocked(api.listAllArchetypes).mockRejectedValue(new Error("API Error"));

			await expect(store.loadAll()).rejects.toThrow("API Error");
			expect(store.error).toBe("API Error");
		});
	});

	describe("loadByRole", () => {
		it("loads archetypes filtered by role", async () => {
			const mockArchetypes = [
				createMockArchetype({ id: 1, name: "GK Basic", role: "GK" }),
			];
			vi.mocked(api.listArchetypesByRole).mockResolvedValue(mockArchetypes);

			await store.loadByRole("GK");

			expect(api.listArchetypesByRole).toHaveBeenCalledWith("GK");
			expect(store.getArchetypesForRole("GK")).toEqual(mockArchetypes);
		});
	});

	describe("create", () => {
		it("creates a new archetype and refreshes the list", async () => {
			const newArchetype = createMockArchetype({ id: 3, name: "New Arch" });
			vi.mocked(api.createArchetype).mockResolvedValue(newArchetype);

			const result = await store.create("New Arch", "ST", [
				{ metric_key: " // gitleaks:allowtest", weight: 1.0, inverted: false },
			]);

			expect(api.createArchetype).toHaveBeenCalledWith("New Arch", "ST", [
				{ metric_key: " // gitleaks:allowtest", weight: 1.0, inverted: false },
			]);
			expect(result).toEqual(newArchetype);
			expect(store.error).toBeNull();
		});
	});

	describe("update", () => {
		it("updates an existing archetype", async () => {
			const updatedArchetype = createMockArchetype({
				id: 1,
				name: "Updated Arch",
			});
			vi.mocked(api.updateArchetype).mockResolvedValue(updatedArchetype);

			const result = await store.update(1, "Updated Arch", [
				{ metric_key: " // gitleaks:allownew_metric", weight: 1.0, inverted: false },
			]);

			expect(api.updateArchetype).toHaveBeenCalledWith(
				1,
				"Updated Arch",
				[{ metric_key: " // gitleaks:allownew_metric", weight: 1.0, inverted: false }]
			);
			expect(result).toEqual(updatedArchetype);
		});
	});

	describe("remove", () => {
		it("deletes an archetype by id", async () => {
			vi.mocked(api.deleteArchetype).mockResolvedValue(undefined);

			await store.remove(1);

			expect(api.deleteArchetype).toHaveBeenCalledWith(1);
		});
	});

	describe("getArchetypesForRole", () => {
		it("returns archetypes filtered by role", async () => {
			const stArchetypes = [
				createMockArchetype({ id: 1, role: "ST", name: "ST 1" }),
				createMockArchetype({ id: 2, role: "ST", name: "ST 2" }),
			];
			const cbArchetypes = [
				createMockArchetype({ id: 3, role: "D", name: "D 1" }),
			];

			// Load all archetypes and check filtering
			vi.mocked(api.listAllArchetypes).mockResolvedValue([
				...stArchetypes,
				...cbArchetypes,
			]);
			await store.loadAll();

			expect(store.getArchetypesForRole("ST")).toEqual(stArchetypes);
			expect(store.getArchetypesForRole("D")).toEqual(cbArchetypes);
			expect(store.getArchetypesForRole("GK")).toEqual([]);
		});

		it("groups archetypes by role correctly", async () => {
			const gkArchetypes = [
				createMockArchetype({ id: 1, role: "GK", name: "GK 1" }),
				createMockArchetype({ id: 2, role: "GK", name: "GK 2" }),
			];
			const dmArchetypes = [
				createMockArchetype({ id: 3, role: "DM", name: "DM 1" }),
			];

			vi.mocked(api.listAllArchetypes).mockResolvedValue([
				...gkArchetypes,
				...dmArchetypes,
			]);
			await store.loadAll();

			expect(store.getArchetypesForRole("GK")).toHaveLength(2);
			expect(store.getArchetypesForRole("DM")).toHaveLength(1);
		});
	});

	describe("selectArchetype", () => {
		it("selects an archetype for a slot", async () => {
			const archetype = createMockArchetype({ id: 5 });
			vi.mocked(api.getArchetype).mockResolvedValue(archetype);

			await store.selectArchetype(5, "slot1");

			expect(store.getSelectedForSlot("slot1")).toEqual(archetype);
		});

		it("throws when archetype not found", async () => {
			vi.mocked(api.getArchetype).mockRejectedValue(new Error("Not found"));

			await expect(store.selectArchetype(999, "slot1")).rejects.toThrow(
				"Not found"
			);
		});
	});

	describe("getSelectedForSlot", () => {
		it("returns the selected archetype for a slot", async () => {
			const archetype = createMockArchetype({ id: 7 });
			vi.mocked(api.getArchetype).mockResolvedValue(archetype);

			await store.selectArchetype(7, "scouting-slot");

			expect(store.getSelectedForSlot("scouting-slot")).toEqual(archetype);
		});

		it("returns undefined for non-existent slot", () => {
			expect(store.getSelectedForSlot("nonexistent")).toBeUndefined();
		});
	});

	describe("clearError", () => {
		it("clears error state", async () => {
			vi.mocked(api.listAllArchetypes).mockRejectedValue(
				new Error("Some error")
			);

			await store.loadAll().catch(() => {
				// Expected to throw
			});

			expect(store.error).toBeTruthy();
			store.clearError();
			expect(store.error).toBeNull();
		});
	});

	describe("reset", () => {
		it("resets store to initial state", async () => {
			const mockArchetypes = [createMockArchetype({ id: 1 })];
			vi.mocked(api.listAllArchetypes).mockResolvedValue(mockArchetypes);
			await store.loadAll();

			store.reset();

			expect(store.archetypes).toEqual([]);
			expect(store.error).toBeNull();
			expect(store.loading).toBe(false);
		});
	});
});
