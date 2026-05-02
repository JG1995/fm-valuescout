<script lang="ts">
	import { onMount } from "svelte";
	import { goto } from "$app/navigation";

	onMount(() => {
		goto("/scouting");
	});
</script>

<div class="redirect-page">
	<p>Redirecting to Scouting...</p>
</div>

<style>
	.redirect-page {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 100vh;
		background: var(--color-background, #0f0f1a);
		color: var(--color-text-secondary, #999);
	}
</style>
