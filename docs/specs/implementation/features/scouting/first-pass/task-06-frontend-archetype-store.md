# Task 06 - Frontend Archetype Store & API Layer

## Overview

Create the TypeScript type definitions, Tauri invoke wrappers, and Svelte 5 store for archetypes. The store manages archetype state using runes (`$state`, `$derived`) and provides methods for CRUD operations.

Uses TDD: install vitest, write failing store tests, then implement to make them pass.

## Files to Create/Modify

- Create: `src/lib/types/archetype.ts` — TypeScript interfaces matching Rust types
- Create: `src/lib/api/archetypes.ts` — Tauri invoke wrappers
- Create: `src/lib/stores/archetype-store.svelte.ts` — Svelte 5 rune-based store
- Create: `src/lib/stores/archetype-store.test.ts` — Unit tests for store logic

## Context

### Svelte 5 Runes Pattern

Svelte 5 uses `$state()` for reactive state and `$derived` for computed values. Stores are plain `.svelte.ts` files (not using the old `writable()`/`readable()` pattern):

```typescript
// src/lib/stores/example.svelte.ts
let count = $state(0);
let doubled = $derived(count * 2);

function increment() {
    count++;
}

export function getExampleStore() {
    return {
        get count() { return count; },
        get doubled() { return doubled; },
        increment,
    };
}
```

### Tauri Invoke Pattern

From the existing `src/routes/+page.svelte`, the pattern is:

```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("command_name", { param1, param2 });
```

Commands defined in Task 05:
- `create_archetype_cmd` — `(name: string, role: string, metrics: MetricWeight[]) → Archetype`
- `list_archetypes_by_role` — `(role: string) → Archetype[]`
- `list_all_archetypes_cmd` — `() → Archetype[]`
- `get_archetype_cmd` — `(id: number) → Archetype`
- `update_archetype_cmd` — `(id: number, name: string, metrics: MetricWeight[]) → Archetype`
- `delete_archetype_cmd` — `(id: number) → void`

### Role String Values

From the parser coarse role system: `GK`, `D`, `WB`, `DM`, `M`, `AM`, `ST`.

## Steps

- [ ] **Step 1: Install and configure vitest**

Add vitest and required plugins as devDependencies:

```bash
bun add -d vitest @vitest/coverage-v8 vite-tsconfig-paths
```

Add a `test` script to `package.json`:

```json
{
  "scripts": {
    "test": "vitest run",
    "test:watch": "vitest"
  }
}
```

Create or update `vitest.config.ts`:

```typescript
import { defineConfig } from "vitest/config";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
    test: {
        include: ["src/**/*.test.ts"],
    },
    plugins: [tsconfigPaths()],
});
```

This configures vitest to resolve `$lib` path aliases via tsconfig and finds all test files under `src/`.

- [ ] **Step 2: Create TypeScript type definitions**

Create `src/lib/types/archetype.ts`:

```typescript
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
```

- [ ] **Step 3: Write failing tests for the archetype store**

Create `src/lib/stores/archetype-store.test.ts`:

