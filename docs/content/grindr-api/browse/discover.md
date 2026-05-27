# Discover, WIP

> [!NOTE]
> This page is a work in progress. Endpoints below haven't been fully researched. Help needed.

## Get discover profiles, WIP

Response type: `DiscoverResponse` (undocumented).

```
GET /v3/discover
```

Query:

- `geohash` — [Geohash](/grindr-api/browse/location#geohash)
- `previewedProfiles` — array of long integers

Response:

If subscription is purchased,

Unknown, WIP.

If subscription is not purchased,

- `status` — string, `"SubscriptionRequired"`
- `body` — object
  - `upsell` — string, `"Unknown"`
  - `profiles` — array, unknown

## Get discover profiles (legacy), WIP

Response type: `DiscoverResponse` (undocumented).

```
GET /v2/discover
```

Query:

- `geohash` — [Geohash](/grindr-api/browse/location#geohash)

Response:

If subscription is purchased,

Unknown, WIP.

If subscription is not purchased,

- `status` — string, `"SubscriptionRequired"`
- `body` — object
  - `upsell` — string, `"Unknown"`

## Post a pass, WIP

```
POST /v1/me/pass/{profileId}
```
