# Conversations

## Get conversations

Requires [Authorization](/grindr-api/api-authorization).

*Also `POST /v3/inbox`, seems to be aliased to v4 now*

```
POST /v4/inbox
```

Query (optional):

- `page` — 1-based number, pagination, optional

Body (optional):

- `unreadOnly` — boolean
- `chemistryOnly` — boolean
- `favoritesOnly` — boolean
- `rightNowOnly` — boolean
- `onlineNowOnly` — boolean
- `distanceMeters` — "double" number value or `null`
- `positions` — array of integers, [sexual position IDs](/grindr-api/users/profiles#sexual-position-id)

Response:

- `entries` — array of [Conversation](/grindr-api/messaging/conversations#conversation)
- `showsFreeHeaderLabel` — boolean
- `totalFullConversations` — number, e.g. `"5"`
- `totalPartialConversations` — number, e.g. `0`
- `maxDisplayLockCount` — number, e.g. `99`
- `nextPage` — integer, e.g. `2`

## Get conversations (v3 alias)

Requires [Authorization](/grindr-api/api-authorization).

```
POST /v3/inbox
```

Query (optional):

- `page` — 1-based number, pagination, optional

Body (optional):

- `unreadOnly` — boolean
- `chemistryOnly` — boolean
- `favoritesOnly` — boolean
- `rightNowOnly` — boolean
- `onlineNowOnly` — boolean
- `distanceMeters` — "double" number value or `null`
- `positions` — array of integers, [sexual position IDs](/grindr-api/users/profiles#sexual-position-id)

Response:

- `entries` — array of [Conversation](/grindr-api/messaging/conversations#conversation)
- `showsFreeHeaderLabel` — boolean
- `totalFullConversations` — number, e.g. `"5"`
- `totalPartialConversations` — number, e.g. `0`
- `maxDisplayLockCount` — number, e.g. `99`
- `nextPage` — integer, e.g. `2`

## Get conversations by IDs

Note: in this v1 endpoint, each `participants` item only includes `profileId`, `primaryMediaHash`, `lastOnline`, and `distanceMetres`. Fields `onlineUntil`, `position`, `isInAList`, and `hasDatingPotential` are absent.

```
POST /v1/inbox/conversation
```

Body:

Array of [Conversation ID](/grindr-api/messaging/conversations#conversation-id).

Response:

Array of [ConversationData](/grindr-api/messaging/conversations#conversationdata).

## Delete a conversation

Requires [Authorization](/grindr-api/api-authorization).

Deletes the conversation on your side. Does not delete the conversation for other chat's participant.

Repeated requests are completed without errors.

```
DELETE /v4/chat/conversation/{conversationId}
```

Response:

Empty.

## Pin a conversation

Requires [Authorization](/grindr-api/api-authorization).

Affects sorting position in [get conversations](/grindr-api/messaging/conversations#get-conversations) endpoint response.

Requests on nonexistent conversations seem to be affecting them after they have been created.

Repeated requests are completed without errors.

```
POST /v4/chat/conversation/{conversationId}/pin
```

Response:

Empty.

## Unpin a conversation

Requires [Authorization](/grindr-api/api-authorization).

Affects sorting position in [get conversations](/grindr-api/messaging/conversations#get-conversations) endpoint response. Requests on nonexistent conversations seem to be affecting them after they have been created.

Repeated requests are completed without errors.

```
POST /v4/chat/conversation/{conversationId}/unpin
```

Response:

Empty.

## Mute a conversation

Requires [Authorization](/grindr-api/api-authorization).

Requests on nonexistent conversations seem to be affecting them after they have been created.

Repeated requests are completed without errors.

```
POST /v1/push/conversation/{conversationId}/mute
```

Response:

Empty

## Unmute a conversation

Requires [Authorization](/grindr-api/api-authorization).

Requests on nonexistent conversations seem to be affecting them after they have been created.

Repeated requests are completed without errors.

```
POST /v1/push/conversation/{conversationId}/unmute
```

Response:

Empty

## Get shared media in conversation

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v5/chat/media/shared/images/with-me/{conversationId}
```

Response:

- `images` — array of [ChatImageBody](/grindr-api/messaging/messages#chatimagebody)

## Refresh messages

Requires [Authorization](/grindr-api/api-authorization).

Requests the messages from the message id

```
POST /v4/chat/conversation/{conversationId}/message-by-id
```

Body:

- `messageIds` — array of strings

Response:

- `messages` — array of [Message](/grindr-api/messaging/messages#message)

## Mark messages as read

Requires [Authorization](/grindr-api/api-authorization).

Regardless of messageId passed, the whole conversation's [`unreadCount`](/grindr-api/messaging/conversations#conversation) will be reset to 0. messageId is taken into account to present the "Read" label to sender.

If you'd like to mark conversation as read but don't show it to other participant, you could pass a valid but nonexistent messageId, such as `0:00000000-0000-0000-0000-000000000000`.

Invalid messageIds will cause HTTP status 400 Bad Request errors.

```
POST /v4/chat/conversation/{conversationId}/read/{messageId}
```

Response:

Empty.

## AI chat suggestions

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v1/chat/suggestions
```

Query:

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)

Response:

- `suggestions` — array of objects
  - `id` — UUIDv3
  - `text` — string
  - `type` — string, `SAVED_PHRASE` | `SMART_PHRASE`

## Chat AI summary feedback, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

```
POST /v1/chat/summary/feedback
```

## Conversation ID

String with two long integers separated by `:`, e.g. `"12345678:23456789"`. Long integers are IDs of [Profile](/grindr-api/users/profiles#profile). The order of these IDs is always from smaller ID to higher ID, regardless of who started the chat.

## ConversationParticipant

- `profileId` — integer, [Profile ID](/grindr-api/users/profiles#profilemin)
- `primaryMediaHash` — string or `null`, see [Media -> Public CDN files](/grindr-api/media/public-cdn-files)
- `lastOnline` — unix timestamp in milliseconds
- `onlineUntil` — unix timestamp in milliseconds or `null`
- `distanceMetres` — float number or `null`
- `position` — [Sexual position ID](/grindr-api/users/profiles#sexual-position-id) or `null`
- `isInAList` — boolean
- `hasDatingPotential` — boolean

## ConversationPreview

- `conversationId` — object
  - `value` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — see [Message](/grindr-api/messaging/messages#message) for format
- `chat1MessageId` — string with UUIDv4, second part of `messageId`
- `senderId` — [Profile ID](/grindr-api/users/profiles#profilemin)
- `type` — [Message type](/grindr-api/messaging/messages#message-type)
- `chat1Type` — [Chat1MessageType](/grindr-api/messaging/messages#chat1messagetype)
- `text` — string or `null`
- `url` — unknown or `null`
- `lat` — unknown or `null`
- `lon` — unknown or `null`
- `albumId` — integer or `null`
- `albumContentId` — unknown or `null`
- `albumContentReply` — unknown or `null`
- `duration` — unknown or `null`
- `imageHash` — string or `null`
- `photoContentReply` — unknown or `null`

## ConversationData

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `name` — string, profile name, may be an empty string, e.g. `""`
- `participants` — array of [ConversationParticipant](/grindr-api/messaging/conversations#conversationparticipant)
- `lastActivityTimestamp` — unix timestamp in milliseconds
- `unreadCount` — integer
- `preview` — [ConversationPreview](/grindr-api/messaging/conversations#conversationpreview)
- `muted` — boolean
- `pinned` — boolean
- `favorite` — boolean
- `context` — unknown or `null`
- `onlineUntil` — unknown or `null`
- `translatable` — boolean
- `rightNow` — string, e.g. `"NOT_ACTIVE"`
- `hasUnreadThrob` — boolean

## Conversation

Conversation object.

- `type` — string, e.g. `"full_conversation_v1"`
- `data` — [ConversationData](/grindr-api/messaging/conversations#conversationdata)

## InboxFilterRequest

- `unreadOnly` — boolean
- `chemistryOnly` — boolean
- `favoritesOnly` — boolean
- `rightNowOnly` — boolean
- `onlineNowOnly` — boolean
- `distanceMeters` — "double" number value or `null`
- `positions` — array of integers, [sexual position IDs](/grindr-api/users/profiles#sexual-position-id)

## InboxResponse

- `entries` — array of [Conversation](/grindr-api/messaging/conversations#conversation)
- `showsFreeHeaderLabel` — boolean
- `totalFullConversations` — number, e.g. `"5"`
- `totalPartialConversations` — number, e.g. `0`
- `maxDisplayLockCount` — number, e.g. `99`
- `nextPage` — integer, e.g. `2`

## ConversationProfileMini

- `profileId` — long integer
- `name` — string
- `mediaHash` — string, See [Media](/grindr-api/media/) or `null`
- `onlineUntil` — unknown or `null`
- `distance` — number or `null`
- `showDistance` — boolean

## ChatSuggestionsResponse

- `suggestions` — array of objects
  - `id` — UUIDv3
  - `text` — string
  - `type` — string, `SAVED_PHRASE` | `SMART_PHRASE`
