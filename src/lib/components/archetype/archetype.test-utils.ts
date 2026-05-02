import type { Archetype } from "$lib/types/archetype";

/**
 * Create a mock archetype for testing.
 */
export function createMockArchetype(overrides: Partial<Archetype> = {}): Archetype {
	return {
		id: 1,
		name: "Test Archetype",
		role: "ST",
		metrics: [
			{ metric_key: "attacking.goals_per_90", // gitleaks:allow weight: 0.6, inverted: false },
			{ metric_key: "attacking.shots_per_90", // gitleaks:allow weight: 0.4, inverted: false },
		],
		is_default: false,
		created_at: "2026-05-02T00:00:00",
		updated_at: "2026-05-02T00:00:00",
		...overrides,
	};
}

/**
 * Create a list of mock archetypes for testing.
 */
export function createMockArchetypes(): Archetype[] {
	return [
		createMockArchetype({ id: 1, name: "Goal Poacher", role: "ST" }),
		createMockArchetype({ id: 2, name: "Target Man", role: "ST" }),
		createMockArchetype({ id: 3, name: "Deep Playmaker", role: "M" }),
		createMockArchetype({ id: 4, name: "Sweeper Keeper", role: "GK" }),
		createMockArchetype({ id: 5, name: "Ball Playing CB", role: "D" }),
	];
}
