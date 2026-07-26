<script lang="ts">
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Label } from "$lib/components/ui/label";
	import { changeEmail } from "$lib/api/account";
	import { SpinnerGap } from "phosphor-svelte";

	let email = $state("");
	let password = $state("");
	let loading = $state(false);

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!email || !password) return;
		loading = true;
		try {
			await changeEmail(email, password);
			toast.success("Email updated");
			email = "";
			password = "";
		} catch (err) {
			console.error(err);
			toast.error("Failed to update email");
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex w-full px-4">
	<main class="pb-(--content-pb) flex flex-col gap-4 w-full max-w-120 m-auto pt-2">
		<form onsubmit={onSubmit} class="flex flex-col gap-4">
			<div class="grid gap-1.5">
				<Label for="new-email">New email</Label>
				<Input id="new-email" type="email" bind:value={email} disabled={loading} required />
			</div>
			<div class="grid gap-1.5">
				<Label for="confirm-password">Current password</Label>
				<Input id="confirm-password" type="password" bind:value={password} disabled={loading} required />
			</div>
			<Button type="submit" disabled={loading || !email || !password}>
				{#if loading}<SpinnerGap class="size-4 animate-spin mr-2" />{/if}
				Update email
			</Button>
		</form>
	</main>
</div>