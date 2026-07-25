<script lang="ts">
	import { onMount } from "svelte";

	import { getReceivedTaps } from "$lib/api/interest/taps";
	import type { TapProfile } from "$lib/model/interest/tap-profile";
	import { tapTypes } from "$lib/model/interest/taps";
	import UserAvatar from "$lib/components/UserAvatar.svelte";
	import * as Item from "$lib/components/ui/item";
	import { SpinnerGap } from "phosphor-svelte";
	import { toast } from "svelte-sonner";

	let profiles = $state<TapProfile[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			const res = await getReceivedTaps();
			profiles = res.profiles ?? [];
		} catch (e) {
			console.error(e);
			error = "Failed to load taps";
			toast("Failed to load taps");
		} finally {
			loading = false;
		}
	});
</script>

<div class="px-4 pb-8">
	{#if loading}
		<div class="flex justify-center py-16">
			<SpinnerGap class="size-6 animate-spin text-muted-foreground" />
		</div>
	{:else if error}
		<div class="text-center py-16 text-muted-foreground text-sm">{error}</div>
	{:else if profiles.length === 0}
		<div class="text-center py-16 text-muted-foreground text-sm">
			No taps yet
		</div>
	{:else}
		<div class="flex flex-col gap-2">
			{#each profiles as profile (profile.profileId)}
				<a href={`/profile/${profile.profileId}`}>
					<Item.Root variant="outline">
						<Item.Media>
							<UserAvatar mediaHash={profile.profileImageMediaHash} size="md" />
						</Item.Media>
						<Item.Content>
							<Item.Title class="truncate">
								{profile.displayName ?? "Unknown"}
							</Item.Title>
							<Item.Description class="text-xs">
								{tapTypes[profile.tapType as keyof typeof tapTypes] ?? "Tap"}
								{#if profile.isMutual}
									· Mutual
								{/if}
							</Item.Description>
						</Item.Content>
					</Item.Root>
				</a>
			{/each}
		</div>
	{/if}
</div>