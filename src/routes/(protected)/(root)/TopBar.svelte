<script lang="ts">
	import toast from "svelte-french-toast";
	import LocationChange from "./LocationChange.svelte";
	import Filters from "./GridFilters.svelte";
	import ProgressiveBlur from "$lib/components/ProgressiveBlur.svelte";
	import { Button, buttonVariants } from "$lib/components/ui/button";
	import { SlidersHorizontalIcon } from "phosphor-svelte";
	import { Tween } from "svelte/motion";
	import { expoOut } from "svelte/easing";
	import { onMount, untrack } from "svelte";
	import * as ToggleGroup from "$lib/components/ui/toggle-group";
	import {
		getPreferences,
		setPreferences,
	} from "$lib/app-data/preferences.svelte";
	import { defaultFilters } from "$lib/components/filters/filters";

	let {
		onUpdatePreferences,
		onRefreshGrid,
	}: {
		onUpdatePreferences: () => void;
		onRefreshGrid: () => void;
	} = $props();

	let expanded = $state(false);
	let expansion = new Tween(0, { duration: 600, easing: expoOut });
	let openFilters = $state({
		all: false,
		age: false,
		position: false,
	});

	let from: HTMLDivElement;
	let to: HTMLDivElement;

	let fromPos = $state({ left: 0, top: 0 });
	let toPos = $state({ left: 0, top: 0 });

	$effect(() => {
		if (expansion.current === expansion.target) return;
		untrack(() => {
			const fromRect = from.getBoundingClientRect();
			const toRect = to.getBoundingClientRect();
			fromPos = {
				left: fromRect.left,
				top: fromRect.top,
			};
			toPos = {
				left: toRect.left + 16,
				top: toRect.top,
			};
		});
	});

	let lastScrollY: number = $state(0);

	onMount(() => {
		expansion.set(window.scrollY > 0 ? 0 : 1, {
			duration: 0,
		});
		lastScrollY = window.scrollY;
	});

	let booleanFilters = $state({
		isFavorite: false,
		isOnline: false,
		isRightNow: false,
		isFresh: false,
	});

	let filters: {
		age: number[] | null;
		position: number[] | null;
	} = $state({
		age: null,
		position: null,
	});

	onMount(() => {
		getPreferences().then(({ gridSearchFilters = defaultFilters }) => {
			booleanFilters = {
				isFavorite: gridSearchFilters.isFavorite,
				isOnline: gridSearchFilters.isOnline,
				isRightNow: gridSearchFilters.isRightNow,
				isFresh: gridSearchFilters.isFresh || false,
			};
			filters = {
				age: gridSearchFilters.age ?? null,
				position: gridSearchFilters.positions ?? null,
			};
		});
	});

	const booleanFiltersValue = $derived(
		Object.entries(booleanFilters)
			.filter(([_, v]) => v)
			.map(([k, _]) => k),
	);
</script>

<svelte:window
	onscroll={(e) => {
		expanded = window.scrollY - lastScrollY < 0;
		expansion.target = expanded ? 1 : 0;
		fromPos = from.getBoundingClientRect();
		toPos = to.getBoundingClientRect();
		lastScrollY = window.scrollY;
	}}
/>
<ProgressiveBlur
	class="fixed top-0 left-0 w-full z-10"
	bgClass="bg-linear-to-b from-background to-transparent"
	contentClass="flex flex-col pt-4"
	direction="topToBottom"
>
	<div
		class="overflow-hidden px-4"
		style="height: {expansion.current * 40}px; opacity: {expansion.current === 1
			? '1'
			: '0'}; pointer-events: {expansion.current > 0.5 ? 'auto' : 'none'};"
		bind:this={to}
	>
		<LocationChange expansion={1} onUpdate={onUpdatePreferences} />
	</div>

	<div class="flex overflow-x-auto scrollbar-thin p-4 pt-0 gap-0.5">
		{#if expansion.current > 0 && expansion.current < 1}
			<div
				class="absolute w-[calc(100%-16px-16px)] pointer-events-none"
				style="left: {fromPos.left +
					(toPos.left - fromPos.left) *
						expansion.current}px; top: {fromPos.top +
					(toPos.top - fromPos.top) * expansion.current}px;"
			>
				<LocationChange expansion={expansion.current} class="relative" />
			</div>
		{/if}
		<div
			class="shrink-0 overflow-hidden"
			style="width: {(1 - expansion.current) *
				44}px; opacity: {expansion.current === 0
				? '1'
				: '0'}; pointer-events: {expansion.current < 0.5 ? 'auto' : 'none'};"
			bind:this={from}
		>
			<LocationChange expansion={0} onUpdate={onUpdatePreferences} />
		</div>
		<Button variant="secondary" onclick={() => (openFilters.all = true)}>
			<SlidersHorizontalIcon />
		</Button>
		<Button
			variant="secondary"
			onclick={() => (openFilters.age = true)}
			class={{ "bg-white hover:bg-neutral-200 text-popover": filters.age !== null }}
		>
			Age
		</Button>
		<Button
			variant="secondary"
			onclick={() => (openFilters.position = true)}
			class={{ "bg-white hover:bg-neutral-200 text-popover": filters.position !== null }}
		>
			Position
		</Button>
		<ToggleGroup.Root
			type="multiple"
			variant="default"
			bind:value={
				() => booleanFiltersValue,
				(values) => {
					async function updateFilters() {
						const oldBooleanFilters = booleanFilters;
						booleanFilters = {
							...oldBooleanFilters,
							isFavorite: values.includes("isFavorite"),
							isOnline: values.includes("isOnline"),
							isRightNow: values.includes("isRightNow"),
							isFresh: values.includes("isFresh"),
						};
						try {
							const { gridSearchFilters: oldFilters = defaultFilters } =
								await getPreferences();
							await setPreferences({
								gridSearchFilters: { ...oldFilters, ...booleanFilters },
							});
							onRefreshGrid();
						} catch (e) {
							toast.error("Failed to update filters");
							booleanFilters = oldBooleanFilters;
						}
					}
					updateFilters();
				}
			}
			size="sm"
			class="h-9"
		>
			<ToggleGroup.Item
				value="isOnline"
				class={buttonVariants({ variant: "secondary" })}
			>
				Online
			</ToggleGroup.Item>
			<ToggleGroup.Item
				value="isRightNow"
				class={buttonVariants({ variant: "secondary" })}
			>
				Right now
			</ToggleGroup.Item>
			<ToggleGroup.Item
				value="isFresh"
				class={buttonVariants({ variant: "secondary" })}
			>
				Fresh
			</ToggleGroup.Item>
		</ToggleGroup.Root>
	</div>
</ProgressiveBlur>
<div class="h-20"></div>
<Filters onUpdate={onRefreshGrid} bind:open={openFilters.all} />
