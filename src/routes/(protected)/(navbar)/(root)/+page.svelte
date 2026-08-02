<script lang="ts">
	import PullToRefresh from "$lib/components/PullToRefresh.svelte";
	import { getPreferences } from "$lib/app-data/preferences.svelte";
	import { gridState } from "./grid-state.svelte";
	import Grid from "./Grid.svelte";
	import LocationChooser from "./LocationEmpty.svelte";
	import TopBar from "./top-bar/TopBar.svelte";

	let preferences = $state(getPreferences());

	let topBar: TopBar | null = $state(null);

	function onPullRefresh() {
		gridState.refresh();
	}
</script>

<svelte:head>
	<title>Open Grind</title>
</svelte:head>
{#await preferences then { geohash }}
	{#if geohash === null}
		<main class="m-auto flex flex-1 max-w-full">
			<LocationChooser onUpdate={() => (preferences = getPreferences())} />
		</main>
	{:else}
		<PullToRefresh
			refreshing={gridState.loading}
			disabled={false}
			onrefresh={onPullRefresh}
		>
			<main class="flex flex-col p-4 gap-4">
				<TopBar
					onUpdatePreferences={() => (preferences = getPreferences())}
					onRefreshGrid={() => gridState.refresh()}
					bind:this={topBar}
				/>
				<Grid {geohash} onResetFilters={() => void topBar?.resetFilters()} />
			</main>
		</PullToRefresh>
	{/if}
{/await}
