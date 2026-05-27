# Albums

WIP. No idea what SpankBank, pressie albums and paywalled albums are.

## Get my albums

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v1/albums
```

Response:

- `albums` — array of [MyAlbum](/grindr-api/messaging/albums#myalbum)

## Get an album

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v2/albums/{albumId}
```

Response:

[AlbumDetailsResponse](/grindr-api/messaging/albums#albumdetailsresponse)

Errors:

- `403` — if you don't have access to album or it doesn't exist

## Rename an album

Requires [Authorization](/grindr-api/api-authorization).

```
PUT /v2/albums/{albumId}
```

Body:

- `albumName` — string or `null`

Response:

- `albumId` — integer
- `albumName` — string or `null`

## Get an album media poster

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v1/albums/{albumId}/content/{contentId}/poster
```

Response:

- `blurredPosterUrl` — URL
- `posterUrl` — URL

## Record view of an album

```
GET /v3/albums/{albumId}/view
```

Response:

Empty.

Errors:

- `403` — Repeated requests after invoking this endpoint on view ONCE albums cause HTTP status 403 Forbidden and `Action not permitted` error.

## Record view of media in an album

Requires [Authorization](/grindr-api/api-authorization).

Repeated requests after reaching remainingViews=0 do not cause any errors.

```
POST /v1/albums/{albumId}/view/content/{contentId}
```

Response:

- `remainingViews` — integer

## Get info about profile's album

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v2/albums/shares
```

Body:

- `profileId` — long integer

Response:

- `profileId` — long integer
- `hasAlbum` — boolean
- `hasSharedWithMe` — boolean

## Get albums shared by a profile

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v2/albums/shares/{profileId}
```

Response:

- `albums` — array of objects
  - *everything from [AlbumMin](/grindr-api/messaging/albums#albummin)*
  - *everything from [AlbumExpiration](/grindr-api/messaging/albums#albumexpiration)*
  - `content` — [AlbumContentMin](/grindr-api/messaging/albums#albumcontentmin), a single blurred preview content item
  - `contentCount` — object
    - `imageCount` — integer
    - `videoCount` — integer

## Create an album

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v2/albums
```

Body:

- `albumName` — string or `null`

Response:

- `albumId` — long integer

Errors:

- `402` — Payment required if you reached [limit](/grindr-api/messaging/albums#get-album-limits) for number of created albums

## Delete an album

Requires [Authorization](/grindr-api/api-authorization).

```
DELETE /v1/albums/{albumId}
```

Response:

Empty

Errors:

- `403` — Repeated requests cause 403 Forbidden and `Action not permitted` error.

## Upload media to an album

Requires [Authorization](/grindr-api/api-authorization).

Repeated requests with the same file (its contents) are skipped and a cached result from the first upload request is returned.

```
POST /v1/albums/{albumId}/content
```

Query (optional):

- `width` — number, optional, doesn't affect the resulting image, optional
- `height` — number, optional, doesn't affect the resulting image, optional
- `isFresh` — boolean, optional, unknown how it affects the resulting image, WIP, optional

Body:

Content-Type: `multipart/form-data`

- `content` — binary

Response:

- `contentId` — long integer
- `contentUrl` — string or `null`

## Reorder media in an album

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v1/albums/{albumId}/content/order
```

Body:

- `contentIds` — array of long integers, each Media file ID must appear exactly once

## Delete media from an album

Requires [Authorization](/grindr-api/api-authorization).

Technically, this does not delete the media from CDN. All signed URLs will continue to work until expired. Uploading same file will result in getting it assigned the same `contentId`.

```
DELETE /v1/albums/{albumId}/content/{contentId}
```

Response:

Empty

## Albums content processing, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v1/albums/{albumId}/content/{contentId}/processing
```

Response:

- `processing` — boolean

## Pics, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v1/pics/limited/status
```

Response:

- `available` — integer
- `total` — integer

## Pics expiring, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v4/pics/expiring
```

## Pics expiring status, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v4/pics/expiring/status
```

## Videos expiring status, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v4/videos/expiring/status
```

## Get album shares

Requires [Authorization](/grindr-api/api-authorization).

Returns profiles the album was shared with.

```
GET /v1/albums/{albumId}/shares
```

Response:

- `profileIds` — array of integers

## Share an album

Requires [Authorization](/grindr-api/api-authorization).

Automatically sends the shared album to chat with all listed profiles.

```
POST /v4/albums/{albumId}/shares
```

Body:

- `profiles` — array of objects
  - `expirationType` — [AlbumExpirationType](/grindr-api/messaging/albums#albumexpirationtype)
  - `profileId` — integer

Response:

Empty

## Unshare an album

Requires [Authorization](/grindr-api/api-authorization).

```
PUT /v1/albums/{albumId}/unshares
```

Body:

- `profiles` — array of objects
  - `profileId` — long integer
  - `shareId` — integer, can be `0`

Response:

Empty

## Unshare an album from everybody, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Unknown, returns 403

```
PUT /v1/albums/{albumId}/shares/remove
```

## Albums content chat list-by-id, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

Unknown, `{"ids":[852120758]}` returns 400

```
POST /v1/albums/{albumId}/content/chat/list-by-id
```

Query:

- `isFresh` — boolean

Body:

- `ids` — array of long integers

## Get album limits

Requires [Authorization](/grindr-api/api-authorization).

Interestingly, /v2/albums/storage appears to exist, though not used in current version of APK.

```
GET /v1/albums/storage
```

Response:

- `subscriptionType` — string, e.g. `FreeAlbums`
- `maxAlbums` — integer
- `maxContentItemsPerAlbum` — integer
- `maxShares` — integer
- `maxViewableAlbums` — integer
- `maxViewableVideos` — integer
- `maxContentSize` — long integer, size in bytes
- `maxContentSizeHumanReadable` — string, incorrectly uses decimal multiples notation (MB) when in fact calculates binary notation (MiB), so API's `120.00 MB` is actually 120 MiB or 125.8291 MB
- `maxVideoLength` — long integer, length in milliseconds (1/1000th of a second)
- `minVideoLength` — long integer, length in milliseconds (1/1000th of a second)
- `maxShareableAlbums` — integer
- `maxVideosPerAlbum` — integer

## Albums red dot, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

This may just be tracking but could also be related to something else

```
PUT /v1/albums/red-dot
```

Response:

Empty.

## Gets albums shared with us

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v3/pressie-albums/feed
```

Body (optional):

- `isFavorite` — boolean, optional, only albums shared by [favorite](/grindr-api/users/favorites) users
- `isOnline` — boolean, optional, only albums shared by currently online users
- `onlyVideo` — boolean, optional, only albums with at least one video
- `blur` — boolean, optional, blur media urls in response

Response:

- `profileFeeds` — array of objects
  - `profileId` — integer
  - `paywallStatus` — string, e.g. `ALLOW`
  - `seen` — boolean
  - `content` — object
  - `profile` — [PressieProfileMini](/grindr-api/messaging/albums#pressieprofilemini)
- `sharedAlbums` — array of objects
  - *everything from [AlbumPreview](/grindr-api/messaging/albums#albumpreview)*
  - `albumViewable` — boolean
  - `albumVersion` — integer
  - `expiresat` — unix timestamp in milliseconds; note: observed key spelling, may also appear as `expiresAt` or `null`
  - `name` — string or `null`
  - `ownerProfileId` — integer
  - `imageCount` — integer
  - `videoCount` — integer
  - `coverContent` — object
    - `id` — long integer
    - `contentType` — string
    - `coverContent` — string or `null`
    - `status` — string
  - `profile` — [PressieProfileMini](/grindr-api/messaging/albums#pressieprofilemini)
- `experimentStatus` — number
- `nonEmptyPersonalAlbumCount` — number
- `emptyAlbumId` — unknown or `null`

## Pressie albums feed paywall

```
POST /v3/pressie-albums/feed/paywall/
```

Response:

- `albumPaywallContent` — array of objects
  - `albumId` — long integer
  - `profile` — [PressieProfileMini](/grindr-api/messaging/albums#pressieprofilemini)
  - `paywallCoverUrl` — see [Media -> Signed CDN files](/grindr-api/media/signed-cdn-files)
  - `paywallUrls` — see [Media -> Signed CDN files](/grindr-api/media/signed-cdn-files)
  - `albumsItemCount` — integer

## Pressie albums feed profile ID, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
GET /v3/pressie-albums/feed/{profileId}
```

## Pressie albums feed update read, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v3/pressie-albums/feed/update/read
```

## AlbumExpirationType

Album expiration type.

Previously shared [albums in chat](/grindr-api/messaging/messages#albummessagebody) inherit new `expirationType` settings from newer sharings of the album.

- `"INDEFINITE"` or `0` — "Indefinitely"
- `"ONCE"` or `1` — "View Once", limited by 30 minutes from request
- `"TEN_MINUTES"` or `2` — "For 10 Minutes"
- `"ONE_HOUR"` or `3` — "For 60 Minutes"
- `"ONE_DAY"` or `4` — "For 24 Hours"

## AlbumPreview

- `albumId` — long integer
- `albumNumber` — integer or `null` if album has expired or was locked
- `totalAlbumsShared` — integer or `null` if album has expired or was locked
- `hasUnseenContent` — boolean

## AlbumMin

- *everything from [AlbumPreview](/grindr-api/messaging/albums#albumpreview)*
- `albumName` — string, appears to always be `null`
- `profileId` — integer
- `albumViewable` — boolean

## AlbumDetails

- `sharedCount` — integer
- `createdAt` — string, date formatted as ISO 8601, e.g. `2026-03-27T20:39:00`
- `updatedAt` — string, date formatted as ISO 8601, e.g. `2026-03-27T20:39:00`

## AlbumExpiration

- `expiresAt` — unix timestamp in milliseconds or `null`
- `expirationType` — [AlbumExpirationType](/grindr-api/messaging/albums#albumexpirationtype)

## AlbumContentMin

- `contentId` — long integer
- `contentType` — string
- `coverUrl` — [AlbumCoverUrl](/grindr-api/messaging/albums#albumcoverurl)
- `statusId` — unknown integer, WIP

## AlbumContent

- *everything from [AlbumContentMin](/grindr-api/messaging/albums#albumcontentmin)*
- `thumbUrl` — string, unblurred preview, see [Media -> Signed CDN files](/grindr-api/media/signed-cdn-files)
- `url` — string, original file, see [Media -> Signed CDN files](/grindr-api/media/signed-cdn-files), may be `""` if `remainingViews` is 0
- `processing` — boolean
- `rejectionId` — unknown or `null`

## MyAlbum

- *everything from [AlbumDetails](/grindr-api/messaging/albums#albumdetails)*
- `albumId` — long integer
- `albumName` — see [Album name](/grindr-api/messaging/albums#album-name) or `null`
- `profileId` — integer
- `version` — integer
- `content` — [AlbumContent](/grindr-api/messaging/albums#albumcontent)
- `isShareable` — boolean

## AlbumContentWithRemainingViews

- *everything from [AlbumContent](/grindr-api/messaging/albums#albumcontent)*
- `remainingViews` — integer, might be -1; absent if this is your own album, optional

## AlbumDetailsResponse

- *everything from [AlbumMin](/grindr-api/messaging/albums#albummin)*
- *everything from [AlbumDetails](/grindr-api/messaging/albums#albumdetails)*
- `content` — array of [AlbumContentWithRemainingViews](/grindr-api/messaging/albums#albumcontentwithremainingviews)

## AlbumShareInfo

- `profileId` — long integer
- `hasAlbum` — boolean
- `hasSharedWithMe` — boolean

## AlbumsSharedByProfileResponse

- `albums` — array of objects
  - *everything from [AlbumMin](/grindr-api/messaging/albums#albummin)*
  - *everything from [AlbumExpiration](/grindr-api/messaging/albums#albumexpiration)*
  - `content` — [AlbumContentMin](/grindr-api/messaging/albums#albumcontentmin), a single blurred preview content item
  - `contentCount` — object
    - `imageCount` — integer
    - `videoCount` — integer

## Album name

String, may be empty (`""`) or `null`, non-string values are coerced into string.

Maximum length: 255 UTF-8 bytes, which is 255 characters for ASCII strings (1 ASCII character is encoded as 1 byte) but less if you include emojis or non-ascii characters (2+ bytes/one codepoint).

- `albumName` — string or `null`

## AlbumRenameResponse

- `albumId` — integer
- `albumName` — string or `null`

## AlbumShareRequest

- `profiles` — array of objects
  - `expirationType` — [AlbumExpirationType](/grindr-api/messaging/albums#albumexpirationtype)
  - `profileId` — integer

## AlbumUnshareRequest

- `profiles` — array of objects
  - `profileId` — long integer
  - `shareId` — integer, can be `0`

## AlbumStorageLimits

- `subscriptionType` — string, e.g. `FreeAlbums`
- `maxAlbums` — integer
- `maxContentItemsPerAlbum` — integer
- `maxShares` — integer
- `maxViewableAlbums` — integer
- `maxViewableVideos` — integer
- `maxContentSize` — long integer, size in bytes
- `maxContentSizeHumanReadable` — string, incorrectly uses decimal multiples notation (MB) when in fact calculates binary notation (MiB), so API's `120.00 MB` is actually 120 MiB or 125.8291 MB
- `maxVideoLength` — long integer, length in milliseconds (1/1000th of a second)
- `minVideoLength` — long integer, length in milliseconds (1/1000th of a second)
- `maxShareableAlbums` — integer
- `maxVideosPerAlbum` — integer

## PressieProfileMini

- `profileId` — long integer, optional
- `name` — string, optional
- `profileUrl` — string or `null`, optional
- `onlineUntil` — unknown or `null`, optional
- `distanceKm` — number or `null`, optional

## PressieAlbumsFeedResponse

- `profileFeeds` — array of objects
  - `profileId` — integer
  - `paywallStatus` — string, e.g. `ALLOW`
  - `seen` — boolean
  - `content` — object
  - `profile` — [PressieProfileMini](/grindr-api/messaging/albums#pressieprofilemini)
- `sharedAlbums` — array of objects
  - *everything from [AlbumPreview](/grindr-api/messaging/albums#albumpreview)*
  - `albumViewable` — boolean
  - `albumVersion` — integer
  - `expiresat` — unix timestamp in milliseconds; note: observed key spelling, may also appear as `expiresAt` or `null`
  - `name` — string or `null`
  - `ownerProfileId` — integer
  - `imageCount` — integer
  - `videoCount` — integer
  - `coverContent` — object
    - `id` — long integer
    - `contentType` — string
    - `coverContent` — string or `null`
    - `status` — string
  - `profile` — [PressieProfileMini](/grindr-api/messaging/albums#pressieprofilemini)
- `experimentStatus` — number
- `nonEmptyPersonalAlbumCount` — number
- `emptyAlbumId` — unknown or `null`

## PressieAlbumsPaywallResponse

- `albumPaywallContent` — array of objects
  - `albumId` — long integer
  - `profile` — [PressieProfileMini](/grindr-api/messaging/albums#pressieprofilemini)
  - `paywallCoverUrl` — see [Media -> Signed CDN files](/grindr-api/media/signed-cdn-files)
  - `paywallUrls` — see [Media -> Signed CDN files](/grindr-api/media/signed-cdn-files)
  - `albumsItemCount` — integer

## AlbumCoverUrl

String with URL or `null`, blurred downscaled preview.

JPEG photo with the first frame of video in case of video files.

Becomes unavailable (`AccessDenied`) after album has expired.

See [Media -> Signed CDN files](/grindr-api/media/signed-cdn-files).
