<script lang="ts">
	import { format, isToday, startOfToday } from "date-fns";
	import { getMessageContext } from "./context";

	let { dayStart }: { dayStart: number } = $props();

	const { firstInStack, lastInStack, indexInStack, msgOut } = $derived(
		getMessageContext()(),
	);
</script>

<span class="text-center text-xs text-muted-foreground mb-4">
	{#if isToday(dayStart)}
		Today
	{:else if startOfToday().getTime() - dayStart < 7 * 24 * 60 * 60 * 1000}
		{format(dayStart, "EEEE")}
	{:else}
		{format(dayStart, "E, LLL d")}
	{/if}
</span>
