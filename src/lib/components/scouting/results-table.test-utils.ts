import type { PlayerScore } from "$lib/scoring";

/**
 * Create a mock player score for testing.
 */
export function createMockPlayerScore(overrides: Partial<PlayerScore> = {}): PlayerScore {
	return {
		playerId: 1,
		fmUid: 12345,
		name: "Test Player",
		club: "Test Club",
		positions: "ST",
		age: 25,
		transferValue: 10_000_000,
		role: "ST",
		rawScore: 75.5,
		valueAdjustedScore: 85.2,
		metricPercentiles: {
			"attacking.goals_per_90": 80,
			"attacking.shots_per_90": 70,
		},
		...overrides,
	};
}

/**
 * Create a list of mock player scores for testing.
 */
export function createMockScoredPlayers(): PlayerScore[] {
	return [
		createMockPlayerScore({
			playerId: 1,
			fmUid: 1001,
			name: "Lionel Messi",
			age: 36,
			transferValue: 15_000_000,
			club: "Inter Miami",
			rawScore: 92.5,
			valueAdjustedScore: 88.3,
		}),
		createMockPlayerScore({
			playerId: 2,
			fmUid: 1002,
			name: "Erling Haaland",
			age: 24,
			transferValue: 200_000_000,
			club: "Manchester City",
			rawScore: 95.0,
			valueAdjustedScore: 45.0,
		}),
		createMockPlayerScore({
			playerId: 3,
			fmUid: 1003,
			name: "Jude Bellingham",
			age: 20,
			transferValue: 120_000_000,
			club: "Real Madrid",
			rawScore: 88.0,
			valueAdjustedScore: 52.8,
		}),
		createMockPlayerScore({
			playerId: 4,
			fmUid: 1004,
			name: "Young Prospect",
			age: 18,
			transferValue: 5_000_000,
			club: "Youth Team",
			rawScore: 70.0,
			valueAdjustedScore: 98.0,
		}),
	];
}
