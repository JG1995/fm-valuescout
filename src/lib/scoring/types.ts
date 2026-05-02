import type { MetricWeight } from "$lib/types/archetype";

/** A player's score for a specific archetype. */
export interface PlayerScore {
	/** Player's database ID (player_seasons.id). */
	playerId: number;
	/** Player's FM UID. */
	fmUid: number;
	/** Player display name. */
	name: string;
	/** Club name. */
	club: string | null;
	/** All positions this player can play (raw FM positions). */
	positions: string;
	/** Age. */
	age: number | null;
	/** Transfer value (high estimate). */
	transferValue: number | null;
	/** The archetype role this score is for. */
	role: string;
	/** Raw weighted score (0-100). */
	rawScore: number;
	/** Value-adjusted score: rawScore / (transferValue / medianValue). */
	valueAdjustedScore: number;
	/** Per-metric breakdown: metric_key → percentile. */
	metricPercentiles: Record<string, number>;
}

/** Pre-computed percentile cache for a dataset. */
export interface PercentileCache {
	/** Map: metric_key → sorted array of values (non-null only). */
	metricValues: Map<string, number[]>;
	/** Map: metric_key → player count (including nulls, for total N). */
	metricCounts: Map<string, number>;
}

/** A player record flattened for scoring. */
export interface ScorablePlayer {
	playerId: number;
	fmUid: number;
	name: string;
	club: string | null;
	positions: string;
	age: number | null;
	transferValueHigh: number | null;
	data: Record<string, unknown> | null; // ParsedPlayer as plain object
}
