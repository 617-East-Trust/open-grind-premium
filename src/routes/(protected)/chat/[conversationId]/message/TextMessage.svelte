<script lang="ts">
	import { getMessageContext, getMessageMetaContext } from "./context";
	import type { TextMessage } from "$lib/model/message";

	let { message }: { message: TextMessage["body"] } = $props();

	const { lastInStack, msgOut } = $derived(getMessageContext()());
	const { clone, setRef, adornments } = $derived(getMessageMetaContext()());

	let el: HTMLDivElement | null = $state(null);
	$effect(() => {
		setRef(el ?? null);
	});
</script>

<div
	class={[
		"py-2 px-3 rounded-xl w-fit max-w-100 text-black shrink-0 relative overflow-visible select-text",
		{
			"bg-message-bubble-in": !msgOut,
			"ms-3": !msgOut && !clone,
			"rounded-es-none": lastInStack && !msgOut,
			"bg-message-bubble-out": msgOut,
			"me-3": msgOut && !clone,
			"rounded-ee-none": lastInStack && msgOut,
		},
	]}
	bind:this={el}
>
	{#if lastInStack}
		<svg
			viewBox="0 0 11 20"
			width="12"
			class={[
				"absolute bottom-0",
				{
					"fill-message-bubble-in right-full translate-x-[0.5px]": !msgOut,
					"fill-message-bubble-out -scale-x-100 left-full -translate-x-[0.5px]":
						msgOut,
				},
			]}
		>
			<g transform="translate(11 -12.5)" fill-rule="evenodd">
				<path
					d="M-6 16h6v17c-.193-2.84-.876-5.767-2.05-8.782-.904-2.325-2.446-4.485-4.625-6.48A1 1 0 01-6 16z"
					transform="matrix(1 0 0 -1 0 49)"
				></path>
			</g>
		</svg>
	{/if}
	<span class="whitespace-pre-wrap">{message.text}</span>
	{@render adornments?.()}
</div>
