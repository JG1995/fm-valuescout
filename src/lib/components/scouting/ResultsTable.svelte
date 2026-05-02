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

	// Sort state
	type SortColumn = "name" | "age" | "transferValue" | "rawScore" | "valueAdjustedScore" | `metric.${string}`;
	let sortColumn = $state<SortColumn>("valueAdjustedScore");
	let sortDirection = $state<"asc" | "desc">("desc");

	// Column visibility state (for metric columns only)
	let visibleMetricKeys = $state<Set<string>>(new Set());

	// Initialize visible metrics when archetype changes
	$effect(() => {
		if (archetype) {
			visibleMetricKeys = new Set(archetype.metrics.map((m) => m.metric_key));
		}
	});

	/**
	 * Sorted scores based on current sort column and direction.
	 */
	const sortedScores = $derived.by(() => {
		const sorted = [...scores].sort((a, b) => {
			const aVal = getSortValue(a, sortColumn);
			const bVal = getSortValue(b, sortColumn);
			return aVal - bVal;
		});
		return sortDirection === "desc" ? sorted : sorted.reverse();
	});

	/**
	 * All metric keys from archetype (for column rendering).
	 */
	const metricKeys = $derived(archetype?.metrics.map((m) => m.metric_key) ?? []);

	/**
	 * Visible metric keys (sorted by archetype order).
	 */
	const displayedMetricKeys = $derived(
		metricKeys.filter((key) => visibleMetricKeys.has(key))
	);

	/**
	 * Toggle visibility of a metric column.
	 */
	function toggleMetricColumn(key: string) {
		if (visibleMetricKeys.has(key)) {
			visibleMetricKeys.delete(key);
		} else {
			visibleMetricKeys.add(key);
		}
		visibleMetricKeys = new Set(visibleMetricKeys);
	}

	/**
	 * Handle column header click - toggle sort direction or change column.
	 */
	function handleHeaderClick(column: SortColumn) {
		if (sortColumn === column) {
			sortDirection = sortDirection === "asc" ? "desc" : "asc";
		} else {
			sortColumn = column;
			sortDirection = "desc";
		}
	}

	/**
	 * Get sort indicator for a column.
	 */
	function getSortIndicator(column: string): string {
		if (sortColumn !== column) return "";
		return sortDirection === "asc" ? " \u25B2" : " \u25BC";
	}
</script>

