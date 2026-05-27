# Right Now, WIP

> [!NOTE]
> This page is a work in progress. Endpoints below haven't been fully researched.

## Get active right now post, WIP

Response type: `RightNowGetActivePostResponse` (undocumented).

```
GET /v3/rightnow/active-post
```

## Get other user right now post, WIP

Response type: `RightNowGetOtherUserPostResponse` (undocumented).

```
GET /v3/rightnow/profiles/{profileId}
```

## Update right now post, WIP

Body type: `UpdatePostRequest` (undocumented).

```
PATCH /{version}/rightnow/posts/{postId}
```

## Create right now post v4, WIP

Body type: `CreatePostRequest` (undocumented).

Response type: `RightNowCreatePostResponse` (undocumented).

```
POST /v4/rightnow/posts
```

## Create right now post v3, WIP

Body type: `CreatePostRequest` (undocumented).

Response type: `RightNowCreatePostResponse` (undocumented).

```
POST /v3/rightnow/posts
```

## Update right now post settings, WIP

Body type: `UpdatePostSettingsRequest` (undocumented).

```
PATCH /v1/rightnow/posts/{postId}/settings
```

## Get right now feed, WIP

Response type: `RightNowGetFeedResponse` (undocumented).

```
GET /v5/rightnow/feed
```

Query (optional):

- `sort` — string, optional
- `hosting` — boolean, optional
- `ageMin` — integer, optional
- `ageMax` — integer, optional
- `sexualPositions` — string, optional

## Upload right now media, WIP

Response type: `RightNowMediaUploadResponse` (undocumented).

```
POST /v1/media/upload
```

Query (optional):

- `img_1_bottom` — integer, optional
- `img_1_left` — integer, optional
- `img_1_right` — integer, optional
- `img_1_top` — integer, optional

Body:

Binary file.

## Get right now google play sku, WIP

Response type: `RightNowSkuResponse` (undocumented).

```
GET /v1/rightnow/googleplay/sku
```

## Create right now request, WIP

Body type: `RightNowCreateRequestData` (undocumented).

```
POST /v1/rightnow/requests
```

## RightNowStatus

- `"NOT_ACTIVE"`
- `"HOSTING"`
- `"NOT_HOSTING"`
