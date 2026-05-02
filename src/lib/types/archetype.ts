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
	/** Position role (coarse): "GK" | "D" | "WB" | "DM" | "M" | "AM" | "ST" */
	role: string;
	metrics: MetricWeight[];
	is_default: boolean;
	created_at: string;
	updated_at: string;
}

/** All valid archetype role strings (coarse system). */
export type ArchetypeRole = "GK" | "D" | "WB" | "DM" | "M" | "AM" | "ST";

/** Role display names for UI. */
export const ROLE_LABELS: Record<ArchetypeRole, string> = {
	GK: "Goalkeeper",
	D: "Defender",
	WB: "Wing Back",
	DM: "Defensive Midfielder",
	M: "Midfielder",
	AM: "Attacking Midfielder / Winger",
	ST: "Striker",
};

// Coarse roles match parser::types::Role exactly: GK, D, WB, DM, M, AM, ST
// No mapping needed — PARSER_ROLE_TO_ARCHETYPE_ROLES was removed (identity mapping)
