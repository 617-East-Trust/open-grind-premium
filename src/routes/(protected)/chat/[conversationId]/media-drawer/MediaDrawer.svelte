<script lang="ts">
	import {
		FolderOpenIcon,
		ImageIcon,
		NavigationArrowIcon,
		PlusIcon,
		CheckIcon,
		SpinnerGap,
	} from "phosphor-svelte";
	import { toast } from "svelte-sonner";

	import {
		addMediaToDrawer,
		asPublicMediaHash,
		imageHashFromUrl,
	} from "$lib/api/chat-media";
	import { type DrawerMedia, getDrawerMedia } from "$lib/api/drawer";
	import ApiErrorDisplay from "$lib/components/ApiErrorDisplay.svelte";
	import { Badge } from "$lib/components/ui/badge";
	import { Button } from "$lib/components/ui/button";
	import * as Drawer from "$lib/components/ui/drawer";
	import * as Empty from "$lib/components/ui/empty";
	import { Skeleton } from "$lib/components/ui/skeleton";
	import * as Tabs from "$lib/components/ui/tabs";
	import type { Message } from "$lib/model/message";
	import { SelectionSet } from "$lib/util/selection.svelte";
	import ToastUnimplemented from "$lib/components/ToastUnimplemented.svelte";

	let {
		open = $bindable(false),
		conversationId,
		onSend,
	}: {
		open?: boolean;
		conversationId: string;
		onSend: (message: Message) => void | Promise<void>;
	} = $props();

	const selected = new SelectionSet<number>(10);
	let media = $state<DrawerMedia[] | null>(null);
	let error = $state<unknown>(null);
	let uploadingCount = $state(0);
	let sending = $state(false);
	let tab = $state("media");
	let fileInput: HTMLInputElement | null = $state(null);

	async function load() {
		media = null;
		error = null;
		try {
			media = await getDrawerMedia(conversationId);
		} catch (err) {
			console.error(err);
			error = err;
		}
	}

	$effect(() => {
		if (open && conversationId) {
			void load();
		}
	});

	function toggle(id: number) {
		selected.toggle(id);
	}

	async function addPhotos(files: FileList | File[]) {
		const list = Array.from(files).filter((f) => f.type.startsWith("image/"));
		if (list.length === 0) return;
		uploadingCount += list.length;
		for (const file of list) {
			try {
				const added = await addMediaToDrawer(file);
				media = [added, ...(media ?? []).filter((m) => m.id !== added.id)];
			} catch (err) {
				console.error(err);
				toast.error(`Couldn't add ${file.name}`);
			} finally {
				uploadingCount -= 1;
			}
		}
	}

	async function sendSelected() {
		if (!media || selected.size === 0 || sending) return;
		sending = true;
		const items = media.filter((m) => selected.has(m.id));
		selected.clear();
		try {
			for (const item of items) {
				item.used = true;
				const hash = asPublicMediaHash(
					imageHashFromUrl(item.url) || String(item.id),
				);
				await onSend({
					type: "Image",
					body: {
						mediaId: item.id,
						width: null,
						height: null,
						url: item.url,
						imageHash: hash,
						takenOnGrindr: item.takenOnGrindr,
						createdAt: item.createdTs,
					},
				});
			}
			toast.success(
				items.length === 1 ? "Photo sent" : `${items.length} photos sent`,
			);
			open = false;
		} catch (err) {
			console.error(err);
			toast.error("Failed to send media");
		} finally {
			sending = false;
		}
	}
</script>

<input
	bind:this={fileInput}
	type="file"
	accept="image/*"
	multiple
	class="hidden"
	onchange={(e) => {
		const input = e.currentTarget as HTMLInputElement;
		if (input.files) void addPhotos(input.files);
		input.value = "";
	}}
/>

