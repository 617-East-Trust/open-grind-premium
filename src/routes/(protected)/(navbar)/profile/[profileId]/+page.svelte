<script lang="ts">
	import { goto } from "$app/navigation";
	import { page } from "$app/state";
	import { CaretLeftIcon, CaretRightIcon, ChatCircleIcon } from "phosphor-svelte";
	import { toast } from "svelte-sonner";

	import { getProfile, invalidateProfile } from "$lib/api/profile";
	import { recordProfileView } from "$lib/api/interest/views";
	import PullToRefresh from "$lib/components/PullToRefresh.svelte";
	import TapButtons from "$lib/components/TapButtons.svelte";
	import ApiErrorDisplay from "$lib/components/ApiErrorDisplay.svelte";
	import Button from "$lib/components/ui/button/button.svelte";
	import { Skeleton } from "$lib/components/ui/skeleton";
	import type { Profile } from "$lib/model/profile";
	import { getAdjacentProfileId } from "$lib/stores/grid-order.svelte";
	import AboutMe from "./AboutMe.svelte";
	import Distance from "./Distance.svelte";
	import Ethnicity from "./Ethnicity.svelte";
	import Genders from "./GendersPronouns.svelte";
	import HealthPractices from "./HealthPractices.svelte";
	import Height from "./HeightWeightBodyType.svelte";
	import HivStatus from "./HivStatus.svelte";
	import ImageCarousel from "./ImageCarousel.svelte";
	import LastTested from "./LastTested.svelte";
	import LookingFor from "./LookingFor.svelte";
	import MeetAt from "./MeetAt.svelte";
	import NSFWPics from "./NSFWPics.svelte";
	import OnlineStatus from "./OnlineStatus.svelte";
	import ProfileTags from "./ProfileTags.svelte";
	import RelationshipStatus from "./RelationshipStatus.svelte";
	import SexualPosition from "./SexualPosition.svelte";
	import Socials from "./Socials.svelte";
	import Tribes from "./Tribes.svelte";

	let { data }: import("./$types").PageProps = $props();

	const profileId = $derived(Number(page.params.profileId));
	const ourProfileId = $derived(data.ourProfileId);
	const isOurProfile = $derived(profileId === ourProfileId);
	const conversationId = $derived(
		[profileId, ourProfileId].toSorted((a, b) => a - b).join(":"),
	);

	let profile = $state<Profile | null>(null);
	let loading = $state(true);
	let refreshing = $state(false);
	let loadError = $state<Error | null>(null);

	// --- Horizontal swipe between grid profiles (grindrx pattern) ----------
	const SWIPE_TRIGGER = 70;
	const prevProfileId = $derived(getAdjacentProfileId(profileId, "prev"));
	const nextProfileId = $derived(getAdjacentProfileId(profileId, "next"));

	let swipeStartX = $state<number | null>(null);
	let swipeStartY = $state<number | null>(null);
	let swipeDx = $state(0);
	let swiping = $state(false);

	function goToProfile(id: number) {
		goto(`/profile/${id}`).catch((err) => console.error(err));
	}

	function onSwipeStart(event: TouchEvent) {
		if (event.touches.length !== 1) {
			swipeStartX = null;
			return;
		}
		// Don't steal horizontal swipes that start on carousel/lightbox chrome
		const target = event.target;
		if (
			target instanceof Element &&
			(target.closest(".pswp") || target.closest("[data-no-profile-swipe]"))
		) {
			swipeStartX = null;
			return;
		}
		swipeStartX = event.touches[0].clientX;
		swipeStartY = event.touches[0].clientY;
		swipeDx = 0;
		swiping = false;
	}

	function onSwipeMove(event: TouchEvent) {
		if (swipeStartX === null || swipeStartY === null) return;
		if (event.touches.length !== 1) {
			swipeStartX = null;
			swipeDx = 0;
			swiping = false;
			return;
		}
		const dx = event.touches[0].clientX - swipeStartX;
		const dy = event.touches[0].clientY - swipeStartY;
		if (!swiping) {
			if (Math.abs(dx) < 10 && Math.abs(dy) < 10) return;
			if (Math.abs(dx) <= Math.abs(dy)) {
				// Vertical — leave to pull-to-refresh / scroll
				swipeStartX = null;
				return;
			}
			swiping = true;
		}
		const canGo = dx < 0 ? nextProfileId !== null : prevProfileId !== null;
		swipeDx = canGo ? dx : dx * 0.25;
	}

	function onSwipeEnd() {
		if (swipeStartX === null) {
			swipeDx = 0;
			swiping = false;
			return;
		}
		const dx = swipeDx;
		swipeStartX = null;
		swipeStartY = null;
		swipeDx = 0;
		swiping = false;
		if (dx <= -SWIPE_TRIGGER && nextProfileId !== null) {
			goToProfile(nextProfileId);
		} else if (dx >= SWIPE_TRIGGER && prevProfileId !== null) {
			goToProfile(prevProfileId);
		}
	}

	async function loadProfile(id: number, isRefresh: boolean) {
		if (!Number.isFinite(id) || id <= 0) return;

		if (isRefresh) {
			if (refreshing || loading) return;
			refreshing = true;
			invalidateProfile(id);
		} else {
			loading = true;
			loadError = null;
			profile = null;
		}

		try {
			const result = await getProfile(id, { force: isRefresh });
			if (id !== profileId) return;
			profile = result;
			loadError = null;
		} catch (error) {
			if (id !== profileId) return;
			const err =
				error instanceof Error ? error : new Error(String(error));
			loadError = err;
			if (isRefresh) {
				toast.error("Failed to refresh profile");
			} else {
				profile = null;
			}
		} finally {
			if (id === profileId) {
				loading = false;
				refreshing = false;
			}
		}
	}

	$effect(() => {
		const id = profileId;
		if (!Number.isFinite(id) || id <= 0) return;
		void loadProfile(id, false);
	});

	function refresh() {
		void loadProfile(profileId, true);
	}

	$effect(() => {
		if (!isOurProfile && profileId > 0) {
			void recordProfileView({ profileId }).catch(() => {});
		}
	});
