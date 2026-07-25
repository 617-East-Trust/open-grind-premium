<script lang="ts">
	import { onMount, onDestroy } from "svelte";

	import { goto } from "$app/navigation";

	import { toast } from "svelte-sonner";

	import L from "leaflet";

	import "leaflet/dist/leaflet.css";

	import { getPreferences } from "$lib/app-data/preferences.svelte";

	import { decodeGeohash } from "$lib/model/geohash";

	import { gridState } from "../(root)/grid-state.svelte";

	import type { FullGridProfile } from "../(root)/grid";

	import { SpinnerGap, MapPinIcon } from "phosphor-svelte";


	let mapEl: HTMLDivElement;

	let map: L.Map | null = null;

	let markersLayer: L.LayerGroup | null = null;

	let loading = $state(true);

	let error = $state<string | null>(null);

	let center = $state<{ lat: number; lon: number } | null>(null);

	// Deterministic jitter so the same profile always lands in the same relative spot
	function jitteredPosition(
		baseLat: number,
		baseLon: number,
		distanceMeters: number | null,
		profileId: number,
	): [number, number] {
		const seed = profileId * 2654435761;
		const angle = ((seed % 360) * Math.PI) / 180;
		// Use reported distance when available, otherwise a modest random radius
		const radiusM =
			distanceMeters && distanceMeters > 0
				? Math.min(distanceMeters * 0.85, 2500)
				: 200 + (seed % 800);
		const dx = (radiusM / 111320) * Math.cos(angle);
		const dy =
			(radiusM / (111320 * Math.cos((baseLat * Math.PI) / 180))) *
			Math.sin(angle);
		return [baseLat + dx, baseLon + dy];
	}

	function createAvatarIcon(profile: FullGridProfile): L.DivIcon {
		const name = (profile.displayName ?? "?").slice(0, 1).toUpperCase();

		const html = `
			<div class="map-pin">
				<div class="map-pin-inner">${name}</div>
			</div>
		`;

		return L.divIcon({
			className: "map-avatar-icon",
			html,
			iconSize: [36, 36],
			iconAnchor: [18, 18],
		});
	}

	async function initMap() {
		try {
			const prefs = await getPreferences();
			if (!prefs.geohash) {
				error = "Set a location first from the Browse tab";
				loading = false;
				return;
			}

			const decoded = decodeGeohash(prefs.geohash);
			center = { lat: decoded.lat, lon: decoded.lon };

			// Kick off / reuse the same grid data the Browse tab uses
			gridState.load(prefs.geohash);

			// Wait briefly for first page of results
			await new Promise((r) => setTimeout(r, 400));

			if (!mapEl) return;

			map = L.map(mapEl, {
				zoomControl: false,
				attributionControl: false,
			}).setView([decoded.lat, decoded.lon], 14);

			L.tileLayer("https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png", {
				maxZoom: 19,
				subdomains: "abcd",
			}).addTo(map);

			L.control.zoom({ position: "bottomright" }).addTo(map);

			markersLayer = L.layerGroup().addTo(map);

			// Center marker (you)
			L.circleMarker([decoded.lat, decoded.lon], {
				radius: 8,
				color: "#22c55e",
				fillColor: "#22c55e",
				fillOpacity: 0.9,
				weight: 2,
			})
				.bindTooltip("You", { direction: "top" })
				.addTo(markersLayer);

			renderMarkers();

			loading = false;
		} catch (e) {
			console.error(e);
			error = "Failed to load map";
			loading = false;
			toast.error("Failed to load map");
		}
	}

	function renderMarkers() {
		if (!map || !markersLayer || !center) return;

		markersLayer.clearLayers();

		// Re-add "you" marker
		L.circleMarker([center.lat, center.lon], {
			radius: 8,
			color: "#22c55e",
			fillColor: "#22c55e",
			fillOpacity: 0.9,
			weight: 2,
		})
			.bindTooltip("You", { direction: "top" })
			.addTo(markersLayer);

		const fullProfiles = gridState.items.filter(
			(p): p is FullGridProfile => p.type === "full",
		);

		for (const profile of fullProfiles) {
			const [lat, lon] = jitteredPosition(
				center.lat,
				center.lon,
				profile.distance,
				profile.id,
			);

			const marker = L.marker([lat, lon], {
				icon: createAvatarIcon(profile),
			});

			marker.bindTooltip(profile.displayName ?? "Profile", {
				direction: "top",
				offset: [0, -12],
			});

			marker.on("click", () => {
				goto(`/profile/${profile.id}`);
			});

			marker.addTo(markersLayer!);
		}
	}

	// Re-render when more profiles load
	$effect(() => {
		const _ = gridState.items.length;
		if (map && center) {
			renderMarkers();
		}
	});

	onMount(() => {
		initMap();
	});

	onDestroy(() => {
		map?.remove();
		map = null;
	});
</script>

<div class="relative flex-1 w-full h-[calc(100dvh-var(--safe-area-top)-80px)]">
	{#if loading}
		<div class="absolute inset-0 z-10 flex items-center justify-center bg-background/80">
			<SpinnerGap class="size-7 animate-spin text-muted-foreground" />
		</div>
	{:else if error}
		<div class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 px-6 text-center">
			<MapPinIcon class="size-10 text-muted-foreground/50" />
			<p class="text-sm text-muted-foreground">{error}</p>
			<a href="/" class="text-sm text-accent underline">Go to Browse</a>
		</div>
	{/if}

	<div bind:this={mapEl} class="absolute inset-0 z-0 bg-muted"></div>
</div>

<style>
	:global(.map-avatar-icon) {
		background: transparent;
		border: none;
	}

	:global(.map-pin) {
		width: 36px;
		height: 36px;
		border-radius: 9999px;
		background: hsl(var(--accent));
		border: 2px solid hsl(var(--background));
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.45);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	:global(.map-pin-inner) {
		color: hsl(var(--accent-foreground));
		font-size: 14px;
		font-weight: 600;
		line-height: 1;
	}

	:global(.leaflet-tooltip) {
		background: hsl(var(--card));
		border: 1px solid hsl(var(--border));
		color: hsl(var(--foreground));
		border-radius: 6px;
		padding: 4px 8px;
		font-size: 12px;
		box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
	}

	:global(.leaflet-tooltip-top:before) {
		border-top-color: hsl(var(--border));
	}
</style>
