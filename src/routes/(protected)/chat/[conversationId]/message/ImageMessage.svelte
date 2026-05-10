<script lang="ts">
	import type { ExpiringImageMessage, ImageMessage } from "$lib/model/message";
	import { getMessageContext, getMessageMetaContext } from "./context";

	let {
		message,
	}: { message: ImageMessage["body"] | ExpiringImageMessage["body"] } =
		$props();

	const { lastInStack, msgOut } = $derived(getMessageContext()());
	const { clone, setRef, adornments } = $derived(getMessageMetaContext()());

	let el: HTMLDivElement | null = $state(null);
	$effect(() => {
		setRef(el ?? null);
	});
</script>

<div
	class={["relative", { "w-2/5 min-w-35 max-w-60 ms-3": !clone }]}
	bind:this={el}
>
	<img
		src={message.url}
		alt=""
		class={[
			"w-full rounded-lg bg-card-foreground/10 object-cover",
			{
				"rounded-es-[6px]": lastInStack && !msgOut,
				"rounded-ee-[6px]": lastInStack && msgOut,
			},
		]}
		style:aspect-ratio={message.width !== null && message.height !== null
			? `${message.width} / ${message.height}`
			: undefined}
	/>
	{@render adornments?.()}
</div>
