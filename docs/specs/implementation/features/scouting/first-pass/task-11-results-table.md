# Task 11 - Results Table Component

## Overview

Create the virtualized, sortable results table that shows all scored players for the selected archetype. Supports column hiding, sorting by any column, and row click navigation to player profiles.

Uses TDD: extract pure logic into testable helpers, write failing tests first, then implement.

## Files to Create/Modify

- Create: `src/lib/components/scouting/table-helpers.test.ts` — Unit tests for table helper functions
- Create: `src/lib/components/scouting/table-helpers.ts` — Extracted pure helper functions
- Create: `src/lib/components/scouting/ResultsTable.svelte` — Main table component (imports helpers)

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

### Pure Logic to Extract

The component contains pure functions that are easily testable:
- `getSortValue(score, key)` — returns numeric sort value for any sort key
- `formatMetricLabel(key)` — converts metric keys to human-readable labels
- `formatValue(value)` — formats numbers for display (M/K suffixes)

## Steps

- [ ] **Step 1: Write failing tests for table helpers**

Create `src/lib/components/scouting/table-helpers.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { getSortValue, formatMetricLabel, formatValue } from "./table-helpers";
import type { PlayerScore } from "$lib/scoring/types";

function makeScore(overrides: Partial<PlayerScore> = {}): PlayerScore {
    return {
        playerId: 1,
        fmUid: 12345,
        name: "Test Player",
        club: "Test FC",
        positions: "ST",
        age: 25,
        transferValue: 10_000_000,
        role: "ST",
        rawScore: 85.5,
        valueAdjustedScore: 92.0,
        metricPercentiles: {
            "attacking.goals_per_90": 75,
            "attacking.assists": 60,
            "defending.tackles": 30,
        },
        ...overrides,
    };
}

describe("getSortValue", () => {
    it("returns rawScore for 'rawScore' key", () => {
        const score = makeScore({ rawScore: 85.5 });
        expect(getSortValue(score, "rawScore")).toBe(85.5);
    });

    it("returns valueAdjustedScore for 'valueAdjustedScore' key", () => {
        const score = makeScore({ valueAdjustedScore: 92.0 });
        expect(getSortValue(score, "valueAdjustedScore")).toBe(92.0);
    });

    it("returns age for 'age' key", () => {
        const score = makeScore({ age: 28 });
        expect(getSortValue(score, "age")).toBe(28);
    });

    it("returns transferValue for 'transferValue' key", () => {
        const score = makeScore({ transferValue: 15_000_000 });
        expect(getSortValue(score, "transferValue")).toBe(15_000_000);
    });

    it("returns null as 0 for 'age'", () => {
        const score = makeScore({ age: null });
        expect(getSortValue(score, "age")).toBe(0);
    });

    it("returns null as 0 for 'transferValue'", () => {
        const score = makeScore({ transferValue: null });
        expect(getSortValue(score, "transferValue")).toBe(0);
    });

    it("returns metric percentile for 'metric.*' keys", () => {
        const score = makeScore();
        expect(getSortValue(score, "metric.attacking.goals_per_90")).toBe(75);
    });

    it("returns 0 for unknown metric keys", () => {
        const score = makeScore();
        expect(getSortValue(score, "metric.unknown_metric")).toBe(0);
    });

    it("returns 0 for unknown sort keys", () => {
        const score = makeScore();
        expect(getSortValue(score, "unknownField")).toBe(0);
    });
});

describe("formatMetricLabel", () => {
    it("converts snake_case dot-paths to Title Case", () => {
        expect(formatMetricLabel("attacking.goals_per_90")).toBe("Goals Per 90");
    });

    it("handles keys without dots", () => {
        expect(formatMetricLabel("minutes_played")).toBe("Minutes Played");
    });

    it("uses only the last segment after the dot", () => {
        expect(formatMetricLabel("attacking.crosses")).toBe("Crosses");
    });

    it("handles multiple underscores in a segment", () => {
        expect(formatMetricLabel("a.b_c_d")).toBe("C D");
    });
});

describe("formatValue", () => {
    it("formats millions as 'X.XM'", () => {
        expect(formatValue(10_000_000)).toBe("10.0M");
        expect(formatValue(15_500_000)).toBe("15.5M");
        expect(formatValue(1_000_000)).toBe("1.0M");
    });

    it("formats thousands as 'XK'", () => {
        expect(formatValue(50_000)).toBe("50K");
        expect(formatValue(999_999)).toBe("999K");
    });

    it("returns '—' for null", () => {
        expect(formatValue(null)).toBe("—");
    });

    it("returns plain number for small values", () => {
        expect(formatValue(500)).toBe("500");
        expect(formatValue(999)).toBe("999");
    });
});
```

Run: `npx vitest run`
Expected: FAIL — file does not exist yet.

- [ ] **Step 2: Create table helper functions**

Create `src/lib/components/scouting/table-helpers.ts`:

```typescript
import type { PlayerScore } from "$lib/scoring/types";

/**
 * Returns the numeric sort value for a given sort key on a PlayerScore.
 * Used by the ResultsTable sortable columns.
 */
export function getSortValue(score: PlayerScore, key: string): number {
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

/**
 * Converts a metric key (e.g., "attacking.goals_per_90") to a human-readable
 * label (e.g., "Goals Per 90").
 * Takes the last segment after the dot and converts snake_case to Title Case.
 */
export function formatMetricLabel(key: string): string {
    const parts = key.split(".");
    const field = parts[parts.length - 1];
    return field
        .split("_")
        .map(w => w.charAt(0).toUpperCase() + w.slice(1))
        .join(" ");
}

/**
 * Formats a numeric value for display in the results table.
 * - null → "—"
 * - >= 1,000,000 → "X.XM"
 * - >= 1,000 → "XK"
 * - otherwise → plain integer
 */
export function formatValue(value: number | null): string {
    if (value === null) return "—";
    if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
    if (value >= 1_000) return `${(value / 1_000).toFixed(0)}K`;
    return value.toFixed(0);
}
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `npx vitest run`
Expected: ALL PASS.

- [ ] **Step 4: Create ResultsTable component**

Create `src/lib/components/scouting/ResultsTable.svelte`:

```svelte
<script lang="ts">
    import type { PlayerScore } from "$lib/scoring/types";
    import type { Archetype } from "$lib/types/archetype";
    import { getSortValue, formatMetricLabel, formatValue } from "./table-helpers";

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

- [ ] **Step 5: Verify TypeScript compilation**

Run: `bun run check`
Expected: SUCCESS.

## Dependencies

- Task 06 (vitest setup) — `table-helpers.test.ts`
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
- All helper function tests pass

## Tests

### Test 1: Table helper functions

**What to test:** Pure functions: `getSortValue`, `formatMetricLabel`, `formatValue` produce correct outputs for all input cases.
**Command:** `npx vitest run src/lib/components/scouting/table-helpers.test.ts`
**Feasibility:** ✅ Unit tests — pure TypeScript functions.

### Test 2: TypeScript compilation

**What to test:** All components compile without errors.
**Command:** `bun run check`
**Feasibility:** ✅ Can be tested — `bun run check`.

### Test 3: Visual verification

**What to test:** Table renders, sorting works, column toggle works.
**Feasibility:** ⚠️ Dependent on running the app — verify in dev mode (`bun run tauri dev`).
