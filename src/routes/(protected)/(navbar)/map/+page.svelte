<script lang="ts">
	/**
	 * Map view — distance-ring placement of cascade profiles.
	 *
	 * Accuracy notes:
	 * - Grindr does not return true lat/lon for nearby profiles.
	 * - We place pins on a circle of radius ≈ reported distanceMeters around
	 *   the session geohash center, using a deterministic angle from profileId
	 *   (plus a golden-angle offset by rank to reduce stack collisions).
	 * - The green accuracy circle is the geohash cell error bounds.
	 */
	import { onMount, onDestroy } from "svelte";
	import { goto } from "$app/navigation";
	import { toast } from "svelte-sonner";
	import L from "leaflet";
	import "leaflet/dist/leaflet.css";
	import "leaflet.markercluster";
	import "leaflet.markercluster/dist/MarkerCluster.css";
	import "leaflet.markercluster/dist/MarkerCluster.Default.css";
	import { MapPinIcon, SpinnerGap } from "phosphor-svelte";

	import { getPreferences } from "$lib/app-data/preferences.svelte";
	import { decodeGeohash } from "$lib/model/geohash";
	import { setGridOrder } from "$lib/stores/grid-order.svelte";
	import type { FullGridProfile } from "../(root)/grid";
	import { gridState } from "../(root)/grid-state.svelte";

	const METERS_PER_DEG_LAT = 111_320;
	/** Golden angle (radians) spreads same-distance pins. */
	const GOLDEN_ANGLE = Math.PI * (3 - Math.sqrt(5));
	/** Cap ring radius so distant free-tier results don't explode the map. */
	const MAX_RING_M = 8_000;
	const MIN_RING_M = 40;

	let mapEl: HTMLDivElement;
	let map: L.Map | null = null;
	let markersLayer: L.MarkerClusterGroup | null = null;
	let youLayer: L.LayerGroup | null = null;
	let loading = $state(true);
	let error = $state<string | null>(null);
	let center = $state<{ lat: number; lon: number } | null>(null);
	let geohashErr = $state<{ latErr: number; lonErr: number } | null>(null);
	let markerCount = $state(0);
	let didFit = false;

	function metersToLat(m: number): number {
		return m / METERS_PER_DEG_LAT;
	}

	function metersToLon(m: number, atLat: number): number {
		return m / (METERS_PER_DEG_LAT * Math.max(0.2, Math.cos((atLat * Math.PI) / 180)));
	}

	/**
	 * Place profile on a ring of ~reported distance.
	 * Angle is deterministic from profileId; rank adds golden-angle spread so
	 * many profiles at similar distances don't stack on one bearing.
	 */
	function ringPosition(
		baseLat: number,
		baseLon: number,
		distanceMeters: number | null,
		profileId: number,
		rank: number,
	): [number, number] {
		const seed = (profileId * 2654435761) >>> 0;
		// Bearing from north (0..2π), mixed with golden angle by list rank
		const bearing =
			((seed % 3600) / 3600) * Math.PI * 2 + rank * GOLDEN_ANGLE;

		let radiusM: number;
		if (distanceMeters != null && distanceMeters > 0) {
			// Use reported distance with a tiny (±6%) deterministic radial jitter
			// so exact-equal distances don't form a perfect ring.
			const jitter = 0.94 + ((seed % 120) / 1000);
			radiusM = Math.min(MAX_RING_M, Math.max(MIN_RING_M, distanceMeters * jitter));
		} else {
			// Unknown distance: near-field pseudo-placement
			radiusM = 180 + (seed % 520);
		}

		// Bearing-from-north: dlat = cos, dlon = sin
		const dLat = metersToLat(radiusM) * Math.cos(bearing);
		const dLon = metersToLon(radiusM, baseLat) * Math.sin(bearing);
		return [baseLat + dLat, baseLon + dLon];
	}

	function formatDistance(metres: number | null): string {
		if (metres == null) return "";
		const miles = metres / 1609.344;
		if (miles < 0.1) return `${Math.round(metres * 3.28084)} ft`;
		if (miles < 10) return `${miles.toFixed(1)} mi`;
		return `${Math.round(miles)} mi`;
	}

	function createAvatarIcon(profile: FullGridProfile): L.DivIcon {
		const name = (profile.displayName ?? "?").slice(0, 1).toUpperCase();
		const hash = profile.profilePhotosHashes?.[0];
		const img = hash
			? `<img class="map-pin-img" src="https://cdns.grindr.com/images/thumb/320x320/${hash}" alt="" loading="lazy" draggable="false" />`
			: `<div class="map-pin-inner">${name}</div>`;

		const html = `<div class="map-pin">${img}</div>`;

		return L.divIcon({
			className: "map-avatar-icon",
			html,
			iconSize: [40, 40],
			iconAnchor: [20, 20],
		});
	}

	function renderYou() {
		if (!map || !youLayer || !center) return;
		youLayer.clearLayers();

		// Geohash accuracy ellipse (cell uncertainty)
		if (geohashErr) {
			const latR = geohashErr.latErr;
			const lonR = geohashErr.lonErr;
			// Approximate as circle using mean error in meters
			const errM = Math.max(
				latR * METERS_PER_DEG_LAT,
				lonR *
					METERS_PER_DEG_LAT *
					Math.cos((center.lat * Math.PI) / 180),
			);
			L.circle([center.lat, center.lon], {
				radius: Math.max(25, errM),
				color: "#22c55e",
				fillColor: "#22c55e",
				fillOpacity: 0.08,
				weight: 1,
				opacity: 0.5,
				dashArray: "4 4",
			})
				.bindTooltip("Your geohash cell", { direction: "top" })
				.addTo(youLayer);
		}

		// Distance rings (visual scale)
		for (const m of [250, 500, 1000, 2000]) {
			L.circle([center.lat, center.lon], {
				radius: m,
				color: "#64748b",
				fill: false,
				weight: 1,
				opacity: 0.25,
				interactive: false,
			}).addTo(youLayer);
		}

		L.circleMarker([center.lat, center.lon], {
			radius: 8,
			color: "#22c55e",
			fillColor: "#22c55e",
			fillOpacity: 0.95,
			weight: 2,
		})
			.bindTooltip("You", { direction: "top" })
			.addTo(youLayer);
	}

	function placedProfiles(): Array<{
		profile: FullGridProfile;
		lat: number;
		lon: number;
	}> {
		const fullProfiles = gridState.items.filter(
			(p): p is FullGridProfile => p.type === "full",
		);
		const sorted = [...fullProfiles].sort((a, b) => {
			const da = a.distance ?? Number.POSITIVE_INFINITY;
			const db = b.distance ?? Number.POSITIVE_INFINITY;
			return da - db;
		});
		// Keep swipe order in sync when browsing from map
		setGridOrder(sorted.map((p) => p.id));

		if (!center) return [];
		return sorted.map((profile, rank) => {
			const [lat, lon] = ringPosition(
				center!.lat,
				center!.lon,
				profile.distance,
				profile.id,
				rank,
			);
			return { profile, lat, lon };
		});
	}

	function renderMarkers() {
		if (!map || !markersLayer || !center) return;
		markersLayer.clearLayers();

		const placed = placedProfiles();
		markerCount = placed.length;
		const bounds: L.LatLngExpression[] = [[center.lat, center.lon]];

		for (const { profile, lat, lon } of placed) {
			bounds.push([lat, lon]);
			const marker = L.marker([lat, lon], {
				icon: createAvatarIcon(profile),
				zIndexOffset: Math.max(
					0,
					5000 - Math.round(profile.distance ?? 5000),
				),
			});
			const distLabel = formatDistance(profile.distance);
			const tip = [profile.displayName ?? "Profile", distLabel]
				.filter(Boolean)
				.join(" · ");
			marker.bindTooltip(tip, {
				direction: "top",
				offset: [0, -14],
			});
			marker.on("click", () => goto(`/profile/${profile.id}`));
			markersLayer.addLayer(marker);
		}

		// Fit once on first meaningful load
		if (!didFit && placed.length >= 3 && placed.length <= 60) {
			try {
				map.fitBounds(L.latLngBounds(bounds), {
					padding: [40, 40],
					maxZoom: 15,
					animate: false,
				});
				didFit = true;
			} catch {
				/* ignore */
			}
		}
	}

	async function waitForGrid(timeoutMs = 8000): Promise<void> {
		const start = Date.now();
		// Kick load if empty
		while (Date.now() - start < timeoutMs) {
			if (!gridState.loading && gridState.items.length > 0) return;
			if (!gridState.loading && gridState.error) return;
			await new Promise((r) => setTimeout(r, 80));
		}
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
			geohashErr = { latErr: decoded.latErr, lonErr: decoded.lonErr };

			gridState.load(prefs.geohash);
			await waitForGrid();

			if (!mapEl) return;

			// Zoom from geohash precision: tighter cell → higher zoom
			const approxCellM = Math.max(
				decoded.latErr * METERS_PER_DEG_LAT,
				decoded.lonErr *
					METERS_PER_DEG_LAT *
					Math.cos((decoded.lat * Math.PI) / 180),
			);
			const startZoom =
				approxCellM < 20 ? 16 : approxCellM < 80 ? 15 : approxCellM < 300 ? 14 : 13;

			map = L.map(mapEl, {
				zoomControl: false,
				attributionControl: false,
			}).setView([decoded.lat, decoded.lon], startZoom);

			L.tileLayer(
				"https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png",
				{
					maxZoom: 19,
					subdomains: "abcd",
				},
			).addTo(map);

			L.control.zoom({ position: "bottomright" }).addTo(map);

			youLayer = L.layerGroup().addTo(map);
			markersLayer = L.markerClusterGroup({
				showCoverageOnHover: false,
				maxClusterRadius: 48,
				spiderfyOnMaxZoom: true,
				disableClusteringAtZoom: 16,
				iconCreateFunction(cluster) {
					const count = cluster.getChildCount();
					const size = count >= 20 ? 48 : count >= 8 ? 42 : 36;
					return L.divIcon({
						html: `<div class="map-cluster" style="width:${size}px;height:${size}px">${count}</div>`,
						className: "map-cluster-icon",
						iconSize: L.point(size, size),
					});
				},
			});
			map.addLayer(markersLayer);

			renderYou();
			renderMarkers();

			// Resolve partials so map gets full distance + photos
			for (let i = 0; i < gridState.partialBatches.length; i++) {
				void gridState.loadBatch(i);
			}

			// Prefetch next cascade page for denser map
			if (gridState.nextPage) {
				void gridState.loadMore();
			}

			loading = false;
		} catch (e) {
			console.error(e);
			error = "Failed to load map";
			loading = false;
			toast.error("Failed to load map");
		}
	}

	// Re-render when more profiles resolve
	$effect(() => {
		const _len = gridState.items.length;
		const _loading = gridState.loading;
		void _len;
		void _loading;
		if (map && center) {
			renderMarkers();
		}
	});

	onMount(() => {
		void initMap();
	});

	onDestroy(() => {
		map?.remove();
		map = null;
	});
