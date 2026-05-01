# Task 12 - Scouting Page Integration

## Overview

Wire all components together into the main scouting page. Replace the existing placeholder `+page.svelte` with the full scouting view that includes pitch, archetype selector, podium, results table, and the state management to connect them.

## Files to Create/Modify

- Modify: `src/routes/+page.svelte` — Replace Tauri template with scouting UI
- Create: `src/lib/stores/scouting-store.svelte.ts` — Scoring state management

## Context

### Page Layout

```
┌─────────────────────────────────────┐
│  Scouting                           │
│  [Season Selector]                  │
├─────────────────────────────────────┤
│                                     │
│         Pitch View                  │
│     (with position slots)           │
│                                     │
├─────────────────────────────────────┤
│         Podium (Top 3)              │
├─────────────────────────────────────┤
│         Results Table               │
│  (sortable, filterable, scrollable) │
└─────────────────────────────────────┘
```

### Data Flow

1. On mount: load archetypes from DB via `archetypeStore.loadAll()`
2. On mount: load players for the current season (use the latest season from the active save)
3. When archetype selected on a pitch slot:
   - Build percentile cache from loaded players
   - Score all players against the selected archetype
   - Update podium + table
4. When no archetype selected: show full database view (all players, no scores)

### State Management

The scouting store holds:
- Loaded players (`ScorablePlayer[]`)
- Current season info
- Computed scores per selected position
- Loading/error state

## Steps

- [ ] **Step 1: Create the scouting store**

Create `src/lib/stores/scouting-store.svelte.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";
import { scoreAllPlayers } from "$lib/scoring";
import type { PlayerScore, ScorablePlayer } from "$lib/scoring/types";
import type { Archetype } from "$lib/types/archetype";

/** Player data from the database (matches Rust PlayerSeasonData). */
interface PlayerSeasonData {
    id: number;
    player_id: number;
    season_id: number;
    fm_uid: number;
    player_name: string;
    club: string | null;
    age: number | null;
    nationality: string | null;
    position: string;
    minutes: number | null;
    transfer_value_high: number | null;
    data: Record<string, unknown> | null;
}

/** State */
let players = $state<ScorablePlayer[]>([]);
let scores = $state<PlayerScore[]>([]);
let activeArchetype = $state<Archetype | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let seasonId = $state<number | null>(null);

/** Convert PlayerSeasonData to ScorablePlayer. */
function toScorable(psd: PlayerSeasonData): ScorablePlayer {
    return {
        playerId: psd.id,
        fmUid: psd.fm_uid,
        name: psd.player_name,
        club: psd.club,
        positions: psd.position,
        age: psd.age,
        transferValueHigh: psd.transfer_value_high,
        data: psd.data,
    };
}

/** Load players for a season. */
async function loadPlayers(seasonIdParam: number) {
    loading = true;
    error = null;
    try {
        const data = await invoke<PlayerSeasonData[]>("get_players_for_season", {
            seasonId: seasonIdParam,
        });
        players = data.map(toScorable);
        seasonId = seasonIdParam;

        // Re-score if archetype is selected
        if (activeArchetype) {
            scores = scoreAllPlayers(players, activeArchetype);
        }
    } catch (e) {
        error = e instanceof Error ? e.message : String(e);
    } finally {
        loading = false;
    }
}

/** Select an archetype and compute scores. */
function selectArchetype(archetype: Archetype | null) {
    activeArchetype = archetype;
    if (archetype && players.length > 0) {
        scores = scoreAllPlayers(players, archetype);
    } else {
        scores = [];
    }
}

/** Clear error. */
function clearError() {
    error = null;
}

export function getScoutingStore() {
    return {
        get players() { return players; },
        get scores() { return scores; },
        get activeArchetype() { return activeArchetype; },
        get loading() { return loading; },
        get error() { return error; },
        get seasonId() { return seasonId; },

        loadPlayers,
        selectArchetype,
        clearError,
    };
}
```

- [ ] **Step 2: Replace +page.svelte**

Replace `src/routes/+page.svelte` entirely:

