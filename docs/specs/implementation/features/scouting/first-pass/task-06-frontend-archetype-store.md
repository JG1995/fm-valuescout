# Task 06 - Frontend Archetype Store & API Layer

## Overview

Create the TypeScript type definitions, Tauri invoke wrappers, and Svelte 5 store for archetypes. The store manages archetype state using runes (`$state`, `$derived`) and provides methods for CRUD operations.

## Files to Create/Modify

- Create: `src/lib/types/archetype.ts` — TypeScript interfaces matching Rust types
- Create: `src/lib/api/archetypes.ts` — Tauri invoke wrappers
- Create: `src/lib/stores/archetype-store.svelte.ts` — Svelte 5 rune-based store

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

From Task 04's seed data, the role strings are:
`"GK"`, `"CB"`, `"FB"`, `"DM"`, `"WB"`, `"CM"`, `"W"`, `"AM"`, `"ST"`

## Steps

- [ ] **Step 1: Create TypeScript type definitions**

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
    D: ["CB", "FB"],  // Defenders can be CB or FB
    WB: ["WB"],
    DM: ["DM"],
    M: ["CM"],
    AM: ["AM", "W"],  // AM can be AM or Winger
    ST: ["ST"],
};
```

- [ ] **Step 2: Create the API layer**

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

- [ ] **Step 3: Create the archetype store**

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

- [ ] **Step 4: Verify TypeScript compilation**

Run: `cd /workspace && bun run check`
Expected: SUCCESS — no TypeScript errors.

## Dependencies

- Task 05 (Tauri archetype commands) — Commands must be registered for `invoke` to work at runtime
- Tasks 01-04 must compile (but TypeScript compilation doesn't require a running backend)

## Success Criteria

- TypeScript types match the Rust `Archetype` and `MetricWeight` shapes
- API layer wraps all 6 Tauri commands
- Store provides reactive state with load, CRUD, and selection management
- `bun run check` passes with no errors
- No imports from non-existent modules

## Tests

### Test 1: TypeScript compilation

**What to test:** All new TypeScript files compile without errors.
**Feasibility:** ✅ Can be tested — `bun run check`.

### Test 2: Type shape consistency

**What to test:** TypeScript `Archetype` interface fields match Rust `Archetype` struct fields.
**Feasibility:** ✅ Can be verified by manual comparison — both use the same JSON serialization.
