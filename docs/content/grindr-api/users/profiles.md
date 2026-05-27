# Profiles

## Profile tags

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v1/tags
```

Response:

Array of [ProfileTagLanguage](/grindr-api/users/profiles#profiletaglanguage).

## Get a profile by ID

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v7/profiles/{id}
```

Response:

- `profiles` — array of [Profile](/grindr-api/users/profiles#profile)

## Get multiple profiles by ID

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v3/profiles
```

Body:

- `targetProfileIds` — array of strings with numeric ids, max profiles number per one request is 150

Response:

- `profiles` — array of [ProfileSummaryWithRightNow](/grindr-api/users/profiles#profilesummarywithrightnow)

## Update own profile (full), WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

[Profile](/grindr-api/users/profiles#profile) object, fully replaces current version.

```
PUT /v3.1/me/profile
```

Body:

[Profile](/grindr-api/users/profiles#profile)

## Update own profile (partial), WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

[Profile](/grindr-api/users/profiles#profile) object, only updates specified keys.

```
PATCH /v4/me/profile
```

Body:

[ProfilePatch](/grindr-api/users/profiles#profilepatch)

## Profile tags suggestions, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
PUT /v4/profile-tags/suggestions
```

## Profile tags translations, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v5/profile-tags/translations
```

Headers:

- `L-Locale` — string

## Upload media

Correct request's `Content-Type` header is required.

```
POST /v5/chat/media/upload
```

Query:

- `length` — long integer
- `looping` — boolean
- `takenOnGrindr` — boolean

Body:

Binary file.

Response:

- `mediaId` — long integer
- `mediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images)
- `url` — URL

## Upload media (legacy)

Requires [Authorization](/grindr-api/api-authorization).

