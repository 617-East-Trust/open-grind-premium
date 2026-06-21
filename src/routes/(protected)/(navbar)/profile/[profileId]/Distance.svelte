<script lang="ts">
	import { NavigationArrowIcon } from "phosphor-svelte";

	let { distance }: { distance: number | null } = $props();

	// API returns distance in metres; convert to miles for display.
	const METRES_PER_MILE = 1609.344;

	function formatDistance(metres: number): string {
		const miles = metres / METRES_PER_MILE;
		if (miles < 0.1) {
			// Less than ~160 m — show as feet for precision.
			const feet = Math.round(metres * 3.28084);
			return `${feet} ft`;
		} else if (miles < 10) {
			return `${miles.toFixed(1)} mi`;
		} else {
			return `${Math.round(miles)} mi`;
		}
	}
</script>

{#if distance !== null}
	<span class="flex items-center gap-1 whitespace-nowrap">
		<NavigationArrowIcon weight="fill" class="rotate-y-180 shrink-0" />
		{formatDistance(distance)}
	</span>
{/if}