</script>

<PullToRefresh
	refreshing={refreshing}
	disabled={loading && !profile}
	onrefresh={refresh}
>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="flex flex-1"
		ontouchstart={onSwipeStart}
		ontouchmove={onSwipeMove}
		ontouchend={onSwipeEnd}
		ontouchcancel={onSwipeEnd}
	>
		<main
			class="w-full max-w-200 flex-1 mx-auto relative"
			style="transform: translateX({swipeDx}px); transition: {swiping
				? 'none'
				: 'transform 0.2s ease'};"
		>
			{#if loading && !profile}
				<div class="flex flex-col">
					<Skeleton
						class="w-full aspect-3/4 max-h-[min(70vh,500px)] rounded-none"
					/>
					<div class="flex flex-col p-4 gap-3">
						<Skeleton class="h-8 w-40 rounded-lg" />
						<Skeleton class="h-4 w-28 rounded" />
						<Skeleton class="h-4 w-36 rounded" />
					</div>
				</div>
			{:else if loadError && !profile}
				<div class="h-full flex min-h-60">
					<ApiErrorDisplay error={loadError} class="m-auto" />
				</div>
			{:else if profile}
				{@const {
					displayName,
					age,
					onlineUntil,
					seen,
					distance,
					sexualPosition,
					height,
					weight,
					bodyType,
					profileTags,
					aboutMe,
					genders,
					pronouns,
					ethnicity,
					relationshipStatus,
					grindrTribes,
					lookingFor,
					meetAt,
					nsfw,
					hivStatus,
					lastTestedDate: lastTestedDateValue,
					sexualHealth: sexualHealthValue,
					socialNetworks,
					medias,
				} = profile}
				<div data-no-profile-swipe>
					<ImageCarousel {medias} />
				</div>
				{#if !isOurProfile}
					<nav class="absolute -translate-y-1/2 right-2 z-10">
						<Button
							size="icon-lg"
							class="size-14"
							href="/chat/{conversationId}"
						>
							<ChatCircleIcon weight="fill" class="size-8" />
						</Button>
					</nav>
				{/if}
				<div class="flex flex-col p-4 pb-12">
					<div class="flex items-center justify-between gap-2">
						<h1 class="text-2xl wrap-break-word min-w-0 flex-1">
							{#if displayName !== null}
								<span class="font-semibold">
									{displayName}
								</span>{:else}<span
									class="font-normal tracking-tight italic text-muted-foreground"
								>
									Someone
								</span>{/if}{#if age !== null}, {age}
							{/if}
						</h1>
						{#if prevProfileId !== null || nextProfileId !== null}
							<div class="flex gap-1 shrink-0">
								<Button
									variant="outline"
									size="icon-sm"
									disabled={prevProfileId === null}
									aria-label="Previous profile"
									onclick={() =>
										prevProfileId !== null && goToProfile(prevProfileId)}
								>
									<CaretLeftIcon class="size-4" weight="bold" />
								</Button>
								<Button
									variant="outline"
									size="icon-sm"
									disabled={nextProfileId === null}
									aria-label="Next profile"
									onclick={() =>
										nextProfileId !== null && goToProfile(nextProfileId)}
								>
									<CaretRightIcon class="size-4" weight="bold" />
								</Button>
							</div>
						{/if}
					</div>
					<div class="flex items-center gap-3 text-sm mt-1">
						<OnlineStatus onlineUntil={onlineUntil ?? null} {seen} />
						<Distance {distance} />
					</div>
					{#if !isOurProfile}
						<div class="mt-3">
							<TapButtons {profileId} />
						</div>
					{/if}
					{#if sexualPosition !== null || height !== null || weight !== null || bodyType !== null}
						<div class="flex items-center gap-3 text-sm mt-2">
							{#if sexualPosition !== null && sexualPosition !== undefined}
								<SexualPosition {sexualPosition} />
							{/if}
							<Height {height} {weight} {bodyType} />
						</div>
					{/if}
					<ProfileTags tags={profileTags} />
					{#if aboutMe !== null}
						<AboutMe>{aboutMe}</AboutMe>
					{/if}
					{#if (genders && genders.length > 0) || (pronouns && pronouns.length > 0) || ethnicity !== null || relationshipStatus !== null || (grindrTribes && grindrTribes.length > 0)}
						<div class="flex flex-col gap-2 mt-4">
							<span class="uppercase text-sm text-muted-foreground"
								>Stats</span
							>
							<Genders {genders} {pronouns} />
							<Tribes tribes={grindrTribes} />
							<Ethnicity {ethnicity} />
							<RelationshipStatus {relationshipStatus} />
						</div>
					{/if}
					{#if (lookingFor && lookingFor.length > 0) || (meetAt && meetAt.length > 0) || nsfw !== null}
						<div class="flex flex-col gap-2 mt-4">
							<span class="uppercase text-sm text-muted-foreground">
								Expectations
							</span>
							<LookingFor {lookingFor} />
							<MeetAt {meetAt} />
							<NSFWPics nsfwPics={nsfw} />
						</div>
					{/if}
					{#if hivStatus !== null || lastTestedDateValue !== null || (sexualHealthValue && sexualHealthValue.length > 0)}
						<div class="flex flex-col gap-2 mt-4">
							<span class="uppercase text-sm text-muted-foreground"
								>Health</span
							>
							<HivStatus {hivStatus} />
							<LastTested lastTestedDate={lastTestedDateValue} />
							<HealthPractices healthPractices={sexualHealthValue} />
						</div>
					{/if}
					{#if socialNetworks && Object.keys(socialNetworks).length > 0}
						<div class="flex flex-col gap-2 mt-4">
							<span class="uppercase text-sm text-muted-foreground"
								>Socials</span
							>
							<Socials socials={socialNetworks} />
						</div>
					{/if}
				</div>
			{/if}
		</main>
	</div>
</PullToRefresh>
