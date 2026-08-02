<script lang="ts">
	import {
		ImageIcon,
		MicrophoneIcon,
		PaperPlaneRightIcon,
		PlusIcon,
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
	import MediaDrawer from "./media-drawer/MediaDrawer.svelte";

	let {
		conversationId,
		onSend,
	}: {
		conversationId: string;
		onSend: (params: Message) => void | Promise<void>;
	} = $props();

	let textContent = $state("");
	let uploading = $state(false);
	let drawerOpen = $state(false);
	let quickFileInput: HTMLInputElement | null = $state(null);

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

	async function quickSendFiles(files: FileList | File[]) {
		const list = Array.from(files)
			.filter((f) => f.type.startsWith("image/"))
			.slice(0, 8);
		if (list.length === 0) return;
		uploading = true;
		let sent = 0;
		try {
			for (const file of list) {
				if (file.size > 12 * 1024 * 1024) continue;
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
				sent += 1;
			}
			if (sent > 0) {
				toast.success(sent === 1 ? "Photo sent" : `${sent} photos sent`);
			} else {
				toast.error("Upload failed");
			}
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
		bind:this={quickFileInput}
		type="file"
		accept="image/*"
		multiple
		class="hidden"
		onchange={(e) => {
			const input = e.currentTarget as HTMLInputElement;
			if (input.files) void quickSendFiles(input.files);
			input.value = "";
		}}
	/>
	<Textarea
		placeholder="Say something..."
		class="min-h-9.5 rounded-[20px] shrink-0 max-h-31.5 py-2 pr-24 h-fit! leading-5 placeholder-shown:truncate"
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
			aria-label="Open media drawer"
			onclick={() => (drawerOpen = true)}
		>
			<PlusIcon weight="bold" class="size-4.5 text-muted-foreground" />
		</Button>
		<Button
			type="button"
			variant="ghost"
			size="icon"
			class="size-9 cursor-pointer p-2"
			disabled={uploading}
			aria-label="Quick send photo"
			onclick={() => quickFileInput?.click()}
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

<MediaDrawer bind:open={drawerOpen} {conversationId} {onSend} />

<style lang="postcss">
	@reference "$layout";
	.actions {
		@apply absolute bottom-0 right-0 flex items-center;
	}
	.button {
		@apply size-9.5;
	}
</style>