<div class="results-table-container">
	{#if scores.length === 0}
		<div class="empty-state">
			<p>No players to display</p>
		</div>
	{:else}
		<!-- Column visibility toggle panel -->
		{#if archetype && metricKeys.length > 0}
			<div class="column-toggles">
				<span class="toggle-label">Columns:</span>
				{#each metricKeys as key}
					<button
						type="button"
						class="toggle-btn"
						class:active={visibleMetricKeys.has(key)}
						onclick={() => toggleMetricColumn(key)}
					>
						{formatMetricLabel(key)}
					</button>
				{/each}
			</div>
		{/if}

		<table class="results-table">
			<thead>
				<tr>
						<th>
							<button type="button" class="sort-header" onclick={() => handleHeaderClick("name")}>
								Name{getSortIndicator("name")}
							</button>
						</th>
						<th>Club</th>
						<th>
							<button type="button" class="sort-header" onclick={() => handleHeaderClick("age")}>
								Age{getSortIndicator("age")}
							</button>
						</th>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick("transferValue")}>
							Value{getSortIndicator("transferValue")}
						</button>
					</th>
					<th>Positions</th>
					{#each displayedMetricKeys as key}
						<th>
							<button type="button" class="sort-header" onclick={() => handleHeaderClick(`metric.${key}`)}>
								{formatMetricLabel(key)}{getSortIndicator(`metric.${key}`)}
							</button>
						</th>
					{/each}
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick("rawScore")}>
							Raw Score{getSortIndicator("rawScore")}
						</button>
					</th>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick("valueAdjustedScore")}>
							Value-Adj Score{getSortIndicator("valueAdjustedScore")}
						</button>
					</th>
				</tr>
			</thead>
			<tbody>
				{#each sortedScores as score (score.playerId)}
					<tr
						class="player-row"
						onclick={() => onrowclick(score)}
						role="button"
						tabindex="0"
						onkeydown={(e) => {
							if (e.key === "Enter" || e.key === " ") {
								onrowclick(score);
							}
						}}
					>
						<td class="name-cell">{score.name}</td>
						<td class="club-cell">{score.club ?? "\u2014"}</td>
						<td class="age-cell">{score.age ?? "\u2014"}</td>
						<td class="value-cell">{formatValue(score.transferValue)}</td>
						<td class="positions-cell">{score.positions}</td>
						{#each displayedMetricKeys as key}
							<td class="metric-cell">
								{score.metricPercentiles[key]?.toFixed(0) ?? "\u2014"}
							</td>
						{/each}
						<td class="score-cell">{score.rawScore.toFixed(1)}</td>
						<td class="adj-score-cell">{score.valueAdjustedScore.toFixed(1)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
</div>

<style>
	.results-table-container {
		width: 100%;
		overflow-x: auto;
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 3rem;
		color: var(--color-text-secondary, #999);
	}

	.empty-state p {
		margin: 0;
		font-size: 1rem;
	}

	.column-toggles {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		padding: 0.75rem;
		background: var(--color-surface-secondary, #1a1a2e);
		border-bottom: 1px solid var(--color-border, #333);
	}

	.toggle-label {
		display: flex;
		align-items: center;
		font-size: 0.75rem;
		color: var(--color-text-secondary, #999);
		margin-right: 0.25rem;
	}

	.toggle-btn {
		background: transparent;
		border: 1px solid var(--color-border, #333);
		color: var(--color-text-secondary, #999);
		font: inherit;
		font-size: 0.75rem;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		cursor: pointer;
		transition: all 0.15s;
	}

	.toggle-btn:hover {
		border-color: var(--color-accent, #6366f1);
		color: var(--color-text-primary, #fff);
	}

	.toggle-btn.active {
		background: var(--color-accent, #6366f1);
		border-color: var(--color-accent, #6366f1);
		color: #fff;
	}

	.results-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}

	thead {
		background: var(--color-surface-hover, #252540);
		position: sticky;
		top: 0;
		z-index: 1;
	}

	th {
		padding: 0.75rem 0.5rem;
		text-align: left;
		font-weight: 600;
		color: var(--color-text-secondary, #999);
		border-bottom: 1px solid var(--color-border, #333);
	}

	.sort-header {
		background: transparent;
		border: none;
		color: inherit;
		font: inherit;
		font-weight: 600;
		cursor: pointer;
		padding: 0;
		text-align: left;
		display: flex;
		align-items: center;
		gap: 0.25rem;
		transition: color 0.15s;
	}

	.sort-header:hover {
		color: var(--color-text-primary, #fff);
	}

	td {
		padding: 0.75rem 0.5rem;
		border-bottom: 1px solid var(--color-border, #333);
		color: var(--color-text-primary, #fff);
	}

	.player-row {
		cursor: pointer;
		transition: background 0.15s;
	}

	.player-row:hover {
		background: var(--color-surface-hover, #252540);
	}

	.player-row:focus {
		outline: 2px solid var(--color-accent, #6366f1);
		outline-offset: -2px;
	}

	.name-cell {
		font-weight: 500;
	}

	.club-cell {
		color: var(--color-text-secondary, #999);
	}

	.positions-cell {
		color: var(--color-text-secondary, #999);
		font-size: 0.8rem;
	}

	.age-cell {
		color: var(--color-text-secondary, #999);
	}

	.value-cell {
		font-family: "SF Mono", "Fira Code", monospace;
	}

	.metric-cell {
		text-align: right;
		font-family: "SF Mono", "Fira Code", monospace;
		color: var(--color-text-secondary, #999);
	}

	.score-cell {
		text-align: right;
		font-family: "SF Mono", "Fira Code", monospace;
	}

	.adj-score-cell {
		text-align: right;
		font-family: "SF Mono", "Fira Code", monospace;
		font-weight: 600;
		color: var(--color-accent, #6366f1);
	}
</style>
