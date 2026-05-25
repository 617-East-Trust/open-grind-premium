# Account

## Validate password complexity, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v3/users/password-validation
```

## Register FCM push token, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v3/gcm-push-tokens
```

## Update password, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v3/users/update-password
```

## Update account email, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v3/users/email
```

Response:

- `profileId` — string with numbers, account's ID
- `sessionId` — [Session ID](/grindr-api/authentication#session-id)
- `authToken` — string, Auth token for session refresh

## Update password via phone, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v4/sms/users/update-password
```

## Create third party account, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v7/users/thirdparty
```

## Forgot password, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v3/users/forgot-password
```

## Exchange Google access token, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v3/users/thirdparty/exchange
```

## Create phone session, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v4/sms/sessions
```

Response:

- `profileId` — string with numbers, account's ID
- `sessionId` — [Session ID](/grindr-api/authentication#session-id)
- `authToken` — string, Auth token for session refresh

## Get preferences

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v3/me/prefs/settings
```

Response:

- `profileId` — integer
- `locationSearchOptOut` — boolean
- `incognito` — boolean
- `hideViewedMe` — boolean
- `approximateDistance` — boolean
- `viewRightNowNsfw` — boolean

## Set preferences

Requires [Authorization](/grindr-api/api-authorization).

```
PUT /v3/me/prefs/settings
```

Body:

- `locationSearchOptOut` — boolean
- `incognito` — boolean
- `hideViewedMe` — boolean
- `approximateDistance` — boolean
- `viewRightNowNsfw` — boolean

Response:

Empty.

## Get visiting settings

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v1/visiting/settings
```

Response:

- `setting` — string, `"AUTO"`, WIP

## Set visiting settings

Requires [Authorization](/grindr-api/api-authorization).

```
PUT /v1/visiting/settings
```

Body:

- `setting` — string, `"AUTO"`, WIP

Response:

Empty.

## Get home location

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v1/visiting/home
```

Response:

- `name` — string, [human-readable name](/grindr-api/browse/location#search-places-by-name) of location
- `lat` — float
- `lon` — float

## Set home location

Requires [Authorization](/grindr-api/api-authorization).

```
PUT /v1/visiting/home
```

Body:

- `lat` — float
- `lon` — float

Response:

- `name` — string, [human-readable name](/grindr-api/browse/location#search-places-by-name) of location
- `lat` — float
- `lon` — float

## AccountPreferences

- `profileId` — integer
- `locationSearchOptOut` — boolean
- `incognito` — boolean
- `hideViewedMe` — boolean
- `approximateDistance` — boolean
- `viewRightNowNsfw` — boolean

## AccountPreferencesUpdate

- `locationSearchOptOut` — boolean
- `incognito` — boolean
- `hideViewedMe` — boolean
- `approximateDistance` — boolean
- `viewRightNowNsfw` — boolean

## VisitingSettings

- `setting` — string, `"AUTO"`, WIP

## HomeLocation

- `name` — string, [human-readable name](/grindr-api/browse/location#search-places-by-name) of location
- `lat` — float
- `lon` — float

## HomeLocationMutation

- `lat` — float
- `lon` — float
