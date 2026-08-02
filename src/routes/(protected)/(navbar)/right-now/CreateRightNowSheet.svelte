<script lang="ts">
	import { HouseIcon, SpinnerGap } from "phosphor-svelte";

	import { Button } from "$lib/components/ui/button";
	import * as Sheet from "$lib/components/ui/sheet";
	import { Switch } from "$lib/components/ui/switch";
	import { Textarea } from "$lib/components/ui/textarea";

	const MAX_TEXT = 140;

	let {
		open = $bindable(false),
		submitting = false,
		onSubmit,
	}: {
		open?: boolean;
		submitting?: boolean;
		onSubmit: (input: {
			text: string;
			hosting: boolean;
		}) => void | Promise<void>;
	} = $props();

	let text = $state("");
	let hosting = $state(false);

	const remaining = $derived(MAX_TEXT - text.length);
	const canSubmit = $derived(!submitting && text.length <= MAX_TEXT);

	$effect(() => {
		if (!open) {
			// Reset draft when closed so next open is clean
			text = "";
			hosting = false;
		}
	});

	async function submit() {
		if (!canSubmit) return;
		await onSubmit({ text: text.trim(), hosting });
	}
</script>

<Sheet.Root bind:open>
	<Sheet.Content side="bottom" class="max-h-[90dvh] rounded-t-3xl gap-0 p-0">
		<Sheet.Header class="px-5 pt-5 pb-3 border-b border-border/60">
			<Sheet.Title>Post Right Now</Sheet.Title>
			<Sheet.Description>
				Visible nearby for 1 hour. Optional text + hosting status.
			</Sheet.Description>
		</Sheet.Header>

		<div class="px-5 py-4 flex flex-col gap-4">
			<div class="flex flex-col gap-2">
				<label for="rn-text" class="text-xs font-medium text-muted-foreground">
					What are you looking for?
				</label>
				<Textarea
					id="rn-text"
					bind:value={text}
					placeholder="Optional — e.g. free tonight, drinks, chill…"
					maxlength={MAX_TEXT}
					rows={4}
					class="min-h-24"
					disabled={submitting}
				/>
				<p
					class={[
						"text-[11px] text-end",
						remaining < 20 ? "text-destructive" : "text-muted-foreground",
					]}
				>
					{remaining}
				</p>
			</div>

			<label
				class="flex items-center justify-between gap-3 rounded-2xl border border-border px-4 py-3"
			>
				<span class="flex items-center gap-2 min-w-0">
					<HouseIcon
						weight={hosting ? "fill" : "regular"}
						class="size-5 shrink-0 text-accent"
					/>
					<span class="min-w-0">
						<span class="block text-sm font-medium">Hosting</span>
						<span class="block text-xs text-muted-foreground">
							Show that people can come to you
						</span>
					</span>
				</span>
				<Switch bind:checked={hosting} disabled={submitting} />
			</label>
		</div>

		<Sheet.Footer
			class="px-5 pb-[calc(1.25rem+var(--safe-area-bottom))] pt-2 border-t border-border/60 flex-row gap-2"
		>
			<Button
				variant="outline"
				class="flex-1"
				disabled={submitting}
				onclick={() => (open = false)}
			>
				Cancel
			</Button>
			<Button class="flex-1" disabled={!canSubmit} onclick={() => void submit()}>
				{#if submitting}
					<SpinnerGap class="size-4 animate-spin" />
					Posting…
				{:else}
					Post for 1 hour
				{/if}
			</Button>
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>
