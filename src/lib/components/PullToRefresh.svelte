<script lang="ts">
	/**
	 * Document-level pull-to-refresh (grindrx Grid pattern).
	 *
	 * Premium layout uses the window as the scroll root with
	 * `overscroll-behavior: none`, so native rubber-band is disabled.
	 * We only engage when scrollY === 0 and the user drags downward.
	 *
	 * Adapted from grindrx `Grid.svelte` (MIT) and intended for profile /
	 * list pages that still use document scroll rather than open-grind's
	 * container + DataRefreshControl stack.
	 */
	import type { Snippet } from "svelte";
	import { ArrowsClockwiseIcon, SpinnerGap } from "phosphor-svelte";

	let {
		refreshing = false,
		disabled = false,
		onrefresh,
		children,
	}: {
		refreshing?: boolean;
		disabled?: boolean;
		onrefresh: () => void | Promise<void>;
		children?: Snippet;
	} = $props();

	const PULL_TRIGGER = 80;
	const PULL_MAX = 120;

	let pullStartY = $state<number | null>(null);
	let pullDistance = $state(0);

	function dampen(distance: number): number {
		return Math.min(PULL_MAX, distance * 0.5);
	}

	function scrollTop(): number {
		return window.scrollY || document.documentElement.scrollTop || 0;
	}

	function onTouchStart(event: TouchEvent) {
		if (disabled || refreshing) return;
		if (scrollTop() > 0) return;
		if (event.touches.length !== 1) return;
		pullStartY = event.touches[0].clientY;
		pullDistance = 0;
	}

	function onTouchMove(event: TouchEvent) {
		if (pullStartY === null) return;
		if (scrollTop() > 0) {
			pullStartY = null;
			pullDistance = 0;
			return;
		}
		const delta = event.touches[0].clientY - pullStartY;
		if (delta <= 0) {
			pullDistance = 0;
			return;
		}
		pullDistance = dampen(delta);
	}

	function onTouchEnd() {
		if (pullStartY === null) return;
		const shouldRefresh = pullDistance >= dampen(PULL_TRIGGER * 2);
		pullStartY = null;
		pullDistance = 0;
		if (shouldRefresh && !refreshing && !disabled) {
			void Promise.resolve(onrefresh()).catch((err) =>
				console.error("[pull-to-refresh]", err),
			);
		}
	}

	const pullActive = $derived(pullDistance > 0);
	const pullReady = $derived(pullDistance >= dampen(PULL_TRIGGER * 2));
	const showIndicator = $derived(pullActive || refreshing);
	const contentOffset = $derived(
		refreshing ? 48 : pullActive ? pullDistance : 0,
	);
</script>

<svelte:window
	ontouchstart={onTouchStart}
	ontouchmove={onTouchMove}
	ontouchend={onTouchEnd}
	ontouchcancel={onTouchEnd}
/>

{#if showIndicator}
	<div
		class="pointer-events-none fixed inset-x-0 top-[calc(var(--safe-area-top)+0.5rem)] z-30 flex justify-center"
		style="transform: translateY({refreshing
			? 8
			: pullDistance}px); transition: {pullStartY === null
			? 'transform 0.2s ease'
			: 'none'};"
	>
		<span
			class="inline-flex items-center gap-2 rounded-full border border-border bg-card/95 px-3 py-1.5 text-xs font-medium text-foreground shadow-md backdrop-blur-sm"
		>
			{#if refreshing}
				<SpinnerGap class="size-3.5 animate-spin" />
				Refreshing
			{:else}
				<ArrowsClockwiseIcon
					class="size-3.5 transition-transform"
					style="transform: rotate({pullReady ? 180 : 0}deg)"
				/>
				{pullReady ? "Release to refresh" : "Pull to refresh"}
			{/if}
		</span>
	</div>
{/if}

<div
	style="transform: translateY({contentOffset}px); transition: {pullStartY ===
		null || refreshing
		? 'transform 0.2s ease'
		: 'none'};"
>
	{#if children}
		{@render children()}
	{/if}
</div>
