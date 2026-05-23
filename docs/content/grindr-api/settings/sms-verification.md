# SMS verification, WIP

> [!NOTE] This page is a work in progress. Endpoints below haven't been fully researched.

## Send SMS code

```
POST /v4/sms/verification/{profileId}/sendcode
```

Body:

- `country_code` — string
- `phone_number` — string

Response:

- `code` — number or `null`
- `message` — string, e.g. `"Profile is not verification required"` or `"Profile is already verified"`

## Verify SMS code

```
POST /v4/sms/verification/{profileId}/verifycode
```

Response:

- `code` — number or `null`
- `message` — string, e.g. `"Profile is not verification required"` or `"Profile is already verified"`

## Request SMS code for password change

```
POST /v4/sms/users/update-password/sendcode
```

Body:

- `country_code` — string
- `phone_number` — string

## Send SMS code (legacy), WIP

```
POST /v4/sms/sendcode
```

Body:

- `country_code` — string
- `phone_number` — string

## Verify SMS code (legacy)

```
POST /v4/sms/verifycode
```

Body:

- `country_code` — string
- `phone_number` — string
- `code` — string

Response:

Empty.

## Face recognition, WIP

```
POST /v4/recognition/face
```

## Spotify token, WIP

Body is URL-encoded (grant_type=authorization_code&code=...&redirect_uri=... or grant_type=refresh_token&refresh_token=... or grant_type=client_credentials).

```
POST /api/token
```

Body:

Content-Type: `application/x-www-form-urlencoded`

Map of string to string.

## Delete account, WIP

Requires [Authorization](/grindr-api/api-authorization).

```
DELETE /v3/me/profile
```

## SmsSendCodeRequest

- `country_code` — string
- `phone_number` — string

## SmsCodeResponse

- `code` — number or `null`
- `message` — string, e.g. `"Profile is not verification required"` or `"Profile is already verified"`

## SmsVerifyCodeLegacyRequest

- `country_code` — string
- `phone_number` — string
- `code` — string
