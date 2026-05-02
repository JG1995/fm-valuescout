<script lang="ts">
	import type { PlayerScore } from "$lib/scoring";
	import PodiumPosition from "./PodiumPosition.svelte";

	interface Props {
		scores: PlayerScore[];
	}

	let { scores }: Props = $props();

	let top3 = $derived.by(() => {
		const sorted = [...scores].sort((a, b) => b.valueAdjustedScore - a.valueAdjustedScore);
		return sorted.slice(0, 3);
	});
</script>

<div class="podium-view">
	<h2 class="podium-title">Top 3</h2>
	<div class="podium-layout">
		<PodiumPosition position={2} score={top3[1] ?? null} />
		<PodiumPosition position={1} score={top3[0] ?? null} />
		<PodiumPosition position={3} score={top3[2] ?? null} />
	</div>
</div>

<style>
	.podium-view {
		padding: 1rem;
		background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
		border-radius: 12px;
		color: #fff;
	}

	.podium-title {
		text-align: center;
		font-size: 1.5rem;
		font-weight: 600;
		margin-bottom: 2rem;
		color: #f1c40f;
	}

	.podium-layout {
		display: flex;
		justify-content: center;
		align-items: flex-end;
		gap: 0.5rem;
		padding: 1rem;
		min-height: 300px;
	}
</style>
