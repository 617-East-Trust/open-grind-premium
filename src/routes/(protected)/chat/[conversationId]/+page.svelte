<script lang="ts">
	import { page } from "$app/state";
	import * as Card from "$lib/components/ui/card";
	import type { Message as MessageType } from "$lib/model/message";
	import { markConversationAsRead, sendMessage } from "$lib/api/messages";
	import { getConversation } from "./messages";
	import MessagesList from "./MessagesList.svelte";
	import MessageComposer from "./MessageComposer.svelte";
	import ChatNavBar from "./ChatNavBar.svelte";

	let { data }: import("./$types").PageProps = $props();

	const ourProfileId = $derived(data.ourProfileId);

	if (page.params.conversationId === undefined)
		throw new Error("conversationId is required");

	const conversationId = $derived(page.params.conversationId);

	async function fetchConversation() {
		const conversation = await getConversation({
			conversationId,
		});
		markConversationAsRead({
			conversationId,
		}).catch((error) =>
			console.error("Failed to mark conversation as read", error),
		);
		return conversation;
	}

	async function onSend({ text }: { text: string }) {
		const {
			profile: { profileId },
		} = await conversation;
		let message: MessageType = {
			type: "Text",
			body: {
				text,
			},
		};
		await sendMessage({
			toUserId: profileId,
			message,
		});
		conversation = fetchConversation(); // TODO: websockets
	}

	let conversation = $derived(fetchConversation());
</script>

<ChatNavBar {conversation} />
<Card.Content class="flex flex-col flex-1 pb-2 px-0 max-h-full min-h-0">
	<MessagesList {ourProfileId} {conversationId} {conversation} />
	<MessageComposer {onSend} />
</Card.Content>
