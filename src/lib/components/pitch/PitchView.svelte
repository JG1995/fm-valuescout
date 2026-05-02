<script lang="ts">
	import type { Archetype } from "$lib/types/archetype";

	interface Props {
		/** Currently selected archetypes for each position slot */
		selectedArchetypes: Record<string, Archetype | undefined>;
		/** Called when user clicks a position to select an archetype */
		onSelectArchetype: (position: string) => void;
	}

	let { selectedArchetypes, onSelectArchetype }: Props = $props();

	/** 4-4-2 formation position keys */
	const POSITIONS = [
		{ key: "ST", label: "ST" },
		{ key: "LW", label: "LW" },
		{ key: "RW", label: "RW" },
		{ key: "CM1", label: "CM" },
		{ key: "DM", label: "DM" },
		{ key: "CM2", label: "CM" },
		{ key: "LB", label: "LB" },
		{ key: "RB", label: "RB" },
		{ key: "LCB", label: "CB" },
		{ key: "RCB", label: "CB" },
		{ key: "GK", label: "GK" },
	];

	// SVG viewBox dimensions
	const VIEWBOX_WIDTH = 600;
	const VIEWBOX_HEIGHT = 900;

	/** Calculate position on pitch (x: 0-1, y: 0-1) */
	function getPositionCoords(position: string): { x: number; y: number } {
		const positions: Record<string, { x: number; y: number }> = {
			// 4-4-2 formation
			ST: { x: 0.5, y: 0.1 }, // Striker - top center
			LW: { x: 0.2, y: 0.25 }, // Left wing
			RW: { x: 0.8, y: 0.25 }, // Right wing
			CM1: { x: 0.35, y: 0.45 }, // Central midfielder 1
			DM: { x: 0.5, y: 0.55 }, // Defensive midfielder
			CM2: { x: 0.65, y: 0.45 }, // Central midfielder 2
			LB: { x: 0.15, y: 0.6 }, // Left back
			RB: { x: 0.85, y: 0.6 }, // Right back
			LCB: { x: 0.35, y: 0.75 }, // Left center back
			RCB: { x: 0.65, y: 0.75 }, // Right center back
			GK: { x: 0.5, y: 0.92 }, // Goalkeeper
		};
		return positions[position] ?? { x: 0.5, y: 0.5 };
	}

	/** Get display text for a position slot */
	function getSlotText(position: string): string {
		const archetype = selectedArchetypes[position];
		return archetype?.name ?? "Select";
	}

	/** Handle position slot click */
	function handleSlotClick(position: string) {
		onSelectArchetype(position);
	}

	/** Convert position coords to SVG coordinates */
	function toSvgCoords(x: number, y: number): { svgX: number; svgY: number } {
		return {
			svgX: x * VIEWBOX_WIDTH,
			svgY: y * VIEWBOX_HEIGHT,
		};
	}
</script>

<div class="pitch-container">
	<svg
		class="pitch-svg"
		viewBox="0 0 {VIEWBOX_WIDTH} {VIEWBOX_HEIGHT}"
		preserveAspectRatio="xMidYMid meet"
	>
		<!-- Pitch background (green field) -->
		<rect
			x="0"
			y="0"
			width={VIEWBOX_WIDTH}
			height={VIEWBOX_HEIGHT}
			fill="#2d8a3e"
		/>

		<!-- Pitch markings -->
		<!-- Outer boundary -->
		<rect
			x="25"
			y="25"
			width={VIEWBOX_WIDTH - 50}
			height={VIEWBOX_HEIGHT - 50}
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Center line -->
		<line
			x1="25"
			y1={VIEWBOX_HEIGHT / 2}
			x2={VIEWBOX_WIDTH - 25}
			y2={VIEWBOX_HEIGHT / 2}
			stroke="white"
			stroke-width="3"
		/>

		<!-- Center circle -->
		<circle
			cx={VIEWBOX_WIDTH / 2}
			cy={VIEWBOX_HEIGHT / 2}
			r="70"
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Center spot -->
		<circle cx={VIEWBOX_WIDTH / 2} cy={VIEWBOX_HEIGHT / 2} r="4" fill="white" />

		<!-- Top penalty area -->
		<rect
			x="150"
			y="25"
			width={VIEWBOX_WIDTH - 300}
			height="150"
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Top goal area (6-yard box) -->
		<rect
			x="230"
			y="25"
			width={VIEWBOX_WIDTH - 460}
			height="70"
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Top penalty spot -->
		<circle cx={VIEWBOX_WIDTH / 2} cy="130" r="4" fill="white" />

		<!-- Bottom penalty area -->
		<rect
			x="150"
			y={VIEWBOX_HEIGHT - 175}
			width={VIEWBOX_WIDTH - 300}
			height="150"
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Bottom goal area (6-yard box) -->
		<rect
			x="230"
			y={VIEWBOX_HEIGHT - 95}
			width={VIEWBOX_WIDTH - 460}
			height="70"
			fill="none"
			stroke="white"
			stroke-width="3"
		/>

		<!-- Bottom penalty spot -->
		<circle cx={VIEWBOX_WIDTH / 2} cy={VIEWBOX_HEIGHT - 130} r="4" fill="white" />

		<!-- Position slot buttons -->
		{#each POSITIONS as pos (pos.key)}
			{@const coords = getPositionCoords(pos.key)}
			{@const svgCoords = toSvgCoords(coords.x, coords.y)}
			{@const text = getSlotText(pos.key)}
			{@const isSelected = selectedArchetypes[pos.key] !== undefined}

			<g class="position-slot" transform="translate({svgCoords.svgX}, {svgCoords.svgY})">
				<!-- Slot circle background -->
				<circle
					cx="0"
					cy="0"
					r="40"
					fill={isSelected ? "#1a5c2a" : "rgba(0, 0, 0, 0.4)"}
					stroke={isSelected ? "#ffd700" : "white"}
					stroke-width="2"
					cursor="pointer"
				/>

				<!-- Slot content (text) - clickable area -->
				<foreignObject x="-40" y="-40" width="80" height="80">
					<button
						type="button"
						class="slot-button"
						class:selected={isSelected}
						data-position={pos.key}
						onclick={() => handleSlotClick(pos.key)}
					>
						<span class="slot-label">{pos.label}</span>
						<span class="slot-value">{text}</span>
					</button>
				</foreignObject>
			</g>
		{/each}
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

	.position-slot {
		transition: transform 0.2s ease;
	}

	.position-slot:hover {
		transform: scale(1.1);
	}

	.slot-button {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		background: transparent;
		border: none;
		cursor: pointer;
		color: white;
		font-family: "Segoe UI", system-ui, sans-serif;
		text-align: center;
		padding: 4px;
	}

	.slot-label {
		font-size: 11px;
		font-weight: 600;
		opacity: 0.8;
		line-height: 1;
	}

	.slot-value {
		font-size: 10px;
		font-weight: 400;
		line-height: 1.2;
		max-width: 70px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.slot-button.selected .slot-value {
		color: #ffd700;
		font-weight: 600;
	}
</style>
