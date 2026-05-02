<script lang="ts">
	import type { Archetype, ArchetypeRole } from "$lib/types/archetype";

	interface Props {
		role: ArchetypeRole;
		archetypes: Archetype[];
		selectedArchetypeId: number | null;
		onselect: (archetype: Archetype | null) => void;
		onedit: (archetype: Archetype) => void;
		oncreate: () => void;
		ondelete: (archetype: Archetype) => void;
		onclose: () => void;
	}

	let { role, archetypes, selectedArchetypeId, onselect, onedit, oncreate, ondelete, onclose }: Props = $props();

	const filteredArchetypes = $derived.by(() => archetypes.filter((a) => a.role === role));

	function handleDelete(e: MouseEvent, arch: Archetype) {
		e.stopPropagation();
		if (arch.is_default) return; // Cannot delete defaults
		const confirmed = window.confirm(`Delete archetype "${arch.name}"? This cannot be undone.`);
		if (confirmed) {
			ondelete(arch);
		}
	}

	function handleEdit(e: MouseEvent, arch: Archetype) {
		e.stopPropagation();
		onedit(arch);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="selector-overlay"
	onclick={onclose}
	onkeydown={(e) => e.key === "Escape" && onclose()}
	role="dialog"
	aria-modal="true"
	aria-label="Archetype selector"
	tabindex="-1"
>
	<!-- Keyboard handler prevents Escape from bubbling to overlay and closing modal -->
	<div class="selector-panel" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
		<div class="selector-header">
			<h3>Select Archetype ({role})</h3>
			<button class="close-btn" onclick={onclose} aria-label="Close">✕</button>
		</div>

		<ul class="archetype-list">
			{#each filteredArchetypes as arch (arch.id)}
				<li>
					<button
						class="archetype-option"
						class:selected={selectedArchetypeId === arch.id}
						onclick={() => onselect(arch)}
					>
						<span class="arch-name">{arch.name}</span>
						{#if arch.is_default}
							<span class="badge">Default</span>
						{/if}
					</button>
					<div class="arch-actions">
						<button
							class="icon-btn"
							onclick={(e) => handleEdit(e, arch)}
							title="Edit"
							aria-label="Edit {arch.name}"
						>✏</button>
						{#if !arch.is_default}
							<button
								class="icon-btn delete-btn"
								onclick={(e) => handleDelete(e, arch)}
								title="Delete"
								aria-label="Delete {arch.name}"
							>🗑</button>
						{/if}
					</div>
				</li>
			{:else}
				<li class="empty-state">No archetypes available for this role.</li>
			{/each}
		</ul>

		<button class="create-btn" onclick={oncreate}>
			+ Create New Archetype
		</button>
	</div>
</div>

<style>
	.selector-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.selector-panel {
		background: var(--color-surface, #1e1e1e);
		border-radius: 12px;
		width: 360px;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
	}

	.selector-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px;
		border-bottom: 1px solid var(--color-border, #333);
	}

	.selector-header h3 {
		margin: 0;
		font-size: 1rem;
		color: var(--color-text-primary, #fff);
	}

	.close-btn {
		background: none;
		border: none;
		color: #999;
		font-size: 1.2rem;
		cursor: pointer;
	}

	.close-btn:hover {
		color: #fff;
	}

	.archetype-list {
		list-style: none;
		margin: 0;
		padding: 8px;
		overflow-y: auto;
		flex: 1;
	}

	.archetype-list li {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.archetype-option {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		background: #2a2a2a;
		border: 1px solid #3a3a3a;
		border-radius: 6px;
		color: #ddd;
		cursor: pointer;
		text-align: left;
		transition: background 0.1s;
	}

	.archetype-option:hover {
		background: #3a3a3a;
	}

	.archetype-option.selected {
		border-color: #4caf50;
		background: rgba(76, 175, 80, 0.15);
	}

	.arch-name {
		flex: 1;
	}

	.badge {
		font-size: 0.65rem;
		background: #444;
		padding: 2px 6px;
		border-radius: 4px;
		color: #aaa;
	}

	.arch-actions {
		display: flex;
		gap: 4px;
	}

	.icon-btn {
		background: none;
		border: none;
		color: #888;
		cursor: pointer;
		font-size: 0.9rem;
		padding: 4px;
	}

	.icon-btn:hover {
		color: #fff;
	}

	.delete-btn:hover {
		color: #ef5350;
	}

	.create-btn {
		margin: 8px;
		padding: 10px;
		background: #333;
		border: 1px dashed #555;
		border-radius: 6px;
		color: #aaa;
		cursor: pointer;
		transition: background 0.1s;
	}

	.create-btn:hover {
		background: #3a3a3a;
		color: #fff;
	}

	.empty-state {
		padding: 2rem;
		text-align: center;
		color: #888;
		font-size: 0.875rem;
	}
</style>
