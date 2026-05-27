# Grid

[Cascade](#get-cascade) returns stuff like advertisements, upsells and partial profiles, presumably ranking by algorithms or paid subscriptions. [Search](#search) returns full profiles, seemingly ranked simply by distance.

## Get Cascade

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v4/cascade
```

Query:

- *everything from [CascadeQuery](/grindr-api/browse/grid#cascadequery)*

Response:

- `items` — array of [CascadeItem](/grindr-api/browse/grid#cascadeitem)
- `nextPage` — integer
- `shuffled` — boolean
- `hiddenProfiles` — unknown
- `hiddenProfileInfo` — unknown

## Get Cascade (legacy)

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v3/cascade
```

Query:

- *everything from [CascadeQuery](/grindr-api/browse/grid#cascadequery)*
- `exploreUuid` — string, unknown, WIP, optional
- `sexualHealth` — see [Sexual health](/grindr-api/users/profiles#sexual-health), optional

Response:

- `items` — array of [CascadeItem](/grindr-api/browse/grid#cascadeitem)
- `nextPage` — integer
- `shuffled` — boolean
- `hiddenProfiles` — unknown
- `hiddenProfileInfo` — unknown

## Search

Requires [Authorization](/grindr-api/api-authorization).

Results array appears to be capped to 600 per page. Use `searchAfterProfileId` or `searchAfterDistance` for pagination.

```
GET /v7/search
```

Query:

- *everything from [GridQuery](/grindr-api/browse/grid#gridquery)*
- `online` — boolean, optional
- `ageMinimum` — integer, optional
- `ageMaximum` — integer, optional
- `heightMinimum` — float, optional
- `heightMaximum` — float, optional
- `weightMinimum` — float, optional
- `weightMaximum` — float, optional
- `grindrTribesIds` — array of integers concatenated with a `,`, `-1` means "Not Specified", see [Tribes](/grindr-api/users/profiles#tribes), optional
- `lookingForIds` — array of integers concatenated with a `,`, `-1` means "Not Specified", see [Looking for](/grindr-api/users/profiles#looking-for), optional
- `relationshipStatusIds` — array of integers concatenated with a `,`, `-1` means "Not Specified", see [Relationship status](/grindr-api/users/profiles#relationship-status), optional
- `bodyTypeIds` — string, see [Body type](/grindr-api/users/profiles#body-type), optional
- `sexualPositionIds` — array of integers concatenated with a `,`, `-1` means "Not Specified", see [Sexual position](/grindr-api/users/profiles#sexual-position-id), optional
- `meetAtIds` — array of integers concatenated with a `,`, `-1` means "Not Specified", see [Meet at](/grindr-api/users/profiles#meet-at), optional
- `nsfwIds` — array of integers concatenated with a `,`, `-1` means "Not Specified", see [Accept NSFW pics](/grindr-api/users/profiles#accept-nsfw-pics), optional
- `profileTags` — string, see [Profile tags](/grindr-api/users/profiles#profile-tags), optional
- `searchAfterDistance` — string, optional
- `searchAfterProfileId` — string, optional
- `freeFilter` — boolean, optional

Response:

- `profiles` — array of [SearchProfileResult](/grindr-api/browse/grid#searchprofileresult)
- `lastDistanceInKm` — float
- `lastProfileId` — integer
- `inserts` — object
  - `mpuFree` — integer
  - `mpuXtra` — integer
  - `boostUpsell` — array of integers
  - `mrecCascadeFirst` — integer
  - `mrecCascadeSecond` — integer
  - `mrecCascadeThird` — integer

## GridQuery

- `nearbyGeoHash` — [Geohash](/grindr-api/browse/location#geohash)
- `exploreGeoHash` — [Geohash](/grindr-api/browse/location#geohash), optional
- `photoOnly` — boolean, optional
- `faceOnly` — boolean, optional
- `notRecentlyChatted` — boolean, optional
- `hasAlbum` — boolean, optional
- `fresh` — boolean, optional
- `genders` — string, see [Get genders](/grindr-api/users/profiles#get-genders), optional
- `pageNumber` — integer, optional

## CascadeQuery

- *everything from [GridQuery](/grindr-api/browse/grid#gridquery)*
- `onlineOnly` — boolean, optional
- `ageMin` — integer, optional
- `ageMax` — integer, optional
- `heightCmMin` — float, optional
- `heightCmMax` — float, optional
- `weightGramsMin` — float, optional
- `weightGramsMax` — float, optional
- `tribes` — string, see [Tribes](/grindr-api/users/profiles#tribes), optional
- `lookingFor` — string, see [Looking for](/grindr-api/users/profiles#looking-for), optional
- `relationshipStatuses` — string, see [Relationship status](/grindr-api/users/profiles#relationship-status), optional
- `bodyTypes` — string, see [Body type](/grindr-api/users/profiles#body-type), optional
- `sexualPositions` — array of integers concatenated with a `,`, see [Sexual position ID](/grindr-api/users/profiles#sexual-position-id), optional
- `meetAt` — string, see [Meet at](/grindr-api/users/profiles#meet-at), optional
- `nsfwPics` — string, see [Accept NSFW pics](/grindr-api/users/profiles#accept-nsfw-pics), optional
- `tags` — string, see [Profile tags](/grindr-api/users/profiles#profile-tags), optional
- `rightNow` — boolean, optional
- `favorites` — boolean, optional
- `showSponsoredProfiles` — boolean, optional
- `shuffle` — boolean, optional
- `hot` — boolean, optional

## CascadeItem

- `type` — string
- `data` — object, shape depends on `type` — [full_profile_v1](/grindr-api/browse/grid#full_profile_v1), [partial_profile_v1](/grindr-api/browse/grid#partial_profile_v1), [advert_v1](/grindr-api/browse/grid#advert_v1), [top_picks_v1](/grindr-api/browse/grid#top_picks_v1), [explore_aggregation_v1](/grindr-api/browse/grid#explore_aggregation_v1), [boost_upsell_v1](/grindr-api/browse/grid#boost_upsell_v1), [unlimited_mpu_v1](/grindr-api/browse/grid#unlimited_mpu_v1), [xtra_mpu_v1](/grindr-api/browse/grid#xtra_mpu_v1), [fav_header_v1](/grindr-api/browse/grid#fav_header_v1)

## CascadeResponse

- `items` — array of [CascadeItem](/grindr-api/browse/grid#cascadeitem)
- `nextPage` — integer
- `shuffled` — boolean
- `hiddenProfiles` — unknown
- `hiddenProfileInfo` — unknown

## full_profile_v1

- *everything from [CascadeResponseProfile](/grindr-api/browse/grid#cascaderesponseprofile)*

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- *everything from [ProfileFields](/grindr-api/users/profiles#profilefields)*
- `@type` — string, `"CascadeItemData$FullProfileV1"`
- `tribes` — array of integers, see [Tribes](/grindr-api/users/profiles#tribes)
- `socialNetworks` — array of [SocialNetwork](/grindr-api/users/profiles#socialnetwork)
- `takenOnGrindrMetadata` — object
  - *key is [Media hash](/grindr-api/media/index#media)*
  - `takenOnGrindr` — boolean
  - `createdAt` — unix timestamp in milliseconds

Only for [v4/cascade](/grindr-api/browse/grid#get-cascade):

- `age` — integer, optional
- `heightCm` — integer, optional
- `weightGrams` — integer, optional
- `bodyType` — integer, see [Body type](/grindr-api/users/profiles#body-type)

## partial_profile_v1

- *everything from [CascadeResponseProfile](/grindr-api/browse/grid#cascaderesponseprofile)*
- `upsellItemType` — string, e.g. `"xtra_mpu_v1"`

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$PartialProfileV1"`

## advert_v1

- `cascadePlacementName` — string, e.g. `"mrec-cascade-first"`

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$Advert"`

## top_picks_v1

Empty for [v4/cascade](/grindr-api/browse/grid#get-cascade).

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$TopPicksV1"`

## explore_aggregation_v1

- `uuid` — string, UUIDv4
- `headerName` — string, e.g. `"🌎 Explore"`
- `source` — string, e.g. `"cascade"`
- `items` — array of objects. `@type` determines contents: *`"ExploreAggregationItem$Location"`*: `data.onlineCount` (integer), `data.uuid` (UUIDv3 string), `data.location` (`id` integer, `name` string e.g. `"Minneapolis"`, `suffix` string e.g. `"🇺🇸"`, `lat` float, `lon` float), `data.profiles` (array of objects with `profileImageUrl` URL string). *`"ExploreAggregationItem$Cta"`*: empty.

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$ExploreAggregationV1"`

## boost_upsell_v1

Empty for [v4/cascade](/grindr-api/browse/grid#get-cascade).

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$BoostUpsellV1"`

## unlimited_mpu_v1

Empty for [v4/cascade](/grindr-api/browse/grid#get-cascade).

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$UnlimitedMpuV1"`

## xtra_mpu_v1

Empty for [v4/cascade](/grindr-api/browse/grid#get-cascade).

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$XtraMpuV1"`

## fav_header_v1

- `available` — integer
- `displayed` — integer
- `total` — integer

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `@type` — string, `"CascadeItemData$FavHeaderV1"`

## SearchProfileResult

- `age` — integer or `null`
- `displayName` — string
- `distance` — float
- `hasFaceRecognition` — boolean
- `isFavorite` — boolean
- `new` — boolean
- `lastChatTimestamp` — number, may be `0`
- `lastViewed` — unix timestamp in milliseconds or `null`
- `lastUpdatedTime` — unix timestamp in milliseconds
- `medias` — array of [ProfileMedia](/grindr-api/users/profiles#profilemedia) or `null`
- `profileId` — integer
- `profileImageMediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images) or `null`
- `profileTags` — array of [Profile tags](/grindr-api/users/profiles#profile-tags)
- `seen` — unix timestamp in milliseconds
- `showAge` — boolean
- `showDistance` — boolean
- `approximateDistance` — boolean
- `boosting` — boolean
- `hasAlbum` — boolean
- `gender` — array of integers or `[-1]`, see [Get genders](/grindr-api/users/profiles#get-genders)

## SearchProfilesResponse

- `profiles` — array of [SearchProfileResult](/grindr-api/browse/grid#searchprofileresult)
- `lastDistanceInKm` — float
- `lastProfileId` — integer
- `inserts` — object
  - `mpuFree` — integer
  - `mpuXtra` — integer
  - `boostUpsell` — array of integers
  - `mrecCascadeFirst` — integer
  - `mrecCascadeSecond` — integer
  - `mrecCascadeThird` — integer

## CascadeResponseProfile

Profile data returned in cascade items.

- `profileId` — integer
- `onlineUntil` — unix timestamp in milliseconds
- `displayName` — string
- `distanceMeters` — integer, optional
- `rightNow` — [RightNowStatus](/grindr-api/right-now#rightnowstatus)
- `unreadCount` — integer
- `isVisiting` — boolean
- `isPopular` — boolean

Only for [v3/cascade](/grindr-api/browse/grid#get-cascade-legacy):

- `lastOnline` — unix timestamp in milliseconds
- `photoMediaHashes` — array of strings, see [Media](/grindr-api/media/index#media)
- `lookingFor` — array of integers, see [Looking for](/grindr-api/users/profiles#looking-for)
- `sexualPosition` — integer, see [Sexual position ID](/grindr-api/users/profiles#sexual-position-id), optional
- `approximateDistance` — boolean
- `isFavorite` — boolean
- `isBoosting` — boolean
- `hasChattedInLast24Hrs` — boolean
- `hasUnviewedSpark` — boolean
- `isTeleporting` — boolean
- `isRoaming` — boolean
- `isRightNow` — boolean
- `hasUnreadThrob` — boolean
- `isBlockable` — boolean, optional
- `isBoostingSomewhereElse` — boolean

Only for [v4/cascade](/grindr-api/browse/grid#get-cascade):

- `primaryImageUrl` — string, URL
- `favorite` — boolean
- `viewed` — boolean
- `chatted` — boolean
- `roaming` — boolean
