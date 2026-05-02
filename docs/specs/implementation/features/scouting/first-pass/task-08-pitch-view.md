# Task 08 - Pitch View Component

## Overview

Create the interactive football pitch SVG component showing all 11 position slots in a standard 4-4-2 formation. Each slot is clickable and displays the selected archetype name (or a placeholder).

Uses TDD: write failing tests first, then implement to make them pass.

## Files to Create/Modify

- Create: `src/lib/components/pitch/pitch-positions.test.ts` — Unit tests for position data
- Create: `src/lib/components/pitch/pitch-positions.ts` — Position configuration data
- Create: `src/lib/components/pitch/PositionSlot.svelte` — Individual position slot
- Create: `src/lib/components/pitch/PitchView.svelte` — Main pitch component
- Create: `src/lib/components/pitch/pitch.css` — Pitch and slot styling

## Context

### Pitch Layout

The pitch uses a standard 4-4-2 formation with these positions:

- GK (1)
- DL, DC, DC, DR (4)
- ML, MC, MC, MR (4)
- STC, STC (2)

Total: 11 slots.

### Slot Data Model

Each position slot needs:

- A unique `slotId` (e.g., "GK", "CB-L", "CB-R", "LB", "RB", "LM", "CM-L", "CM-R", "RM", "LS", "RS")
- A display label
- The archetype `role` it maps to (coarse roles: GK for GK, D for CB-L/CB-R/LB/RB, M for CM-L/CM-R, AM for LM/RM, ST for LS/RS)
- X/Y coordinates for positioning on the pitch (percentage-based)

### Interaction

Clicking a slot dispatches an event to the parent with the `slotId`. The parent (ScoutingPage) handles opening the archetype selector. The slot displays:

- Selected archetype name (if one is selected)
- "Select archetype" placeholder (if none selected)

## Steps

- [ ] **Step 1: Write failing tests for pitch positions data**

Create `src/lib/components/pitch/pitch-positions.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { PITCH_POSITIONS } from "./pitch-positions";
import type { ArchetypeRole } from "$lib/types/archetype";

// Valid roles from the Archetype type
const VALID_ROLES: ArchetypeRole[] = ["GK", "D", "M", "AM", "ST"];

describe("PITCH_POSITIONS", () => {
	it("has exactly 11 positions", () => {
		expect(PITCH_POSITIONS).toHaveLength(11);
	});

	it("all slotIds are unique", () => {
		const ids = PITCH_POSITIONS.map((p) => p.slotId);
		const unique = new Set(ids);
		expect(unique.size).toBe(ids.length);
	});

	it("all roles are valid ArchetypeRole values", () => {
		for (const pos of PITCH_POSITIONS) {
			expect(VALID_ROLES).toContain(pos.role);
		}
	});

	it("all x/y coordinates are in 0-100 range", () => {
		for (const pos of PITCH_POSITIONS) {
			expect(pos.x).toBeGreaterThanOrEqual(0);
			expect(pos.x).toBeLessThanOrEqual(100);
			expect(pos.y).toBeGreaterThanOrEqual(0);
			expect(pos.y).toBeLessThanOrEqual(100);
		}
	});

	it("no duplicate slotIds", () => {
		const ids = PITCH_POSITIONS.map((p) => p.slotId);
		const seen = new Set<string>();
		for (const id of ids) {
			expect(seen.has(id)).toBe(false);
			seen.add(id);
		}
	});

	it("positions form a valid 4-4-2 formation", () => {
		const byRole: Record<string, number> = {};
		for (const pos of PITCH_POSITIONS) {
			byRole[pos.role] = (byRole[pos.role] ?? 0) + 1;
		}
		// 4-4-2: 2 ST, 4 midfield (2 M + 2 AM), 4 defenders (all D), 1 GK
		expect(byRole["ST"]).toBe(2);
		expect(byRole["M"] + (byRole["AM"] ?? 0)).toBe(4);
		expect(byRole["D"]).toBe(4);
		expect(byRole["GK"]).toBe(1);
	});
});
```