```svelte
<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";

    import PitchView from "$lib/components/pitch/PitchView.svelte";
    import ArchetypeSelector from "$lib/components/archetype/ArchetypeSelector.svelte";
    import ArchetypeEditor from "$lib/components/archetype/ArchetypeEditor.svelte";
    import PodiumView from "$lib/components/scouting/PodiumView.svelte";
    import ResultsTable from "$lib/components/scouting/ResultsTable.svelte";

    import { getArchetypeStore } from "$lib/stores/archetype-store.svelte";
    import { getScoutingStore } from "$lib/stores/scouting-store.svelte";
    import { PITCH_POSITIONS } from "$lib/components/pitch/pitch-positions";

    const archetypeStore = getArchetypeStore();
    const scoutingStore = getScoutingStore();

    // Selector/editor state
    let selectorOpen = $state(false);
    let editorOpen = $state(false);
    let activeSlotId = $state<string | null>(null);
    let editingArchetype = $state<Archetype | null>(null);
    let activeRole = $state<ArchetypeRole>("GK");

    import type { Archetype, ArchetypeRole, MetricWeight } from "$lib/types/archetype";

    // Available archetypes for the active role
    let roleArchetypes = $derived.by(() => {
        const role = activeRole;
        return archetypeStore.archetypes.filter(a => a.role === role);
    });

    onMount(async () => {
        await archetypeStore.loadAll();

        // Try to load the latest season's players
        try {
            const saves = await invoke<{ id: number }[]>("list_saves");
            if (saves.length > 0) {
                const latestSeason = await invoke<{ id: number } | null>("get_latest_season", {
                    saveId: saves[0].id,
                });
                if (latestSeason) {
                    await scoutingStore.loadPlayers(latestSeason.id);
                }
            }
        } catch {
            // No saves or seasons yet — empty state
        }
    });

    function handleSlotClick(slotId: string) {
        const pos = PITCH_POSITIONS.find(p => p.slotId === slotId);
        if (!pos) return;

        activeSlotId = slotId;
        activeRole = pos.role;
        selectorOpen = true;
    }

    function handleArchetypeSelect(archetype: Archetype | null) {
        if (activeSlotId) {
            archetypeStore.selectArchetype(activeSlotId, archetype);
        }
        // Set the active archetype for scoring
        scoutingStore.selectArchetype(archetype);
        selectorOpen = false;
    }

    function handleCreateArchetype() {
        editingArchetype = null;
        editorOpen = true;
    }

    function handleEditArchetype(archetype: Archetype) {
        editingArchetype = archetype;
        editorOpen = true;
    }

    async function handleDeleteArchetype(archetype: Archetype) {
        const success = await archetypeStore.remove(archetype.id);
        if (!success) {
            // Error is set in the store — could show a toast
        }
    }

    async function handleSaveArchetype(name: string, metrics: MetricWeight[]) {
        let saved: Archetype | null;
        if (editingArchetype) {
            saved = await archetypeStore.update(editingArchetype.id, name, metrics);
        } else {
            saved = await archetypeStore.create(name, activeRole, metrics);
        }
        if (saved && activeSlotId) {
            archetypeStore.selectArchetype(activeSlotId, saved);
            scoutingStore.selectArchetype(saved);
        }
        editorOpen = false;
    }

    function handleRowClick(score: PlayerScore) {
        // Navigate to player profile — for now, just log
        console.log("Navigate to player profile:", score.playerId, score.name);
        // Future: goto(`/player/${score.playerId}`);
    }

    import type { PlayerScore } from "$lib/scoring/types";

    // Empty state
    let hasPlayers = $derived(scoutingStore.players.length > 0);
</script>

<main class="scouting-page">
    <header class="scouting-header">
        <h1>Moneyball Scouting</h1>
        {#if scoutingStore.error}
            <div class="error-banner">
                {scoutingStore.error}
                <button onclick={scoutingStore.clearError}>✕</button>
            </div>
        {/if}
        {#if archetypeStore.error}
            <div class="error-banner">
                {archetypeStore.error}
                <button onclick={archetypeStore.clearError}>✕</button>
            </div>
        {/if}
    </header>

    {#if !hasPlayers}
        <div class="empty-state">
            <p>No player data loaded.</p>
            <p>Import a CSV to start scouting.</p>
        </div>
    {:else}
        <section class="pitch-section">
            <PitchView
                selectedArchetypes={archetypeStore.selectedArchetypes}
                onslotclick={handleSlotClick}
            />
        </section>

        {#if scoutingStore.scores.length > 0}
            <section class="podium-section">
                <PodiumView scores={scoutingStore.scores} />
            </section>

            <section class="table-section">
                <ResultsTable
                    scores={scoutingStore.scores}
                    archetype={scoutingStore.activeArchetype}
                    onrowclick={handleRowClick}
                />
            </section>
        {:else}
            <div class="full-db-section">
                <h2 class="section-title">All Players</h2>
                <p class="section-hint">Select a position on the pitch to see archetype scoring.</p>
                <ResultsTable
                    scores={scoutingStore.players.map(p => ({
                        playerId: p.playerId,
                        fmUid: p.fmUid,
                        name: p.name,
                        club: p.club,
                        positions: p.positions,
                        age: p.age,
                        transferValue: p.transferValueHigh,
                        role: "",
                        rawScore: 0,
                        valueAdjustedScore: 0,
                        metricPercentiles: {},
                    }))}
                    archetype={null}
                    onrowclick={handleRowClick}
                />
            </div>
    {/if}

    <!-- Overlays -->
    {#if selectorOpen}
        <ArchetypeSelector
            {activeRole}
            archetypes={roleArchetypes}
            selectedArchetypeId={activeSlotId ? archetypeStore.getSelectedForSlot(activeSlotId)?.id ?? null : null}
            onselect={handleArchetypeSelect}
            onedit={handleEditArchetype}
            oncreate={handleCreateArchetype}
            ondelete={handleDeleteArchetype}
            onclose={() => selectorOpen = false}
        />
    {/if}

    {#if editorOpen}
        <ArchetypeEditor
            {activeRole}
            archetype={editingArchetype}
            onsave={handleSaveArchetype}
            onclose={() => editorOpen = false}
        />
    {/if}
</main>

<style>
    .scouting-page {
        max-width: 800px;
        margin: 0 auto;
        padding: 16px;
        font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    }

    .scouting-header h1 {
        text-align: center;
        color: #fff;
        font-size: 1.5rem;
        margin: 0 0 16px 0;
    }

    .error-banner {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 12px;
        background: #b71c1c;
        color: #fff;
        border-radius: 6px;
        font-size: 0.85rem;
        margin-bottom: 12px;
    }

    .error-banner button {
        background: none;
        border: none;
        color: #fff;
        cursor: pointer;
    }

    .empty-state {
        text-align: center;
        padding: 40px 16px;
        color: #888;
    }

    .empty-state p:first-child {
        font-size: 1.1rem;
        color: #aaa;
        margin-bottom: 8px;
    }

    .full-db-section {
        padding: 16px 0;
    }

    .section-title {
        text-align: center;
        color: #fff;
        font-size: 1.1rem;
        margin: 0 0 4px 0;
    }

    .section-hint {
        text-align: center;
        color: #666;
        font-size: 0.85rem;
        margin: 0 0 16px 0;
    }

    .pitch-section {
        margin-bottom: 24px;
    }

    .podium-section {
        margin-bottom: 16px;
    }

    .table-section {
        margin-bottom: 32px;
    }

    @media (prefers-color-scheme: dark) {
        :root {
            color: #f6f6f6;
            background-color: #1a1a1a;
        }
    }
</style>
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `bun run check`
Expected: SUCCESS — all components and stores resolve correctly.

- [ ] **Step 4: Verify Rust compilation**

Run: `cd src-tauri && cargo check`
Expected: SUCCESS.

- [ ] **Step 5: Run full test suite**

Run: `cd src-tauri && cargo test --lib`
Expected: ALL PASS.

Run: `bun run check`
Expected: SUCCESS.

## Dependencies

- Task 05 (Tauri commands) — `list_saves`, `get_latest_season`, `get_players_for_season`
- Task 06 (frontend archetype store) — `getArchetypeStore()`
- Task 07 (scoring engine) — `scoreAllPlayers`
- Task 08 (pitch view) — `PitchView` component
- Task 09 (archetype selector) — `ArchetypeSelector`, `ArchetypeEditor`
- Task 10 (podium view) — `PodiumView`
- Task 11 (results table) — `ResultsTable`

## Success Criteria

- Page loads and shows empty state when no players are loaded
- When players exist: pitch renders, user can click slots to select archetypes
- Selecting an archetype triggers scoring and updates podium + table
- Full end-to-end flow: load data → click slot → select archetype → see results
- All TypeScript compiles without errors
- All Rust tests pass
- Dark theme consistent throughout

## Tests

### Test 1: TypeScript compilation

**What to test:** All components and stores compile.
**Feasibility:** ✅ Can be tested — `bun run check`.

### Test 2: Rust compilation

**What to test:** All Tauri commands compile and register.
**Feasibility:** ✅ Can be tested — `cargo check`.

### Test 3: End-to-end smoke test

**What to test:** Launch app, verify scouting page renders.
**Feasibility:** ⚠️ Dependent on running the Tauri app — `bun run tauri dev`.
