/** A single metric entry in an archetype's scoring configuration. */
export interface MetricWeight {
	/** ParsedPlayer field key (e.g., "attacking.goals_per_90") */
	metric_key: string;
	/** Weight 0.0–1.0. All weights in an archetype sum to ~1.0. */
	weight: number;
	/** If true, lower values are better (e.g., "fouls_made_per_90"). */
	inverted: boolean;
}

/** A scoring archetype for a position role. */
export interface Archetype {
	id: number;
	name: string;
	/** Position role: "GK" | "CB" | "FB" | "DM" | "WB" | "CM" | "W" | "AM" | "ST" */
	role: string;
	metrics: MetricWeight[];
	is_default: boolean;
	created_at: string;
	updated_at: string;
}

/** All valid archetype role strings. */
export type ArchetypeRole = "GK" | "CB" | "FB" | "DM" | "WB" | "CM" | "W" | "AM" | "ST";

/** Role display names for UI. */
export const ROLE_LABELS: Record<ArchetypeRole, string> = {
	GK: "Goalkeeper",
	CB: "Center Back",
	FB: "Full Back",
	DM: "Defensive Midfielder",
	WB: "Wing Back",
	CM: "Central Midfielder",
	W: "Winger",
	AM: "Attacking Midfielder",
	ST: "Striker",
};

/**
 * Map FM parser Role enum values to archetype roles.
 * The parser uses: GK, D, WB, DM, M, AM, ST
 * Archetypes use: GK, CB, FB, DM, WB, CM, W, AM, ST
 * This mapping is used when scoring a player against archetypes.
 */
export const PARSER_ROLE_TO_ARCHETYPE_ROLES: Record<string, ArchetypeRole[]> = {
	GK: ["GK"],
	D: ["CB", "FB"], // Defenders can be CB or FB
	WB: ["WB"],
	DM: ["DM"],
	M: ["CM"],
	AM: ["AM", "W"], // AM can be AM or Winger
	ST: ["ST"],
};