```typescript
import { describe, it, expect, vi, beforeEach } from "vitest";
import type { Archetype, ArchetypeRole, MetricWeight } from "$lib/types/archetype";

// Mock the Tauri invoke — the API layer calls this
vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

// Helper to make a valid Archetype
function makeArchetype(overrides: Partial<Archetype> = {}): Archetype {
    return {
        id: 1,
        name: "Test Archetype",
        role: "ST",
        metrics: [
            { metric_key: "attacking.goals_per_90", weight: 0.6, inverted: false },
            { metric_key: "chance_creation.assists_per_90", weight: 0.4, inverted: false }, // gitleaks:allow
        ],
        is_default: true,
        created_at: "2024-01-01T00:00:00Z",
        updated_at: "2024-01-01T00:00:00Z",
        ...overrides,
    };
}

describe("Archetype Store", () => {
    beforeEach(() => {
        vi.resetModules();
        vi.mocked(invoke).mockReset();
    });

    describe("loadAll", () => {
        it("populates archetypes array", async () => {
            const archetypes = [
                makeArchetype({ id: 1, name: "Poacher", role: "ST" }),
                makeArchetype({ id: 2, name: "Anchor", role: "DM" }),
            ];
            vi.mocked(invoke).mockResolvedValue(archetypes);

            const { getArchetypeStore } = await import("./archetype-store.svelte");
            const store = getArchetypeStore();

            await store.loadAll();

            expect(store.archetypes).toHaveLength(2);
            expect(store.archetypes[0].name).toBe("Poacher");
            expect(store.archetypes[1].name).toBe("Anchor");
            expect(store.loading).toBe(false);
        });

        it("sets error on failure", async () => {
            vi.mocked(invoke).mockRejectedValue(new Error("DB error"));

            const { getArchetypeStore } = await import("./archetype-store.svelte");
            const store = getArchetypeStore();

            await store.loadAll();

            expect(store.error).toBe("DB error");
            expect(store.loading).toBe(false);
        });
    });

    describe("loadByRole", () => {
        it("replaces only that role's archetypes in the cache", async () => {
            // First, loadAll returns ST + DM
            vi.mocked(invoke).mockResolvedValue([
                makeArchetype({ id: 1, role: "ST", name: "Striker A" }),
                makeArchetype({ id: 2, role: "DM", name: "Defender A" }),
            ]);

            const { getArchetypeStore } = await import("./archetype-store.svelte");
            const store = getArchetypeStore();
            await store.loadAll();

            // Now loadByRole returns new ST archetypes
            vi.mocked(invoke).mockResolvedValue([
                makeArchetype({ id: 3, role: "ST", name: "Striker B" }),
            ]);

            await store.loadByRole("ST" as ArchetypeRole);

            expect(store.archetypes).toHaveLength(2);
            // DM archetype should remain
            expect(store.archetypes.find(a => a.role === "DM")?.name).toBe("Defender A");
            // ST should be replaced
            expect(store.archetypes.find(a => a.role === "ST")?.name).toBe("Striker B");
        });
    });

    describe("create", () => {
        it("adds new archetype to list", async () => {
            vi.mocked(invoke).mockResolvedValue([]);

            const { getArchetypeStore } = await import("./archetype-store.svelte");
            const store = getArchetypeStore();
            await store.loadAll();

            const newArch = makeArchetype({ id: 10, name: "New Arch" });
            vi.mocked(invoke).mockResolvedValue(newArch);

            const result = await store.create("New Arch", "ST" as ArchetypeRole, newArch.metrics);

            expect(result).not.toBeNull();
            expect(result!.name).toBe("New Arch");
            expect(store.archetypes).toHaveLength(1);
        });
    });

    describe("update", () => {
        it("replaces archetype in list and updates selected references", async () => {
            const original = makeArchetype({ id: 5, name: "Old Name" });
            vi.mocked(invoke).mockResolvedValue([original]);

            const { getArchetypeStore } = await import("./archetype-store.svelte");
            const store = getArchetypeStore();
            await store.loadAll();

            // Select it
            store.selectArchetype("GK", original);
            expect(store.getSelectedForSlot("GK")?.name).toBe("Old Name");

            // Update it
            const updated = makeArchetype({ id: 5, name: "New Name" });
            vi.mocked(invoke).mockResolvedValue(updated);

            await store.update(5, "New Name", updated.metrics);

            expect(store.archetypes[0].name).toBe("New Name");
            // Selected reference should also be updated
            expect(store.getSelectedForSlot("GK")?.name).toBe("New Name");
        });
    });

    describe("remove", () => {
        it("deletes from list and clears selected references", async () => {
            const arch = makeArchetype({ id: 7, name: "To Delete" });
            vi.mocked(invoke).mockResolvedValue([arch]);

            const { getArchetypeStore } = await import("./archetype-store.svelte");
            const store = getArchetypeStore();
            await store.loadAll();

            // Select it
            store.selectArchetype("CB-L", arch);
            expect(store.getSelectedForSlot("CB-L")?.name).toBe("To Delete");

            // Delete it
            vi.mocked(invoke).mockResolvedValue(undefined);
            const success = await store.remove(7);

            expect(success).toBe(true);
            expect(store.archetypes).toHaveLength(0);
            expect(store.getSelectedForSlot("CB-L")).toBeNull();
        });
    });

    describe("archetypesByRole", () => {
        it("groups archetypes by role", async () => {
            const archetypes = [
                makeArchetype({ id: 1, role: "ST", name: "Poacher" }),
                makeArchetype({ id: 2, role: "ST", name: "Complete Forward" }),
                makeArchetype({ id: 3, role: "D", name: "Ball Playing Defender" }),
            ];
            vi.mocked(invoke).mockResolvedValue(archetypes);

            const { getArchetypeStore } = await import("./archetype-store.svelte");
            const store = getArchetypeStore();
            await store.loadAll();

            const byRole = store.archetypesByRole;
            expect(byRole["ST"]).toHaveLength(2);
            expect(byRole["D"]).toHaveLength(1);
            expect(byRole["DM"]).toBeUndefined();
        });
    });
});
```

