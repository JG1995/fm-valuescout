<script lang="ts">
	import type { Archetype } from "$lib/types/archetype";
	import { PITCH_POSITIONS } from "./pitch-positions";
	import PositionSlot from "./PositionSlot.svelte";

	interface Props {
		/** Currently selected archetypes for each position slot */
		selectedArchetypes: Record<string, Archetype | null>;
		/** Called when user clicks a position to select an archetype */
		onslotclick: (slotId: string) => void;
	}

	let { selectedArchetypes, onslotclick }: Props = $props();

	// SVG viewBox dimensions (portrait orientation for football pitch)
	const VIEWBOX_WIDTH = 600;
	const VIEWBOX_HEIGHT = 900;

	/**
	 * Handle slot click - delegates to parent's onslotclick callback
	 */
	function handleSlotClick(slotId: string) {
		onslotclick(slotId);
	}

	/**
	 * Get archetype for a given slotId
	 */
	function getArchetypeForSlot(slotId: string): Archetype | null {
		return selectedArchetypes[slotId] ?? null;
	}

	// Pre-calculate pitch dimensions for markings
	const PITCH_MARGIN = 25;
	const PITCH_WIDTH = VIEWBOX_WIDTH - PITCH_MARGIN * 2;
	const PITCH_HEIGHT = VIEWBOX_HEIGHT - PITCH_MARGIN * 2;
	const PENALTY_AREA_WIDTH = PITCH_WIDTH * 0.4;
	const GOAL_AREA_WIDTH = PITCH_WIDTH * 0.2;
	const PENALTY_AREA_HEIGHT = 150;
	const GOAL_AREA_HEIGHT = 70;
	const CENTER_CIRCLE_RADIUS = 70;
	const HALF_WIDTH = VIEWBOX_WIDTH / 2;
	const HALF_HEIGHT = VIEWBOX_HEIGHT / 2;
</script>

<div class="pitch-container">
	<svg
		class="pitch-svg"
		viewBox="0 0 {VIEWBOX_WIDTH} {VIEWBOX_HEIGHT}"
		preserveAspectRatio="xMidYMid meet"
		role="img"
		aria-label="Football pitch with position slots"
	>
		<!-- Pitch background (green field) -->
		<rect
			class="pitch-background"
			x="0"
			y="0"
			width={VIEWBOX_WIDTH}
			height={VIEWBOX_HEIGHT}
			fill="#2d8a3e"
		/>

		<!-- Pitch boundary -->
		<rect
			class="pitch-line"
			x={PITCH_MARGIN}
			y={PITCH_MARGIN}
			width={PITCH_WIDTH}
			height={PITCH_HEIGHT}
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Center line -->
		<line
			class="pitch-line"
			x1={PITCH_MARGIN}
			y1={HALF_HEIGHT}
			x2={VIEWBOX_WIDTH - PITCH_MARGIN}
			y2={HALF_HEIGHT}
			stroke="white"
			stroke-width="3"
		/>

		<!-- Center circle -->
		<circle
			class="pitch-circle"
			cx={HALF_WIDTH}
			cy={HALF_HEIGHT}
			r={CENTER_CIRCLE_RADIUS}
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Center spot -->
		<circle
			class="pitch-fill"
			cx={HALF_WIDTH}
			cy={HALF_HEIGHT}
			r="4"
			fill="white"
		/>

		<!-- Top penalty area -->
		<rect
			class="pitch-line"
			x={HALF_WIDTH - PENALTY_AREA_WIDTH / 2}
			y={PITCH_MARGIN}
			width={PENALTY_AREA_WIDTH}
			height={PENALTY_AREA_HEIGHT}
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Top goal area (6-yard box) -->
		<rect
			class="pitch-line"
			x={HALF_WIDTH - GOAL_AREA_WIDTH / 2}
			y={PITCH_MARGIN}
			width={GOAL_AREA_WIDTH}
			height={GOAL_AREA_HEIGHT}
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Top penalty spot -->
		<circle
			class="pitch-fill"
			cx={HALF_WIDTH}
			cy={PITCH_MARGIN + 100}
			r="4"
			fill="white"
		/>

		<!-- Bottom penalty area -->
		<rect
			class="pitch-line"
			x={HALF_WIDTH - PENALTY_AREA_WIDTH / 2}
			y={VIEWBOX_HEIGHT - PITCH_MARGIN - PENALTY_AREA_HEIGHT}
			width={PENALTY_AREA_WIDTH}
			height={PENALTY_AREA_HEIGHT}
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Bottom goal area (6-yard box) -->
		<rect
			class="pitch-line"
			x={HALF_WIDTH - GOAL_AREA_WIDTH / 2}
			y={VIEWBOX_HEIGHT - PITCH_MARGIN - GOAL_AREA_HEIGHT}
			width={GOAL_AREA_WIDTH}
			height={GOAL_AREA_HEIGHT}
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Bottom penalty spot -->
		<circle
			class="pitch-fill"
			cx={HALF_WIDTH}
			cy={VIEWBOX_HEIGHT - PITCH_MARGIN - 100}
			r="4"
			fill="white"
		/>

		<!-- Position slots layer -->
		<g class="position-slots">
			{#each PITCH_POSITIONS as position (position.slotId)}
				{@const archetype = getArchetypeForSlot(position.slotId)}
				<foreignObject
					x={(position.x / 100) * VIEWBOX_WIDTH - 40}
					y={(position.y / 100) * VIEWBOX_HEIGHT - 40}
					width="80"
					height="80"
				>
					<PositionSlot
						{position}
						{archetype}
						onclick={handleSlotClick}
					/>
				</foreignObject>
			{/each}
		</g>
	</svg>
</div>

<style>
	.pitch-container {
		width: 100%;
		max-width: 600px;
		margin: 0 auto;
	}

	.pitch-svg {
		width: 100%;
		height: auto;
		display: block;
		border-radius: 8px;
		box-shadow:
			0 4px 6px rgba(0, 0, 0, 0.1),
			0 0 20px rgba(45, 138, 62, 0.3);
	}
</style>
