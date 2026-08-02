<script lang="ts">
	import { TerminalIcon } from "phosphor-svelte";

	import { Button } from "$lib/components/ui/button";
	import { commandCenterState } from "./command-center-state.svelte";

	const isMobile =
		typeof navigator !== "undefined" &&
		/Android|iPhone|iPad|iPod|Mobile/i.test(navigator.userAgent);
</script>

<Button
	variant="outline"
	size="sm"
	onclick={() => (commandCenterState.open = true)}
	class={["gap-1.5", !isMobile && "pe-1.5"]}
	aria-label="Open command center"
>
	<TerminalIcon class="size-4" />
	{#if !isMobile}
		<span class="text-xs text-muted-foreground font-mono">⌘K</span>
	{/if}
</Button>
