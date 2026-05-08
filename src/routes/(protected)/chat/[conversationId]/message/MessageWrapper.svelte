<script lang="ts">
	import { setMessageMetaContext } from "./context";
	import type { Snippet } from "svelte";

	let {
		clone = false,
		setRef,
		adornments,
		children,
	}: {
		clone?: boolean;
		setRef: (el: HTMLElement | null) => void;
		adornments?: Snippet;
		children: Snippet;
	} = $props();

	setMessageMetaContext(() => ({
		clone,
		setRef: clone ? () => {} : setRef,
		adornments: clone ? undefined : adornments,
	}));
</script>

{@render children()}
