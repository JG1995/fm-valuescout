import { getMetricValue } from "./metric-accessor";
import type { ScorablePlayer, PercentileCache } from "./types";

/**
 * Compute the percentile rank of a value within a sorted array.
 * Uses linear interpolation (same as Excel PERCENTRANK.INC).
 * Returns 0 for empty arrays.
 */
export function computePercentile(value: number, sortedValues: number[]): number {
	if (sortedValues.length === 0) return 0;
	if (sortedValues.length === 1) {
		return value >= sortedValues[0] ? 100 : 0;
	}

	// Check if all values are the same
	const first = sortedValues[0];
	if (sortedValues.every(v => v === first)) {
		return 50; // All same value → return midpoint
	}

	// Find insertion point
	let low = 0;
	let high = sortedValues.length - 1;

	if (value <= sortedValues[low]) return 0;
	if (value >= sortedValues[high]) return 100;

	// Binary search for position
	while (low <= high) {
		const mid = Math.floor((low + high) / 2);
		if (sortedValues[mid] < value) {
			low = mid + 1;
		} else if (sortedValues[mid] > value) {
			high = mid - 1;
		} else {
			// Exact match — interpolate between duplicates
			// Find first occurrence
			let first = mid;
			while (first > 0 && sortedValues[first - 1] === value) first--;
			// Find last occurrence
			let last = mid;
			while (last < sortedValues.length - 1 && sortedValues[last + 1] === value) last++;

			// Use midpoint of the duplicate range
			const avgRank = (first + last) / 2;
			return (avgRank / (sortedValues.length - 1)) * 100;
		}
	}

	// Interpolate between low-1 and low
	const lower = sortedValues[low - 1];
	const upper = sortedValues[low];
	const fraction = (value - lower) / (upper - lower);
	return ((low - 1 + fraction) / (sortedValues.length - 1)) * 100;
}

/**
 * Build a percentile cache from a list of players and the metrics to track.
 * Collects all non-null values for each metric and sorts them.
 */
export function buildPercentileCache(
	players: ScorablePlayer[],
	metricKeys: string[],
): PercentileCache {
	const metricValues = new Map<string, number[]>();
	const metricCounts = new Map<string, number>();

	for (const key of metricKeys) {
		const values: number[] = [];
		let count = 0;
		for (const player of players) {
			count++;
			const val = getMetricValue(player.data, key);
			if (val !== null) {
				values.push(val);
			}
		}
		values.sort((a, b) => a - b);
		metricValues.set(key, values);
		metricCounts.set(key, count);
	}

	return { metricValues, metricCounts };
}
