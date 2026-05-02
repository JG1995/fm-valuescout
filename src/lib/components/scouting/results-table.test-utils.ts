import type { ScoredPlayer } from "$lib/scoring/score";
import type { ParsedPlayer } from "$lib/scoring/score";

/**
 * Create a mock parsed player for testing.
 */
export function createMockParsedPlayer(overrides: Partial<ParsedPlayer> = {}): ParsedPlayer {
	return {
		id: "1",
		name: "Test Player",
		position: "ST",
		age: 25,
		transfer_value: 10000000,
		club: "Test Club",
		...overrides,
	};
}

/**
 * Create a mock scored player for testing.
 */
export function createMockScoredPlayer(overrides: Partial<ScoredPlayer> = {}): ScoredPlayer {
	const player = createMockParsedPlayer(overrides.player as Partial<ParsedPlayer> || {});
	return {
		player,
		archetypeId: 1,
		rawScore: 75.5,
		valueAdjustedScore: 85.2,
		percentileByMetric: new Map([
			["attacking.goals_per_90", 80],
			["attacking.shots_per_90", 70],
		]),
		...overrides,
	};
}

/**
 * Create a list of mock scored players for testing.
 */
export function createMockScoredPlayers(): ScoredPlayer[] {
	return [
		createMockScoredPlayer({
			player: createMockParsedPlayer({
				id: "1",
				name: "Lionel Messi",
				age: 36,
				transfer_value: 15000000,
				club: "Inter Miami",
			}),
			rawScore: 92.5,
			valueAdjustedScore: 88.3,
		}),
		createMockScoredPlayer({
			player: createMockParsedPlayer({
				id: "2",
				name: "Erling Haaland",
				age: 24,
				transfer_value: 200000000,
				club: "Manchester City",
			}),
			rawScore: 95.0,
			valueAdjustedScore: 45.0,
		}),
		createMockScoredPlayer({
			player: createMockParsedPlayer({
				id: "3",
				name: "Jude Bellingham",
				age: 20,
				transfer_value: 120000000,
				club: "Real Madrid",
			}),
			rawScore: 88.0,
			valueAdjustedScore: 52.8,
		}),
		createMockScoredPlayer({
			player: createMockParsedPlayer({
				id: "4",
				name: "Young Prospect",
				age: 18,
				transfer_value: 5000000,
				club: "Youth Team",
			}),
			rawScore: 70.0,
			valueAdjustedScore: 98.0,
		}),
	];
}
