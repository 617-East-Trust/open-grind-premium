<script lang="ts">
	import { goto } from "$app/navigation";
	import { FunnelIcon } from "phosphor-svelte";
	import { toast } from "svelte-sonner";

	import { setPreferences } from "$lib/app-data/preferences.svelte";
	import { Badge } from "$lib/components/ui/badge";
	import * as Command from "$lib/components/ui/command";
	import {
		commandCenterClose,
		commandCenterState,
	} from "../command-center-state.svelte";
	import { parseFilterGridQuery } from "./filter-grid-query";

	const result = $derived(parseFilterGridQuery(commandCenterState.query));
	const canApply = $derived(result.validCount > 0 && result.invalidCount === 0);

	async function apply() {
		if (!canApply) return;
		try {
			await setPreferences({ gridSearchFilters: result.filters });
			commandCenterClose();
			// Hard navigate so Browse reloads cascade with the new filters
			await goto("/", { invalidateAll: true });
			// Force a full reload of the browse grid state if already on /
			if (typeof window !== "undefined") {
				window.dispatchEvent(new CustomEvent("og:grid-filters-changed"));
			}
			toast.success(
				`Applied ${result.validCount} filter${result.validCount === 1 ? "" : "s"}`,
			);
		} catch (error) {
			console.error(error);
			toast.error("Failed to apply filters");
		}
	}
</script>

<Command.Group heading="Filter grid...">
	<Command.Item
		value={commandCenterState.query || "?"}
		disabled={!canApply}
		class={canApply ? undefined : "text-muted-foreground"}
		onSelect={() => void apply()}
	>
		<FunnelIcon />
		<div class="flex min-w-0 flex-1 flex-col gap-2">
			{#if result.parsed.length === 0}
				<span>
					Type a filter query, e.g.
					<code class="rounded-xs bg-muted px-1 py-px font-mono text-sm">
						online=true&age=21-40&rightnow=true
					</code>
				</span>
			{:else}
				<div class="flex flex-wrap gap-1">
					{#each result.parsed as filter, index (index)}
						{#if filter.valid}
							<Badge variant="secondary">
								<span class="opacity-60">{filter.key}</span>
								{filter.valueText}
							</Badge>
						{:else}
							<Badge variant="destructive">
								<span class="opacity-80">{filter.key}</span>
								{filter.error}
							</Badge>
						{/if}
					{/each}
				</div>
				{#if canApply}
					<span class="text-xs text-muted-foreground">
						Press Enter to apply {result.validCount}
						filter{result.validCount === 1 ? "" : "s"}
					</span>
				{:else}
					<span class="text-xs text-destructive">
						Fix the highlighted filters to apply
					</span>
				{/if}
			{/if}
		</div>
	</Command.Item>
</Command.Group>
