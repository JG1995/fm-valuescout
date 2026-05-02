<script lang="ts">
	import type { ScoredPlayer } from "$lib/scoring/score";

	interface ResultsTableProps {
		scoredPlayers: ScoredPlayer[];
		onPlayerClick: (playerId: string) => void;
	}

	let { scoredPlayers, onPlayerClick }: ResultsTableProps = $props();

	// Sort state
	let sortColumn = $state<'name' | 'club' | 'age' | 'value' | 'rawScore' | 'valueAdjScore'>('valueAdjScore');
	let sortDirection = $state<'asc' | 'desc'>('desc');

	/**
	 * Sorted players based on current sort column and direction.
	 * Uses $derived.by for complex derived state.
	 */
	const sortedPlayers = $derived.by(() => {
		const sorted = [...scoredPlayers].sort((a, b) => {
			let aVal: number | string;
			let bVal: number | string;

			switch (sortColumn) {
				case 'name':
					aVal = String(a.player.name);
					bVal = String(b.player.name);
					break;
				case 'club':
					aVal = String(a.player.club);
					bVal = String(b.player.club);
					break;
				case 'age':
					aVal = a.player.age;
					bVal = b.player.age;
					break;
				case 'value':
					aVal = a.player.transfer_value;
					bVal = b.player.transfer_value;
					break;
				case 'rawScore':
					aVal = a.rawScore;
					bVal = b.rawScore;
					break;
				case 'valueAdjScore':
					aVal = a.valueAdjustedScore;
					bVal = b.valueAdjustedScore;
					break;
				default:
					aVal = a.valueAdjustedScore;
					bVal = b.valueAdjustedScore;
			}

			// Handle string vs number comparison
			if (typeof aVal === 'string' && typeof bVal === 'string') {
				return aVal.localeCompare(bVal);
			}
			return (aVal as number) - (bVal as number);
		});

		return sortDirection === 'desc' ? sorted : sorted.reverse();
	});

	/**
	 * Handle column header click - toggle sort direction or change column
	 */
	function handleHeaderClick(column: 'name' | 'club' | 'age' | 'value' | 'rawScore' | 'valueAdjScore') {
		if (sortColumn === column) {
			// Toggle direction if same column
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			// Switch to new column, default to descending
			sortColumn = column;
			sortDirection = 'desc';
		}
	}

	/**
	 * Handle row click - notify parent with player id
	 */
	function handleRowClick(playerId: string) {
		onPlayerClick(playerId);
	}

	/**
	 * Format transfer value as currency string
	 */
	function formatCurrency(value: number): string {
		if (value >= 1000000) {
			return `€${(value / 1000000).toFixed(1)}M`;
		} else if (value >= 1000) {
			return `€${(value / 1000).toFixed(0)}K`;
		}
		return `€${value}`;
	}

	/**
	 * Format score to 1 decimal place
	 */
	function formatScore(score: number): string {
		return score.toFixed(1);
	}

	/**
	 * Get sort indicator for a column
	 */
	function getSortIndicator(column: string): string {
		if (sortColumn !== column) return '';
		return sortDirection === 'asc' ? ' ▲' : ' ▼';
	}
</script>

<div class="results-table-container">
	{#if scoredPlayers.length === 0}
		<div class="empty-state">
			<p>No players to display</p>
		</div>
	{:else}
		<table class="results-table">
			<thead>
				<tr>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick('name')}>
							Name{getSortIndicator('name')}
						</button>
					</th>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick('club')}>
							Club{getSortIndicator('club')}
						</button>
					</th>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick('age')}>
							Age{getSortIndicator('age')}
						</button>
					</th>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick('value')}>
							Value{getSortIndicator('value')}
						</button>
					</th>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick('rawScore')}>
							Raw Score{getSortIndicator('rawScore')}
						</button>
					</th>
					<th>
						<button type="button" class="sort-header" onclick={() => handleHeaderClick('valueAdjScore')}>
							Value-Adj Score{getSortIndicator('valueAdjScore')}
						</button>
					</th>
				</tr>
			</thead>
			<tbody>
				{#each sortedPlayers as scoredPlayer (scoredPlayer.player.id)}
					<tr
						class="player-row"
						onclick={() => handleRowClick(scoredPlayer.player.id)}
						role="button"
						tabindex="0"
						onkeydown={(e) => {
							if (e.key === 'Enter' || e.key === ' ') {
								handleRowClick(scoredPlayer.player.id);
							}
						}}
					>
						<td class="name-cell">{scoredPlayer.player.name}</td>
						<td class="club-cell">{scoredPlayer.player.club}</td>
						<td class="age-cell">{scoredPlayer.player.age}</td>
						<td class="value-cell">{formatCurrency(scoredPlayer.player.transfer_value)}</td>
						<td class="score-cell">{formatScore(scoredPlayer.rawScore)}</td>
						<td class="adj-score-cell">{formatScore(scoredPlayer.valueAdjustedScore)}</td>
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

	.age-cell {
		color: var(--color-text-secondary, #999);
	}

	.value-cell {
		font-family: 'SF Mono', 'Fira Code', monospace;
	}

	.score-cell {
		text-align: right;
		font-family: 'SF Mono', 'Fira Code', monospace;
	}

	.adj-score-cell {
		text-align: right;
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-weight: 600;
		color: var(--color-accent, #6366f1);
	}
</style>
