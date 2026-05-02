<script lang="ts">
	import type { Archetype } from "$lib/types/archetype";
	import type { PitchPosition } from "./pitch-positions";

	interface Props {
		position: PitchPosition;
		archetype: Archetype | null;
		onclick: (slotId: string) => void;
	}

	let { position, archetype, onclick }: Props = $props();

	const isSelected = $derived(archetype !== null);
	const displayText = $derived(archetype?.name ?? "Select");

	function handleClick() {
		onclick(position.slotId);
	}
</script>

<div
	class="pitch-slot"
	style="left: {position.x}%; top: {position.y}%;"
	data-slot-id={position.slotId}
>
	<button
		type="button"
		class="slot-button"
		class:selected={isSelected}
		class:selected-green={isSelected}
		onclick={handleClick}
		aria-label="Select archetype for {position.label}"
	>
		<span class="slot-label">{position.label}</span>
		<span class="slot-value">{displayText}</span>
	</button>
</div>

<style>
	.pitch-slot {
		position: absolute;
		transform: translate(-50%, -50%);
		cursor: pointer;
		transition: transform 0.15s ease;
		width: 80px;
		height: 80px;
	}

	.pitch-slot:hover {
		transform: translate(-50%, -50%) scale(1.1);
	}

	.slot-button {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		background: rgba(0, 0, 0, 0.4);
		border: 2px solid white;
		border-radius: 50%;
		cursor: pointer;
		color: white;
		font-family: "Segoe UI", system-ui, sans-serif;
		text-align: center;
		padding: 4px;
		transition:
			background-color 0.2s ease,
			border-color 0.2s ease;
	}

	.slot-button:hover {
		background: rgba(0, 0, 0, 0.6);
	}

	.slot-button:focus {
		outline: 2px solid #ffd700;
		outline-offset: 2px;
	}

	.slot-button:focus:not(:focus-visible) {
		outline: none;
	}

	/* Selected state: green border */
	.slot-button.selected-green {
		border-color: #4caf50;
		background: rgba(26, 92, 42, 0.8);
	}

	.slot-button.selected-green .slot-value {
		color: #ffd700;
		font-weight: 600;
	}

	.slot-label {
		font-size: 11px;
		font-weight: 600;
		opacity: 0.8;
		line-height: 1;
		margin-bottom: 2px;
	}

	.slot-value {
		font-size: 10px;
		font-weight: 400;
		line-height: 1.2;
		max-width: 65px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
