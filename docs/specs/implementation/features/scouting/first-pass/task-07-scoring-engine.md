# Task 07 - Scoring Engine (TypeScript)

## Overview

Implement the player scoring algorithm in TypeScript. This is a pure computation module that scores players against archetypes using weighted percentiles. All scoring happens client-side per the design spec.

## Files to Create/Modify

- Create: `src/lib/scoring/percentiles.ts` — Percentile computation
- Create: `src/lib/scoring/score.ts` — Archetype scoring and value-adjusted scoring
- Create: `src/lib/scoring/types.ts` — Scoring-specific types
- Create: `src/lib/scoring/metric-accessor.ts` — Extract metric values from ParsedPlayer data
- Create: `src/lib/scoring/index.ts` — Public API barrel export

## Context

### Scoring Algorithm (from design spec)

1. For each metric in archetype: compute percentile within loaded dataset (0-100)
2. For INVERTED metrics: use `(100 - percentile)`
3. Weighted sum: `score = sum(percentile * weight)` for all metrics → result is 0-100
4. Value-adjusted: `score / (transfer_value / median_value)` — if player has no transfer value, use median

### Player Data Shape

Players are `PlayerSeasonData` from the Rust backend, with `data: ParsedPlayer | null`. The `ParsedPlayer` is the JSON blob with nested stat structs:

```typescript
interface ParsedPlayer {
    uid: number;
    name: string;
    positions: Position[];  // { role: Role, sides: Side[] }
    transfer_value: { low?: number; high?: number; currency_symbol?: string; raw?: string };
    attacking: { goals_per_90?: number; xg_per_90?: number; ... };
    chance_creation: { assists_per_90?: number; pass_completion_rate?: number; ... };
    movement: { dribbles_per_90?: number; distance_per_90?: number; ... };
    defending: { tackles_per_90?: number; interceptions_per_90?: number; ... };
    aerial: { aerial_challenge_rate?: number; ... };
    goalkeeping: { saves_per_90?: number; ... };
    discipline: { fouls_made_per_90?: number; ... };
    match_outcome: { average_rating?: number; ... };
}
```

Metric keys are dot-separated: `"attacking.goals_per_90"` → access `parsedPlayer.attacking.goals_per_90`.

### Percentile Calculation

Use the standard "percentile rank" method: `percentile = (count_below + 0.5 * count_equal) / total * 100`. This is the same as Excel's `PERCENTRANK.INC`.

For a player with value `v` in a dataset of `n` values:
- Sort all values
- Find rank position
- `percentile = rank / (n - 1) * 100` (using linear interpolation)

When a player has `null`/`undefined` for a metric, they get percentile 0 (worst case).

## Steps

- [ ] **Step 1: Create scoring types**

Create directory `src/lib/scoring/` and file `src/lib/scoring/types.ts`:

```typescript
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
```

- [ ] **Step 2: Write the failing tests for metric accessor**

Create `src/lib/scoring/metric-accessor.test.ts`:

```typescript
import { describe, it, expect } from "vitest"; // Or the project's test runner
import { getMetricValue } from "./metric-accessor";

describe("getMetricValue", () => {
    const player = {
        attacking: { goals_per_90: 0.5, xg_per_90: 0.3 },
        chance_creation: { assists_per_90: 0.2, pass_completion_rate: 0.85 },
        defending: { tackles_per_90: 2.1 },
        movement: { dribbles_per_90: 1.5 },
    };

    it("extracts top-level metric", () => {
        expect(getMetricValue(player, "attacking.goals_per_90")).toBe(0.5);
    });

    it("extracts nested metric", () => {
        expect(getMetricValue(player, "chance_creation.pass_completion_rate")).toBe(0.85);
    });

    it("returns null for missing category", () => {
        expect(getMetricValue(player, "goalkeeping.saves_per_90")).toBeNull();
    });

    it("returns null for missing field in category", () => {
        expect(getMetricValue(player, "attacking.shots_per_90")).toBeNull();
    });

    it("returns null for null data", () => {
        expect(getMetricValue(null, "attacking.goals_per_90")).toBeNull();
    });

    it("returns null for malformed key", () => {
        expect(getMetricValue(player, "invalid")).toBeNull();
    });

    it("returns null for deeply nested key", () => {
        expect(getMetricValue(player, "a.b.c")).toBeNull();
    });
});
```

- [ ] **Step 3: Implement metric accessor**

Create `src/lib/scoring/metric-accessor.ts`:

