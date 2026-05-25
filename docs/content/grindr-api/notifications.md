# Notifications

## Acknowledge notifications, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /public/v1/notifications/ack
```

Body:

- `notificationId` — string
- `source` — string, `"WEBSOCKET"` | `"PUSH"`, optional

Response:

Empty.

## NotificationsAckRequest

- `notificationId` — string
- `source` — string, `"WEBSOCKET"` | `"PUSH"`, optional