You must ensure thumbCoords's width and height dimensions are equal, i.e. y2-y1 must equal to x2-x1. Submitting non-square thumbnail won't trigger any errors and it will be uploaded to CDN, however attempting to use such illegal thumbnail dimensions image in [Edit profile photos](/grindr-api/users/profiles#edit-profile-photos) will result in it being silently dropped/skipped.

Also there is a legacy `POST /v3/me/profile/images`.

```
POST /v4/media/upload
```

Query:

- `thumbCoords` — [RectF](/grindr-api/users/profiles#rectf)
- `takenOnGrindr` — boolean

Body:

Binary file.

Response:

- `hash` — string
- `imageSizes` — array of objects
  - `size` — integer or `null`
  - `fullUrl` — string
  - `thumbnail` — boolean or `null`
  - `state` — string
  - `mediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images)
  - `rejectionReason` — string or `null`
- `mediaId` — integer

## Edit profile photos

Requires [Authorization](/grindr-api/api-authorization).

Setting both `primaryImageHash` and `secondaryImageHashes` to `null` works. But setting `primaryImageHash` to a hash value while setting `secondaryImageHashes` to null causes HTTP status 400 Bad Request error. It's recommended to just use `[]` for `secondaryImageHashes` rather than `null`.

Supplied images must have square thumbnails, otherwise they will be silently skipped. See [Upload media](/grindr-api/users/profiles#upload-media).

Repeating `primaryImageHash` value in `secondaryImageHashes` array will result in secondaryImageHashes's entry being silently dropped from supplied request array. Repeating `secondaryImageHashes` values will result in successfully saving the array as-is to the server, however official mobile client seems to drop repeating media when processing `secondaryImageHashes` response.

```
PUT /v3/me/profile/images
```

Body:

- `primaryImageHash` — string or `null`
- `secondaryImageHashes` — array of strings, max. length: 5 or `null`

Response:

Empty.

## Delete profile photos

Requires [Authorization](/grindr-api/api-authorization).

This endpoint removes photo from your profile as well as deletes the media from CDN.

```
DELETE /v3/me/profile/images
```

Body (yes, body, not query):

- `media_hashes` — array of strings

Response:

Empty.

## Get my profile photos

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v3.1/me/profile/images
```

Response:

- `medias` — array of [ProfileImageState](/grindr-api/users/profiles#profileimagestate)

## Check if profiles are reachable, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v4/profiles/reachable
```

Body:

- `profileIds` — array of strings with numeric ids

Response:

- `profileIds` — array of strings with numeric ids

## Get profile insights v1, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v1/profile-insights/{profileId}
```

## Get profile insights v2, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v2/profile-insights/{profileId}
```

## Get pronouns

Requires [Authorization](/grindr-api/api-authorization).

Gender and pronouns are **managed fields** — profile fields that aren't hardcoded but pulled dynamically from the server.

```
GET /v1/pronouns
```

Response:

Array of [Pronoun](/grindr-api/users/profiles#pronoun).

## Get genders

```
GET /public/v2/genders
```

Response:

Array of [Gender](/grindr-api/users/profiles#gender).

## Get genders (v1 alias)

```
GET /public/v1/genders
```

Response:

Array of [Gender](/grindr-api/users/profiles#gender).

## Suggest gender or pronoun

Requires [Authorization](/grindr-api/api-authorization).

```
PUT /v4/genderpronoun/suggestions
```

Body:

- `category` — string, either `gender` or `pronoun`
- `suggestedValue` — string

Response:

Empty

## RectF

Array of 4 floats or `nulls`: Bottom edge ("y2") in pixels, Left edge ("x1") in pixels, Right edge ("x2") in pixels, Top edge ("y1") in pixels. When used in query, stringified as follows: `y2,x1,x2,y1`.

## ProfileMaskedMin

Minimal masked profile.

- `distance` — number or `null`
- `profileImageMediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images) or `null`
- `isFavorite` — boolean

## ProfileMasked

Masked profile (extra fields revealed).

- *everything from [ProfileMaskedMin](/grindr-api/users/profiles#profilemaskedmin)*
- `lastViewed` — number or `null`
- `seen` — unix timestamp in milliseconds or `null`
- `rightNow` — [RightNowStatus](/grindr-api/right-now#rightnowstatus)
- `sexualPosition` — integer or `null`, optional
- `foundVia` — [ViewSourceEnum](/grindr-api/interest/views#viewsourceenum) or `null`, optional

## ProfileMin

Minimal profile fields.

- `profileId` — string with numeric id
- `displayName` — string or `null`
- `onlineUntil` — unix timestamp in milliseconds or `null`, optional

## ProfileMedia

- `mediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images)
- `type` — integer
- `state` — integer
- `reason` — string or `null`
- `takenOnGrindr` — boolean or `null`
- `createdAt` — unix timestamp in milliseconds or `null`

## ProfileShort

Short profile, combines ProfileMasked and ProfileMin.

- *everything from [ProfileMasked](/grindr-api/users/profiles#profilemasked)*
- *everything from [ProfileMin](/grindr-api/users/profiles#profilemin)*
- `age` — integer, may be `0` or `null`
- `showAge` — boolean
- `showDistance` — boolean
- `approximateDistance` — boolean
- `lastChatTimestamp` — number, may be `0` or `null`
- `isNew` — boolean
- `lastUpdatedTime` — unix timestamp in milliseconds, may be `0`
- `medias` — array of [ProfileMedia](/grindr-api/users/profiles#profilemedia)

## ProfileFields

Optional profile fields.

- `meetAt` — array of [Meet at](/grindr-api/users/profiles#meet-at), optional
- `vaccines` — array of [Vaccines](/grindr-api/users/profiles#vaccines), optional
- `genders` — array of integers, optional
- `pronouns` — array of integers, optional

## ProfileRightNow

Right Now status info on a profile.

- `rightNowText` — string or `null`
- `rightNowPosted` — unix timestamp in milliseconds or `null`
- `rightNowDistance` — long integer or `null`
- `rightNowThumbnailUrl` — string or `null`
- `rightNowFullImageUrl` — string or `null`

## ProfileExtraFields

Extra profile fields.

- `nsfw` — [Accept NSFW pics](/grindr-api/users/profiles#accept-nsfw-pics) or `null`
- `verifiedInstagramId` — string or `null`
- `isBlockable` — boolean or `null`
- `showTribes` — boolean
- `showPosition` — boolean

## TravelPlanEmbedded

- `endDateUtc` — unix timestamp in milliseconds or `null`
- `geohash` — [Geohash](/grindr-api/browse/location#geohash)
- `id` — long integer or `null`
- `locationName` — string
- `showOnProfile` — boolean or `null`
- `startDateUtc` — unix timestamp in milliseconds or `null`

## RightNowMedia

- `mediaId` — long integer or `null`
- `thumbnailUrl` — string
- `fullImageUrl` — string
- `contentType` — string
- `isNsfw` — boolean or `null`

## Profile

Full profile object.

- *everything from [ProfileShort](/grindr-api/users/profiles#profileshort)*
- *everything from [ProfileFields](/grindr-api/users/profiles#profilefields)*
- *everything from [ProfileRightNow](/grindr-api/users/profiles#profilerightnow)*
- *everything from [ProfileExtraFields](/grindr-api/users/profiles#profileextrafields)*
- `aboutMe` — string or `null`
- `ethnicity` — [Ethnicity](/grindr-api/users/profiles#ethnicity) or `null`
- `relationshipStatus` — [Relationship status](/grindr-api/users/profiles#relationship-status) or `null`
- `grindrTribes` — array of [Tribes](/grindr-api/users/profiles#tribes)
- `lookingFor` — array of [Looking for](/grindr-api/users/profiles#looking-for)
- `bodyType` — [Body type](/grindr-api/users/profiles#body-type) or `null`
- `hivStatus` — [HIV status](/grindr-api/users/profiles#hiv-status) or `null`
- `lastTestedDate` — unix timestamp in milliseconds or `null`
- `height` — number in centimeters or `null`
- `weight` — number in grams or `null`
- `socialNetworks` — object
  - `twitter` — [SocialNetwork](/grindr-api/users/profiles#socialnetwork), may be absent
  - `facebook` — [SocialNetwork](/grindr-api/users/profiles#socialnetwork), may be absent
  - `instagram` — [SocialNetwork](/grindr-api/users/profiles#socialnetwork), may be absent
- `identity` — unknown or `null`
- `hashtags` — array
- `profileTags` — array of [Profile tags](/grindr-api/users/profiles#profile-tags)
- `tapped` — boolean
- `tapType` — boolean or `null`
- `lastReceivedTapTimestamp` — unix timestamp in milliseconds or `null`
- `isTeleporting` — boolean
- `isRoaming` — boolean
- `arrivalDays` — number or `null`
- `unreadCount` — number, optional
- `lastThrobTimestamp` — unknown
- `sexualHealth` — array of [Sexual health](/grindr-api/users/profiles#sexual-health)
- `isVisiting` — boolean
- `travelPlans` — array of [TravelPlanEmbedded](/grindr-api/users/profiles#travelplanembedded)
- `isInAList` — boolean
- `tribesImInto` — array of [Tribes](/grindr-api/users/profiles#tribes)
- `showVipBadge` — boolean
- `rightNowShareLocation` — string, `"NONE"` or `null`
- `rightNowMedias` — array of [RightNowMedia](/grindr-api/users/profiles#rightnowmedia)

## ProfilePatch

- *everything from [Profile](/grindr-api/users/profiles#profile)*

## ProfileSummaryWithRightNow

- *everything from [ProfileShort](/grindr-api/users/profiles#profileshort)*
- *everything from [ProfileRightNow](/grindr-api/users/profiles#profilerightnow)*

## SocialNetwork

Social network user reference.

- `userId` — string, username or `null`
- `site` — unknown, e.g. `"twitter"` | `"facebook"` | `"instagram"`

## ProfileTagLanguage

- `language` — string
- `categoryCollection` — array of objects
  - `text` — string
  - `possessiveText` — string or `null`
  - `tags` — array of objects
    - `tagId` — integer
    - `text` — string
    - `key` — string

## ChatMediaUploadResponse

- `mediaId` — long integer
- `mediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images)
- `url` — URL

## LegacyMediaUploadResponse

- `hash` — string
- `imageSizes` — array of objects
  - `size` — integer or `null`
  - `fullUrl` — string
  - `thumbnail` — boolean or `null`
  - `state` — string
  - `mediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images)
  - `rejectionReason` — string or `null`
- `mediaId` — integer

## EditProfilePhotosRequest

- `primaryImageHash` — string or `null`
- `secondaryImageHashes` — array of strings, max. length: 5 or `null`

## DeleteProfilePhotosRequest

- `media_hashes` — array of strings

## ProfileImageState

> [!NOTE]
> This type hasn't been researched yet

Profile media state.

- `mediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images)
- `type` — integer
- `state` — integer

## Pronoun

- `pronounId` — integer
- `pronoun` — string, e.g. `"-"` or `"They/Them/Theirs"`

## Gender

- `genderId` — integer
- `gender` — string
- `displayGroup` — integer
- `sortProfile` — integer or `null`
- `sortFilter` — integer or `null`
- `genderPlural` — string or `null`
- `excludeOnProfileSelection` — array of integers or `null`
- `excludeOnFilterSelection` — array of integers or `null`
- `alsoClassifiedAs` — array of integers

## GenderPronounSuggestionRequest

- `category` — string, either `gender` or `pronoun`
- `suggestedValue` — string

## Sexual position ID

Sexual position ID.

- `1` — Top
- `2` — Bottom
- `3` — Versatile
- `4` — Vers Bottom
- `5` — Vers Top
- `6` — Side

## Ethnicity

Ethnicity.

- `1` — Asian
- `2` — Black
- `3` — Latino
- `4` — Middle Eastern
- `5` — Mixed
- `6` — Native American
- `7` — White
- `8` — Other
- `9` — South Asian

## Relationship status

Relationship status.

- `1` — Single
- `2` — Dating
- `3` — Exclusive
- `4` — Committed
- `5` — Partnered
- `6` — Engaged
- `7` — Married
- `8` — Open Relationship

## Body type

Body type.

- `1` — Toned
- `2` — Average
- `3` — Large
- `4` — Muscular
- `5` — Slim
- `6` — Stocky

## HIV status

HIV status. Not to be confused with [Sexual health](/grindr-api/users/profiles#sexual-health).

- `1` — Negative
- `2` — Negative, on PrEP
- `3` — Positive
- `4` — Positive, undetectable

## Accept NSFW pics

Accept NSFW pics.

- `1` — Never
- `2` — Not At First
- `3` — Yes Please

## Meet at

Meet at preferences.

- `1` — My Place
- `2` — Your Place
- `3` — Bar
- `4` — Coffee Shop
- `5` — Restaurant

## Sexual health

Sexual health (also "Health Practices"). Not to be confused with [HIV status](/grindr-api/users/profiles#hiv-status).

- `1` — Condoms
- `2` — I'm on doxyPEP
- `3` — I'm on PrEP
- `4` — I'm HIV undetectable
- `5` — Prefer to discuss

## Looking for

Looking for.

- `2` — Chat
- `3` — Dates
- `4` — Friends
- `5` — Networking
- `6` — Relationship
- `7` — Hookups

## Tribes

Tribe.

- `1` — Bear
- `2` — Clean-Cut
- `3` — Daddy
- `4` — Discreet
- `5` — Geek
- `6` — Jock
- `7` — Leather
- `8` — Otter
- `9` — Poz
- `10` — Rugged
- `11` — Trans
- `12` — Twink
- `13` — Sober

## Vaccines

Vaccine.

- `1` — COVID-19
- `2` — Monkeypox
- `3` — Meningitis
