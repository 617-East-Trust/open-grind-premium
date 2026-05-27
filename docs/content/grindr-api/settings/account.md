# Account

## Login via email, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

> [!NOTE]
> URL is dynamic and currently unknown.

Body type: `LoginEmailRequest` (undocumented).

Response type: `AuthResponse` (undocumented).

```
POST /v?/sessions/email
```

## Create account via email, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

> [!NOTE]
> URL is dynamic and currently unknown.

Body type: `CreateAccountEmailRequest` (undocumented).

Response type: `FirstPartyCreateAccountResponse` (undocumented).

```
POST /v?/users/registration/email
```

## Login via third party, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

> [!NOTE]
> URL is dynamic and currently unknown.

Body type: `ThirdPartyRequest` (undocumented).

Response type: `ThirdPartyAuthResponse` (undocumented).

```
POST /v?/sessions/thirdparty
```

## Create third party session, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

> [!NOTE]
> URL is dynamic and currently unknown.

Body type: `ThirdPartySessionRequest` (undocumented).

Response type: `ThirdPartyAuthResponse` (undocumented).

```
POST /v?/sessions/thirdparty/session
```

## Validate password complexity, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `ValidatePasswordComplexityRequest` (undocumented).

```
POST /v3/users/password-validation
```

## Register FCM push token, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `FcmPushRequest` (undocumented).

```
POST /v3/gcm-push-tokens
```

## Update password, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `ChangePasswordRequest` (undocumented).

Response type: `ChangePasswordResponse` (undocumented).

```
POST /v3/users/update-password
```

## Update account email, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `UpdateEmailRequest` (undocumented).

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

Body type: `ChangePasswordPhoneRequest` (undocumented).

Response type: `ChangePasswordResponse` (undocumented).

```
POST /v4/sms/users/update-password
```

## Create third party account, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `CreateThirdPartyAccountRequest` (undocumented).

Response type: `ThirdPartyCreateAccountResponse` (undocumented).

```
POST /v7/users/thirdparty
```

## Forgot password, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `ForgotPwdEmailRequest` (undocumented).

Response type: `ForgotPwdEmailResponse` (undocumented).

```
POST /v3/users/forgot-password
```

## Exchange Google access token, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `GoogleAccessTokenRequest` (undocumented).

Response type: `GoogleAccessTokenResponse` (undocumented).

```
POST /v3/users/thirdparty/exchange
```

## Create phone session, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Body type: `LoginPhoneRequest` (undocumented).

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
