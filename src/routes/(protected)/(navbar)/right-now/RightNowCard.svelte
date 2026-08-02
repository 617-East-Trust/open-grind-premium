<script lang="ts">
	import { HouseIcon, MapPinIcon, TimerIcon } from "phosphor-svelte";

	import { Badge } from "$lib/components/ui/badge";
	import UserAvatar from "$lib/components/UserAvatar.svelte";
	import {
		formatDistanceMeters,
		formatRightNowRemaining,
		rightNowRemainingMs,
		statusLabel,
		type RightNowFeedItem,
	} from "$lib/model/right-now";
	import { getNow, subscribeNow } from "$lib/now.svelte";

	let { item }: { item: RightNowFeedItem } = $props();

	$effect(() => subscribeNow());

	const remaining = $derived(
		formatRightNowRemaining(rightNowRemainingMs(item.postedAt, getNow())),
	);
	const distance = $derived(formatDistanceMeters(item.distance));
	const isHosting = $derived(String(item.status ?? "").toUpperCase() === "HOSTING");
	const hasPostImage = $derived(
		Boolean(item.thumbnailUrl || item.fullImageUrl),
	);
</script>

<a
	href="/profile/{item.profileId}"
	class="block rounded-2xl border border-border bg-card overflow-hidden hover:bg-muted/40 transition-colors"
>
	{#if hasPostImage}
		<div class="relative aspect-[4/3] w-full bg-muted">
			<img
				src={item.fullImageUrl || item.thumbnailUrl || ""}
				alt=""
				class="absolute inset-0 size-full object-cover"
				loading="lazy"
				draggable="false"
			/>
			<div
				class="absolute inset-x-0 bottom-0 h-20 bg-linear-to-t from-black/70 to-transparent"
			></div>
			<div class="absolute bottom-2 left-2 right-2 flex items-end gap-2">
				<div
					class="size-10 shrink-0 rounded-full overflow-hidden border-2 border-background/80"
				>
					<UserAvatar
						mediaHash={item.profileImageMediaHash}
						class="size-full"
						size="md"
					/>
				</div>
				<div class="min-w-0 flex-1">
					<p class="text-sm font-semibold text-white truncate drop-shadow">
						{item.displayName ?? "Profile"}
						{#if item.age != null}
							<span class="font-normal opacity-90">, {item.age}</span>
						{/if}
					</p>
				</div>
				<Badge
					variant={isHosting ? "default" : "secondary"}
					class="shrink-0 gap-1 shadow-sm"
				>
					{#if isHosting}
						<HouseIcon weight="fill" class="size-3" />
					{/if}
					{statusLabel(item.status)}
				</Badge>
			</div>
		</div>
	{:else}
		<div class="flex items-start gap-3 p-3 pb-0">
			<div class="size-12 shrink-0 rounded-full overflow-hidden">
				<UserAvatar
					mediaHash={item.profileImageMediaHash}
					class="size-full"
					size="md"
				/>
			</div>
			<div class="min-w-0 flex-1 pt-0.5">
				<div class="flex items-center gap-2 flex-wrap">
					<p class="text-sm font-semibold truncate">
						{item.displayName ?? "Profile"}
						{#if item.age != null}
							<span class="font-normal text-muted-foreground"
								>, {item.age}</span
							>
						{/if}
					</p>
					<Badge
						variant={isHosting ? "default" : "secondary"}
						class="shrink-0 gap-1"
					>
						{#if isHosting}
							<HouseIcon weight="fill" class="size-3" />
						{/if}
						{statusLabel(item.status)}
					</Badge>
				</div>
			</div>
		</div>
	{/if}

	<div class="p-3 flex flex-col gap-2">
		{#if item.text}
			<p class="text-sm leading-snug whitespace-pre-wrap break-words">
				{item.text}
			</p>
		{:else if !hasPostImage}
			<p class="text-sm text-muted-foreground italic">No status text</p>
		{/if}

		<div
			class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground"
		>
			{#if distance}
				<span class="inline-flex items-center gap-1">
					<MapPinIcon class="size-3.5" />
					{distance}
				</span>
			{/if}
			{#if remaining}
				<span class="inline-flex items-center gap-1">
					<TimerIcon class="size-3.5" />
					{remaining}
				</span>
			{/if}
		</div>
	</div>
</a>
