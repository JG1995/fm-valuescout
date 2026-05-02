import { getMetricValue } from "./metric-accessor";
import { computePercentile, buildPercentileCache } from "./percentiles";
import type { ScorablePlayer, PlayerScore, PercentileCache } from "./types";
import type { Archetype, MetricWeight } from "$lib/types/archetype";

/**
 * Compute the median transfer value from a list of players.
 * Uses only non-null values. Returns 1 if no values exist (prevents division by zero).
 */
export function computeMedianTransferValue(players: ScorablePlayer[]): number {
	const values = players
		.map(p => p.transferValueHigh)
		.filter((v): v is number => v !== null && v > 0)
		.sort((a, b) => a - b);

	if (values.length === 0) return 1;

	const mid = Math.floor(values.length / 2);
	return values.length % 2 !== 0
		? values[mid]
		: (values[mid - 1] + values[mid]) / 2;
}

/**
 * Score a single player against an archetype.
 *
 * For each metric:
 * 1. Get the player's value
 * 2. Compute percentile within the dataset
 * 3. If inverted: use (100 - percentile)
 * 4. Multiply by weight
 * 5. Sum all weighted percentiles → raw score (0-100)
 *
 * Value-adjusted score = rawScore / (transferValue / medianValue)
 */
export function scorePlayer(
	player: ScorablePlayer,
	archetype: Archetype,
	allMetricValues: Record<string, number[]>,
	medianTransferValue: number,
): PlayerScore {
	let rawScore = 0;
	const metricPercentiles: Record<string, number> = {};

	for (const metric of archetype.metrics) {
		const playerValue = getMetricValue(player.data, metric.metric_key);
		const sortedValues = allMetricValues[metric.metric_key] ?? [];

		let percentile: number;
		if (playerValue === null) {
			percentile = 0; // Worst case for missing data
		} else {
			percentile = computePercentile(playerValue, sortedValues);
		}

		if (metric.inverted) {
			percentile = 100 - percentile;
		}

		metricPercentiles[metric.metric_key] = percentile;
		rawScore += percentile * metric.weight;
	}

	// Value-adjusted score
	const transferValue = player.transferValueHigh ?? medianTransferValue;
	const valueRatio = transferValue / medianTransferValue;
	const valueAdjustedScore = valueRatio > 0 ? rawScore / valueRatio : rawScore;

	return {
		playerId: player.playerId,
		fmUid: player.fmUid,
		name: player.name,
		club: player.club,
		positions: player.positions,
		age: player.age,
		transferValue: player.transferValueHigh,
		role: archetype.role,
		rawScore,
		valueAdjustedScore,
		metricPercentiles,
	};
}

/**
 * Score all players against a specific archetype.
 * Builds the percentile cache from the player data and the archetype's metrics.
 */
export function scoreAllPlayers(
	players: ScorablePlayer[],
	archetype: Archetype,
): PlayerScore[] {
	const metricKeys = archetype.metrics.map(m => m.metric_key);
	const cache = buildPercentileCache(players, metricKeys);

	// Convert cache to plain record for scoring
	const allMetricValues: Record<string, number[]> = {};
	for (const [key, values] of cache.metricValues) {
		allMetricValues[key] = values;
	}

	const medianTransferValue = computeMedianTransferValue(players);

	return players.map(player =>
		scorePlayer(player, archetype, allMetricValues, medianTransferValue)
	);
}
