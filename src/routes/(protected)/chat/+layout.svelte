<script lang="ts">
	import ConversationsList from "./ConversationsList.svelte";
	import * as Card from "$lib/components/ui/card";
	import { page } from "$app/state";
	import * as Resizable from "$lib/components/ui/resizable";
	import { MediaQuery } from "svelte/reactivity";

	let {
		children,
	}: {
		children: import("svelte").Snippet;
	} = $props();

	let paneGroup: HTMLElement | null = $state(null);
	let conversationsListCollapsedSizePercentage = $state(0);
	let conversationsListMinWidthPercentage = $state(0);
	let pageContentMinWidthPercentage = $state(0);

	$effect(() => {
		if (!paneGroup) return;
		const observer = new ResizeObserver(() => {
			if (!paneGroup) return;
			conversationsListCollapsedSizePercentage = 117 / paneGroup.offsetWidth;
			conversationsListMinWidthPercentage = 200 / paneGroup.offsetWidth;
			pageContentMinWidthPercentage = 280 / paneGroup.offsetWidth;
		});
		observer.observe(paneGroup);
		return () => observer.disconnect();
	});

	const isChatSelected = $derived(page.params.conversationId !== undefined);

	const mobile = new MediaQuery("(width < 424px)");
</script>

<main class="flex w-full h-dvh">
	{#if !mobile.current}
		<Resizable.PaneGroup
			direction="horizontal"
			class="max-w-300 mx-auto max-h-full h-auto! max-xs:hidden!"
			bind:ref={paneGroup}
			autoSaveId="/(protected)/chat/layout"
		>
			<Resizable.Pane
				defaultSize={43}
				minSize={conversationsListMinWidthPercentage * 100}
				collapsedSize={conversationsListCollapsedSizePercentage * 100}
				collapsible
				class="min-w-29.25"
			>
				<ConversationsList class="pb-24 pe-0.75" />
			</Resizable.Pane>
			<Resizable.Handle
				class="cursor-col-resize! px-2 bg-transparent"
				withHandle
			/>
			<Resizable.Pane
				defaultSize={57}
				minSize={pageContentMinWidthPercentage * 100}
			>
				<div class="flex-1 self-stretch p-4 ps-1 pb-18 h-full">
					<Card.Root
						class={[
							"h-full rounded-2xl p-0 gap-0 relative dark:ring-neutral-800",
							{
								"bg-card/20 ring-0": !isChatSelected,
							},
						]}
					>
						{@render children?.()}
					</Card.Root>
				</div>
			</Resizable.Pane>
		</Resizable.PaneGroup>
	{/if}
	{#if isChatSelected}
		{#if mobile.current}
			<div class="flex-1 self-stretch h-full flex flex-col max-w-full">
				{@render children?.()}
			</div>
		{/if}
	{:else}
		<ConversationsList class="xs:hidden pb-24" />
	{/if}
</main>
