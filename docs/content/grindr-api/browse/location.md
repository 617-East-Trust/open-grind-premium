# Location

## Search places by name

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v3/places/search
```

Query:

- `placeName` — string, e.g. `Paris`

Response:

- `places` — array of [Place](/grindr-api/browse/location#place)

## Update location

Requires [Authorization](/grindr-api/api-authorization).

```
PUT /v4/location
```

Body:

- `geohash` — [Geohash](/grindr-api/browse/location#geohash)

Response:

Empty.

## Geohash

Grindr requires geohash to be exactly 12 characters long. <https://en.wikipedia.org/wiki/Geohash>

Example: `ezjmgyern222` (Madrid, Spain)

> [!WARNING]: Setting geohash inside of United Kingdom will lead to the account being locked and resetting geohash being impossible until you submit legal documents for age verification.

Geohash explorer: <https://geohash.softeng.co/>

## Place

- `name` — string
- `address` — string or `null`
- `lat` — number
- `lon` — number
- `placeId` — string with number
- `importance` — float
