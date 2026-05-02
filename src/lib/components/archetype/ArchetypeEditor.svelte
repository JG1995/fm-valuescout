<script lang="ts">
	import type { Archetype, ArchetypeRole, MetricWeight } from "$lib/types/archetype";

	interface Props {
		role: ArchetypeRole;
		archetype?: Archetype | null;
		/** Called with (name, metrics) on successful save. Metrics are normalized to sum to 1.0. */
		/** Called when user clicks cancel or close. */
		onsave: (name: string, metrics: MetricWeight[]) => void;
		onclose: () => void;
	}

	let { role, archetype = null, onsave, onclose }: Props = $props();

	let name = $state("");
	let metrics = $state<MetricWeight[]>([
		{ metric_key: "", weight: 1.0, inverted: false },
	]);

	// Sync local state when archetype prop changes (e.g., when switching from create to edit mode)
	$effect(() => {
		name = archetype?.name ?? "";
		metrics = archetype?.metrics.map((m) => ({ ...m })) ?? [
			{ metric_key: "", weight: 1.0, inverted: false },
		];
	});

	let error = $state<string | null>(null);

	let totalWeight = $derived.by(() =>
		metrics.reduce((sum, m) => sum + m.weight, 0)
	);

	function addMetric() {
		metrics = [...metrics, { metric_key: "", weight: 0.1, inverted: false }];
	}

	function removeMetric(index: number) {
		if (metrics.length <= 1) return;
		metrics = metrics.filter((_, i) => i !== index);
	}

	function updateMetricKey(index: number, value: string) {
		metrics = metrics.map((m, i) => (i === index ? { ...m, metric_key: value } : m));
	}

	function updateMetricWeight(index: number, value: number) {
		metrics = metrics.map((m, i) => (i === index ? { ...m, weight: value } : m));
	}

	function updateMetricInverted(index: number, value: boolean) {
		metrics = metrics.map((m, i) => (i === index ? { ...m, inverted: value } : m));
	}

	function handleSave() {
		error = null;
		const trimmedName = name.trim();
		if (!trimmedName) {
			error = "Name cannot be empty.";
			return;
		}
		const hasEmptyKey = metrics.some((m) => !m.metric_key.trim());
		if (hasEmptyKey) {
			error = "All metrics must have a non-empty key.";
			return;
		}
		const hasNegativeWeight = metrics.some((m) => m.weight <= 0);
		if (hasNegativeWeight) {
			error = "All weights must be positive.";
			return;
		}
		// Normalize weights to sum to 1.0
		const normalized = metrics.map((m) => ({
			...m,
			weight: m.weight / totalWeight,
		}));
		onsave(trimmedName, normalized);
	}
</script>

<div
	class="editor-overlay"
	onclick={onclose}
	onkeydown={(e) => e.key === "Escape" && onclose()}
	role="dialog"
	aria-modal="true"
	aria-label="Archetype editor"
	tabindex="-1"
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="editor-panel" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
		<div class="editor-header">
			<h3>{archetype ? "Edit" : "Create"} Archetype ({role})</h3>
			<button class="close-btn" onclick={onclose} aria-label="Close">✕</button>
		</div>

		<div class="editor-body">
			<label class="field">
				<span>Name</span>
				<input
					type="text"
					value={name}
					oninput={(e) => (name = e.currentTarget.value)}
					placeholder="Archetype name"
					maxlength={100}
				/>
			</label>

			<div class="metrics-section">
				<div class="metrics-header">
					<span>Metrics</span>
					<span class="weight-total">Total weight: {totalWeight.toFixed(2)}</span>
				</div>

				{#each metrics as metric, i (i)}
					<div class="metric-row" data-testid="metric-row">
						<input
							type="text"
							value={metric.metric_key}
							placeholder="metric key (e.g., attacking.goals_per_90)"
							oninput={(e) => updateMetricKey(i, e.currentTarget.value)}
							data-testid="metric-key-input"
						/>
						<input
							type="number"
							value={metric.weight}
							min="0"
							max="1"
							step="0.05"
							oninput={(e) =>
								updateMetricWeight(i, parseFloat(e.currentTarget.value) || 0)}
							aria-label="Metric weight"
							data-testid="metric-weight-input"
						/>
						<label class="inverted-toggle">
							<input
								type="checkbox"
								checked={metric.inverted}
								onchange={(e) => updateMetricInverted(i, e.currentTarget.checked)}
								aria-label="Invert metric"
								data-testid="metric-inverted-checkbox"
							/>
							<span>Inv</span>
						</label>
						<button
							class="remove-btn"
							disabled={metrics.length <= 1}
							onclick={() => removeMetric(i)}
							data-testid="remove-metric-btn"
							aria-label="Remove metric"
						>
							✕
						</button>
					</div>
				{/each}

				<button class="add-metric-btn" onclick={addMetric} data-testid="add-metric-btn">
					+ Add Metric
				</button>
			</div>

			{#if error}
				<div class="error" data-testid="error-message">{error}</div>
			{/if}
		</div>

		<div class="editor-footer">
			<button class="cancel-btn" onclick={onclose} data-testid="cancel-btn">Cancel</button>
			<button class="save-btn" onclick={handleSave} data-testid="save-btn">Save</button>
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
		background: var(--color-surface, #1e1e1e);
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
		border-bottom: 1px solid var(--color-border, #333);
	}

	.editor-header h3 {
		margin: 0;
		font-size: 1rem;
		color: var(--color-text-primary, #fff);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-secondary, #999);
		font-size: 1.2rem;
		cursor: pointer;
	}

	.close-btn:hover {
		color: var(--color-text-primary, #fff);
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
		color: var(--color-text-secondary, #aaa);
	}

	.field input[type="text"] {
		padding: 8px;
		background: var(--color-surface-hover, #2a2a2a);
		border: 1px solid var(--color-border, #3a3a3a);
		border-radius: 4px;
		color: var(--color-text-primary, #fff);
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
		color: var(--color-text-secondary, #aaa);
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
		background: var(--color-surface-hover, #2a2a2a);
		border: 1px solid var(--color-border, #3a3a3a);
		border-radius: 4px;
		color: var(--color-text-primary, #fff);
		font-size: 0.8rem;
	}

	.metric-row input[type="number"] {
		width: 60px;
		padding: 6px;
		background: var(--color-surface-hover, #2a2a2a);
		border: 1px solid var(--color-border, #3a3a3a);
		border-radius: 4px;
		color: var(--color-text-primary, #fff);
		font-size: 0.8rem;
	}

	.inverted-toggle {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.75rem;
		color: var(--color-text-secondary, #aaa);
	}

	.remove-btn {
		background: none;
		border: none;
		color: var(--color-text-secondary, #666);
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
		background: var(--color-surface-hover, #2a2a2a);
		border: 1px dashed var(--color-border, #555);
		border-radius: 4px;
		color: var(--color-text-secondary, #888);
		cursor: pointer;
		font-size: 0.8rem;
	}

	.add-metric-btn:hover {
		background: var(--color-surface-active, #333);
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
		border-top: 1px solid var(--color-border, #333);
	}

	.cancel-btn {
		padding: 8px 16px;
		background: var(--color-surface-hover, #333);
		border: none;
		border-radius: 6px;
		color: var(--color-text-secondary, #ccc);
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
