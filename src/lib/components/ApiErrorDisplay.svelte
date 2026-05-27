<script lang="ts">
	import * as clipboard from "@tauri-apps/plugin-clipboard-manager";
	import { toast } from "svelte-sonner";

	import { ApiError } from "$lib/api/api-error";
	import {
		getPreferences,
		setPreferences,
	} from "$lib/app-data/preferences.svelte";
	import { Button } from "$lib/components/ui/button";

	let {
		error,
		class: className,
		buttonVariant = "outline",
	}: {
		error: unknown;
		class?: import("svelte/elements").ClassValue;
		buttonVariant?: import("$lib/components/ui/button").ButtonVariant;
	} = $props();

	function getCopyText(): string {
		if (error instanceof ApiError) {
			return error.copyableText();
		}
		if (error instanceof Error) {
			return JSON.stringify(
				{ error: error.message, stack: error.stack },
				null,
				2,
			);
		}
		return String(error);
	}

	function copy() {
		clipboard
			.writeText(getCopyText())
			.then(async () => {
				toast.success("Copied to clipboard");
				if (
					await getPreferences().then((p) => p.warnBeforeCopyingErrorDetails)
				) {
					toast.warning("Be mindful of what you share in the internet!", {
						description:
							"Error details may contain your personal and sensitive data. Redact before sharing them with others.",
						duration: 7000,
					});
					void setPreferences({ warnBeforeCopyingErrorDetails: false });
				}
			})
			.catch((e) => console.error(e));
	}
</script>

<div class={["flex flex-col items-center gap-2 p-4", className]}>
	<p class="text-muted-foreground text-sm">Something went wrong</p>
	<Button variant={buttonVariant} size="sm" onclick={copy}>Copy details</Button>
</div>
