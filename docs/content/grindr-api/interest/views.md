# Views

## Get views number

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v6/views/eyeball
```

Response:

- `viewedCount` — number or `null`
- `mostRecent` — object or `null`
  - `profileId` — string with number
  - `photoHash` — string, See [Media](/grindr-api/media/)
  - `timestamp` — unix timestamp in milliseconds

## Get viewers list

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v7/views/list
```

Response:

- `totalViewers` — integer
- `previews` — array of [ViewPreview](/grindr-api/interest/views#viewpreview)
- `profiles` — array of [ViewerProfile](/grindr-api/interest/views#viewerprofile)

## Record profile views (batch), WIP

> [!NOTE] This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v4/views
```

Body:

- `viewedProfileIds` — array of strings with numeric profile IDs
- `foundVia` — [ViewSourceEnum](/grindr-api/interest/views#viewsourceenum) or `null`

## Record single profile view, WIP

> [!NOTE] This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v4/views/{profileId}
```

## Record profile view v2, WIP

> [!NOTE] This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v5/views/{profileId}
```

Body:

- `foundVia` — [ViewSourceEnum](/grindr-api/interest/views#viewsourceenum) or `null`
- `source` — [ViewSourceEnum](/grindr-api/interest/views#viewsourceenum)

## ViewSourceEnum

View source.

- `DISCOVER`
- `FOR_YOU`
- `UNKNOWN` — (fallback)

## ViewsEyeballResponse

- `viewedCount` — number or `null`
- `mostRecent` — object or `null`
  - `profileId` — string with number
  - `photoHash` — string, See [Media](/grindr-api/media/)
  - `timestamp` — unix timestamp in milliseconds

## ViewPreview

- *everything from [ProfileMasked](/grindr-api/users/profiles#profilemasked)*
- `isInBadNeighborhood` — boolean
- `isViewedMeFreshFace` — boolean
- `isSecretAdmirer` — boolean
- `viewedCount` — object
  - `totalCount` — integer
  - `maxDisplayCount` — integer

## ViewerProfile

- *everything from [ViewPreview](/grindr-api/interest/views#viewpreview)*
- *everything from [ProfileShort](/grindr-api/users/profiles#profileshort)*
- `hasFaceRecognition` — boolean
- `isIncognito` — boolean
- `boosting` — boolean
- `showUnlockReward` — boolean
- `unreadMessageCount` — integer
- `hasChatted` — boolean

## ViewsListResponse

- `totalViewers` — integer
- `previews` — array of [ViewPreview](/grindr-api/interest/views#viewpreview)
- `profiles` — array of [ViewerProfile](/grindr-api/interest/views#viewerprofile)
