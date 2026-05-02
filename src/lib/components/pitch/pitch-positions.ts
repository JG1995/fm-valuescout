import type { ArchetypeRole } from "$lib/types/archetype";

/**
 * Represents a single position slot on the pitch in 4-4-2 formation.
 */
export interface PitchPosition {
	/** Unique identifier for this slot (e.g., "GK", "CB-L", "LS") */
	slotId: string;
	/** Display label shown in the slot (e.g., "GK", "CB-L") */
	label: string;
	/** Coarse archetype role for matching archetypes to positions */
	role: ArchetypeRole;
	/** Horizontal position as percentage (0-100) from left edge */
	x: number;
	/** Vertical position as percentage (0-100) from top edge */
	y: number;
}

/**
 * 4-4-2 formation position configuration.
 * Layout:
 *   - 1 GK (goalkeeper)
 *   - 4 D (defenders): LB, CB-L, CB-R, RB
 *   - 4 M/AM (midfielders/wings): LM, CM-L, CM-R, RM
 *   - 2 ST (strikers): LS, RS
 */
export const PITCH_POSITIONS: PitchPosition[] = [
	// Strikers (top of pitch)
	{ slotId: "LS", label: "LS", role: "ST", x: 35, y: 8 },
	{ slotId: "RS", label: "RS", role: "ST", x: 65, y: 8 },
	// Midfielders (middle band)
	{ slotId: "LM", label: "LM", role: "AM", x: 12, y: 32 },
	{ slotId: "CM-L", label: "CM-L", role: "M", x: 37, y: 32 },
	{ slotId: "CM-R", label: "CM-R", role: "M", x: 63, y: 32 },
	{ slotId: "RM", label: "RM", role: "AM", x: 88, y: 32 },
	// Defenders (back line)
	{ slotId: "LB", label: "LB", role: "D", x: 12, y: 58 },
	{ slotId: "CB-L", label: "CB-L", role: "D", x: 37, y: 58 },
	{ slotId: "CB-R", label: "CB-R", role: "D", x: 63, y: 58 },
	{ slotId: "RB", label: "RB", role: "D", x: 88, y: 58 },
	// Goalkeeper (bottom center)
	{ slotId: "GK", label: "GK", role: "GK", x: 50, y: 82 },
];
