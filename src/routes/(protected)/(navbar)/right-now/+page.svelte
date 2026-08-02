<script lang="ts">
	import { onMount } from "svelte";

	import {
		ArrowsClockwiseIcon,
		DropIcon,
		HouseIcon,
		MapPinIcon,
		PlusIcon,
		SpinnerGap,
		TimerIcon,
	} from "phosphor-svelte";
	import { toast } from "svelte-sonner";

	import PullToRefresh from "$lib/components/PullToRefresh.svelte";
	import { Button } from "$lib/components/ui/button";
	import {
		formatRightNowRemaining,
		rightNowRemainingMs,
		statusLabel,
	} from "$lib/model/right-now";
	import { getNow, subscribeNow } from "$lib/now.svelte";

	import CreateRightNowSheet from "./CreateRightNowSheet.svelte";
	import RightNowCard from "./RightNowCard.svelte";
	import {
		rightNowState,
		type HostingFilter,
	} from "./right-now-state.svelte";

	const filters: { id: HostingFilter; label: string }[] = [
		{ id: "all", label: "All" },
		{ id: "hosting", label: "Hosting" },
		{ id: "not_hosting", label: "Not hosting" },
	];

	$effect(() => subscribeNow());

	const activeRemaining = $derived(
		formatRightNowRemaining(
			rightNowRemainingMs(rightNowState.activePost?.postedAt ?? null, getNow()),
		),
	);

	onMount(() => {
		void rightNowState.load({ force: true });
	});

	async function onRefresh(options: { silent?: boolean } = {}) {
		await rightNowState.refresh();
		if (!rightNowState.error && !options.silent) {
			toast.success(
				rightNowState.items.length
					? `Updated · ${rightNowState.items.length} nearby`
					: "No active Right Now posts nearby",
			);
		}
	}
</script>

<PullToRefresh
	refreshing={rightNowState.loading}
	disabled={!rightNowState.geohash && rightNowState.loadedOnce}
	onrefresh={() => onRefresh({ silent: true })}