Run: `npx vitest run`
Expected: FAIL — file does not exist yet.

- [ ] **Step 2: Create position configuration**

Create directory `src/lib/components/pitch/` and file `src/lib/components/pitch/pitch-positions.ts`:

```typescript
import type { ArchetypeRole } from "$lib/types/archetype";

export interface PitchPosition {
	/** Unique slot identifier. */
	slotId: string;
	/** Display label on the pitch. */
	label: string;
	/** Archetype role this slot maps to. */
	role: ArchetypeRole;
	/** X position as percentage (0 = left, 100 = right). */
	x: number;
	/** Y position as percentage (0 = top/goal, 100 = bottom/own goal). */
	y: number;
}

/**
 * 4-4-2 formation positions.
 * Y axis: 0 = opponent goal (top), 100 = own goal (bottom).
 * X axis: 0 = left, 100 = right.
 */
export const PITCH_POSITIONS: PitchPosition[] = [
	// Strikers (top)
	{ slotId: "LS", label: "LS", role: "ST", x: 35, y: 8 },
	{ slotId: "RS", label: "RS", role: "ST", x: 65, y: 8 },

	// Midfielders
	{ slotId: "LM", label: "LM", role: "AM", x: 12, y: 32 },
	{ slotId: "CM-L", label: "CM-L", role: "M", x: 37, y: 32 },
	{ slotId: "CM-R", label: "CM-R", role: "M", x: 63, y: 32 },
	{ slotId: "RM", label: "RM", role: "AM", x: 88, y: 32 },

	// Defenders
	{ slotId: "LB", label: "LB", role: "D", x: 12, y: 58 },
	{ slotId: "CB-L", label: "CB-L", role: "D", x: 37, y: 58 },
	{ slotId: "CB-R", label: "CB-R", role: "D", x: 63, y: 58 },
	{ slotId: "RB", label: "RB", role: "D", x: 88, y: 58 },

	// Goalkeeper
	{ slotId: "GK", label: "GK", role: "GK", x: 50, y: 82 },
];
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `npx vitest run`
Expected: ALL PASS.

- [ ] **Step 4: Create PositionSlot component**

Create `src/lib/components/pitch/PositionSlot.svelte`:

```svelte
<script lang="ts">
    import type { PitchPosition } from "./pitch-positions";
    import type { Archetype } from "$lib/types/archetype";

    interface Props {
        position: PitchPosition;
        selectedArchetype: Archetype | null;
        onclick: (slotId: string) => void;
    }

    let { position, selectedArchetype, onclick }: Props = $props();

    let displayName = $derived(
        selectedArchetype?.name ?? "Select"
    );

    let hasSelection = $derived(selectedArchetype !== null);
</script>

<button
    class="position-slot"
    class:has-selection={hasSelection}
    style="left: {position.x}%; top: {position.y}%;"
    onclick={() => onclick(position.slotId)}
    title="{position.label}: {displayName}"
>
    <span class="slot-label">{position.label}</span>
    <span class="slot-archetype">{displayName}</span>
</button>

