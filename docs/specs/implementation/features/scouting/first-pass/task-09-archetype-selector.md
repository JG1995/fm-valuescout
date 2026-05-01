# Task 09 - Archetype Selector Component

## Overview

Create a dropdown/modal component that appears when a user clicks a pitch position slot. It shows all available archetypes for that position's role and allows selecting, creating, editing, and deleting archetypes.

## Files to Create/Modify

- Create: `src/lib/components/archetype/ArchetypeSelector.svelte` — Dropdown selector
- Create: `src/lib/components/archetype/ArchetypeEditor.svelte` — Create/edit modal

## Context

### Interaction Flow

1. User clicks a pitch position slot (e.g., "CB-L")
2. `ArchetypeSelector` opens, filtered to show archetypes for role "CB"
3. User can:
   - Click an existing archetype to select it → selector closes, slot updates
   - Click "Create New" → `ArchetypeEditor` opens in create mode
   - Click edit icon on an archetype → `ArchetypeEditor` opens in edit mode
   - Click delete icon on a custom archetype → confirmation → delete

### ArchetypeEditor

The editor is a modal with:
- Name input field
- Metric list (key, weight, inverted toggle)
- Add metric button
- Save / Cancel buttons
- Weight validation (must sum to ~1.0 — normalized on save)

This is Phase 2 per the design spec ("Archetype creation modal with metric picker" is nice-to-have). For MVP, the editor can be a simple form with a text input for metric key, number input for weight, and checkbox for inverted.

## Steps

- [ ] **Step 1: Create ArchetypeSelector component**

Create directory `src/lib/components/archetype/` and file `src/lib/components/archetype/ArchetypeSelector.svelte`:

```svelte
<script lang="ts">
    import type { Archetype, ArchetypeRole } from "$lib/types/archetype";

    interface Props {
        role: ArchetypeRole;
        archetypes: Archetype[];
        selectedArchetypeId: number | null;
        onselect: (archetype: Archetype | null) => void;
        onedit: (archetype: Archetype) => void;
        oncreate: () => void;
        ondelete: (archetype: Archetype) => void;
        onclose: () => void;
    }

    let { role, archetypes, selectedArchetypeId, onselect, onedit, oncreate, ondelete, onclose }: Props = $props();

    function handleDelete(e: MouseEvent, arch: Archetype) {
        e.stopPropagation();
        if (arch.is_default) return; // Cannot delete defaults
        const confirmed = window.confirm(`Delete archetype "${arch.name}"? This cannot be undone.`);
        if (confirmed) {
            ondelete(arch);
        }
    }

    function handleEdit(e: MouseEvent, arch: Archetype) {
        e.stopPropagation();
        onedit(arch);
    }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="selector-overlay" onclick={onclose} onkeydown={(e) => e.key === "Escape" && onclose()}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="selector-panel" onclick={(e) => e.stopPropagation()}>
        <div class="selector-header">
            <h3>Select Archetype ({role})</h3>
            <button class="close-btn" onclick={onclose}>✕</button>
        </div>

        <ul class="archetype-list">
            {#each archetypes as arch (arch.id)}
                <li>
                    <button
                        class="archetype-option"
                        class:selected={selectedArchetypeId === arch.id}
                        onclick={() => onselect(arch)}
                    >
                        <span class="arch-name">{arch.name}</span>
                        {#if arch.is_default}
                            <span class="badge">Default</span>
                        {/if}
                    </button>
                    <div class="arch-actions">
                        <button
                            class="icon-btn"
                            onclick={(e) => handleEdit(e, arch)}
                            title="Edit"
                        >✏</button>
                        {#if !arch.is_default}
                            <button
                                class="icon-btn delete-btn"
                                onclick={(e) => handleDelete(e, arch)}
                                title="Delete"
                            >🗑</button>
                        {/if}
                    </div>
                </li>
            {/each}
        </ul>

        <button class="create-btn" onclick={oncreate}>
            + Create New Archetype
        </button>
    </div>
</div>

<style>
    .selector-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
    }

    .selector-panel {
        background: #1e1e1e;
        border-radius: 12px;
        width: 360px;
        max-height: 80vh;
        display: flex;
        flex-direction: column;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    }

    .selector-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px;
        border-bottom: 1px solid #333;
    }

    .selector-header h3 {
        margin: 0;
        font-size: 1rem;
        color: #fff;
    }

    .close-btn {
        background: none;
        border: none;
        color: #999;
        font-size: 1.2rem;
        cursor: pointer;
    }

    .close-btn:hover {
        color: #fff;
    }

    .archetype-list {
        list-style: none;
        margin: 0;
        padding: 8px;
        overflow-y: auto;
        flex: 1;
    }

    .archetype-list li {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .archetype-option {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 12px;
        background: #2a2a2a;
        border: 1px solid #3a3a3a;
        border-radius: 6px;
        color: #ddd;
        cursor: pointer;
        text-align: left;
        transition: background 0.1s;
    }

    .archetype-option:hover {
        background: #3a3a3a;
    }

    .archetype-option.selected {
        border-color: #4caf50;
        background: rgba(76, 175, 80, 0.15);
    }

    .arch-name {
        flex: 1;
    }

    .badge {
        font-size: 0.65rem;
        background: #444;
        padding: 2px 6px;
        border-radius: 4px;
        color: #aaa;
    }

    .arch-actions {
        display: flex;
        gap: 4px;
    }

    .icon-btn {
        background: none;
        border: none;
        color: #888;
        cursor: pointer;
        font-size: 0.9rem;
        padding: 4px;
    }

    .icon-btn:hover {
        color: #fff;
    }

    .delete-btn:hover {
        color: #ef5350;
    }

    .create-btn {
        margin: 8px;
        padding: 10px;
        background: #333;
        border: 1px dashed #555;
        border-radius: 6px;
        color: #aaa;
        cursor: pointer;
        transition: background 0.1s;
    }

    .create-btn:hover {
        background: #3a3a3a;
        color: #fff;
    }
</style>
```

