<script lang="ts">
	import { page } from "$app/state";
	import { UserIcon } from "phosphor-svelte";

	import DisplayName from "$lib/components/DisplayName.svelte";
	import * as Avatar from "$lib/components/ui/avatar";
	import { Badge } from "$lib/components/ui/badge";
	import * as Item from "$lib/components/ui/item";
	import type { Conversation } from "$lib/model/conversation";
	import ConversationRelativeTimeDynamic from "./ConversationRelativeTimeDynamic.svelte";

	let {
		conversation,
	}: {
		conversation: Conversation;
	} = $props();

	const preview = $derived(conversation.data.preview);
	const participant = $derived(conversation.data.participants[0]);

	const selected = $derived(
		page.params.conversationId === conversation.data.conversationId,
	);
</script>

{#snippet avatar()}
	<Item.Media class="p-2 translate-y-0! rounded-2xl relative">
		<Avatar.Root class="size-20 after:rounded-xl">
			{#if conversation.data.unreadCount > 0}
				<Badge
					class="px-[5.5px] @[9rem]:hidden absolute right-0 top-0 translate-x-1/4 -translate-y-1/4"
				>
					{conversation.data.unreadCount}
				</Badge>
			{/if}
			{#if participant.primaryMediaHash}
				<Avatar.Image
					src="https://cdns.grindr.com/images/thumb/320x320/{participant.primaryMediaHash}"
					alt="Avatar"
					class="rounded-xl"
				/>
			{/if}
			<Avatar.Fallback class="bg-neutral-700 rounded-xl">
				<UserIcon
					weight="fill"
					color="var(--color-stone-400)"
					class="size-10"
				/>
			</Avatar.Fallback>
		</Avatar.Root>
	</Item.Media>
{/snippet}
{#snippet content()}
	<Item.Content class="flex-1 min-w-0">
		<Item.Title
			class={[
				"truncate inline min-w-0 w-auto",
				{
					"text-muted-foreground": !conversation.data.name,
				},
			]}
		>
			<DisplayName name={conversation.data.name} />
		</Item.Title>
		<Item.Description
			class={{
				"font-medium text-white": conversation.data.unreadCount > 0,
			}}
		>
			{#if preview !== null}
				{#if preview.text !== null}
					{preview.text}
				{:else if preview.albumId !== null}
					Album
				{:else if preview.imageHash !== null || preview.type === "Image"}
					Photo
				{:else}
					<span class="preview-not-available"> Preview not available </span>
				{/if}
			{:else}
				<span class="preview-not-available"> Preview not available </span>
			{/if}
		</Item.Description>
	</Item.Content>
	<Item.Actions class="flex flex-col items-end gap-1 min-w-0">
		<span
			class="text-muted-foreground font-medium text-right truncate max-w-full"
		>
			<ConversationRelativeTimeDynamic
				date={conversation.data.lastActivityTimestamp}
			/>
		</span>
		{#if conversation.data.unreadCount > 0}
			<Badge class="px-[5.5px] @max-[9rem]:hidden">
				{conversation.data.unreadCount}
			</Badge>
		{/if}
	</Item.Actions>
{/snippet}
<Item.Root
	variant={selected ? "muted" : "outline"}
	class="p-0 gap-0 flex items-stretch flex-nowrap @container min-w-24"
>
	<a
		href="/profile/{participant.profileId}"
		class="rounded-l-2xl @max-[9rem]:hidden"
	>
		{@render avatar()}
	</a>
	<a
		href="/chat/{conversation.data.conversationId}"
		class="flex flex-1 self-stretch items-center p-4 ps-2 rounded-r-2xl min-w-0 gap-0.5 @max-[9rem]:hidden"
	>
		{@render content()}
	</a>
	<a
		href="/chat/{conversation.data.conversationId}"
		class="rounded-2xl @[9rem]:hidden min-w-24"
	>
		{@render avatar()}
	</a>
</Item.Root>

<style lang="postcss">
	@reference "$layout";
	.preview-not-available {
		@apply font-normal tracking-tight italic text-muted-foreground;
	}
</style>