```typescript
/**
 * Extract a metric value from a ParsedPlayer data object using a dot-separated key.
 * Key format: "category.field_name" (e.g., "attacking.goals_per_90").
 * Returns null if the path doesn't exist or the value is null/undefined.
 */
export function getMetricValue(
    data: Record<string, unknown> | null,
    key: string,
): number | null {
    if (!data) return null;

    const parts = key.split(".");
    if (parts.length !== 2) return null;

    const [category, field] = parts;
    const categoryObj = data[category];
    if (!categoryObj || typeof categoryObj !== "object") return null;

    const value = (categoryObj as Record<string, unknown>)[field];
    if (value === null || value === undefined) return null;
    if (typeof value !== "number") return null;

    return value;
}
```

- [ ] **Step 4: Write the failing tests for percentile computation**

Append to `src/lib/scoring/metric-accessor.test.ts` or create a new test file `src/lib/scoring/percentiles.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { computePercentile, buildPercentileCache } from "./percentiles";
import type { ScorablePlayer } from "./types";

describe("computePercentile", () => {
    it("returns 0 for empty values", () => {
        expect(computePercentile(5, [])).toBe(0);
    });

    it("returns 100 for highest value", () => {
        expect(computePercentile(10, [1, 5, 10])).toBe(100);
    });

    it("returns 0 for lowest value", () => {
        expect(computePercentile(1, [1, 5, 10])).toBe(0);
    });

    it("returns 50 for median in 3-element set", () => {
        expect(computePercentile(5, [1, 5, 10])).toBe(50);
    });

    it("handles duplicate values", () => {
        // All same value → percentile should be 100 (or 50 for mid-rank)
        const result = computePercentile(5, [5, 5, 5]);
        expect(result).toBeGreaterThan(0);
        expect(result).toBeLessThanOrEqual(100);
    });

    it("returns 0 for value not in list", () => {
        expect(computePercentile(0, [1, 5, 10])).toBe(0);
    });
});

describe("buildPercentileCache", () => {
    const players: ScorablePlayer[] = [
        {
            playerId: 1, fmUid: 100, name: "A", club: null,
            positions: "ST", age: 25, transferValueHigh: 1000000,
            data: { attacking: { goals_per_90: 0.3 } },
        },
        {
            playerId: 2, fmUid: 200, name: "B", club: null,
            positions: "ST", age: 27, transferValueHigh: 2000000,
            data: { attacking: { goals_per_90: 0.6 } },
        },
        {
            playerId: 3, fmUid: 300, name: "C", club: null,
            positions: "ST", age: 22, transferValueHigh: null,
            data: null, // Missing data
        },
    ];

    it("builds cache for specified metrics", () => {
        const cache = buildPercentileCache(players, ["attacking.goals_per_90"]);
        const values = cache.metricValues.get("attacking.goals_per_90")!;
        expect(values).toHaveLength(2); // Only 2 non-null values
        expect(values).toEqual([0.3, 0.6]);
    });

    it("handles all-null metric", () => {
        const cache = buildPercentileCache(players, ["chance_creation.assists_per_90"]);
        const values = cache.metricValues.get("chance_creation.assists_per_90")!;
        expect(values).toHaveLength(0);
    });
});
```

- [ ] **Step 5: Implement percentile computation**

Create `src/lib/scoring/percentiles.ts`:

```typescript
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
```

- [ ] **Step 6: Write the failing tests for scoring**