Run: `bun run test`
Expected: FAIL — `archetype-store.svelte.ts` does not exist yet.

- [ ] **Step 4: Create the API layer**

Create directory `src/lib/api/` and file `src/lib/api/archetypes.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { Archetype, MetricWeight } from "$lib/types/archetype";

export async function createArchetype(
    name: string,
    role: string,
    metrics: MetricWeight[],
): Promise<Archetype> {
    return invoke<Archetype>("create_archetype_cmd", { name, role, metrics });
}

export async function listArchetypesByRole(role: string): Promise<Archetype[]> {
    return invoke<Archetype[]>("list_archetypes_by_role", { role });
}

export async function listAllArchetypes(): Promise<Archetype[]> {
    return invoke<Archetype[]>("list_all_archetypes_cmd");
}

export async function getArchetype(id: number): Promise<Archetype> {
    return invoke<Archetype>("get_archetype_cmd", { id });
}

export async function updateArchetype(
    id: number,
    name: string,
    metrics: MetricWeight[],
): Promise<Archetype> {
    return invoke<Archetype>("update_archetype_cmd", { id, name, metrics });
}

export async function deleteArchetype(id: number): Promise<void> {
    return invoke("delete_archetype_cmd", { id });
}
```

- [ ] **Step 5: Create the archetype store**

Create directory `src/lib/stores/` and file `src/lib/stores/archetype-store.svelte.ts`:

