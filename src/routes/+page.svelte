<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api/core";

	import PitchView from "$lib/components/pitch/PitchView.svelte";
	import ArchetypeSelector from "$lib/components/archetype/ArchetypeSelector.svelte";
	import ArchetypeEditor from "$lib/components/archetype/ArchetypeEditor.svelte";
	import PodiumView from "$lib/components/scouting/PodiumView.svelte";
	import ResultsTable from "$lib/components/scouting/ResultsTable.svelte";

	import { getArchetypeStore } from "$lib/stores/archetype-store.svelte";
	import { getScoutingStore } from "$lib/stores/scouting-store.svelte";
	import { PITCH_POSITIONS } from "$lib/components/pitch/pitch-positions";

	import type { Archetype, ArchetypeRole, MetricWeight } from "$lib/types/archetype";
	import type { PlayerScore } from "$lib/scoring/types";

	const archetypeStore = getArchetypeStore();
	const scoutingStore = getScoutingStore();

	let selectorOpen = $state(false);
	let editorOpen = $state(false);
	let activeSlotId = $state<string | null>(null);
	let editingArchetype = $state<Archetype | null>(null);
	let activeRole = $state<ArchetypeRole>("GK");

	let roleArchetypes = $derived.by(() => {
		return archetypeStore.archetypes.filter((a) => a.role === activeRole);
	});

	let hasPlayers = $derived(scoutingStore.players.length > 0);

	onMount(async () => {
		await archetypeStore.loadAll();

		try {
			const saves = await invoke<{ id: number }[]>("list_saves");
			if (saves.length > 0) {
				const latestSeason = await invoke<{ id: number } | null>(
					"get_latest_season",
					{ saveId: saves[0].id }
				);
				if (latestSeason) {
					await scoutingStore.loadPlayers(latestSeason.id);
				}
			}
		} catch {
			// No saves or seasons yet — empty state
		}
	});

	function handleSlotClick(slotId: string) {
		const pos = PITCH_POSITIONS.find((p) => p.slotId === slotId);
		if (!pos) return;

		activeSlotId = slotId;
		activeRole = pos.role;
		selectorOpen = true;
	}

	function handleArchetypeSelect(archetype: Archetype | null) {
		if (activeSlotId) {
			// Store's selectArchetype takes (archetypeId, slotId)
			if (archetype) {
				archetypeStore.selectArchetype(archetype.id, activeSlotId);
			}
		}
		scoutingStore.selectArchetype(archetype);
		selectorOpen = false;
	}

	function handleCreateArchetype() {
		editingArchetype = null;
		editorOpen = true;
	}

	function handleEditArchetype(archetype: Archetype) {
		editingArchetype = archetype;
		editorOpen = true;
	}

	async function handleDeleteArchetype(archetype: Archetype) {
		await archetypeStore.remove(archetype.id);
	}

	async function handleSaveArchetype(name: string, metrics: MetricWeight[]) {
		let saved: Archetype | null = null;

		if (editingArchetype) {
			saved = await archetypeStore.update(editingArchetype.id, name, metrics);
		} else {
			saved = await archetypeStore.create(name, activeRole, metrics);
		}

		if (saved && activeSlotId) {
			// Store's selectArchetype takes (archetypeId, slotId)
			archetypeStore.selectArchetype(saved.id, activeSlotId);
			scoutingStore.selectArchetype(saved);
		}
		editorOpen = false;
	}

	function handleRowClick(score: PlayerScore) {
		console.log("Navigate to player profile:", score.playerId, score.name);
	}

	const selectedArchetypesForPitch = $derived.by(() => {
		const result: Record<string, Archetype | null> = {};
		for (const [slotId, archetype] of Object.entries(archetypeStore.selectedArchetypes)) {
			result[slotId] = archetype ?? null;
		}
		return result;
	});

	function getSelectedArchetypeIdForSlot(slotId: string | null): number | null {
		if (!slotId) return null;
		const selected = archetypeStore.getSelectedForSlot(slotId);
		return selected?.id ?? null;
	}