<style>
    .position-slot {
        position: absolute;
        transform: translate(-50%, -50%);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        width: 80px;
        height: 56px;
        border: 2px solid rgba(255, 255, 255, 0.6);
        border-radius: 8px;
        background: rgba(0, 0, 0, 0.4);
        color: white;
        cursor: pointer;
        transition: background-color 0.15s, border-color 0.15s;
        padding: 4px;
        box-sizing: border-box;
    }

    .position-slot:hover {
        background: rgba(0, 0, 0, 0.6);
        border-color: rgba(255, 255, 255, 0.9);
    }

    .position-slot.has-selection {
        border-color: #4caf50;
        background: rgba(76, 175, 80, 0.25);
    }

    .slot-label {
        font-size: 0.7rem;
        font-weight: 600;
        text-transform: uppercase;
        opacity: 0.8;
    }

    .slot-archetype {
        font-size: 0.65rem;
        max-width: 72px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
</style>
```

- [ ] **Step 5: Create PitchView component**

Create `src/lib/components/pitch/PitchView.svelte`:

```svelte
<script lang="ts">
    import { PITCH_POSITIONS } from "./pitch-positions";
    import PositionSlot from "./PositionSlot.svelte";
    import type { Archetype } from "$lib/types/archetype";

    interface Props {
        selectedArchetypes: Record<string, Archetype | null>;
        onslotclick: (slotId: string) => void;
    }

    let { selectedArchetypes, onslotclick }: Props = $props();
</script>

<div class="pitch-container">
    <div class="pitch">
        <!-- Pitch markings -->
        <div class="pitch-field">
            <div class="center-circle"></div>
            <div class="center-line"></div>
            <div class="goal-area goal-area-top"></div>
            <div class="goal-area goal-area-bottom"></div>
        </div>

        <!-- Position slots -->
        {#each PITCH_POSITIONS as position (position.slotId)}
            <PositionSlot
                {position}
                selectedArchetype={selectedArchetypes[position.slotId] ?? null}
                onclick={onslotclick}
            />
        {/each}
    </div>
</div>

<style>
    .pitch-container {
        width: 100%;
        max-width: 600px;
        margin: 0 auto;
    }

    .pitch {
        position: relative;
        width: 100%;
        /* Aspect ratio roughly 2:3 for a vertical pitch view */
        padding-bottom: 140%;
        background: #2e7d32;
        border-radius: 8px;
        overflow: hidden;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    }

    .pitch-field {
        position: absolute;
        inset: 0;
    }

    .center-circle {
        position: absolute;
        width: 30%;
        padding-bottom: 30%;
        left: 35%;
        top: 35%;
        border: 2px solid rgba(255, 255, 255, 0.3);
        border-radius: 50%;
    }

    .center-line {
        position: absolute;
        left: 5%;
        right: 5%;
        top: 50%;
        height: 2px;
        background: rgba(255, 255, 255, 0.3);
    }

    .goal-area {
        position: absolute;
        left: 25%;
        right: 25%;
        height: 12%;
        border: 2px solid rgba(255, 255, 255, 0.3);
    }

    .goal-area-top {
        top: 0;
        border-top: none;
    }

    .goal-area-bottom {
        bottom: 0;
        border-bottom: none;
    }
</style>
```

- [ ] **Step 6: Verify TypeScript compilation**

Run: `bun run check`
Expected: SUCCESS — no errors.

## Dependencies

- Task 06 (frontend types) — `Archetype`, `ArchetypeRole` types
- Task 06 (vitest setup) — `pitch-positions.test.ts`

## Success Criteria

- Pitch renders with 11 position slots in 4-4-2 formation
- Each slot displays position label and selected archetype name
- Slots without selection show "Select" placeholder
- Clicking a slot fires the `onslotclick` callback with the `slotId`
- Slots with a selected archetype have a green border highlight
- Pitch has visual markings (center circle, center line, goal areas)
- `bun run check` passes
- All unit tests pass

## Tests

### Test 1: Pitch positions data

**What to test:** `PITCH_POSITIONS` has exactly 11 positions, unique slotIds, valid ArchetypeRole values, x/y in 0–100 range, and correct 4-4-2 formation.
**Command:** `npx vitest run src/lib/components/pitch/pitch-positions.test.ts`
**Feasibility:** ✅ Unit test — pure data validation.

### Test 2: Component renders

**What to test:** PitchView renders 11 position slots with correct labels.
**Feasibility:** ⚠️ Svelte component testing — verify in dev mode (`bun run tauri dev`).

### Test 3: TypeScript compilation

**What to test:** All components compile without errors.
**Command:** `bun run check`
**Feasibility:** ✅ Can be tested — `bun run check`.
