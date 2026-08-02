import "leaflet";

declare module "leaflet" {
	interface MarkerClusterGroupOptions {
		showCoverageOnHover?: boolean;
		maxClusterRadius?: number | ((zoom: number) => number);
		spiderfyOnMaxZoom?: boolean;
		disableClusteringAtZoom?: number;
		iconCreateFunction?: (cluster: MarkerCluster) => L.DivIcon;
	}

	class MarkerClusterGroup extends FeatureGroup {
		constructor(options?: MarkerClusterGroupOptions);
		addLayer(layer: Layer): this;
		clearLayers(): this;
	}

	interface MarkerCluster extends Marker {
		getChildCount(): number;
		getAllChildMarkers(): Marker[];
	}

	function markerClusterGroup(
		options?: MarkerClusterGroupOptions,
	): MarkerClusterGroup;
}

declare module "leaflet.markercluster" {
	// side-effect import extends L
}
