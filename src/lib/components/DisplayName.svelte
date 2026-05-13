<script lang="ts">
	let { name }: { name: string | null } = $props();
</script>

{#if name}
	{name}
{:else}
	<span class="font-normal tracking-tight italic">Someone</span>
{/if}