>
<div class="flex flex-col flex-1 min-h-0">
	<header
		class="sticky top-0 z-10 px-4 pt-3 pb-2 bg-background/90 backdrop-blur-md border-b border-border/60"
	>
		<div class="flex items-center justify-between gap-2 mb-3">
			<div class="flex items-center gap-2 min-w-0">
				<DropIcon weight="fill" class="size-5 text-accent shrink-0" />
				<div class="min-w-0">
					<h1 class="text-base font-semibold leading-tight">Right Now</h1>
					<p class="text-[11px] text-muted-foreground truncate">
						{#if rightNowState.source}
							via {rightNowState.source === "feed" ? "live feed" : "nearby grid"}
						{:else}
							Live posts · expire after 1 hour
						{/if}
					</p>
				</div>
			</div>
			<div class="flex items-center gap-1 shrink-0">
				<Button
					variant="ghost"
					size="icon"
					disabled={rightNowState.loading}
					onclick={() => void onRefresh()}
					aria-label="Refresh"
				>
					<ArrowsClockwiseIcon
						class={[
							"size-5",
							rightNowState.loading && "animate-spin",
						]}
					/>
				</Button>
				<Button
					size="sm"
					class="gap-1"
					disabled={rightNowState.posting}
					onclick={() => {
						if (rightNowState.hasActivePost) {
							toast.message("You already have an active post", {
								description: "End it first to post again.",
							});
							return;
						}
						rightNowState.composeOpen = true;
					}}
				>
					<PlusIcon class="size-4" weight="bold" />
					Post
				</Button>
			</div>
		</div>

		<div class="flex gap-1.5 overflow-x-auto no-scrollbar pb-0.5">
			{#each filters as filter}
				<button
					type="button"
					class={[
						"px-3 py-1.5 rounded-full text-xs border transition-colors shrink-0",
						rightNowState.hostingFilter === filter.id
							? "bg-accent text-accent-foreground border-accent font-medium"
							: "bg-muted/40 text-muted-foreground border-border hover:bg-muted",
					]}
					onclick={() => void rightNowState.setHostingFilter(filter.id)}
				>
					{filter.label}
				</button>
			{/each}
		</div>
	</header>

	<div class="flex-1 overflow-y-auto px-4 py-4 pb-28">
		{#if rightNowState.activePost}
			{@const active = rightNowState.activePost}
			<section
				class="mb-4 rounded-2xl border border-accent/40 bg-accent/10 p-3 max-w-lg mx-auto w-full"
			>
				<div class="flex items-start justify-between gap-3">
					<div class="min-w-0 flex-1">
						<p class="text-xs font-medium text-accent mb-1">Your post</p>
						<p class="text-sm font-medium truncate">
							{statusLabel(active.status)}
							{#if active.text}
								<span class="font-normal text-muted-foreground">
									· {active.text}
								</span>
							{/if}
						</p>
						<div
							class="mt-1 flex flex-wrap gap-x-3 gap-y-1 text-[11px] text-muted-foreground"
						>
							{#if String(active.status ?? "").toUpperCase() === "HOSTING"}
								<span class="inline-flex items-center gap-1">
									<HouseIcon class="size-3.5" weight="fill" />
									Hosting
								</span>
							{/if}
							{#if activeRemaining}
								<span class="inline-flex items-center gap-1">
									<TimerIcon class="size-3.5" />
									{activeRemaining}
								</span>
							{/if}
						</div>
					</div>
					<Button
						variant="outline"
						size="sm"
						disabled={rightNowState.ending}
						onclick={() => void rightNowState.endPost()}
					>
						{#if rightNowState.ending}
							<SpinnerGap class="size-4 animate-spin" />
						{:else}
							End
						{/if}
					</Button>
				</div>
			</section>
		{/if}

		{#if rightNowState.loading && rightNowState.items.length === 0}
			<div class="flex justify-center py-20">
				<SpinnerGap class="size-7 animate-spin text-muted-foreground" />
			</div>
		{:else if !rightNowState.geohash}
			<div
				class="flex flex-col items-center justify-center gap-3 py-20 px-6 text-center"
			>
				<MapPinIcon class="size-10 text-muted-foreground/50" />
				<p class="text-sm text-muted-foreground">
					Set a location on the Browse tab to see Right Now posts nearby.
				</p>
				<a href="/" class="text-sm text-accent underline">Go to Browse</a>
			</div>
		{:else if rightNowState.error && rightNowState.items.length === 0}
			<div class="flex flex-col items-center gap-3 py-20 text-center px-6">
				<p class="text-sm text-muted-foreground">
					{rightNowState.errorMessage ?? "Failed to load"}
				</p>
				<Button variant="outline" size="sm" onclick={() => void onRefresh()}>
					Retry
				</Button>
			</div>
		{:else if rightNowState.items.length === 0}
			<div
				class="flex flex-col items-center justify-center gap-3 py-20 px-6 text-center"
			>
				<DropIcon class="size-10 text-muted-foreground/40" weight="fill" />
				<p class="text-sm font-medium">Nobody's posting Right Now</p>
				<p class="text-xs text-muted-foreground max-w-xs">
					Posts only last an hour. Be the first nearby.
				</p>
				{#if !rightNowState.hasActivePost}
					<Button
						size="sm"
						class="mt-1 gap-1"
						onclick={() => (rightNowState.composeOpen = true)}
					>
						<PlusIcon class="size-4" weight="bold" />
						Post Right Now
					</Button>
				{/if}
			</div>
		{:else}
			<div class="flex flex-col gap-3 max-w-lg mx-auto w-full">
				{#each rightNowState.items as item (item.profileId + ":" + (item.postId ?? "") + ":" + (item.postedAt ?? ""))}
					<RightNowCard {item} />
				{/each}
			</div>
		{/if}
	</div>
</div>
</PullToRefresh>

<CreateRightNowSheet
	bind:open={rightNowState.composeOpen}
	submitting={rightNowState.posting}
	onSubmit={(input) => rightNowState.createPost(input)}
/>
