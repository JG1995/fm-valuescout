# Task 11 - Results Table Component

## Overview

Create the virtualized, sortable results table that shows all scored players for the selected archetype. Supports column hiding, sorting by any column, and row click navigation to player profiles.

## Files to Create/Modify

- Create: `src/lib/components/scouting/ResultsTable.svelte` — Main table component

## Context

### Table Columns

From the design spec:
- Name
- Club
- All positions (string)
- Age
- Transfer Value
- Raw Score
- Value-Adjusted Score
- Key metrics for the selected archetype (dynamic based on archetype metrics)

Columns can be hidden (not removed) via a column visibility toggle.

### Key Requirements

- **Sortable**: Click column header to sort ascending/descending
- **Virtualized scroll**: Handle large player lists efficiently
- **Row click**: Navigate to Player Profile (for MVP, this can just log/select — Player Profile page is separate)
- **Hidden columns**: Toggle visibility, not remove

### Virtualization Note

For MVP, virtualized scroll can be achieved with CSS `overflow-y: auto` and a max-height. True virtualization (rendering only visible rows) can be deferred if performance is acceptable with 500-1000 players. The design spec says "virtualized scroll" but this can start as a simple scrollable table and be optimized later if needed.

## Steps

- [ ] **Step 1: Create ResultsTable component**

Create `src/lib/components/scouting/ResultsTable.svelte`:

