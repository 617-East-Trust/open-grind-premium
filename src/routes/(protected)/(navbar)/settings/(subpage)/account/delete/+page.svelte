<script lang="ts">
	import { goto } from "$app/navigation";
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import * as AlertDialog from "$lib/components/ui/alert-dialog";
	import { deleteAccount } from "$lib/api/account";
	import { getAccounts } from "$lib/account-store.svelte";
	import { SpinnerGap, WarningIcon } from "phosphor-svelte";

	let password = $state("");
	let loading = $state(false);
	let confirmOpen = $state(false);

	async function onDelete() {
		if (!password) return;
		loading = true;
		try {
			await deleteAccount(password);
			await getAccounts().logout();
			toast.success("Account deleted");
			goto("/auth/sign-in");
		} catch (err) {
			console.error(err);
			toast.error("Failed to delete account");
			confirmOpen = false;
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex w-full px-4">
	<main class="pb-(--content-pb) flex flex-col gap-6 w-full max-w-120 m-auto pt-4">
		<div class="flex flex-col items-center gap-3 text-center px-4">
			<WarningIcon class="size-10 text-destructive" weight="fill" />
			<h2 class="text-lg font-semibold">Delete account</h2>
			<p class="text-sm text-muted-foreground">
				This action is permanent. All your data, messages, and profile will be removed and cannot be recovered.
			</p>
		</div>
		<div class="grid gap-1.5">
			<Label for="delete-password">Confirm with your password</Label>
			<Input
				id="delete-password"
				type="password"
				bind:value={password}
				disabled={loading}
				placeholder="Current password"
			/>
		</div>
		<Button
			variant="destructive"
			disabled={!password || loading}
			onclick={() => (confirmOpen = true)}
		>
			Delete my account
		</Button>
	</main>
</div>

<AlertDialog.Root bind:open={confirmOpen}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>Are you absolutely sure?</AlertDialog.Title>
			<AlertDialog.Description>
				This will permanently delete your Grindr account. This cannot be undone.
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={loading}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action
				onclick={onDelete}
				disabled={loading}
				class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
			>
				{#if loading}<SpinnerGap class="size-4 animate-spin mr-2" />{/if}
				Yes, delete forever
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>