Create `src/lib/scoring/score.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { scorePlayer, computeMedianTransferValue, scoreAllPlayers } from "./score";
import type { ScorablePlayer } from "./types";
import type { Archetype } from "$lib/types/archetype";

describe("computeMedianTransferValue", () => {
    it("computes median of non-null values", () => {
        const players: ScorablePlayer[] = [
            { playerId: 1, fmUid: 1, name: "A", club: null, positions: "ST", age: null, transferValueHigh: 100, data: {} },
            { playerId: 2, fmUid: 2, name: "B", club: null, positions: "ST", age: null, transferValueHigh: 200, data: {} },
            { playerId: 3, fmUid: 3, name: "C", club: null, positions: "ST", age: null, transferValueHigh: 300, data: {} },
        ];
        expect(computeMedianTransferValue(players)).toBe(200);
    });

    it("returns 1 when all values are null", () => {
        const players: ScorablePlayer[] = [
            { playerId: 1, fmUid: 1, name: "A", club: null, positions: "ST", age: null, transferValueHigh: null, data: {} },
        ];
        expect(computeMedianTransferValue(players)).toBe(1);
    });
});

describe("scorePlayer", () => {
    const archetype: Archetype = {
        id: 1,
        name: "Test Striker",
        role: "ST",
        metrics: [
            { metric_key: "attacking.goals_per_90", weight: 0.6, inverted: false },
            { metric_key: "chance_creation.assists_per_90", weight: 0.4, inverted: false },
        ],
        is_default: true,
        created_at: "",
        updated_at: "",
    };

    const allValues = {
        "attacking.goals_per_90": [0.1, 0.2, 0.3, 0.5, 0.8],
        "chance_creation.assists_per_90": [0.1, 0.2, 0.3, 0.4, 0.5],
    };

    it("scores a player with perfect metrics at 100", () => {
        const player: ScorablePlayer = {
            playerId: 1, fmUid: 1, name: "Best", club: "Club A",
            positions: "ST", age: 25, transferValueHigh: 1000000,
            data: { attacking: { goals_per_90: 0.8 }, chance_creation: { assists_per_90: 0.5 } },
        };
        const result = scorePlayer(player, archetype, allValues, 1000000);
        expect(result.rawScore).toBe(100);
        expect(result.valueAdjustedScore).toBe(100);
    });

    it("scores a player with worst metrics at 0", () => {
        const player: ScorablePlayer = {
            playerId: 2, fmUid: 2, name: "Worst", club: null,
            positions: "ST", age: 20, transferValueHigh: 500000,
            data: { attacking: { goals_per_90: 0.1 }, chance_creation: { assists_per_90: 0.1 } },
        };
        const result = scorePlayer(player, archetype, allValues, 500000);
        expect(result.rawScore).toBe(0);
    });

    it("handles missing metrics with 0 percentile", () => {
        const player: ScorablePlayer = {
            playerId: 3, fmUid: 3, name: "No Data", club: null,
            positions: "ST", age: 30, transferValueHigh: null,
            data: {},
        };
        const result = scorePlayer(player, archetype, allValues, 1000000);
        expect(result.rawScore).toBe(0);
    });

    it("handles inverted metrics", () => {
        const invertedArch: Archetype = {
            ...archetype,
            metrics: [
                { metric_key: "discipline.fouls_made_per_90", weight: 1.0, inverted: true },
            ],
        };
        const invValues = { "discipline.fouls_made_per_90": [1.0, 2.0, 3.0, 4.0, 5.0] };
        const player: ScorablePlayer = {
            playerId: 4, fmUid: 4, name: "Clean", club: null,
            positions: "ST", age: 25, transferValueHigh: 1000000,
            data: { discipline: { fouls_made_per_90: 1.0 } },
        };
        const result = scorePlayer(player, invertedArch, invValues, 1000000);
        // Lowest fouls (1.0) → percentile 0 → inverted → 100
        expect(result.rawScore).toBe(100);
    });

    it("computes value-adjusted score", () => {
        const player: ScorablePlayer = {
            playerId: 5, fmUid: 5, name: "Value", club: null,
            positions: "ST", age: 25, transferValueHigh: 500000,
            data: { attacking: { goals_per_90: 0.8 }, chance_creation: { assists_per_90: 0.5 } },
        };
        const result = scorePlayer(player, archetype, allValues, 1000000);
        // rawScore = 100, value = 500000, median = 1000000
        // valueAdjusted = 100 / (500000 / 1000000) = 100 / 0.5 = 200
        expect(result.rawScore).toBe(100);
        expect(result.valueAdjustedScore).toBe(200);
    });
});
```

- [ ] **Step 7: Implement the scoring function**

Create `src/lib/scoring/score.ts`:

