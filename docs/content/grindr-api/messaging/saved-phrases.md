# Saved phrases

## Get saved phrases

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v1/chat/phrases
```

Response:

- `phrases` — array of [SavedPhrase](/grindr-api/messaging/saved-phrases#savedphrase)

## Add a saved phrase

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v1/chat/phrases
```

Body:

- `text` — string

Response:

- `phrase` — [SavedPhrase](/grindr-api/messaging/saved-phrases#savedphrase)

## Get saved phrases (legacy)

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v3/me/prefs
```

Response:

- `phrases` — object
  - *key is phrase ID (uuid)*
  - `phraseId` — string, uuid
  - `phraseText` — string
  - `timestamp` — unix timestamp in milliseconds
  - `frequency` — integer, see [Track phrase usage frequency](/grindr-api/messaging/saved-phrases#track-phrase-usage-frequency)

## Add a saved phrase (legacy)

Requires [Authorization](/grindr-api/api-authorization).

This endpoint is somewhat broken and sometimes throws 500 ISE error or .

```
POST /v3/me/prefs/phrases
```

Body:

- `phrase` — string

Response:

- `phrase` — [SavedPhrase](/grindr-api/messaging/saved-phrases#savedphrase)

## Delete a saved phrase

Requires [Authorization](/grindr-api/api-authorization).

```
DELETE /v3/me/prefs/phrases/{id}
```

Response:

Empty.

## Track phrase usage frequency

Requires [Authorization](/grindr-api/api-authorization).

Doesn't appear to influence the phrase's sorting position in [Get saved phrases](/grindr-api/messaging/saved-phrases#get-saved-phrases) response. Increments value in [Get saved phrases (legacy)](/grindr-api/messaging/saved-phrases#get-saved-phrases-legacy) endpoint.

```
POST /v4/phrases/frequency/{id}
```

Response:

Empty.

## SavedPhrase

- `id` — string
- `text` — string
- `type` — string, e.g. `"user"`

## SavedPhrasesResponse

- `phrases` — array of [SavedPhrase](/grindr-api/messaging/saved-phrases#savedphrase)

## LegacyPrefsResponse

- `phrases` — object
  - *key is phrase ID (uuid)*
  - `phraseId` — string, uuid
  - `phraseText` — string
  - `timestamp` — unix timestamp in milliseconds
  - `frequency` — integer, see [Track phrase usage frequency](/grindr-api/messaging/saved-phrases#track-phrase-usage-frequency)