<Drawer.Root bind:open>
	<Drawer.Content
		class="mx-auto max-w-200 border-none bg-transparent p-0 shadow-none before:hidden h-[min(85dvh,720px)]"
	>
		<div
			class="flex h-full flex-col rounded-t-4xl border border-border bg-popover shadow-xl"
		>
			<div class="mx-auto my-3 h-1.5 w-20 shrink-0 rounded-full bg-muted"></div>
			<div class="flex items-center justify-between px-4 pb-2">
				<h2 class="text-sm font-semibold">Media drawer</h2>
				{#if selected.size > 0}
					<Button size="sm" disabled={sending} onclick={() => void sendSelected()}>
						{#if sending}
							<SpinnerGap class="size-4 animate-spin" />
						{:else}
							Send
							<Badge
								variant="secondary"
								class="bg-primary-foreground/15 text-primary-foreground"
							>
								{selected.size}
							</Badge>
						{/if}
					</Button>
				{/if}
			</div>

			<Tabs.Root bind:value={tab} class="flex min-h-0 flex-1 flex-col gap-0">
				<div class="min-h-0 flex-1 overflow-y-auto px-4 pb-24">
					<Tabs.Content value="media" class="mt-0">
						{#if error !== null}
							<div class="flex min-h-48">
								<ApiErrorDisplay {error} class="m-auto" />
							</div>
						{:else if media === null}
							<div class="grid grid-cols-3 gap-1 sm:grid-cols-4">
								{#each Array(12)}
									<Skeleton class="aspect-square rounded-md" />
								{/each}
							</div>
						{:else if media.length === 0 && uploadingCount === 0}
							<Empty.Root class="py-10">
								<Empty.Header>
									<Empty.Media variant="icon">
										<ImageIcon weight="fill" />
									</Empty.Media>
									<Empty.Title>No media yet</Empty.Title>
									<Empty.Description>
										Add photos to the drawer, then select and send.
									</Empty.Description>
								</Empty.Header>
								<Empty.Content>
									<Button onclick={() => fileInput?.click()}>
										<PlusIcon weight="bold" />
										Add photo
									</Button>
								</Empty.Content>
							</Empty.Root>
						{:else}
							<div class="grid grid-cols-3 gap-1 sm:grid-cols-4">
								<button
									type="button"
									class="flex aspect-square flex-col items-center justify-center gap-1 rounded-md bg-muted/60 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
									aria-label="Add photo"
									onclick={() => fileInput?.click()}
								>
									<PlusIcon weight="bold" class="size-6" />
									<span class="text-xs font-medium">Add</span>
								</button>
								{#each Array(uploadingCount)}
									<Skeleton class="aspect-square rounded-md" />
								{/each}
								{#each media as item (item.id)}
									{@const isSelected = selected.has(item.id)}
									<button
										type="button"
										class={[
											"relative aspect-square overflow-hidden rounded-md",
											{
												"cursor-pointer":
													selected.canSelectMore || isSelected,
											},
										]}
										aria-pressed={isSelected}
										onclick={() => toggle(item.id)}
									>
										<img
											src={item.url}
											alt=""
											class="size-full object-cover bg-muted"
											draggable="false"
											loading="lazy"
										/>
										{#if isSelected}
											<div
												class="absolute inset-0 flex items-center justify-center bg-primary/45 outline-2 -outline-offset-2 outline-primary"
											>
												<div
													class="flex size-8 items-center justify-center rounded-full bg-primary"
												>
													<CheckIcon weight="bold" class="size-5 text-white" />
												</div>
											</div>
										{:else if item.used}
											<div
												class="absolute inset-0 flex items-center justify-center bg-black/50"
											>
												<span class="text-sm font-medium text-white">Sent</span>
											</div>
										{/if}
									</button>
								{/each}
							</div>
						{/if}
					</Tabs.Content>
					<Tabs.Content value="albums" class="mt-0 py-8">
						<button
							type="button"
							class="w-full text-center text-sm text-muted-foreground"
							onclick={() =>
								toast(ToastUnimplemented, {
									componentProps: { feature: "Sharing albums", issue: 33 },
								})}
						>
							Album sharing not implemented yet
						</button>
					</Tabs.Content>
					<Tabs.Content value="location" class="mt-0 py-8">
						<button
							type="button"
							class="w-full text-center text-sm text-muted-foreground"
							onclick={() =>
								toast(ToastUnimplemented, {
									componentProps: { feature: "Sharing location", issue: 35 },
								})}
						>
							Location sharing not implemented yet
						</button>
					</Tabs.Content>
				</div>

				<Drawer.Footer
					class="absolute inset-x-0 bottom-0 items-center border-t border-border bg-popover/95 pt-2 pb-[calc(0.5rem+var(--safe-area-bottom))] backdrop-blur"
				>
					<Tabs.List class="w-full max-w-sm">
						<Tabs.Trigger value="media" class="flex-1 flex-col gap-0.5 h-auto py-1.5">
							<ImageIcon weight="fill" class="size-5" />
							Media
						</Tabs.Trigger>
						<Tabs.Trigger value="albums" class="flex-1 flex-col gap-0.5 h-auto py-1.5">
							<FolderOpenIcon weight="fill" class="size-5" />
							Albums
						</Tabs.Trigger>
						<Tabs.Trigger
							value="location"
							class="flex-1 flex-col gap-0.5 h-auto py-1.5"
						>
							<NavigationArrowIcon weight="fill" class="size-5" />
							Location
						</Tabs.Trigger>
					</Tabs.List>
				</Drawer.Footer>
			</Tabs.Root>
		</div>
	</Drawer.Content>
</Drawer.Root>
