<script lang="ts">
	import { UserPlusIcon } from "phosphor-svelte";

	import * as Item from "$lib/components/ui/item";
</script>

<Item.Root variant="outline">
	{#snippet child({ props })}
		<a href="/auth/sign-in" {...props}>
			<Item.Media>
				<UserPlusIcon weight="regular" class="size-5" />
			</Item.Media>
			<Item.Content class="min-w-0">
				<Item.Title class="truncate min-w-0 w-full inline-block text-left">
					Add Account
				</Item.Title>
			</Item.Content>
			<Item.Actions>
				<UserPlusIcon class="size-4 opacity-40" />
			</Item.Actions>
		</a>
	{/snippet}
</Item.Root>
