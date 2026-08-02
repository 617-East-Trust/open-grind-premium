<script lang="ts">
	import { goto } from "$app/navigation";
	import { page } from "$app/state";
	import { toast } from "svelte-sonner";

	import { setPreferences } from "$lib/app-data/preferences.svelte";
	import * as Command from "$lib/components/ui/command";
	import { commandCenterClose } from "../command-center-state.svelte";

	let {
		geohash,
	}: {
		geohash: string | null;
	} = $props();

	async function setLocation(hash: string) {
		try {
			await setPreferences({ geohash: hash });
			await goto("/", { replaceState: page.url.pathname === "/" });
			toast.success("Location updated");
		} catch (error) {
			console.error(error);
			toast.error("Failed to save location");
		}
	}
</script>

<Command.Item
	value={geohash === null ? "@" : `@${geohash}`}
	disabled={geohash === null || geohash.length !== 12}
	class={geohash === null ? "text-muted-foreground" : "gap-0 font-mono"}
	onSelect={() => {
		if (geohash === null || geohash.length !== 12) return;
		void setLocation(geohash);
		commandCenterClose();
	}}
>
	{#if geohash === null}
		Enter the 12-character geohash to set your location
	{:else}
		@{geohash}
		{#if geohash.length < 12}
			<span class="text-muted-foreground ms-1 text-xs">
				({12 - geohash.length} more)
			</span>
		{/if}
	{/if}
</Command.Item>