```svelte
<script lang="ts">
    import type { PlayerScore } from "$lib/scoring/types";
    import type { Archetype } from "$lib/types/archetype";

    interface Props {
        scores: PlayerScore[];
        archetype: Archetype | null;
        onrowclick: (score: PlayerScore) => void;
    }

    let { scores, archetype, onrowclick }: Props = $props();

    // Sorting state
    let sortKey = $state<string>("valueAdjustedScore");
    let sortDir = $state<"asc" | "desc">("desc");

    // Column visibility
    let hiddenColumns = $state<Set<string>>(new Set());

    // Sorted scores
    let sorted = $derived.by(() => {
        return [...scores].sort((a, b) => {
            const aVal = getSortValue(a, sortKey);
            const bVal = getSortValue(b, sortKey);
            const cmp = aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
            return sortDir === "asc" ? cmp : -cmp;
        });
    });

    // Dynamic metric columns from archetype
    let metricColumns = $derived.by(() => {
        if (!archetype) return [];
        return archetype.metrics.map(m => ({
            key: `metric.${m.metric_key}`,
            label: formatMetricLabel(m.metric_key),
        }));
    });

    function getSortValue(score: PlayerScore, key: string): number {
        if (key.startsWith("metric.")) {
            const metricKey = key.slice(7);
            return score.metricPercentiles[metricKey] ?? 0;
        }
        switch (key) {
            case "name": return score.name.charCodeAt(0);
            case "rawScore": return score.rawScore;
            case "valueAdjustedScore": return score.valueAdjustedScore;
            case "age": return score.age ?? 0;
            case "transferValue": return score.transferValue ?? 0;
            default: return 0;
        }
    }

    function formatMetricLabel(key: string): string {
        // Take the last part after the dot, convert snake_case to Title Case
        const parts = key.split(".");
        const field = parts[parts.length - 1];
        return field
            .split("_")
            .map(w => w.charAt(0).toUpperCase() + w.slice(1))
            .join(" ");
    }

    function toggleSort(key: string) {
        if (sortKey === key) {
            sortDir = sortDir === "asc" ? "desc" : "asc";
        } else {
            sortKey = key;
            sortDir = "desc";
        }
    }

    function toggleColumn(key: string) {
        const next = new Set(hiddenColumns);
        if (next.has(key)) {
            next.delete(key);
        } else {
            next.add(key);
        }
        hiddenColumns = next;
    }

    function formatValue(value: number | null): string {
        if (value === null) return "—";
        if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
        if (value >= 1_000) return `${(value / 1_000).toFixed(0)}K`;
        return value.toFixed(0);
    }
</script>

<div class="results-table-container">
    <!-- Column visibility toggle -->
    <div class="column-toggle">
        <button class="toggle-btn" onclick={() => {
            const panel = document.getElementById("column-panel");
            if (panel) panel.classList.toggle("visible");
        }}>Columns ▾</button>
        <div id="column-panel" class="column-panel">
            {#each metricColumns as col (col.key)}
                <label class="column-checkbox">
                    <input
                        type="checkbox"
                        checked={!hiddenColumns.has(col.key)}
                        onchange={() => toggleColumn(col.key)}
                    />
                    <span>{col.label}</span>
                </label>
            {/each}
        </div>
    </div>

    <!-- Table -->
    <div class="table-scroll">
        <table>
            <thead>
                <tr>
                    <th class="sortable" onclick={() => toggleSort("name")}>
                        Name {sortKey === "name" ? (sortDir === "asc" ? "↑" : "↓") : ""}
                    </th>
                    <th>Club</th>
                    <th>Positions</th>
                    <th class="sortable" onclick={() => toggleSort("age")}>
                        Age {sortKey === "age" ? (sortDir === "asc" ? "↑" : "↓") : ""}
                    </th>
                    <th class="sortable" onclick={() => toggleSort("transferValue")}>
                        Value {sortKey === "transferValue" ? (sortDir === "asc" ? "↑" : "↓") : ""}
                    </th>
                    <th class="sortable" onclick={() => toggleSort("rawScore")}>
                        Score {sortKey === "rawScore" ? (sortDir === "asc" ? "↑" : "↓") : ""}
                    </th>
                    <th class="sortable" onclick={() => toggleSort("valueAdjustedScore")}>
                        Value Adj. {sortKey === "valueAdjustedScore" ? (sortDir === "asc" ? "↑" : "↓") : ""}
                    </th>
                    {#each metricColumns as col (col.key)}
                        {#if !hiddenColumns.has(col.key)}
                            <th class="sortable" onclick={() => toggleSort(col.key)}>
                                {col.label} {sortKey === col.key ? (sortDir === "asc" ? "↑" : "↓") : ""}
                            </th>
                        {/if}
                    {/each}
                </tr>
            </thead>
            <tbody>
                {#each sorted as score (score.playerId)}
                    <tr onclick={() => onrowclick(score)}>
                        <td class="name-cell">{score.name}</td>
                        <td>{score.club ?? "—"}</td>
                        <td class="pos-cell">{score.positions}</td>
                        <td>{score.age ?? "—"}</td>
                        <td class="num">{formatValue(score.transferValue)}</td>
                        <td class="num">{score.rawScore.toFixed(1)}</td>
                        <td class="num highlight">{score.valueAdjustedScore.toFixed(1)}</td>
                        {#each metricColumns as col (col.key)}
                            {#if !hiddenColumns.has(col.key)}
                                <td class="num">
                                    {(score.metricPercentiles[col.key.slice(7)] ?? 0).toFixed(0)}
                                </td>
                            {/if}
                        {/each}
                    </tr>
                {/each}
            </tbody>
        </table>
    </div>
</div>

<style>
    .results-table-container {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .column-toggle {
        position: relative;
        align-self: flex-end;
    }

    .toggle-btn {
        padding: 4px 12px;
        background: #2a2a2a;
        border: 1px solid #3a3a3a;
        border-radius: 4px;
        color: #aaa;
        cursor: pointer;
        font-size: 0.8rem;
    }

    .toggle-btn:hover {
        background: #333;
    }

    .column-panel {
        display: none;
        position: absolute;
        right: 0;
        top: 100%;
        background: #1e1e1e;
        border: 1px solid #333;
        border-radius: 6px;
        padding: 8px;
        z-index: 50;
        min-width: 200px;
    }

    .column-panel.visible {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .column-checkbox {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 0.8rem;
        color: #ccc;
        cursor: pointer;
    }

    .table-scroll {
        max-height: 500px;
        overflow-y: auto;
        border-radius: 8px;
        border: 1px solid #333;
    }

    table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.85rem;
    }

    thead {
        position: sticky;
        top: 0;
        z-index: 10;
    }

    th {
        background: #1a1a1a;
        color: #aaa;
        font-weight: 600;
        text-align: left;
        padding: 10px 12px;
        border-bottom: 2px solid #333;
        white-space: nowrap;
        font-size: 0.75rem;
        text-transform: uppercase;
    }

    th.sortable {
        cursor: pointer;
        user-select: none;
    }

    th.sortable:hover {
        color: #fff;
    }

    td {
        padding: 8px 12px;
        border-bottom: 1px solid #222;
        color: #ddd;
    }

    tr {
        cursor: pointer;
        transition: background 0.1s;
    }

    tbody tr:hover {
        background: #2a2a2a;
    }

    .name-cell {
        font-weight: 600;
    }

    .pos-cell {
        font-size: 0.75rem;
        color: #888;
    }

    .num {
        text-align: right;
        font-family: monospace;
        font-size: 0.8rem;
    }

    .highlight {
        color: #4caf50;
        font-weight: 600;
    }
</style>
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `bun run check`
Expected: SUCCESS.

## Dependencies

- Task 07 (scoring engine) — `PlayerScore` type
- Task 06 (frontend types) — `Archetype` type

## Success Criteria

- Table renders with all columns: name, club, positions, age, value, raw score, value-adjusted score, archetype metric percentiles
- Clicking a column header sorts the table (toggles asc/desc)
- Metric columns are dynamic based on selected archetype
- Columns can be hidden via the column toggle panel
- Row click fires `onrowclick` callback with the `PlayerScore`
- Table has scrollable body with sticky header
- Dark theme styling consistent with the app
- `bun run check` passes

## Tests

### Test 1: TypeScript compilation

**What to test:** Component compiles.
**Feasibility:** ✅ Can be tested — `bun run check`.

### Test 2: Visual verification

**What to test:** Table renders, sorting works, column toggle works.
**Feasibility:** ⚠️ Dependent on running the app — verify in dev mode.
