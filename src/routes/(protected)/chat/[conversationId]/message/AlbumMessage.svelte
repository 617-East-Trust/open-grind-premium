<script lang="ts">
	import "photoswipe/style.css";
	import toast from "svelte-french-toast";
	import PhotoSwipeLightbox from "photoswipe/lightbox";
	import { ImagesIcon, LockIcon, VideoIcon } from "phosphor-svelte";
	import type { AlbumMessage } from "$lib/model/message";
	import { getMessageContext, getMessageMetaContext } from "./context";
	import { getAlbumContent, type AlbumContentResponse } from "$lib/api/album";

	let { message }: { message: AlbumMessage["body"] } = $props();

	const { lastInStack, msgOut } = $derived(getMessageContext()());
	const { clone, setRef, adornments } = $derived(getMessageMetaContext()());

	let el: HTMLElement | null = $state(null);
	$effect(() => {
		setRef(el ?? null);
	});

	const className: import("svelte/elements").ClassValue = $derived([
		"aspect-3/4 h-auto relative",
		{
			"ring ring-accent": message.hasUnseenContent,
			"w-2/5 min-w-35 max-w-60 ms-3": !clone,
			"size-full": clone,
		},
	]);

	const contentClass: import("svelte/elements").ClassValue = $derived([
		"rounded-xl",
		{
			"rounded-es-[6px]": lastInStack && !msgOut,
			"rounded-ee-[6px]": lastInStack && msgOut,
		},
	]);

	let open = $state(false);
	let loading = $state(false);
	let album:
		| (AlbumContentResponse & {
				content: (AlbumContentResponse["content"][number] & {
					width: number;
					height: number;
				})[];
		  })
		| null = $state(null);

	$effect(() => {
		if (open && !album) {
			async function load() {
				loading = true;
				try {
					album = await getAlbumContent(message.albumId).then(async (res) => ({
						...res,
						content: await Promise.all(
							res.content.map(async (slide) => {
								if (slide.contentType.startsWith("video/")) {
									const video = document.createElement("video");
									video.src = slide.url ?? "";
									video.load();
									await new Promise<void>((resolve, reject) => {
										if (video.readyState >= 1) resolve();
										video.addEventListener("loadedmetadata", () => resolve(), {
											once: true,
										});
										video.addEventListener("error", () => reject(), {
											once: true,
										});
									});
									video.remove();
									return {
										...slide,
										width: video.videoWidth,
										height: video.videoHeight,
									};
								} else {
									const img = document.createElement("img");
									img.src = slide.url ?? "";
									await new Promise<void>((resolve, reject) => {
										if (img.complete) resolve();
										img.addEventListener("load", () => resolve(), {
											once: true,
										});
										img.addEventListener("error", () => reject(), {
											once: true,
										});
									});
									img.remove();
									return {
										...slide,
										src: img.src,
										width: img.naturalWidth,
										height: img.naturalHeight,
									};
								}
							}),
						),
					}));
					console.log(album);
				} catch (error) {
					console.error(error);
					toast.error("Failed to load album content");
					open = false;
				} finally {
					loading = false;
				}
			}
			load();
		}
	});

	$effect(() => {
		if (!open || !album) return;
		let lightbox = new PhotoSwipeLightbox({
			showHideAnimationType: "fade",
			pswpModule: () => import("photoswipe"),
			mainClass: `pswp--buttons-visible`,
		});
		lightbox.addFilter("numItems", () => {
			return album?.content.length ?? 0;
		});
		lightbox.addFilter("itemData", (_, index) => {
			if (!album) throw new Error("Album not loaded");
			const { url, width, height } = album.content[index];
			return {
				src: url,
				width,
				height,
				// TODO: thumburl?
			};
		});
		lightbox.on("contentLoad", (e) => {
			const { content } = e;

			const slide = album?.content[content.index];
			if (slide?.contentType.startsWith("video/")) {
				e.preventDefault();

				content.element = document.createElement("div");
				const video = document.createElement("video");
				video.src = slide.url;
				video.controls = true;
				video.playsInline = true;
				video.className = "max-w-full max-h-[80vh]";
				content.element.appendChild(video);

				content.state = "loading";

				if (video.readyState >= 3) {
					content.onLoaded();
				} else {
					video.addEventListener("loadeddata", () => {
						content.onLoaded();
					});

					video.addEventListener("error", (e) => {
						content.onError();
					});
				}
			}
		});
		lightbox.on("closingAnimationEnd", () => {
			open = false;
		});
		lightbox.init();
		lightbox.loadAndOpen(0);
		return () => lightbox.destroy();
	});
</script>

{#if message.isViewable}
	<button
		class={[
			className,
			contentClass,
			{ "cursor-pointer": !loading && !open, "opacity-50": loading },
		]}
		onclick={() => (open = true)}
		disabled={loading || open}
		bind:this={el}
	>
		<img
			src={message.coverUrl}
			alt=""
			class="w-full rounded-[inherit] bg-card-foreground/10 h-full object-cover absolute top-0 left-0"
		/>
		<div class={["absolute top-0 left-0 size-full @container", contentClass]}>
			<div
				class="*:bg-card *:rounded-full *:w-[20cqw] *:aspect-square *:p-2 absolute bottom-1/5 left-1/2 -translate-x-1/2 flex items-center gap-1 px-2 py-0.5"
			>
				{#if message.hasPhoto}
					<div>
						<ImagesIcon
							width="100%"
							height="auto"
							weight="fill"
							color="var(--color-neutral-200)"
						/>
					</div>
				{/if}
				{#if message.hasVideo}
					<div>
						<VideoIcon
							width="100%"
							height="auto"
							weight="fill"
							color="var(--color-neutral-200)"
						/>
					</div>
				{/if}
			</div>
		</div>
		{@render adornments?.()}
	</button>
{:else}
	<div class={[className, contentClass]} bind:this={el}>
		<div
			class="size-full flex justify-center items-center bg-card-foreground/10 rounded-lg"
		>
			<LockIcon weight="fill" size={36} color="var(--color-neutral-600)" />
		</div>
		{@render adornments?.()}
	</div>
{/if}

<style>
	:global(.pswp__img) {
		object-fit: contain;
	}
</style>
