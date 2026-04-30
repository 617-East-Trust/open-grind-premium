<script lang="ts">
	import { page } from "$app/state";
	import { getProfile } from "$lib/api/profile";
	import { Skeleton } from "$lib/components/ui/skeleton";
	import ImageCarousel from "./ImageCarousel.svelte";
	import Distance from "./Distance.svelte";
	import OnlineStatus from "./OnlineStatus.svelte";
	import SexualPosition from "./SexualPosition.svelte";
	import Height from "./HeightWeightBodyType.svelte";
	import ProfileTags from "./ProfileTags.svelte";
	import AboutMe from "./AboutMe.svelte";
	import Genders from "./GendersPronouns.svelte";
	import Ethnicity from "./Ethnicity.svelte";
	import RelationshipStatus from "./RelationshipStatus.svelte";
	import Tribes from "./Tribes.svelte";
	import LookingFor from "./LookingFor.svelte";
	import MeetAt from "./MeetAt.svelte";
	import NSFWPics from "./NSFWPics.svelte";
	import Socials from "./Socials.svelte";
	import HivStatus from "./HivStatus.svelte";
	import LastTested from "./LastTested.svelte";
	import HealthPractices from "./HealthPractices.svelte";

	const profile = $derived(getProfile(Number(page.params.profileId)));
</script>

<main class="pb-16">
	{#await profile}
		<Skeleton />
	{:then profile}
		{@const {
			displayName,
			age,
			onlineUntil,
			seen,
			distance,
			showDistance,
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
		<ImageCarousel {medias} />
		<div class="flex flex-col p-4 pb-12">
			<h1 class="text-2xl wrap-break-word">
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
			<div class="flex items-center gap-3 text-sm mt-1">
				<OnlineStatus onlineUntil={onlineUntil ?? null} {seen} />
				<Distance {distance} />
			</div>
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
					<span class="uppercase text-sm text-muted-foreground">Stats</span>
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
					<span class="uppercase text-sm text-muted-foreground">Health</span>
					<HivStatus {hivStatus} />
					<LastTested lastTestedDate={lastTestedDateValue} />
					<HealthPractices healthPractices={sexualHealthValue} />
				</div>
			{/if}
			{#if socialNetworks && Object.keys(socialNetworks).length > 0}
				<div class="flex flex-col gap-2 mt-4">
					<span class="uppercase text-sm text-muted-foreground">Socials</span>
					<Socials socials={socialNetworks} />
				</div>
			{/if}
		</div>
	{/await}
</main>