- [ ] **Step 2: Create ArchetypeEditor component**

Create `src/lib/components/archetype/ArchetypeEditor.svelte`:

```svelte
<script lang="ts">
    import type { Archetype, ArchetypeRole, MetricWeight } from "$lib/types/archetype";

    interface Props {
        role: ArchetypeRole;
        archetype?: Archetype | null;
        onsave: (name: string, metrics: MetricWeight[]) => void;
        onclose: () => void;
    }

    let { role, archetype, onsave, onclose }: Props = $props();

    let name = $state(archetype?.name ?? "");
    let metrics = $state<MetricWeight[]>(
        archetype?.metrics.map(m => ({ ...m })) ?? [{ metric_key: "", weight: 1.0, inverted: false }]
    );
    let error = $state<string | null>(null);

    let totalWeight = $derived(metrics.reduce((sum, m) => sum + m.weight, 0));

    function addMetric() {
        metrics = [...metrics, { metric_key: "", weight: 0.1, inverted: false }];
    }

    function removeMetric(index: number) {
        if (metrics.length <= 1) return;
        metrics = metrics.filter((_, i) => i !== index);
    }

    function updateMetric(index: number, field: keyof MetricWeight, value: string | number | boolean) {
        metrics = metrics.map((m, i) =>
            i === index ? { ...m, [field]: value } : m
        );
    }

    function handleSave() {
        error = null;
        const trimmedName = name.trim();
        if (!trimmedName) {
            error = "Name cannot be empty.";
            return;
        }
        const hasEmptyKey = metrics.some(m => !m.metric_key.trim());
        if (hasEmptyKey) {
            error = "All metrics must have a non-empty key.";
            return;
        }
        const hasNegativeWeight = metrics.some(m => m.weight <= 0);
        if (hasNegativeWeight) {
            error = "All weights must be positive.";
            return;
        }
        onsave(trimmedName, metrics);
    }
</script>

<div class="editor-overlay">
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="editor-panel" onclick={(e) => e.stopPropagation()}>
        <div class="editor-header">
            <h3>{archetype ? "Edit" : "Create"} Archetype ({role})</h3>
            <button class="close-btn" onclick={onclose}>✕</button>
        </div>

        <div class="editor-body">
            <label class="field">
                <span>Name</span>
                <input type="text" bind:value={name} placeholder="Archetype name" maxlength={100} />
            </label>

            <div class="metrics-section">
                <div class="metrics-header">
                    <span>Metrics</span>
                    <span class="weight-total">Total weight: {totalWeight.toFixed(2)}</span>
                </div>

                {#each metrics as metric, i (i)}
                    <div class="metric-row">
                        <input
                            type="text"
                            value={metric.metric_key}
                            placeholder="metric key (e.g., attacking.goals_per_90)"
                            oninput={(e) => updateMetric(i, "metric_key", e.currentTarget.value)}
                        />
                        <input
                            type="number"
                            value={metric.weight}
                            min="0"
                            max="1"
                            step="0.05"
                            oninput={(e) => updateMetric(i, "weight", parseFloat(e.currentTarget.value) || 0)}
                        />
                        <label class="inverted-toggle">
                            <input
                                type="checkbox"
                                checked={metric.inverted}
                                onchange={(e) => updateMetric(i, "inverted", e.currentTarget.checked)}
                            />
                            <span>Inv</span>
                        </label>
                        <button
                            class="remove-btn"
                            disabled={metrics.length <= 1}
                            onclick={() => removeMetric(i)}
                        >✕</button>
                    </div>
                {/each}

                <button class="add-metric-btn" onclick={addMetric}>+ Add Metric</button>
            </div>

            {#if error}
                <div class="error">{error}</div>
            {/if}
        </div>

        <div class="editor-footer">
            <button class="cancel-btn" onclick={onclose}>Cancel</button>
            <button class="save-btn" onclick={handleSave}>Save</button>
        </div>
    </div>
</div>

<style>
    .editor-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 200;
    }

    .editor-panel {
        background: #1e1e1e;
        border-radius: 12px;
        width: 480px;
        max-height: 85vh;
        display: flex;
        flex-direction: column;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    }

    .editor-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px;
        border-bottom: 1px solid #333;
    }

    .editor-header h3 {
        margin: 0;
        font-size: 1rem;
        color: #fff;
    }

    .close-btn {
        background: none;
        border: none;
        color: #999;
        font-size: 1.2rem;
        cursor: pointer;
    }

    .editor-body {
        padding: 16px;
        overflow-y: auto;
        flex: 1;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 4px;
        margin-bottom: 16px;
    }

    .field span {
        font-size: 0.8rem;
        color: #aaa;
    }

    .field input[type="text"] {
        padding: 8px;
        background: #2a2a2a;
        border: 1px solid #3a3a3a;
        border-radius: 4px;
        color: #fff;
    }

    .metrics-section {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .metrics-header {
        display: flex;
        justify-content: space-between;
        font-size: 0.8rem;
        color: #aaa;
    }

    .weight-total {
        font-family: monospace;
    }

    .metric-row {
        display: flex;
        gap: 6px;
        align-items: center;
    }

    .metric-row input[type="text"] {
        flex: 2;
        padding: 6px;
        background: #2a2a2a;
        border: 1px solid #3a3a3a;
        border-radius: 4px;
        color: #fff;
        font-size: 0.8rem;
    }

    .metric-row input[type="number"] {
        width: 60px;
        padding: 6px;
        background: #2a2a2a;
        border: 1px solid #3a3a3a;
        border-radius: 4px;
        color: #fff;
        font-size: 0.8rem;
    }

    .inverted-toggle {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 0.75rem;
        color: #aaa;
    }

    .remove-btn {
        background: none;
        border: none;
        color: #666;
        cursor: pointer;
        padding: 4px;
    }

    .remove-btn:hover:not(:disabled) {
        color: #ef5350;
    }

    .remove-btn:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }

    .add-metric-btn {
        padding: 6px;
        background: #2a2a2a;
        border: 1px dashed #555;
        border-radius: 4px;
        color: #888;
        cursor: pointer;
        font-size: 0.8rem;
    }

    .add-metric-btn:hover {
        background: #333;
    }

    .error {
        color: #ef5350;
        font-size: 0.85rem;
        margin-top: 8px;
    }

    .editor-footer {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
        padding: 16px;
        border-top: 1px solid #333;
    }

    .cancel-btn {
        padding: 8px 16px;
        background: #333;
        border: none;
        border-radius: 6px;
        color: #ccc;
        cursor: pointer;
    }

    .save-btn {
        padding: 8px 16px;
        background: #4caf50;
        border: none;
        border-radius: 6px;
        color: #fff;
        cursor: pointer;
    }

    .save-btn:hover {
        background: #43a047;
    }
</style>
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `bun run check`
Expected: SUCCESS.

## Dependencies

- Task 06 (frontend types) — `Archetype`, `ArchetypeRole`, `MetricWeight`
- Task 08 (pitch view) — Not a hard dependency, but the selector is triggered by pitch clicks

## Success Criteria

- ArchetypeSelector shows a list of archetypes filtered by role
- Clicking an archetype calls `onselect` and can close the selector
- Edit/delete actions are available on each archetype
- Default archetypes cannot be deleted (button hidden)
- ArchetypeEditor allows creating/editing with name and metrics
- Editor validates: non-empty name, non-empty metric keys, positive weights
- Editor shows total weight sum
- Both components compile without errors
- Dark theme styling consistent with the app

## Tests

### Test 1: TypeScript compilation

**What to test:** Components compile without errors.
**Command:** `bun run check`
**Feasibility:** ✅ Can be tested — `bun run check`.

### Test 2: Visual verification

**What to test:** Selector opens, lists archetypes by role, edit/delete actions work, editor modal validates inputs.
**Feasibility:** ⚠️ No Svelte testing library installed. Svelte component rendering tests require `@testing-library/svelte` (not in scope for this plan). Verify in dev mode (`bun run tauri dev`).

### Rationale

Both components in this task (`ArchetypeSelector.svelte`, `ArchetypeEditor.svelte`) are pure Svelte UI — event handlers, conditional rendering, and form state. No extractable pure logic functions. The `handleSave` validation in `ArchetypeEditor` (empty name, empty keys, negative weights) is tightly coupled to Svelte `$state` and the form lifecycle. The automated gate is `bun run check` (type safety).
