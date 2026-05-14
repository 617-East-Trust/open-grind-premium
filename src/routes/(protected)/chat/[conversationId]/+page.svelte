<script lang="ts">
	import { untrack } from "svelte";
	import { page } from "$app/state";
	import * as Card from "$lib/components/ui/card";
	import type { Message } from "$lib/model/message";
	import { ConversationState } from "./conversation-state.svelte";
	import MessagesList from "./MessagesList.svelte";
	import MessageComposer from "./MessageComposer.svelte";
	import ChatNavBar from "./ChatNavBar.svelte";

	let { data }: import("./$types").PageProps = $props();

	if (page.params.conversationId === undefined)
		throw new Error("conversationId is required");

	const conversationId = $derived(page.params.conversationId as string);
	const ourProfileId = $derived(data.ourProfileId);

	let conversationState = $state(
		untrack(() => new ConversationState(conversationId, data.ourProfileId)),
	);

	$effect(() => {
		if (
			conversationId !== conversationState.conversationId ||
			ourProfileId !== conversationState.ourProfileId
		) {
			conversationState = new ConversationState(conversationId, ourProfileId);
		}
	});
</script>

<ChatNavBar {conversationState} />
<Card.Content class="flex flex-col flex-1 pb-2 px-0 max-h-full min-h-0">
	<MessagesList {conversationState} />
	<MessageComposer
		onSend={(message: Message) => conversationState.send(message)}
	/>
</Card.Content>