</script>

<div class="relative flex-1 w-full h-[calc(100dvh-var(--safe-area-top)-80px)]">
	{#if loading}
		<div
			class="absolute inset-0 z-10 flex items-center justify-center bg-background/80"
		>
			<SpinnerGap class="size-7 animate-spin text-muted-foreground" />
		</div>
	{:else if error}
		<div
			class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 px-6 text-center"
		>
			<MapPinIcon class="size-10 text-muted-foreground/50" />
			<p class="text-sm text-muted-foreground">{error}</p>
			<a href="/" class="text-sm text-accent underline">Go to Browse</a>
		</div>
	{:else}
		<div
			class="absolute top-3 left-3 z-10 rounded-full border border-border bg-card/90 px-3 py-1.5 text-[11px] text-muted-foreground shadow backdrop-blur-sm"
		>
			{markerCount} nearby · markercluster · distance rings (not GPS)
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
		width: 40px;
		height: 40px;
		border-radius: 9999px;
		background: hsl(var(--accent));
		border: 2px solid hsl(var(--background));
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.45);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
	}

	:global(.map-pin-img) {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	:global(.map-pin-inner) {
		color: hsl(var(--accent-foreground));
		font-size: 14px;
		font-weight: 600;
		line-height: 1;
	}

	:global(.map-cluster-icon) {
		background: transparent;
		border: none;
	}

	:global(.map-cluster) {
		border-radius: 9999px;
		background: hsl(var(--accent));
		color: hsl(var(--accent-foreground));
		border: 2px solid hsl(var(--background));
		box-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 13px;
		font-weight: 700;
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
