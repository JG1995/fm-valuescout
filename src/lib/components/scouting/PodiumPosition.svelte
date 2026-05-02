<script lang="ts">
	import type { PlayerScore } from "$lib/scoring";

	interface Props {
		position: 1 | 2 | 3;
		score: PlayerScore | null;
	}

	let { position, score }: Props = $props();

	const labels: Record<1 | 2 | 3, string> = {
		1: "1st",
		2: "2nd",
		3: "3rd",
	};

	const platformClasses: Record<1 | 2 | 3, string> = {
		1: "tallest",
		2: "medium",
		3: "short",
	};

	const medalClasses: Record<1 | 2 | 3, string> = {
		1: "gold",
		2: "silver",
		3: "bronze",
	};

	function formatScore(score: number): string {
		return score.toFixed(1);
	}
</script>

<div class="podium-position" class:has-player={score !== null}>
	<div class="player-card" class:placeholder={score === null}>
		{#if score !== null}
			<div class="position-badge medal-{medalClasses[position]}">
				<span class="badge-label">{labels[position]}</span>
			</div>

			<div class="player-info">
				<span class="player-name">{score.name}</span>
				<span class="player-club">{score.club ?? "—"}</span>
			</div>

			<div class="scores">
				<div class="score-row">
					<span class="score-label">Score:</span>
					<span class="score-value">{formatScore(score.rawScore)}</span>
				</div>
				<div class="score-row">
					<span class="score-label">Value Adj.:</span>
					<span class="score-value">{formatScore(score.valueAdjustedScore)}</span>
				</div>
			</div>
		{:else}
			<span class="placeholder-text">{labels[position]}</span>
		{/if}
	</div>

	<div class="podium-platform {platformClasses[position]}">
		<span class="position-number">{position}</span>
	</div>
</div>

<style>
	.podium-position {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 140px;
	}

	.player-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 0.75rem 0.5rem;
		text-align: center;
		min-height: 120px;
		width: 100%;
	}

	.player-card.placeholder {
		justify-content: center;
		opacity: 0.5;
	}

	.placeholder-text {
		font-size: 1.5rem;
		color: #555;
	}

	.position-badge {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: 700;
		font-size: 0.875rem;
		margin-bottom: 0.5rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
	}

	.badge-label {
		font-weight: 700;
	}

	.medal-gold {
		background: linear-gradient(135deg, #ffd700 0%, #ffaa00 100%);
		color: #1a1a2e;
	}

	.medal-silver {
		background: linear-gradient(135deg, #c0c0c0 0%, #a0a0a0 100%);
		color: #1a1a2e;
	}

	.medal-bronze {
		background: linear-gradient(135deg, #cd7f32 0%, #a0522d 100%);
		color: #fff;
	}

	.player-info {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		margin-bottom: 0.5rem;
	}

	.player-name {
		font-size: 0.875rem;
		font-weight: 600;
		color: #fff;
		word-break: break-word;
	}

	.player-club {
		font-size: 0.75rem;
		color: #98c1d9;
	}

	.scores {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		width: 100%;
	}

	.score-row {
		display: flex;
		justify-content: space-between;
		font-size: 0.75rem;
	}

	.score-label {
		color: #a0a0a0;
	}

	.score-value {
		color: #f1c40f;
		font-weight: 600;
	}

	.podium-platform {
		width: 100%;
		border-radius: 8px 8px 0 0;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.tallest {
		height: 60px;
		background: linear-gradient(to bottom, #ffd700, #b8860b);
	}

	.medium {
		height: 45px;
		background: linear-gradient(to bottom, #c0c0c0, #808080);
	}

	.short {
		height: 30px;
		background: linear-gradient(to bottom, #cd7f32, #8b4513);
	}

	.position-number {
		font-weight: 700;
		font-size: 1rem;
		color: rgba(0, 0, 0, 0.5);
	}
</style>
