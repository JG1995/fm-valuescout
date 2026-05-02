<script lang="ts">
	import type { ScoredPlayer } from "$lib/scoring/score";

	interface PodiumViewProps {
		/** Pre-sorted scored players, top 3 (already sorted by value-adjusted score) */
		scoredPlayers: ScoredPlayer[];
		/** Name of the archetype for the heading */
		archetypeName: string;
	}

	let { scoredPlayers, archetypeName }: PodiumViewProps = $props();

	// Extract top 3 players for podium
	const topPlayers = $derived.by(() => {
		return scoredPlayers.slice(0, 3);
	});

	// Position labels: 2nd (index 0), 1st (index 1), 3rd (index 2)
	const podiumPositions = [
		{ place: 2, label: "2nd", medal: "silver" },
		{ place: 1, label: "1st", medal: "gold" },
		{ place: 3, label: "3rd", medal: "bronze" },
	];

	/**
	 * Format a score to 1 decimal place
	 */
	function formatScore(score: number): string {
		return score.toFixed(1);
	}

	/**
	 * Get the player's club if available
	 */
	function getClub(player: ScoredPlayer): string | undefined {
		return player.player["club"] as string | undefined;
	}
</script>

<div class="podium-container">
	<h2 class="podium-heading">
		Top Players for {archetypeName}
	</h2>

	{#if topPlayers.length === 0}
		<div class="empty-state">
			<p>No players available</p>
		</div>
	{:else}
		<div class="podium-layout">
			{#each podiumPositions as position, index}
				{@const player = topPlayers[index]}
				{@const hasPlayer = player !== undefined}

				<div
					class="podium-item"
					class:position-1={position.place === 1}
					class:position-2={position.place === 2}
					class:position-3={position.place === 3}
					class:has-player={hasPlayer}
					data-position={position.place}
				>
					{#if hasPlayer}
						<!-- Medal indicator -->
						<div class="medal-indicator medal-{position.medal}">
							<span class="medal-label">{position.label}</span>
						</div>

						<!-- Podium stand -->
						<div class="podium-stand">
							<div class="stand-top"></div>
							<div class="stand-base"></div>
						</div>

						<!-- Player info -->
						<div class="player-card">
							<h3 class="player-name">{player.player.name}</h3>
							{#if getClub(player)}
								<p class="player-club">{getClub(player)}</p>
							{/if}
							<div class="player-scores">
								<div class="score-row">
									<span class="score-label">Raw:</span>
									<span class="score-value">{formatScore(player.rawScore)}</span>
								</div>
								<div class="score-row">
									<span class="score-label">Value:</span>
									<span class="score-value">{formatScore(player.valueAdjustedScore)}</span>
								</div>
							</div>
						</div>
					{:else}
						<!-- Empty slot -->
						<div class="empty-slot">
							<span class="position-placeholder">{position.label}</span>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.podium-container {
		padding: 1rem;
		background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
		border-radius: 12px;
		color: #fff;
	}

	.podium-heading {
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

	.podium-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 140px;
	}

	/* Position heights: 1st tallest, 2nd/3rd shorter */
	.podium-item.position-1 {
		height: 220px;
	}

	.podium-item.position-2 {
		height: 170px;
	}

	.podium-item.position-3 {
		height: 170px;
	}

	.medal-indicator {
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

	.medal-label {
		font-weight: 700;
	}

	.podium-stand {
		display: flex;
		flex-direction: column;
		align-items: center;
		width: 100%;
	}

	.stand-top {
		width: 100%;
		height: 60px;
		background: linear-gradient(180deg, #3d5a80 0%, #293241 100%);
		border-radius: 8px 8px 0 0;
	}

	.stand-base {
		width: 100%;
		height: 40px;
		background: linear-gradient(180deg, #293241 0%, #1a1a2e 100%);
	}

	/* First place stand is taller */
	.position-1 .stand-top {
		height: 80px;
		background: linear-gradient(180deg, #4a6fa5 0%, #3d5a80 100%);
	}

	.player-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 0.75rem 0.5rem;
		text-align: center;
	}

	.player-name {
		font-size: 0.875rem;
		font-weight: 600;
		margin: 0 0 0.25rem 0;
		color: #fff;
		word-break: break-word;
	}

	.player-club {
		font-size: 0.75rem;
		color: #98c1d9;
		margin: 0 0 0.5rem 0;
	}

	.player-scores {
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

	.empty-state {
		text-align: center;
		padding: 3rem;
		color: #a0a0a0;
	}

	.empty-slot {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		opacity: 0.5;
	}

	.position-placeholder {
		font-size: 1.5rem;
		color: #555;
	}
</style>
