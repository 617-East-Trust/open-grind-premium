# Messages

## Get messages in a conversation

Requires [Authorization](/grindr-api/api-authorization).

Invoking this endpoint does not [mark messages as read](/grindr-api/messaging/conversations#mark-messages-as-read).

```
GET /v5/chat/conversation/{conversationId}/message
```

Query (optional):

- `pageKey` — string, optional, return messages with IDs before specified value, optional
- `profile` — boolean (`profile=true` | `profile=` + any other value), optional

Response:

- `lastReadTimestamp` — unix timestamp of recipient's side in milliseconds or `null`; is not affected by yours calls to [mark messages as read](/grindr-api/messaging/conversations#mark-messages-as-read)
- `messages` — array of [Message](/grindr-api/messaging/messages#message)
- `metadata` — object
  - `translate` — boolean
  - `hasSharedAlbums` — boolean
  - `isInAList` — boolean
- `profile` — object if `profile` query parameter is set to `true` or `null`

## Get a single message in a conversation

Requires [Authorization](/grindr-api/api-authorization).

```
GET /v4/chat/conversation/{conversationId}/message/{messageId}
```

Response:

- `message` — [Message](/grindr-api/messaging/messages#message)

## Send a message to a conversation

Requires [Authorization](/grindr-api/api-authorization).

Please don't use this for spam. Be civil.

When `replyToMessageId` is used in HTTP API appears to cause 400 Bad Request error.

See also: [Send a message to a conversation via ws](/grindr-api/websocket/commands#send-a-message-to-a-conversation-via-ws)

```
POST /v4/chat/message/send
```

Body:

- `type` — [Message type](/grindr-api/messaging/messages#message-type)
- `target` — [MessageTarget](/grindr-api/messaging/messages#messagetarget)
- `body` — object with [Message contents](/grindr-api/messaging/messages#message-contents) or `null`

Response:

HTTP status 201. [Message](/grindr-api/messaging/messages#message) object.

## Unsend a message

Requires [Authorization](/grindr-api/api-authorization).

Turns a message in chat into "This message was unsent."

Repeated requests are completed without errors.

```
POST /v4/chat/message/unsend
```

Body:

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — string, must be sent by you

Response:

Empty.

Errors:

- `500` — Internal Error if conversation or message was not found or if it wasn't sent by you

## Delete a message

Requires [Authorization](/grindr-api/api-authorization).

Deletes a message on your side. Does not delete message for other chat participant.

Repeated requests are completed without errors.

```
POST /v4/chat/message/delete
```

Body:

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — string, must be sent by you

Response:

Empty.

Errors:

- `500` — Internal Error if conversation or message was not found

## Send typing indicator, WIP

> [!NOTE]
> This endpoint hasn't been researched yet

Requires [Authorization](/grindr-api/api-authorization).

WIP, does not seem to work.

```
POST /v4/chatstatus/typing
```

Body:

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `status` — string, either `"Typing"` or `"Cleared"`

Response:

Empty.

Errors:

- `403` — Action not permitted if conversation not found

## React to a message

Requires [Authorization](/grindr-api/api-authorization).

There is no discovered way to undo the reaction as of yet.

Repeated requests are completed without errors.

```
POST /v4/chat/message/reaction
```

Body:

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — string
- `reactionType` — integer, (`1` is "🔥")

Response:

Empty.

Errors:

- `500` — Internal Error if conversation or message was not found

## MessageReaction

- `profileId` — integer
- `reactionType` — integer, (`1` is "🔥")

## Message type

Message type discriminator. There also appears to be a related `chat1Type`, could be legacy type.

- `"Album"`
- `"AlbumContentReaction"`
- `"AlbumContentReply"`
- `"Audio"`
- `"ExpiringAlbum"`
- `"ExpiringAlbumV2"`
- `"ExpiringImage"`
- `"Video"`
- `"Gaymoji"`
- `"Generative"`
- `"Giphy"`
- `"Image"`
- `"Location"`
- `"PrivateVideo"`
- `"ProfileLink"`
- `"ProfilePhotoReply"`
- `"Retract"`
- `"Text"`
- `"Unknown"`
- `"NonExpiringVideo"`
- `"VideoCall"`

## Chat1MessageType

Legacy message type.

- `"map"`
- `"image"`
- `"expiring_album"`
- `"expiring_image"`
- `"private_video"`
- `"expiring_video"`
- `"gaymoji"`
- `"giphy"`
- `"audio"`
- `"video_call"`
- `"video_call_v3"`
- `"audio_call"`
- `"text"`
- `"unknown"`
- `"retracted"`
- `"retracted_location"`
- `"album_share"`
- `"album_react"`
- `"album_content_reaction"`
- `"album_content_reply"`

## AlbumMessageBody

Body for `"Album"` message type. Includes everything from AlbumPreview and AlbumExpiration.

- *everything from [AlbumPreview](/grindr-api/messaging/albums#albumpreview)*
- *everything from [AlbumExpiration](/grindr-api/messaging/albums#albumexpiration)*
- `coverUrl` — string or `null`
- `ownerProfileId` — number or `null` if album has expired or was locked
- `isViewable` — boolean
- `hasVideo` — boolean
- `hasPhoto` — boolean
- `viewableUntil` — number or `null`

## AlbumContentReactionBody

Body for `"AlbumContentReaction"` message type. Implies "🔥" reaction as there does not appear to be any choice.

- `albumId` — integer
- `ownerProfileId` — integer or `null` if album has expired or was locked
- `albumContentId` — integer
- `previewUrl` — string or `null` if album has expired or was locked, see [Signed CDN files -> Chat media](/grindr-api/media/signed-cdn-files#chat-media)
- `expiresAt` — unix timestamp in milliseconds or `null`
- `viewable` — boolean

## AlbumContentReplyBody

Body for `"AlbumContentReply"` message type. Extends AlbumContentReaction.

- *everything from [AlbumContentReactionBody](/grindr-api/messaging/messages#albumcontentreactionbody)*
- `albumContentReply` — string
- `contentType` — string or `null` if album has expired or was locked

## AudioMessageBody

Body for `"Audio"` message type.

- `mediaId` — number
- `mediaHash` — string, See [Media -> Public CDN files -> Profile Images](/grindr-api/media/public-cdn-files#profile-images) or `null`
- `url` — string, see [Signed CDN files -> Chat media](/grindr-api/media/signed-cdn-files#chat-media)
- `contentType` — string, e.g. `audio/aac`
- `length` — number in milliseconds (1/1000th of a second) or `null`
- `expiresAt` — unix timestamp in milliseconds, 15 minutes

## VideoMessageBody

Body for `"Video"` message type. Additionally, for expiring videos `viewsRemaining` is set; capped at `2147483647` for "unlimited" views.

- `mediaId` — number or `null`
- `url` — string or `null`
- `fileCacheKey` — string
- `contentType` — string or `null`
- `length` — number
- `maxViews` — integer or `null`
- `looping` — boolean or `null`
- `viewsRemaining` — integer, optional

## PrivateVideoBody

Body for `"PrivateVideo"` message type. Extends [Video](#video).

- *everything from [VideoMessageBody](/grindr-api/messaging/messages#videomessagebody)*
- `viewCount` — integer

## GaymojiBody

Body for `"Gaymoji"` message type.

- `imageHash` — string

## GiphyBody

Body for `"Giphy"` message type. URLs point at `https://media0.giphy.com`.

- `id` — string
- `urlPath` — full URL to gif file
- `stillPath` — single frame, URL to gif file
- `previewPath` — string
- `width` — integer
- `height` — integer
- `imageHash` — string

## ImageMessageBody

Body for `"Image"` message type. Additional fields are present only for regular (non-expiring) images.

- `mediaId` — number
- `width` — integer or `null`
- `height` — integer or `null`
- `url` — string
- `imageHash` — string
- `takenOnGrindr` — boolean or `null`
- `createdAt` — unix timestamp in milliseconds or `null`

## ChatImageBody

"ChatImage" message body.

- `mediaId` — number
- `url` — string
- `expiresAt` — unix timestamp in milliseconds
- `takenOnGrindr` — boolean
- `createdTs` — unix timestamp in milliseconds

## ExpiringImageBody

Body for `"ExpiringImage"` message type. Extends [Image](#image) but with `url` set to `null` and additional `viewsRemaining`.

To get a single-use expiring image URL, you need to call [Get a single message in a conversation](/grindr-api/messaging/messages#get-a-single-message-in-a-conversation). After one call, `url` in message body will be replaced with `null` and image will no longer be viewable.

- *everything from [ImageMessageBody](/grindr-api/messaging/messages#imagemessagebody)*
- `url` — string or `null`
- `viewsRemaining` — number or `null`

## LocationMessageBody

Body for `"Location"` message type.

- `lat` — number
- `lon` — number

## ProfilePhotoReplyBody

Body for `"ProfilePhotoReply"` message type. Unknown, WIP.

- `imageHash` — string
- `photoContentReply` — string

## RetractMessageBody

Body for `"Retract"` message type. Unknown, WIP.

- `targetMessageId` — string

## TextMessageBody

Body for `"Text"` message type.

- `text` — string

## VideoCallMessageBody

Body for `"VideoCall"` message type. WIP. Only for "status" messages.

- `result` — string or `null`, appears to have the following values: `SUCCESSFUL`, `Duration:`, `Busy`, `BUSY`, `Cancelled`, `Declined`, `DECLINED`, `Missed`, `AB_Unsupported`, `No_Answer`, `UNANSWERED`, `Lite_Unsupport`
- `videoCallDuration` — number or `null`

## UnknownMessageBody

Empty type.

## Message contents

- [AlbumMessageBody](/grindr-api/messaging/messages#albummessagebody)
- [AlbumContentReactionBody](/grindr-api/messaging/messages#albumcontentreactionbody)
- [AlbumContentReplyBody](/grindr-api/messaging/messages#albumcontentreplybody)
- [AudioMessageBody](/grindr-api/messaging/messages#audiomessagebody)
- [VideoMessageBody](/grindr-api/messaging/messages#videomessagebody)
- [PrivateVideoBody](/grindr-api/messaging/messages#privatevideobody)
- [GaymojiBody](/grindr-api/messaging/messages#gaymojibody)
- [GiphyBody](/grindr-api/messaging/messages#giphybody)
- [ImageMessageBody](/grindr-api/messaging/messages#imagemessagebody)
- [ChatImageBody](/grindr-api/messaging/messages#chatimagebody)
- [ExpiringImageBody](/grindr-api/messaging/messages#expiringimagebody)
- [LocationMessageBody](/grindr-api/messaging/messages#locationmessagebody)
- [ProfilePhotoReplyBody](/grindr-api/messaging/messages#profilephotoreplybody)
- [RetractMessageBody](/grindr-api/messaging/messages#retractmessagebody)
- [TextMessageBody](/grindr-api/messaging/messages#textmessagebody)
- [VideoCallMessageBody](/grindr-api/messaging/messages#videocallmessagebody)
- [UnknownMessageBody](/grindr-api/messaging/messages#unknownmessagebody)
- [ExpiringAlbumBody](/grindr-api/messaging/messages#expiringalbumbody)
- [ExpiringAlbumV2Body](/grindr-api/messaging/messages#expiringalbumv2body)
- [NonExpiringVideoBody](/grindr-api/messaging/messages#nonexpiringvideobody)
- [GenerativeBody](/grindr-api/messaging/messages#generativebody)
- [ProfileLinkBody](/grindr-api/messaging/messages#profilelinkbody)

## Message

- `messageId` — string, appears to be a unix timestamp in milliseconds and UUIDv4 separated by `:`, e.g. `"1774296692000:843daee8-1e93-47d6-bc7f-3d981925a393"`
- `conversationId` — string, see [Conversation](/grindr-api/messaging/conversations#conversation)
- `senderId` — number
- `timestamp` — unix timestamp in milliseconds, appears to be same as in `messageId`
- `unsent` — boolean, if this is true, `body` is set to `null`
- `reactions` — array of [MessageReaction](/grindr-api/messaging/messages#messagereaction)
- `type` — string, see [Message type](/grindr-api/messaging/messages#message-type)
- `body` — object with [Message contents](/grindr-api/messaging/messages#message-contents) or `null`
- `replyToMessage` — unknown or `null`
- `dynamic` — boolean, unknown purpose, WIP
- `chat1Type` — string, see [Message type](/grindr-api/messaging/messages#message-type)
- `replyPreview` — unknown or `null`

## ConversationMessagesResponse

- `lastReadTimestamp` — unix timestamp of recipient's side in milliseconds or `null`; is not affected by yours calls to [mark messages as read](/grindr-api/messaging/conversations#mark-messages-as-read)
- `messages` — array of [Message](/grindr-api/messaging/messages#message)
- `metadata` — object
  - `translate` — boolean
  - `hasSharedAlbums` — boolean
  - `isInAList` — boolean
- `profile` — object if `profile` query parameter is set to `true` or `null`

## MessageTarget

- `type` — string, `Direct`, `Group`, `HumanWingman`
- `targetId` — integer

## SendMessageRequest

- `type` — [Message type](/grindr-api/messaging/messages#message-type)
- `target` — [MessageTarget](/grindr-api/messaging/messages#messagetarget)
- `body` — object with [Message contents](/grindr-api/messaging/messages#message-contents) or `null`

Additional body fields for websocket only:

- `replyToMessageId` — string or `null`

## MessageMutationRequest

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — string, must be sent by you

## TypingIndicatorRequest

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `status` — string, either `"Typing"` or `"Cleared"`

## MessageReactionRequest

- `conversationId` — [Conversation ID](/grindr-api/messaging/conversations#conversation-id)
- `messageId` — string
- `reactionType` — integer, (`1` is "🔥")

## ExpiringAlbumBody

Body for `"ExpiringAlbum"` message type. Extends [Album](#album-message-body).

- *everything from [AlbumMessageBody](/grindr-api/messaging/messages#albummessagebody)*

## ExpiringAlbumV2Body

Body for `"ExpiringAlbumV2"` message type. For [AlbumExpirationType](/grindr-api/messaging/albums#albumexpirationtype) = `ONCE` but might have other values if expiration settings were changed later. Extends [ExpiringAlbum](#expiring-album-body).

- *everything from [ExpiringAlbumBody](/grindr-api/messaging/messages#expiringalbumbody)*

## NonExpiringVideoBody

> [!NOTE]
> This type hasn't been researched yet

Body for `"NonExpiringVideo"` message type. Unknown, WIP.

## GenerativeBody

> [!NOTE]
> This type hasn't been researched yet

Body for `"Generative"` message type. Unknown, WIP.

## ProfileLinkBody

> [!NOTE]
> This type hasn't been researched yet

Body for `"ProfileLink"` message type. Unknown, WIP.
