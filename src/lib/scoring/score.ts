/**
 * Scoring Engine for FM ValueScout's moneyball scouting feature.
 *
 * Pure TypeScript functions for computing player scores against archetypes.
 * No dependencies on Tauri or browser APIs.
 */

import type { Archetype } from "$lib/types/archetype";

/**
 * Parsed player data from FM CSV import.
 * This interface represents the structure of player data after being parsed.
 * Individual fields are accessed via dot notation (e.g., player["attacking.goals_per_90"]).
 */
export interface ParsedPlayer {
	/** Unique player identifier */
	id: string;
	/** Player display name */
	name: string;
	/** Player's position/role */
	position: string;
	/** Player age */
	age: number;
	/** Transfer market value in currency units */
	transfer_value: number;
	/** Additional dynamic metric fields accessed via bracket notation */
	[key: string]: unknown;
}

/**
 * A player with computed scoring data.
 */
export interface ScoredPlayer {
	/** The player's parsed data */
	player: ParsedPlayer;
	/** The archetype this score was computed against */
	archetypeId: number;
	/** Raw weighted score (0-100) before value adjustment */
	rawScore: number;
	/** Value-adjusted score (rewards cheap high-performers) */
	valueAdjustedScore: number;
	/** Per-metric percentile values for display */
	percentileByMetric: Map<string, number>;
}

/**
 * Computes the percentile rank of a value within a dataset.
 *
 * @param value - The value to compute percentile for
 * @param allValues - Array of all values in the dataset
 * @returns Percentile rank from 0-100
 *
 * Edge cases:
 * - Empty array: returns 50 (neutral fallback)
 * - Single value: returns 50 (neutral fallback)
 * - All same values: returns 50 (neutral fallback)
 * - Value equals min: returns 0
 * - Value equals max: returns 100
 */
export function computePercentile(value: number, allValues: number[]): number {
	// Edge case: empty array
	if (allValues.length === 0) {
		return 50;
	}

	// Edge case: single value or all same values
	const uniqueValues = [...new Set(allValues)];
	if (uniqueValues.length <= 1) {
		return 50;
	}

	// Sort values for percentile calculation
	const sorted = [...allValues].sort((a, b) => a - b);
	const min = sorted[0];
	const max = sorted[sorted.length - 1];

	// Edge case: value equals minimum
	if (value <= min) {
		return 0;
	}

	// Edge case: value equals or exceeds maximum
	if (value >= max) {
		return 100;
	}

	// Count values strictly less than the given value
	const countLess = sorted.filter((v) => v < value).length;
	const total = sorted.length;

	// Percentile formula: (count less than value / (total - 1)) * 100
	// We use total - 1 to normalize the range since min and max are extremes
	const percentile = (countLess / (total - 1)) * 100;

	return percentile;
}

/**
 * Gets the percentile rank for a player's value within the percentile distribution.
 *
 * @param value - The player's metric value
 * @param percentileValues - Sorted array of all percentile boundary values
 * @returns Percentile rank from 0-100
 */
function getPercentileFromDistribution(
	value: number,
	percentileValues: number[]
): number {
	// Handle empty distribution
	if (percentileValues.length === 0) {
		return 50;
	}

	// Handle single value distribution
	if (percentileValues.length === 1) {
		return 50;
	}

	const sorted = [...percentileValues].sort((a, b) => a - b);
	const min = sorted[0];
	const max = sorted[sorted.length - 1];

	// Value below minimum
	if (value <= min) {
		return 0;
	}

	// Value at or above maximum
	if (value >= max) {
		return 100;
	}

	// Find position in distribution
	const countLess = sorted.filter((v) => v < value).length;
	const total = sorted.length;

	return (countLess / (total - 1)) * 100;
}

/**
 * Computes a player's raw score against an archetype.
 *
 * @param player - Parsed player data
 * @param archetype - Archetype with metric weights
 * @param percentiles - Pre-computed percentile distributions per metric key
 * @returns Weighted score from 0-100
 */
export function computePlayerScore(
	player: ParsedPlayer,
	archetype: Archetype,
	percentiles: Map<string, number[]>
): number {
	let totalScore = 0;

	for (const metric of archetype.metrics) {
		const metricKey = metric.metric_key;
		const weight = metric.weight;

		// Get player's value for this metric (default to 0 if missing)
		const playerValue = (player[metricKey] as number) ?? 0;

		// Get percentile distribution for this metric
		const distribution = percentiles.get(metricKey);

		// Compute percentile (use 0 if distribution not available)
		let percentile: number;
		if (distribution) {
			percentile = getPercentileFromDistribution(playerValue, distribution);
		} else {
			// Missing metric data = 0 percentile (worst score)
			percentile = 0;
		}

		// Apply inversion for metrics where lower is better
		if (metric.inverted) {
			percentile = 100 - percentile;
		}

		// Add weighted contribution to total score
		totalScore += percentile * weight;
	}

	return totalScore;
}

/**
 * Computes a value-adjusted score that rewards cost-efficient players.
 *
 * The value adjustment divides the raw score by the ratio of player transfer
 * value to median transfer value. This means:
 * - A player costing half the median with the same raw score gets 2x the adjusted score
 * - A player costing double the median with the same raw score gets 0.5x the adjusted score
 *
 * @param rawScore - Raw weighted score (0-100)
 * @param transferValue - Player's transfer market value
 * @param medianTransferValue - Median transfer value of the dataset
 * @returns Value-adjusted score (can exceed 100 for cheap high-performers)
 */
export function computeValueAdjustedScore(
	rawScore: number,
	transferValue: number,
	medianTransferValue: number
): number {
	// Handle invalid transfer values
	if (transferValue <= 0) {
		return rawScore;
	}

	if (medianTransferValue <= 0) {
		return rawScore;
	}

	// Value adjustment formula: rawScore / (transferValue / medianTransferValue)
	const valueRatio = transferValue / medianTransferValue;
	const adjustedScore = rawScore / valueRatio;

	return adjustedScore;
}

/**
 * Scores a player against an archetype with full scoring pipeline.
 *
 * @param player - Parsed player data
 * @param archetype - Archetype with metric weights
 * @param percentiles - Pre-computed percentile distributions per metric key
 * @param medianTransferValue - Median transfer value for value adjustment
 * @returns Complete scored player data
 */
export function scorePlayer(
	player: ParsedPlayer,
	archetype: Archetype,
	percentiles: Map<string, number[]>,
	medianTransferValue: number
): ScoredPlayer {
	// Compute raw score
	const rawScore = computePlayerScore(player, archetype, percentiles);

	// Compute value-adjusted score
	const valueAdjustedScore = computeValueAdjustedScore(
		rawScore,
		player.transfer_value,
		medianTransferValue
	);

	// Build per-metric percentile map for display
	const percentileByMetric = new Map<string, number>();
	for (const metric of archetype.metrics) {
		const metricKey = metric.metric_key;
		const playerValue = (player[metricKey] as number) ?? 0;
		const distribution = percentiles.get(metricKey);

		if (distribution) {
			let percentile = getPercentileFromDistribution(playerValue, distribution);
			if (metric.inverted) {
				percentile = 100 - percentile;
			}
			percentileByMetric.set(metricKey, percentile);
		} else {
			percentileByMetric.set(metricKey, 0);
		}
	}

	return {
		player,
		archetypeId: archetype.id,
		rawScore,
		valueAdjustedScore,
		percentileByMetric,
	};
}