```typescript
import type { Archetype, ArchetypeRole } from "$lib/types/archetype";
import * as api from "$lib/api/archetypes";

/** State */
let archetypes = $state<Archetype[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

/** Derived: archetypes grouped by role */
let archetypesByRole = $derived.by<Record<string, Archetype[]>>(() => {
    const map: Record<string, Archetype[]> = {};
    for (const arch of archetypes) {
        if (!map[arch.role]) map[arch.role] = [];
        map[arch.role].push(arch);
    }
    return map;
});

/** Selected archetype per position slot (keyed by slot id like "GK", "CB-L", etc.) */
let selectedArchetypes = $state<Record<string, Archetype | null>>({});

/** Actions */

async function loadAll() {
    loading = true;
    error = null;
    try {
        archetypes = await api.listAllArchetypes();
    } catch (e) {
        error = e instanceof Error ? e.message : String(e);
    } finally {
        loading = false;
    }
}

async function loadByRole(role: ArchetypeRole) {
    try {
        const roleArchetypes = await api.listArchetypesByRole(role);
        // Replace only the archetypes for this role in the local cache
        archetypes = [
            ...archetypes.filter(a => a.role !== role),
            ...roleArchetypes,
        ];
    } catch (e) {
        error = e instanceof Error ? e.message : String(e);
    }
}

function selectArchetype(slotId: string, archetype: Archetype | null) {
    selectedArchetypes[slotId] = archetype;
}

function getSelectedForSlot(slotId: string): Archetype | null {
    return selectedArchetypes[slotId] ?? null;
}

async function create(name: string, role: ArchetypeRole, metrics: Archetype["metrics"]): Promise<Archetype | null> {
    try {
        const created = await api.createArchetype(name, role, metrics);
        archetypes = [...archetypes, created];
        return created;
    } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        return null;
    }
}

async function update(id: number, name: string, metrics: Archetype["metrics"]): Promise<Archetype | null> {
    try {
        const updated = await api.updateArchetype(id, name, metrics);
        archetypes = archetypes.map(a => a.id === id ? updated : a);
        // Also update selected archetype references
        for (const slotId of Object.keys(selectedArchetypes)) {
            if (selectedArchetypes[slotId]?.id === id) {
                selectedArchetypes[slotId] = updated;
            }
        }
        return updated;
    } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        return null;
    }
}

async function remove(id: number): Promise<boolean> {
    try {
        await api.deleteArchetype(id);
        const removed = archetypes.find(a => a.id === id);
        archetypes = archetypes.filter(a => a.id !== id);
        // Clear any selected references to this archetype
        if (removed) {
            for (const slotId of Object.keys(selectedArchetypes)) {
                if (selectedArchetypes[slotId]?.id === id) {
                    selectedArchetypes[slotId] = null;
                }
            }
        }
        return true;
    } catch (e) {
        error = e instanceof Error ? e.message : String(e);
        return false;
    }
}

function clearError() {
    error = null;
}

export function getArchetypeStore() {
    return {
        get archetypes() { return archetypes; },
        get loading() { return loading; },
        get error() { return error; },
        get archetypesByRole() { return archetypesByRole; },
        get selectedArchetypes() { return selectedArchetypes; },

        loadAll,
        loadByRole,
        selectArchetype,
        getSelectedForSlot,
        create,
        update,
        remove,
        clearError,
    };
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `bun run test`
Expected: ALL PASS — all store unit tests pass.

- [ ] **Step 7: Verify TypeScript compilation**

Run: `bun run check`
Expected: SUCCESS — no TypeScript errors.

## Dependencies

- Task 05 (Tauri archetype commands) — Commands must be registered for `invoke` to work at runtime
- Tasks 01-04 must compile (but TypeScript compilation doesn't require a running backend)

## Success Criteria

- Vitest is installed and configured for SvelteKit path aliases
- TypeScript types match the Rust `Archetype` and `MetricWeight` shapes
- API layer wraps all 6 Tauri commands
- Store provides reactive state with load, CRUD, and selection management
- All store unit tests pass (`loadAll`, `loadByRole`, `create`, `update`, `remove`, `archetypesByRole`)
- `bun run check` passes with no errors
- No imports from non-existent modules

## Tests

### Test 1: Store unit tests

**What to test:** Store state transformations — loadAll, loadByRole, create, update, remove, archetypesByRole derived grouping.
**Command:** `bun run test`
**Feasibility:** ✅ Unit tests with mocked `@tauri-apps/api/core` invoke.

### Test 2: TypeScript compilation

**What to test:** All new TypeScript files compile without errors.
**Command:** `bun run check`
**Feasibility:** ✅ Can be tested — `bun run check`.

### Test 3: Type shape consistency

**What to test:** TypeScript `Archetype` interface fields match Rust `Archetype` struct fields.
**Feasibility:** ✅ Can be verified by manual comparison — both use the same JSON serialization.
