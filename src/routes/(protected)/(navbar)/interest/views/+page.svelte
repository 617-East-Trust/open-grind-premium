<script lang="ts">
	import { onMount } from "svelte";

	import { getViews } from "$lib/api/interest/views";
	import type { ViewerProfile } from "$lib/model/interest/views";
	import UserAvatar from "$lib/components/UserAvatar.svelte";
	import * as Item from "$lib/components/ui/item";
	import { SpinnerGap } from "phosphor-svelte";
	import { toast } from "svelte-sonner";

	let viewers = $state<ViewerProfile[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	onMount(async () => {
		try {
			const res = await getViews();
			viewers = res.profiles ?? [];
		} catch (e) {
			console.error(e);
			error = "Failed to load views";
			toast("Failed to load views");
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
	{:else if viewers.length === 0}
		<div class="text-center py-16 text-muted-foreground text-sm">
			No profile views yet
		</div>
	{:else}
		<div class="flex flex-col gap-2">
			{#each viewers as viewer (viewer.profileId)}
				<a href={`/profile/${viewer.profileId}`}>
					<Item.Root variant="outline">
						<Item.Media>
							<UserAvatar mediaHash={viewer.profileImageMediaHash} size="md" />
						</Item.Media>
						<Item.Content>
							<Item.Title class="truncate">
								{viewer.displayName ?? "Unknown"}
							</Item.Title>
							<Item.Description class="text-xs">
								{#if viewer.isSecretAdmirer}
									Secret Admirer
								{:else}
									Viewed you
								{/if}
							</Item.Description>
						</Item.Content>
					</Item.Root>
				</a>
			{/each}
		</div>
	{/if}
</div>