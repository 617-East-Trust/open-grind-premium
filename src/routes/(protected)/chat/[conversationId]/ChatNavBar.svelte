<script lang="ts">
	import { ArrowLeftIcon, UserIcon } from "phosphor-svelte";
	import * as Card from "$lib/components/ui/card";
	import * as Avatar from "$lib/components/ui/avatar";
	import { Skeleton } from "$lib/components/ui/skeleton";
	import ProgressiveBlur from "$lib/components/ProgressiveBlur.svelte";
	import DisplayName from "$lib/components/DisplayName.svelte";
	import type { ConversationState } from "./conversation-state.svelte";

	let { conversationState }: { conversationState: ConversationState } = $props();
</script>

<ProgressiveBlur
	direction="topToBottom"
	class="w-full shrink-0 h-19 absolute z-10"
	bgClass="bg-linear-to-b max-xs:from-background xs:from-card to-transparent"
	contentClass="flex items-center h-full"
	tag="nav"
>
	<a href="/chat" class="flex items-center justify-center w-19 h-full">
		<ArrowLeftIcon size={32} />
	</a>
	{#if conversationState.loading || conversationState.profile === null}
		<div class="py-4 ps-0 flex-1 flex items-center gap-3">
			<Skeleton class="rounded-full size-[37.5px]" />
			<div class="flex flex-col gap-2">
				<Skeleton class="rounded-md w-20 h-4" />
				<Skeleton class="rounded-md w-12 h-3" />
			</div>
		</div>
	{:else if conversationState.error}
		<span class="flex-1">Failed to load conversation</span>
	{:else}
		{@const profile = conversationState.profile}
		<a href="/profile/{profile.profileId}" class="flex-1 ps-0 py-4 pe-4">
			<Card.Header class="flex items-center gap-4 px-0">
				<Avatar.Root class="size-[37.5px] after:rounded-full">
					{#if profile.mediaHash}
						<Avatar.Image
							src="https://cdns.grindr.com/images/thumb/75x75/{profile.mediaHash}"
							alt="Avatar"
							class="rounded-full"
						/>
					{/if}
					<Avatar.Fallback class="bg-neutral-700 rounded-full">
						<UserIcon
							weight="fill"
							color="var(--color-stone-400)"
							class="size-5"
						/>
					</Avatar.Fallback>
				</Avatar.Root>
				<div class="flex flex-col min-w-0">
					<Card.Title
						class={[
							"min-w-0 truncate",
							{
								"text-muted-foreground": !profile.name,
							},
						]}
					>
						<DisplayName name={profile.name} />
					</Card.Title>
					{#if profile.distance === null}
						<Card.Description class="truncate">
							Distance unknown
						</Card.Description>
					{:else}
						<Card.Description class="truncate">
							{#if profile.distance < 1000}
								{Math.round(profile.distance)} m
							{:else}
								{(profile.distance / 1000).toFixed(1)} km
							{/if}
						</Card.Description>
					{/if}
				</div>
			</Card.Header>
		</a>
	{/if}
</ProgressiveBlur>
