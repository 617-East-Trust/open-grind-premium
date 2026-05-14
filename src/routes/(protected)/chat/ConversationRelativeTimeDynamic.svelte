<script lang="ts">
	import { formatTimeRelativeCustom } from "$lib/utils";

	let {
		date,
	}: {
		date: number;
	} = $props();

	let relativeTime = $derived(formatTimeRelativeCustom(date));

	$effect(() => {
		const interval = setInterval(() => {
			relativeTime = formatTimeRelativeCustom(date);
		}, 1000);
		return () => clearInterval(interval);
	});
</script>

{relativeTime}
