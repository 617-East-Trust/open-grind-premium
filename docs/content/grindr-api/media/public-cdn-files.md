# Public CDN files

CDN files that are public are accessible directly using their hash, e.g. profile images.

Base URL: `https://cdns.grindr.com`

## Grindr Gaymoji

```
GET /grindr/chat/gaymoji
```

Response:

- `lastUpdateTime` — unix timestamp in milliseconds
- `gaymoji` — array of objects
  - `name` — string, unique identifier consisting of alphanumeric characters, hyphens and underscores
  - `id` — string, same as `name` + `.png`
  - `category` — [GaymojiCategory](/grindr-api/media/public-cdn-files#gaymojicategory)
- `category` — array of objects
  - `name` — [GaymojiCategory](/grindr-api/media/public-cdn-files#gaymojicategory)
  - `expiredTime` — unix timestamp in milliseconds, may be `0`

## Get gaymoji asset

ID must include file extension (e.g. `name.png`).

```
GET /grindr/chat/gaymoji/{id}
```

Response:

Image binary.

## Profile images

One side will always be the requested size and another will be less or equal to the requested size.

Available sizes for `{size}` parameter:

- `2048x2048` (might be unavailable)
- `1024x1024`
- `480x480`
- `320x320`

```
GET /images/profile/{size}/{mediaHash}
```

Response:

Image binary.

## Thumbnails images

Image will be cropped at center and both sides will be exactly the requested size.

Available sizes for `{size}` parameter:
- `480x480` (might be unavailable)
- `320x320`
- `75x75`

```
GET /images/thumb/{size}/{mediaHash}
```

Response:

Image binary.

## GaymojiCategory

- `"body"`
- `"dating+sex"`
- `"featured"`
- `"holiday"`
- `"mood"`
- `"objects"`
- `"profile"`
- `"wen_ching_taiwan_stickers"`
