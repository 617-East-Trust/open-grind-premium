<script lang="ts">
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import { changePassword } from "$lib/api/account";
	import { SpinnerGap } from "phosphor-svelte";

	let currentPassword = $state("");
	let newPassword = $state("");
	let confirmPassword = $state("");
	let loading = $state(false);

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (newPassword !== confirmPassword) {
			toast.error("New passwords do not match");
			return;
		}
		if (newPassword.length < 8) {
			toast.error("Password must be at least 8 characters");
			return;
		}
		loading = true;
		try {
			await changePassword(currentPassword, newPassword);
			toast.success("Password updated");
			currentPassword = "";
			newPassword = "";
			confirmPassword = "";
		} catch (err) {
			console.error(err);
			toast.error("Failed to update password");
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex w-full px-4">
	<main class="pb-(--content-pb) flex flex-col gap-4 w-full max-w-120 m-auto pt-2">
		<form onsubmit={onSubmit} class="flex flex-col gap-4">
			<div class="grid gap-1.5">
				<Label for="current">Current password</Label>
				<Input id="current" type="password" bind:value={currentPassword} disabled={loading} required />
			</div>
			<div class="grid gap-1.5">
				<Label for="new">New password</Label>
				<Input id="new" type="password" bind:value={newPassword} disabled={loading} required />
			</div>
			<div class="grid gap-1.5">
				<Label for="confirm">Confirm new password</Label>
				<Input id="confirm" type="password" bind:value={confirmPassword} disabled={loading} required />
			</div>
			<Button type="submit" disabled={loading || !currentPassword || !newPassword}>
				{#if loading}<SpinnerGap class="size-4 animate-spin mr-2" />{/if}
				Update password
			</Button>
		</form>
	</main>
</div>