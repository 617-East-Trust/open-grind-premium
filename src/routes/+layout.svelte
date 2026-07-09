<script lang="ts">
	import { IconContext } from "phosphor-svelte";
	import "@fontsource-variable/ibm-plex-sans/wght.css";
	import "@fontsource-variable/ibm-plex-sans/wght-italic.css";

	import "../layout.css";
	import { onMount } from "svelte";
	import { Toaster } from "svelte-sonner";

	import {
		applyAndroidInsets,
		applyBackGestureHandler,
	} from "$lib/android-native-bridge";
	import { invoke } from "@tauri-apps/api/core";

	onMount(() => {
		applyAndroidInsets();
		applyBackGestureHandler();

		// App lifecycle → Rust foreground flag (Doze / WS reconnect on resume).
		const syncForeground = () => {
			const foreground = document.visibilityState === "visible";
			invoke("set_foreground", { foreground }).catch((e: unknown) => {
				console.debug("[lifecycle] set_foreground failed", e);
			});
		};
		syncForeground();
		document.addEventListener("visibilitychange", syncForeground);
		window.addEventListener("focus", syncForeground);
		window.addEventListener("pageshow", syncForeground);
		return () => {
			document.removeEventListener("visibilitychange", syncForeground);
			window.removeEventListener("focus", syncForeground);
			window.removeEventListener("pageshow", syncForeground);
		};
	});

	import RequestBlockedAlert from "$lib/api/request-blocked/RequestBlockedAlert.svelte";
	import favicon from "$lib/assets/favicon.png";

	let {
		children,
	}: {
		children?: import("svelte").Snippet;
	} = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>
<div
	class="fixed inset-x-0 top-0 z-150000 bg-background/50"
	style="height: var(--safe-area-top)"
></div>
<div
	class="fixed inset-x-0 bottom-0 z-150000 bg-background/50"
	style="height: var(--safe-area-bottom)"
></div>
<Toaster
	position="bottom-center"
	toastOptions={{
		style:
			"background-color: var(--accent); color: var(--popover); border: 1px solid var(--border);",
	}}
	expand
/>
<IconContext values={{}}>
	{@render children?.()}
</IconContext>
<RequestBlockedAlert />
