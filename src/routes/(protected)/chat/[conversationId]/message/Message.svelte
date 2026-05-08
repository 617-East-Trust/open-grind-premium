<script lang="ts">
	import { tick } from "svelte";
	import type { ApiResponseMessage, Message } from "$lib/model/message";
	import AlbumMessage from "./AlbumMessage.svelte";
	import ImageMessage from "./ImageMessage.svelte";
	import TextMessage from "./TextMessage.svelte";
	import MessageDateGroup from "./MessageDateGroup.svelte";
	import UnsupportedMessage from "./UnsupportedMessage.svelte";
	import MessageTime from "./MessageTime.svelte";
	import { setMessageContext } from "./context";
	import MessageContextMenu from "./MessageContextMenu.svelte";
	import toast from "svelte-french-toast";

	let {
		message,
		ourProfileId,
		indexInStack,
		stackLength,
		dayStart,
	}: {
		message: ApiResponseMessage;
		ourProfileId: number;
		indexInStack: number;
		stackLength: number;
		dayStart?: number;
	} = $props();

	const msgOut = $derived(message.senderId === ourProfileId);
	const firstInStack = $derived(indexInStack === 0);
	const lastInStack = $derived(indexInStack === stackLength - 1);

	setMessageContext(() => ({
		firstInStack,
		lastInStack,
		indexInStack,
		msgOut,
		timestamp: message.timestamp,
	}));

	let contextMenuOpen:
		| false
		| {
				x: number;
				y: number;
				width: number;
				height: number;
		  } = $state(false);
	let messageElement: HTMLDivElement | undefined = $state();
	let inheritedStyles = $state("");

	const INHERITED_PROPS = [
		"font-size",
		"font-family",
		"font-weight",
		"font-style",
		"font-variant",
		"font-stretch",
		"line-height",
		"letter-spacing",
		"word-spacing",
		"text-transform",
		"text-indent",
		"text-align",
		"text-decoration",
		"color",
		"direction",
		"white-space",
		"word-break",
		"overflow-wrap",
		"tab-size",
		"hyphens",
		"cursor",
		"border-collapse",
		"border-spacing",
		"list-style",
		"list-style-type",
		"quotes",
	];

	function onContextMenu(e: MouseEvent | KeyboardEvent) {
		if (!messageElement) return;
		const { x, y } = messageElement.getBoundingClientRect();
		const computed = getComputedStyle(messageElement);
		inheritedStyles = INHERITED_PROPS.map(
			(prop) => `${prop}: ${computed.getPropertyValue(prop)}`,
		).join("; ");
		contextMenuOpen = {
			x,
			y,
			width: messageElement.clientWidth,
			height: messageElement.offsetHeight,
		};
		tick().then(() => contextMenu?.showModal());
	}

	let contextMenu: HTMLDialogElement | undefined = $state();

	function onReact(reactionId: number) {
		toast.error("TODO: Reacting to messages not implemented yet");
	}
</script>

{#snippet content(clone?: boolean)}
	{#if message.type === "Text"}
		<TextMessage
			message={message.body}
			bind:ref={() => messageElement, (el) => !clone && (messageElement = el)}
			{clone}
		/>
	{:else if message.type === "Image" || message.type === "ExpiringImage"}
		<ImageMessage
			message={message.body}
			bind:ref={() => messageElement, (el) => !clone && (messageElement = el)}
		/>
	{:else if message.type === "Album" || message.type === "ExpiringAlbum" || message.type === "ExpiringAlbumV2"}
		<AlbumMessage
			message={message.body}
			bind:ref={() => messageElement, (el) => !clone && (messageElement = el)}
			{clone}
		/>
	{:else}
		<UnsupportedMessage
			type={message.type}
			bind:ref={() => messageElement, (el) => !clone && (messageElement = el)}
		/>
	{/if}
{/snippet}
<div
	class={[
		"flex flex-col gap-0.5 z-1 relative",
		{
			"mt-3": firstInStack,
		},
	]}
>
	{#if firstInStack && dayStart !== undefined}
		<MessageDateGroup {dayStart} />
	{/if}
	<div
		class={["message-container", { "*:me-auto": !msgOut, "*:ms-auto": msgOut }]}
		role="button"
		tabindex="0"
		aria-label="Message"
		ondblclick={(e) => {
			e.preventDefault();
			onReact(1);
		}}
		onkeydown={(e) => {
			if (e.key === "Enter" || e.key === " ") {
				if (e.key === " ") e.preventDefault();
				onContextMenu(e);
			}
		}}
		oncontextmenu={(e) => {
			e.preventDefault();
			onContextMenu(e);
		}}
		style:visibility={contextMenuOpen ? "hidden" : undefined}
	>
		{@render content()}
	</div>
	{#if lastInStack}
		<MessageTime />
	{/if}
</div>

{#if contextMenuOpen}
	<MessageContextMenu
		{contextMenuOpen}
		{content}
		onClose={() => (contextMenuOpen = false)}
		style={inheritedStyles}
		textContent={message.type === "Text" ? message.body.text : undefined}
	/>
{/if}
