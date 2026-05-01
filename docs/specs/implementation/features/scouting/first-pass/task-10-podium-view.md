# Task 10 - Podium View Component

## Overview

Create the Top-3 podium component that displays the best-scoring players for the selected archetype. Uses a classic 3-2-1 layout with 1st place center (tallest), 2nd place left, 3rd place right.

## Files to Create/Modify

- Create: `src/lib/components/scouting/PodiumView.svelte` — Podium component
- Create: `src/lib/components/scouting/PodiumPosition.svelte` — Individual podium card

## Context

### Podium Layout

From the design spec:
- 1st place: center, tallest platform
- 2nd place: left, medium platform
- 3rd place: right, shortest platform

Each podium card shows:
- Position number (1st, 2nd, 3rd)
- Player name
- Club
- Raw score
- Value-adjusted score

Ties are broken by value-adjusted score (cheaper is better, so higher value-adjusted = better).

### Data Source

The podium receives `PlayerScore[]` (from `src/lib/scoring/types.ts`), already sorted by value-adjusted score descending. The component takes the top 3.

## Steps

- [ ] **Step 1: Create PodiumPosition component**

Create directory `src/lib/components/scouting/` and file `src/lib/components/scouting/PodiumPosition.svelte`:

```svelte
<script lang="ts">
    import type { PlayerScore } from "$lib/scoring/types";

    interface Props {
        position: 1 | 2 | 3;
        score: PlayerScore | null;
    }

    let { position, score }: Props = $props();

    let positionLabel = $derived(
        position === 1 ? "1st" : position === 2 ? "2nd" : "3rd"
    );

    let heightClass = $derived(
        position === 1 ? "tallest" : position === 2 ? "medium" : "short"
    );
</script>

<div class="podium-position" class:empty={!score}>
    {#if score}
        <div class="player-card">
            <span class="position-badge">{positionLabel}</span>
            <div class="player-info">
                <span class="player-name">{score.name}</span>
                <span class="player-club">{score.club ?? "—"}</span>
            </div>
            <div class="scores">
                <div class="score-row">
                    <span class="score-label">Score</span>
                    <span class="score-value">{score.rawScore.toFixed(1)}</span>
                </div>
                <div class="score-row">
                    <span class="score-label">Value Adj.</span>
                    <span class="score-value highlight">{score.valueAdjustedScore.toFixed(1)}</span>
                </div>
            </div>
        </div>
    {:else}
        <div class="player-card placeholder">
            <span class="position-badge">{positionLabel}</span>
            <span class="empty-text">—</span>
        </div>
    {/if}
    <div class="podium-platform {heightClass}">
        <span class="position-number">{position}</span>
    </div>
</div>

<style>
    .podium-position {
        display: flex;
        flex-direction: column;
        align-items: center;
        flex: 1;
    }

    .player-card {
        background: #1e1e1e;
        border-radius: 8px;
        padding: 12px;
        width: 100%;
        max-width: 180px;
        display: flex;
        flex-direction: column;
        gap: 8px;
        border: 1px solid #333;
    }

    .player-card.placeholder {
        align-items: center;
        justify-content: center;
        min-height: 80px;
    }

    .position-badge {
        display: inline-block;
        font-size: 0.7rem;
        font-weight: 700;
        padding: 2px 8px;
        border-radius: 4px;
        background: #333;
        color: #aaa;
        align-self: flex-start;
    }

    .player-info {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .player-name {
        font-weight: 600;
        color: #fff;
        font-size: 0.9rem;
    }

    .player-club {
        font-size: 0.75rem;
        color: #888;
    }

    .scores {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .score-row {
        display: flex;
        justify-content: space-between;
        font-size: 0.8rem;
    }

    .score-label {
        color: #888;
    }

    .score-value {
        font-family: monospace;
        color: #ddd;
    }

    .score-value.highlight {
        color: #4caf50;
        font-weight: 600;
    }

    .empty-text {
        color: #555;
        font-size: 1.5rem;
    }

    .podium-platform {
        width: 100%;
        max-width: 180px;
        border-radius: 0 0 8px 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: #2a2a2a;
        border: 1px solid #333;
        border-top: none;
    }

    .podium-platform.tallest {
        height: 60px;
        background: linear-gradient(to bottom, #ffd700, #b8860b);
    }

    .podium-platform.medium {
        height: 45px;
        background: linear-gradient(to bottom, #c0c0c0, #808080);
    }

    .podium-platform.short {
        height: 30px;
        background: linear-gradient(to bottom, #cd7f32, #8b4513);
    }

    .position-number {
        font-size: 1.2rem;
        font-weight: 700;
        color: rgba(0, 0, 0, 0.5);
    }
</style>
```

- [ ] **Step 2: Create PodiumView component**

Create `src/lib/components/scouting/PodiumView.svelte`:

```svelte
<script lang="ts">
    import type { PlayerScore } from "$lib/scoring/types";
    import PodiumPosition from "./PodiumPosition.svelte";

    interface Props {
        scores: PlayerScore[];
    }

    let { scores }: Props = $props();

    // Sort by value-adjusted score descending, take top 3
    let top3 = $derived.by(() => {
        const sorted = [...scores].sort((a, b) => b.valueAdjustedScore - a.valueAdjustedScore);
        return sorted.slice(0, 3);
    });
</script>

<div class="podium-view">
    <h2 class="podium-title">Top 3</h2>
    <div class="podium-layout">
        <!-- 2nd place (left) -->
        <PodiumPosition position={2} score={top3.length >= 2 ? top3[1] : null} />

        <!-- 1st place (center) -->
        <PodiumPosition position={1} score={top3.length >= 1 ? top3[0] : null} />

        <!-- 3rd place (right) -->
        <PodiumPosition position={3} score={top3.length >= 3 ? top3[2] : null} />
    </div>
</div>

<style>
    .podium-view {
        padding: 16px 0;
    }

    .podium-title {
        text-align: center;
        color: #fff;
        font-size: 1.1rem;
        margin: 0 0 16px 0;
    }

    .podium-layout {
        display: flex;
        align-items: flex-end;
        justify-content: center;
        gap: 12px;
        padding: 0 16px;
    }
</style>
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `bun run check`
Expected: SUCCESS.

## Dependencies

- Task 07 (scoring engine) — `PlayerScore` type

## Success Criteria

- Podium renders with 3-2-1 layout (2nd left, 1st center tallest, 3rd right)
- Each position shows player name, club, raw score, value-adjusted score
- Empty positions show placeholder
- Podium platforms have gold/silver/bronze gradient styling
- `bun run check` passes

## Tests

### Test 1: TypeScript compilation

**What to test:** Components compile without errors.
**Command:** `bun run check`
**Feasibility:** ✅ Can be tested — `bun run check`.

### Test 2: Visual verification

**What to test:** Podium renders correctly with gold/silver/bronze platforms, 3-2-1 layout, empty placeholders.
**Feasibility:** ⚠️ No Svelte testing library installed. Svelte component rendering tests require `@testing-library/svelte` (not in scope for this plan). Verify in dev mode (`bun run tauri dev`).

### Rationale

Both components in this task (`PodiumView.svelte`, `PodiumPosition.svelte`) are pure Svelte UI — conditional rendering, derived sorting, CSS styling. The `top3` derived computation is a one-liner `[...scores].sort(...).slice(0, 3)` tightly coupled to `$derived.by`. No extractable pure logic functions. The automated gate is `bun run check` (type safety).
