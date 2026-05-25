# Misc

## Translate a message

Requires [Authorization](/grindr-api/api-authorization).

Paid feature.

```
POST /v5/chat/translate
```

Body:

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — string
- `targetLanguageCode` — string, e.g. `en`

Response:

- `translatedText` — string

Errors:

- `402` — `User has reached their entitlement limits`

## OCR recognition in chat, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

Appears to be a submitting endpoint rather than a retrieving one.

```
POST /v5/recognition/chat
```

## Rate an AI message suggestion, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v1/wingman/feedback
```

Body:

- `message_id` — string
- `prompt_id` — string
- `rating` — number, e.g. `1`
- `text` — string, feedback text
- `timestamp` — unix timestamp in milliseconds

Response:

Empty object (`{}`).

Errors:

- `400` — bad request

## TranslateMessageRequest

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — string
- `targetLanguageCode` — string, e.g. `en`

## WingmanFeedbackRequest

- `message_id` — string
- `prompt_id` — string
- `rating` — number, e.g. `1`
- `text` — string, feedback text
- `timestamp` — unix timestamp in milliseconds
