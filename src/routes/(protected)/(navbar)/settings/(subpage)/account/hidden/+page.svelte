<script lang="ts">
	import { onMount } from "svelte";
	import { toast } from "svelte-sonner";
	import { getHiddenUsers, unhideUser } from "$lib/api/account";
	import UserAvatar from "$lib/components/UserAvatar.svelte";
	import * as Item from "$lib/components/ui/item";
	import { Button } from "$lib/components/ui/button";
	import { SpinnerGap, UserCircleIcon } from "phosphor-svelte";

	let users = $state<{ profileId: number; displayName?: string | null; profileImageMediaHash?: string | null }[]>([]);
	let loading = $state(true);
	let unhiding = $state<Record<number, boolean>>({});

	onMount(async () => {
		try {
			const res = await getHiddenUsers();
			users = res.profiles ?? [];
		} catch (e) {
			console.error(e);
			toast.error("Failed to load hidden users");
		} finally {
			loading = false;
		}
	});

	async function onUnhide(profileId: number) {
		unhiding[profileId] = true;
		try {
			await unhideUser(profileId);
			users = users.filter((u) => u.profileId !== profileId);
			toast.success("User unhidden");
		} catch (e) {
			console.error(e);
			toast.error("Failed to unhide");
		} finally {
			unhiding[profileId] = false;
		}
	}
</script>

<div class="flex w-full px-4">
	<main class="pb-(--content-pb) flex flex-col gap-2 w-full max-w-120 m-auto pt-2">
		{#if loading}
			<div class="flex justify-center py-16">
				<SpinnerGap class="size-6 animate-spin text-muted-foreground" />
			</div>
		{:else if users.length === 0}
			<div class="flex flex-col items-center gap-3 py-16 text-center">
				<UserCircleIcon class="size-10 text-muted-foreground/40" />
				<p class="text-sm text-muted-foreground">No hidden users</p>
			</div>
		{:else}
			{#each users as user (user.profileId)}
				<Item.Root variant="outline">
					<Item.Media>
						<UserAvatar mediaHash={user.profileImageMediaHash ?? null} size="md" />
					</Item.Media>
					<Item.Content>
						<Item.Title class="truncate">{user.displayName ?? `User #${user.profileId}`}</Item.Title>
					</Item.Content>
					<Item.Actions>
						<Button
							variant="ghost"
							size="sm"
							disabled={unhiding[user.profileId]}
							onclick={() => onUnhide(user.profileId)}
						>
							{#if unhiding[user.profileId]}
								<SpinnerGap class="size-4 animate-spin" />
							{:else}
								Unhide
							{/if}
						</Button>
					</Item.Actions>
				</Item.Root>
			{/each}
		{/if}
	</main>
</div>