</script>

<main class="scouting-page">
	<header class="scouting-header">
		<h1>Moneyball Scouting</h1>
		{#if scoutingStore.error}
			<div class="error-banner">
				<span>{scoutingStore.error}</span>
				<button onclick={() => scoutingStore.clearError()}>✕</button>
			</div>
		{/if}
		{#if archetypeStore.error}
			<div class="error-banner">
				<span>{archetypeStore.error}</span>
				<button onclick={() => archetypeStore.clearError()}>✕</button>
			</div>
		{/if}
	</header>

	{#if !hasPlayers}
		<div class="empty-state">
			<p>No player data loaded.</p>
			<p>Import a CSV to start scouting.</p>
		</div>
	{:else}
		<section class="pitch-section">
			<PitchView
				selectedArchetypes={selectedArchetypesForPitch}
				onslotclick={handleSlotClick}
			/>
		</section>

		{#if scoutingStore.scores.length > 0}
			<section class="podium-section">
				<PodiumView scores={scoutingStore.scores} />
			</section>

			<section class="table-section">
				<ResultsTable
					scores={scoutingStore.scores}
					archetype={scoutingStore.activeArchetype}
					onrowclick={handleRowClick}
				/>
			</section>
		{:else}
			<div class="full-db-section">
				<h2 class="section-title">All Players</h2>
				<p class="section-hint">Select a position on the pitch to see archetype scoring.</p>
				<ResultsTable
					scores={scoutingStore.players.map((p) => ({
						playerId: p.playerId,
						fmUid: p.fmUid,
						name: p.name,
						club: p.club,
						positions: p.positions,
						age: p.age,
						transferValue: p.transferValueHigh,
						role: "",
						rawScore: 0,
						valueAdjustedScore: 0,
						metricPercentiles: {},
					}))}
					archetype={null}
					onrowclick={handleRowClick}
				/>
			</div>
		{/if}
	{/if}

	{#if selectorOpen}
		<ArchetypeSelector
			role={activeRole}
			archetypes={roleArchetypes}
			selectedArchetypeId={getSelectedArchetypeIdForSlot(activeSlotId)}
			onselect={handleArchetypeSelect}
			onedit={handleEditArchetype}
			oncreate={handleCreateArchetype}
			ondelete={handleDeleteArchetype}
			onclose={() => (selectorOpen = false)}
		/>
	{/if}

	{#if editorOpen}
		<ArchetypeEditor
			role={activeRole}
			archetype={editingArchetype}
			onsave={handleSaveArchetype}
			onclose={() => (editorOpen = false)}
		/>
	{/if}
</main>

<style>
	.scouting-page {
		max-width: 800px;
		margin: 0 auto;
		padding: 16px;
		font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
	}

	.scouting-header h1 {
		text-align: center;
		color: #fff;
		font-size: 1.5rem;
		margin: 0 0 16px 0;
	}

	.error-banner {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		background: #b71c1c;
		color: #fff;
		border-radius: 6px;
		font-size: 0.85rem;
		margin-bottom: 12px;
	}

	.error-banner button {
		background: none;
		border: none;
		color: #fff;
		cursor: pointer;
	}

	.empty-state {
		text-align: center;
		padding: 40px 16px;
		color: #888;
	}

	.empty-state p:first-child {
		font-size: 1.1rem;
		color: #aaa;
		margin-bottom: 8px;
	}

	.full-db-section {
		padding: 16px 0;
	}

	.section-title {
		text-align: center;
		color: #fff;
		font-size: 1.1rem;
		margin: 0 0 4px 0;
	}

	.section-hint {
		text-align: center;
		color: #666;
		font-size: 0.85rem;
		margin: 0 0 16px 0;
	}

	.pitch-section {
		margin-bottom: 24px;
	}

	.podium-section {
		margin-bottom: 16px;
	}

	.table-section {
		margin-bottom: 32px;
	}

	@media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #1a1a1a;
		}
	}
</style>
