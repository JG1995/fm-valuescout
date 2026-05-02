<script lang="ts">
	import { onMount } from "svelte";
	import type { Archetype } from "$lib/types/archetype";
	import type { ParsedPlayer, ScoredPlayer } from "$lib/scoring/score";
	import { scorePlayer } from "$lib/scoring/score";
	import { getArchetypeStore } from "$lib/stores/archetype-store.svelte";
	import { getLatestSeason, getPlayersForSeason } from "$lib/api/players";
	import PitchView from "$lib/components/pitch/PitchView.svelte";
	import ArchetypeSelector from "$lib/components/archetype/ArchetypeSelector.svelte";
	import PodiumView from "$lib/components/scouting/PodiumView.svelte";
	import ResultsTable from "$lib/components/scouting/ResultsTable.svelte";

	// ── State ─────────────────────────────────────────────────────────────────

	// Selected archetypes per position slot
	let selectedArchetypes = $state<Record<string, Archetype | undefined>>({});

	// Selector modal state
	let selectorOpen = $state(false);
	let selectorPosition = $state<string | null>(null);

	// Players loaded from backend
	let players = $state<ParsedPlayer[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// ── Derived: archetype store ───────────────────────────────────────────────

	const archetypeStore = getArchetypeStore();
	const archetypes = $derived(archetypeStore.archetypes);

	// ── State: active position (most recently selected) ──────────────────────

	let activePosition = $state<string | null>(null);

	// ── Derived: active archetype ─────────────────────────────────────────────

	const currentArchetype = $derived.by(() => {
		if (!activePosition) return undefined;
		return selectedArchetypes[activePosition];
	});

	// ── Derived: percentiles for scoring ───────────────────────────────────────

	/**
	 * Compute percentile distributions for each metric across all players.
	 * This is used to normalize player scores relative to the dataset.
	 */
	const percentiles = $derived.by(() => {
		if (players.length === 0) return new Map<string, number[]>();

		const distributions = new Map<string, Set<number>>();

		for (const player of players) {
			// Iterate through all known metric keys
			const metricKeys = [
				"attacking.goals_per_90",
				"attacking.xg_per_90",
				"attacking.np_xg_per_90",
				"attacking.shots_per_90",
				"attacking.shots_on_target_per_90",
				"chance_creation.assists_per_90",
				"chance_creation.xa_per_90",
				"chance_creation.key_passes_per_90",
				"chance_creation.progressive_passes_per_90",
				"chance_creation.pass_completion_rate",
				"movement.dribbles_per_90",
				"movement.distance_per_90",
				"defending.tackles_per_90",
				"defending.interceptions_per_90",
				"defending.pressures_per_90",
				"defending.clearances_per_90",
				"defending.tackle_completion_rate",
				"defending.pressure_completion_rate",
				"aerial.aerial_challenge_rate",
				"aerial.aerial_duels_per_90",
				"discipline.fouls_made_per_90",
				"discipline.fouls_against_per_90",
				"match_outcome.win_rate",
				"match_outcome.average_rating",
			];

			for (const key of metricKeys) {
				const value = player[key];
				if (typeof value === "number" && !isNaN(value)) {
					if (!distributions.has(key)) {
						distributions.set(key, new Set());
					}
					distributions.get(key)!.add(value);
				}
			}
		}

		// Convert sets to sorted arrays
		const result = new Map<string, number[]>();
		for (const [key, set] of distributions) {
			result.set(key, [...set].sort((a, b) => a - b));
		}
		return result;
	});

	/**
	 * Compute median transfer value for value adjustment.
	 */
	const medianTransferValue = $derived.by(() => {
		if (players.length === 0) return 1;
		const values = players
			.map((p) => p.transfer_value)
			.filter((v) => v > 0)
			.sort((a, b) => a - b);
		if (values.length === 0) return 1;
		const mid = Math.floor(values.length / 2);
		return values.length % 2 === 0 ? (values[mid - 1] + values[mid]) / 2 : values[mid];
	});

	// ── Derived: scored players (all, sorted by value-adjusted score) ───────────

	const allScoredPlayers = $derived.by(() => {
		if (!currentArchetype || players.length === 0) return [];

		const scored: ScoredPlayer[] = [];
		for (const player of players) {
			const result = scorePlayer(player, currentArchetype, percentiles, medianTransferValue);
			scored.push(result);
		}

		// Sort by value-adjusted score descending
		scored.sort((a, b) => b.valueAdjustedScore - a.valueAdjustedScore);
		return scored;
	});

	// ── Derived: podium players (top 3) ─────────────────────────────────────────

	const podiumPlayers = $derived.by(() => {
		return allScoredPlayers.slice(0, 3);
	});

	// ── Derived: table players (remaining) ───────────────────────────────────────

	const tablePlayers = $derived.by(() => {
		return allScoredPlayers.slice(3);
	});

	// ── Event handlers ─────────────────────────────────────────────────────────

	/**
	 * Handle clicking a position slot on the pitch.
	 * Opens the archetype selector for that position.
	 */
	function handlePositionClick(position: string) {
		activePosition = position;
		selectorPosition = position;
		selectorOpen = true;
	}

	/**
	 * Handle selecting an archetype from the modal.
	 */
	async function handleArchetypeSelect(archetypeId: number) {
		if (!selectorPosition) return;

		// Get the full archetype from the store
		const archetype = archetypes.find((a) => a.id === archetypeId);
		if (archetype) {
			selectedArchetypes = { ...selectedArchetypes, [selectorPosition]: archetype };
		}

		selectorOpen = false;
		selectorPosition = null;
	}

	/**
	 * Handle closing the selector modal without selecting.
	 */
	function handleSelectorClose() {
		selectorOpen = false;
		selectorPosition = null;
	}

	/**
	 * Handle clicking on a player row.
	 * Currently a stub - navigates to player profile.
	 */
	function handlePlayerClick(playerId: string) {
		// TODO: Navigate to player profile page
		console.log("Player clicked:", playerId);
	}

	// ── Data loading ───────────────────────────────────────────────────────────

	onMount(async () => {
		loading = true;
		error = null;

		try {
			// Load archetypes
			await archetypeStore.loadAll();

			// Load players from latest season
			const latest = await getLatestSeason();
			if (latest) {
				const playerData = await getPlayersForSeason(latest.id);
				// Extract ParsedPlayer from data field
				players = playerData
					.filter((p) => p.data !== null)
					.map((p) => p.data as unknown as ParsedPlayer);
			}
		} catch (err) {
			error = err instanceof Error ? err.message : "Failed to load data";
		} finally {
			loading = false;
		}
	});

	// ── Derived: has any archetype selected ────────────────────────────────────

	const hasSelection = $derived(Object.values(selectedArchetypes).some((a) => a !== undefined));
</script>

<div class="scouting-page">
	<header class="page-header">
		<h1>Moneyball Scouting</h1>
		<p class="subtitle">Select archetypes for each position to find the best value players</p>
	</header>

	{#if loading}
		<div class="loading-state">
			<p>Loading players...</p>
		</div>
	{:else if error}
		<div class="error-state">
			<p>Error: {error}</p>
		</div>
	{:else}
		<!-- Pitch Section -->
		<section class="pitch-section">
			<PitchView {selectedArchetypes} onSelectArchetype={handlePositionClick} />
		</section>

		<!-- Archetype Selector Modal -->
		<ArchetypeSelector
			open={selectorOpen}
			position={selectorPosition ?? ""}
			{archetypes}
			onSelect={handleArchetypeSelect}
			onClose={handleSelectorClose}
		/>

		<!-- Results Section -->
		<section class="results-section">
			{#if hasSelection && currentArchetype}
				<!-- Show scoring results when an archetype is selected -->
				<PodiumView scoredPlayers={podiumPlayers} archetypeName={currentArchetype.name} />
				<ResultsTable scoredPlayers={tablePlayers} onPlayerClick={handlePlayerClick} />
			{:else if players.length === 0}
				<!-- Empty state: no players loaded -->
				<div class="empty-state">
					<p>No players loaded. Import a season to get started.</p>
				</div>
			{:else}
				<!-- Full database view: all players sorted by name -->
				<div class="full-database">
					<h2 class="section-heading">All Players</h2>
					<ResultsTable
						scoredPlayers={players.map((p) => ({
							player: p,
							archetypeId: 0,
							rawScore: 0,
							valueAdjustedScore: 0,
							percentileByMetric: new Map(),
						}))}
						onPlayerClick={handlePlayerClick}
					/>
				</div>
			{/if}
		</section>
	{/if}
</div>

<style>
	.scouting-page {
		padding: 1.5rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	.page-header {
		text-align: center;
		margin-bottom: 2rem;
	}

	.page-header h1 {
		font-size: 2rem;
		font-weight: 700;
		color: var(--color-text-primary, #fff);
		margin: 0 0 0.5rem 0;
	}

	.subtitle {
		font-size: 1rem;
		color: var(--color-text-secondary, #999);
		margin: 0;
	}

	.loading-state,
	.error-state,
	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4rem 2rem;
		color: var(--color-text-secondary, #999);
	}

	.error-state {
		color: #ef4444;
	}

	.pitch-section {
		margin-bottom: 2rem;
	}

	.results-section {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.full-database {
		background: var(--color-surface, #1a1a2e);
		border-radius: 12px;
		padding: 1rem;
	}

	.section-heading {
		font-size: 1.25rem;
		font-weight: 600;
		color: var(--color-text-primary, #fff);
		margin: 0 0 1rem 0;
		padding: 0 0.5rem;
	}
</style>