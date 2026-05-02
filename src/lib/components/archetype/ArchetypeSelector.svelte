<script lang="ts">
	import type { Archetype } from "$lib/types/archetype";

	interface ArchetypeSelectorProps {
		open: boolean;
		position: string;
		archetypes: Archetype[];
		onSelect: (archetypeId: number) => void;
		onClose: () => void;
	}

	let { open, position, archetypes, onSelect, onClose }: ArchetypeSelectorProps = $props();

	/**
	 * Group archetypes by role using $derived.by
	 */
	const groupedByRole = $derived.by(() => {
		const grouped: Record<string, Archetype[]> = {};
		for (const arch of archetypes) {
			if (!grouped[arch.role]) {
				grouped[arch.role] = [];
			}
			grouped[arch.role].push(arch);
		}
		return grouped;
	});

	/**
	 * Handle backdrop click - close if clicking the overlay, not the modal content
	 */
	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			onClose();
		}
	}

	/**
	 * Handle archetype selection
	 */
	function handleSelect(archetypeId: number) {
		onSelect(archetypeId);
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="overlay" onclick={handleBackdropClick} role="presentation">
		<dialog open aria-labelledby="archetype-selector-heading">
			<header>
				<h2 id="archetype-selector-heading">
					Select Archetype for <span class="position">{position}</span>
				</h2>
				<button type="button" class="close-button" onclick={onClose} aria-label="Close">
					&times;
				</button>
			</header>

			<div class="content">
				{#if archetypes.length === 0}
					<p class="empty-state">No archetypes available for this position.</p>
				{:else}
					<div class="archetype-list">
						{#each Object.entries(groupedByRole) as [role, roleArchetypes]}
							<div class="role-group">
								<h3 class="role-label">{role}</h3>
								<div class="archetypes">
									{#each roleArchetypes as archetype}
										<button
											type="button"
											class="archetype-button"
											onclick={() => handleSelect(archetype.id)}
										>
											<span class="archetype-name">{archetype.name}</span>
											<span class="metric-count">{archetype.metrics.length} metrics</span>
										</button>
									{/each}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<footer>
				<button type="button" class="cancel-button" onclick={onClose}>Cancel</button>
			</footer>
		</dialog>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	dialog {
		background: var(--color-surface, #1a1a2e);
		border: 1px solid var(--color-border, #333);
		border-radius: 8px;
		padding: 0;
		max-width: 500px;
		width: 90%;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
	}

	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem 1.25rem;
		border-bottom: 1px solid var(--color-border, #333);
	}

	h2 {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: var(--color-text-primary, #fff);
	}

	.position {
		color: var(--color-accent, #6366f1);
	}

	.close-button {
		background: transparent;
		border: none;
		font-size: 1.5rem;
		color: var(--color-text-secondary, #999);
		cursor: pointer;
		padding: 0.25rem;
		line-height: 1;
	}

	.close-button:hover {
		color: var(--color-text-primary, #fff);
	}

	.content {
		flex: 1;
		overflow-y: auto;
		padding: 1rem 1.25rem;
	}

	.empty-state {
		text-align: center;
		color: var(--color-text-secondary, #999);
		padding: 2rem;
	}

	.archetype-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.role-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.role-label {
		margin: 0;
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text-secondary, #999);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.archetypes {
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.archetype-button {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 0.75rem 1rem;
		background: var(--color-surface-hover, #252540);
		border: 1px solid var(--color-border, #333);
		border-radius: 6px;
		cursor: pointer;
		text-align: left;
		transition: background 0.15s, border-color 0.15s;
	}

	.archetype-button:hover {
		background: var(--color-surface-active, #303050);
		border-color: var(--color-accent, #6366f1);
	}

	.archetype-name {
		font-weight: 500;
		color: var(--color-text-primary, #fff);
	}

	.metric-count {
		font-size: 0.8125rem;
		color: var(--color-text-secondary, #999);
	}

	footer {
		padding: 1rem 1.25rem;
		border-top: 1px solid var(--color-border, #333);
		display: flex;
		justify-content: flex-end;
	}

	.cancel-button {
		padding: 0.5rem 1rem;
		background: transparent;
		border: 1px solid var(--color-border, #333);
		border-radius: 6px;
		color: var(--color-text-secondary, #999);
		cursor: pointer;
		font-size: 0.875rem;
		transition: background 0.15s, color 0.15s;
	}

	.cancel-button:hover {
		background: var(--color-surface-hover, #252540);
		color: var(--color-text-primary, #fff);
	}
</style>
