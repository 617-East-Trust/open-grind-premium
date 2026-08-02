<script lang="ts">
	import {
		ImageIcon,
		MicrophoneIcon,
		PaperPlaneRightIcon,
		SpinnerGap,
	} from "phosphor-svelte";
	import { toast } from "svelte-sonner";
	import { expoOut } from "svelte/easing";
	import { fade } from "svelte/transition";

	import {
		asPublicMediaHash,
		imageHashFromUrl,
		uploadChatMediaFromFile,
	} from "$lib/api/chat-media";
	import ToastUnimplemented from "$lib/components/ToastUnimplemented.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Textarea } from "$lib/components/ui/textarea";
	import type { Message } from "$lib/model/message";

	let { onSend }: { onSend: (params: Message) => void | Promise<void> } =
		$props();

	let textContent = $state("");
	let uploading = $state(false);
	let fileInput: HTMLInputElement | null = $state(null);

	async function onSubmit() {
		const text = textContent.trim();
		if (text === "") return;
		try {
			await onSend({ type: "Text", body: { text } });
			textContent = "";
		} catch (error) {
			console.error(error);
			toast.error("Failed to send message");
		}
	}

	async function onPickImage(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		input.value = "";
		if (!file) return;
		if (!file.type.startsWith("image/")) {
			toast.error("Only images are supported");
			return;
		}
		if (file.size > 12 * 1024 * 1024) {
			toast.error("Image too large (max 12MB)");
			return;
		}

		uploading = true;
		try {
			const uploaded = await uploadChatMediaFromFile(file);
			const dims = await readImageDims(file);
			const hash = asPublicMediaHash(
				uploaded.mediaHash || imageHashFromUrl(uploaded.url),
			);
			const url =
				uploaded.url && uploaded.url.startsWith("http")
					? uploaded.url
					: `https://cdns.grindr.com/images/thumb/320x320/${hash}`;
			await onSend({
				type: "Image",
				body: {
					mediaId: uploaded.mediaId,
					width: dims.width,
					height: dims.height,
					url,
					imageHash: hash,
					takenOnGrindr: false,
					createdAt: Date.now(),
				},
			});
			toast.success("Photo sent");
		} catch (error) {
			console.error(error);
			toast.error(
				error instanceof Error ? error.message.slice(0, 120) : "Upload failed",
			);
		} finally {
			uploading = false;
		}
	}

	function readImageDims(
		file: File,
	): Promise<{ width: number | null; height: number | null }> {
		return new Promise((resolve) => {
			const url = URL.createObjectURL(file);
			const img = new Image();
			img.onload = () => {
				resolve({ width: img.naturalWidth, height: img.naturalHeight });
				URL.revokeObjectURL(url);
			};
			img.onerror = () => {
				resolve({ width: null, height: null });
				URL.revokeObjectURL(url);
			};
			img.src = url;
		});
	}
</script>

<form
	class="relative mx-2 shrink-0 min-h-9.5 min-w-0"
	onsubmit={(event) => {
		event.preventDefault();
		onSubmit().catch((error) => console.error(error));
	}}
>
	<input
		bind:this={fileInput}
		type="file"
		accept="image/*"
		class="hidden"
		onchange={(e) => void onPickImage(e)}
	/>
	<Textarea
		placeholder="Say something..."
		class="min-h-9.5 rounded-[20px] shrink-0 max-h-31.5 py-2 pr-18 h-fit! leading-5 placeholder-shown:truncate"
		disabled={uploading}
		onkeydown={(
			event: KeyboardEvent & {
				currentTarget: EventTarget & HTMLTextAreaElement;
			},
		) => {
			if (event.key === "Enter" && !event.shiftKey) {
				event.preventDefault();
				event.currentTarget.form?.requestSubmit();
			}
		}}
		bind:value={textContent}
	/>
	<div class="actions">
		<Button
			type="button"
			variant="ghost"
			size="icon"
			class="size-9 cursor-pointer p-2"
			disabled={uploading}
			aria-label="Send photo"
			onclick={() => fileInput?.click()}
		>
			{#if uploading}
				<SpinnerGap class="size-4.5 animate-spin text-muted-foreground" />
			{:else}
				<ImageIcon weight="fill" class="size-4.5 text-muted-foreground" />
			{/if}
		</Button>
		{#if textContent === ""}
			<div class="button" transition:fade={{ duration: 400, easing: expoOut }}>
				<Button
					type="button"
					variant="ghost"
					size="icon"
					class="size-full cursor-pointer p-2"
					onclick={() => {
						toast(ToastUnimplemented, {
							componentProps: {
								feature: "Voice messages",
								issue: 35,
							},
						});
					}}
				>
					<MicrophoneIcon
						weight="fill"
						color="var(--muted-foreground)"
						class="size-4.5"
					/>
				</Button>
			</div>
		{:else}
			<div class="button" transition:fade={{ duration: 400, easing: expoOut }}>
				<Button
					type="submit"
					variant="ghost"
					size="icon"
					class="size-full cursor-pointer p-2"
					disabled={uploading}
				>
					<PaperPlaneRightIcon
						weight="fill"
						color="var(--primary)"
						class="size-4.5"
					/>
				</Button>
			</div>
		{/if}
	</div>
</form>

<style lang="postcss">
	@reference "$layout";
	.actions {
		@apply absolute bottom-0 right-0 flex items-center;
	}
	.button {
		@apply size-9.5;
	}
</style>
