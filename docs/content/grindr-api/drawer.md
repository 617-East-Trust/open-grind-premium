# Drawer, WIP

> [!NOTE] This page is a work in progress. Endpoints below haven't been fully researched.

## Get media in drawer

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v4/chat/media/drawer
```

Response:

Array of [DrawerMedia](/grindr-api/drawer#drawermedia).

## Get media in drawer for a conversation

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v4/chat/media/drawer/{conversationId}
```

Response:

Array of [DrawerMedia](/grindr-api/drawer#drawermedia).

## Add media to drawer

Requires [Authorization](/grindr-api/api-authorization).

MediaId must be obtained through [uploading](/grindr-api/users/profiles#upload-media).

```
PUT /v4/chat/media/drawer/{mediaId}
```

Response:

Empty.

Errors:

- `500` — Repeated requests cause 500 HTTP status "Internal Error".

## Delete media from drawer

Requires [Authorization](/grindr-api/api-authorization).

Repeated requests are completed without errors.

```
DELETE /v4/chat/media/drawer/{mediaId}
```

Response:

Empty with HTTP status 202.

## DrawerMedia

- `id` — long integer
- `url` — URL
- `contentType` — string
- `createdTs` — unix timestamp in milliseconds
- `used` — boolean
- `takenOnGrindr` — boolean