```typescript
import { getMetricValue } from "./metric-accessor";
import { computePercentile, buildPercentileCache } from "./percentiles";
import type { ScorablePlayer, PlayerScore, PercentileCache } from "./types";
import type { Archetype, MetricWeight } from "$lib/types/archetype";

/**
 * Compute the median transfer value from a list of players.
 * Uses only non-null values. Returns 1 if no values exist (prevents division by zero).
 */
export function computeMedianTransferValue(players: ScorablePlayer[]): number {
    const values = players
        .map(p => p.transferValueHigh)
        .filter((v): v is number => v !== null && v > 0)
        .sort((a, b) => a - b);

    if (values.length === 0) return 1;

    const mid = Math.floor(values.length / 2);
    return values.length % 2 !== 0
        ? values[mid]
        : (values[mid - 1] + values[mid]) / 2;
}

/**
 * Score a single player against an archetype.
 *
 * For each metric:
 * 1. Get the player's value
 * 2. Compute percentile within the dataset
 * 3. If inverted: use (100 - percentile)
 * 4. Multiply by weight
 * 5. Sum all weighted percentiles → raw score (0-100)
 *
 * Value-adjusted score = rawScore / (transferValue / medianValue)
 */
export function scorePlayer(
    player: ScorablePlayer,
    archetype: Archetype,
    allMetricValues: Record<string, number[]>,
    medianTransferValue: number,
): PlayerScore {
    let rawScore = 0;
    const metricPercentiles: Record<string, number> = {};

    for (const metric of archetype.metrics) {
        const playerValue = getMetricValue(player.data, metric.metric_key);
        const sortedValues = allMetricValues[metric.metric_key] ?? [];

        let percentile: number;
        if (playerValue === null) {
            percentile = 0; // Worst case for missing data
        } else {
            percentile = computePercentile(playerValue, sortedValues);
        }

        if (metric.inverted) {
            percentile = 100 - percentile;
        }

        metricPercentiles[metric.metric_key] = percentile;
        rawScore += percentile * metric.weight;
    }

    // Value-adjusted score
    const transferValue = player.transferValueHigh ?? medianTransferValue;
    const valueRatio = transferValue / medianTransferValue;
    const valueAdjustedScore = valueRatio > 0 ? rawScore / valueRatio : rawScore;

    return {
        playerId: player.playerId,
        fmUid: player.fmUid,
        name: player.name,
        club: player.club,
        positions: player.positions,
        age: player.age,
        transferValue: player.transferValueHigh,
        role: archetype.role,
        rawScore,
        valueAdjustedScore,
        metricPercentiles,
    };
}

/**
 * Score all players against a specific archetype.
 * Builds the percentile cache from the player data and the archetype's metrics.
 */
export function scoreAllPlayers(
    players: ScorablePlayer[],
    archetype: Archetype,
): PlayerScore[] {
    const metricKeys = archetype.metrics.map(m => m.metric_key);
    const cache = buildPercentileCache(players, metricKeys);

    // Convert cache to plain record for scoring
    const allMetricValues: Record<string, number[]> = {};
    for (const [key, values] of cache.metricValues) {
        allMetricValues[key] = values;
    }

    const medianTransferValue = computeMedianTransferValue(players);

    return players.map(player =>
        scorePlayer(player, archetype, allMetricValues, medianTransferValue)
    );
}
```

- [ ] **Step 8: Create barrel export**

Create `src/lib/scoring/index.ts`:

```typescript
export { getMetricValue } from "./metric-accessor";
export { computePercentile, buildPercentileCache } from "./percentiles";
```

### Multi-Position Scoring Note

The design spec requires scoring each player against all their eligible positions.
The `scoreAllPlayers` function above scores against a single archetype. For multi-position
scoring, the calling code (scouting page / store) should:

1. For each player, parse their `positions` field to determine eligible archetype roles
   using the `PARSER_ROLE_TO_ARCHETYPE_ROLES` mapping from `$lib/types/archetype.ts`
2. For each eligible role, find the best-fitting archetype (or use user's selected archetype)
3. Call `scoreAllPlayers(players, archetype)` for each archetype
4. Merge results: each player has a score per position

For MVP, the page scores against the single user-selected archetype. Multi-position
best-fit scoring (showing all scores in the full database view) is a future enhancement.

- [ ] **Step 9: Run tests**

Run: `bun test` (or `vitest run` depending on project setup)
Expected: ALL PASS.

Run: `bun run check`
Expected: SUCCESS — TypeScript compilation passes.

## Dependencies

- Task 06 (frontend types) — `Archetype`, `MetricWeight` types

## Success Criteria

- `getMetricValue` correctly extracts nested values from ParsedPlayer data
- `computePercentile` returns correct percentiles for edge cases (empty, single value, exact match, interpolation)
- `scorePlayer` produces correct raw and value-adjusted scores
- Missing metrics produce 0 percentile
- Inverted metrics correctly flip the score direction
- Value-adjusted scoring uses median fallback for null transfer values
- All tests pass

## Tests

### Test 1: Metric accessor

**What to test:** Dot-path access to nested fields, null handling, malformed keys.
**Feasibility:** ✅ Can be tested — pure function.

### Test 2: Percentile computation

**What to test:** Edge cases (empty, single, min, max, median, duplicates).
**Feasibility:** ✅ Can be tested — pure function.

### Test 3: Scoring

**What to test:** Perfect score, worst score, missing data, inverted metrics, value-adjusted calculation.
**Feasibility:** ✅ Can be tested — pure function with constructed test data.
