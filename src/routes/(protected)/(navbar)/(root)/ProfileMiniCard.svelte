<script lang="ts">
	import { ChatIcon, StarIcon } from "phosphor-svelte";

	import { Badge } from "$lib/components/ui/badge";
	import UserAvatar from "$lib/components/UserAvatar.svelte";

	let {
		id,
		displayName = null,
		age = null,
		distance = null,
		medias = null,
		unread = null,
		isFavorite = false,
		hadRecentChat = false,
	}: {
		id: number;
		displayName?: string | null;
		age?: number | null;
		distance?: number | null;
		medias?: { mediaHash: string }[] | null;
		unread?: number | null;
		isFavorite?: boolean;
		hadRecentChat?: boolean;
	} = $props();

	const profilePicture = $derived(medias?.[0]);

	// API returns distance in metres; convert to miles for display.
	const METRES_PER_MILE = 1609.344;
	function formatDistance(metres: number): string {
		const miles = metres / METRES_PER_MILE;
		if (miles < 0.1) {
			return `${Math.round(metres * 3.28084)} ft`;
		} else if (miles < 10) {
			return `${miles.toFixed(1)} mi`;
		} else {
			return `${Math.round(miles)} mi`;
		}
	}
</script>

<a href="/profile/{id}" class="aspect-square relative flex items-end">
	<div class="absolute w-full h-full bg-stone-700">
		<UserAvatar
			mediaHash={profilePicture?.mediaHash ?? null}
			class="size-full"
			size="xl"
		/>
	</div>
	{#if distance}
		<span
			class="absolute top-1 right-1 border-transparent bg-transparent text-[11px] px-1 h-4 tracking-tight font-medium text-white/80 text-shadow-stroke"
		>
			{formatDistance(distance)}
		</span>
	{/if}
	<div class="w-full z-1 flex p-0.5 gap-0.5">
		{#if displayName !== null || age !== null}
			<Badge
				variant="outline"
				class="gap-0 max-w-full bg-popover/20 backdrop-blur-2xl min-w-0 shrink"
			>
				{#if displayName !== null}
					<span class="truncate block shrink font-semibold">
						{displayName}
					</span>
				{/if}
				{#if displayName !== null && age !== null}
					,&nbsp;
				{/if}
				{#if age !== null}
					<span class="truncate line-clamp-1 block max-w-full shrink-0">
						{age}
					</span>
				{/if}
			</Badge>
		{/if}
		{#if unread !== null && unread > 0}
			<span
				class="size-5 bg-primary inline-block rounded-full border border-black/20 shrink-0"
			>
				{unread}
			</span>
		{/if}
		{#if isFavorite || hadRecentChat}
			<div
				class="absolute top-2 inset-s-2 flex gap-1 items-center w-1/6 flex-col"
			>
				{#if isFavorite}
					<div class="badge">
						<StarIcon
							weight="fill"
							class="text-yellow-500 icon size-4/6 m-auto"
						/>
					</div>
				{/if}
				{#if hadRecentChat}
					<div class="badge">
						<ChatIcon
							weight="fill"
							class="text-sky-400 size-3/5 m-auto -translate-y-px"
						/>
					</div>
				{/if}
			</div>
		{/if}
	</div>
</a>

<style lang="postcss">
	@reference "$layout";

	.text-shadow-stroke {
		text-shadow:
			0px 1px 1px rgba(0, 0, 0, 0.2),
			0px 0px 2px rgba(0, 0, 0, 0.2);
	}

	.badge {
		@apply flex bg-popover/40 rounded-full backdrop-blur-2xl w-full h-auto aspect-square border border-white/10;
	}
</style>
