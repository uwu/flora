# Flora SDK API Reference

Generated from Flora v0.1.0

## Event Payloads

These types represent the data received in event handlers.

### `InteractionCreatePayload`

| Field               | Type                | Description |
| ------------------- | ------------------- | ----------- |
| `interaction_id`    | `String`            |             |
| `interaction_token` | `String`            |             |
| `application_id`    | `String`            |             |
| `guild_id`          | `Option`            |             |
| `channel_id`        | `Option`            |             |
| `user`              | `UserPayload`       |             |
| `member`            | `Option`            |             |
| `command_name`      | `String`            |             |
| `data`              | `serde_json::Value` |             |
| `locale`            | `Option`            |             |
| `guild_locale`      | `Option`            |             |

### `MemberPayload`

| Field                          | Type          | Description |
| ------------------------------ | ------------- | ----------- |
| `user`                         | `UserPayload` |             |
| `nick`                         | `Option`      |             |
| `avatar`                       | `Option`      |             |
| `roles`                        | `Vec`         |             |
| `joined_at`                    | `Option`      |             |
| `premium_since`                | `Option`      |             |
| `deaf`                         | `bool`        |             |
| `mute`                         | `bool`        |             |
| `flags`                        | `u32`         |             |
| `pending`                      | `bool`        |             |
| `permissions`                  | `Option`      |             |
| `communication_disabled_until` | `Option`      |             |

### `MessageDeleteBulkPayload`

| Field        | Type     | Description |
| ------------ | -------- | ----------- |
| `ids`        | `Vec`    |             |
| `channel_id` | `String` |             |
| `guild_id`   | `Option` |             |

### `MessageDeletePayload`

| Field        | Type     | Description |
| ------------ | -------- | ----------- |
| `id`         | `String` |             |
| `channel_id` | `String` |             |
| `guild_id`   | `Option` |             |

### `MessagePayload`

| Field        | Type          | Description |
| ------------ | ------------- | ----------- |
| `id`         | `String`      |             |
| `channel_id` | `String`      |             |
| `guild_id`   | `Option`      |             |
| `content`    | `String`      |             |
| `author`     | `UserPayload` |             |
| `member`     | `Option`      |             |

### `MessageUpdatePayload`

| Field              | Type     | Description |
| ------------------ | -------- | ----------- |
| `id`               | `String` |             |
| `channel_id`       | `String` |             |
| `guild_id`         | `Option` |             |
| `content`          | `Option` |             |
| `author`           | `Option` |             |
| `edited_timestamp` | `Option` |             |
| `old`              | `Option` |             |
| `new`              | `Option` |             |

### `ReadyPayload`

| Field       | Type          | Description |
| ----------- | ------------- | ----------- |
| `user`      | `UserPayload` |             |
| `guild_ids` | `Vec`         |             |

### `UserPayload`

| Field           | Type     | Description |
| --------------- | -------- | ----------- |
| `id`            | `String` |             |
| `username`      | `String` |             |
| `discriminator` | `Option` |             |
| `bot`           | `bool`   |             |

## Op Input Types

These types represent the arguments passed to runtime operations.

### `AllowedMentionsInput`

| Field          | Type     | Description |
| -------------- | -------- | ----------- |
| `parse`        | `Option` |             |
| `users`        | `Option` |             |
| `roles`        | `Option` |             |
| `replied_user` | `Option` |             |

### `EmbedAuthorInput`

| Field      | Type     | Description |
| ---------- | -------- | ----------- |
| `name`     | `Option` |             |
| `url`      | `Option` |             |
| `icon_url` | `Option` |             |

### `EmbedFieldInput`

| Field    | Type     | Description |
| -------- | -------- | ----------- |
| `name`   | `String` |             |
| `value`  | `String` |             |
| `inline` | `bool`   |             |

### `EmbedFooterInput`

| Field      | Type     | Description |
| ---------- | -------- | ----------- |
| `text`     | `Option` |             |
| `icon_url` | `Option` |             |

### `EmbedInput`

| Field         | Type     | Description |
| ------------- | -------- | ----------- |
| `title`       | `Option` |             |
| `description` | `Option` |             |
| `url`         | `Option` |             |
| `color`       | `Option` |             |
| `timestamp`   | `Option` |             |
| `footer`      | `Option` |             |
| `image`       | `Option` |             |
| `thumbnail`   | `Option` |             |
| `author`      | `Option` |             |
| `fields`      | `Option` |             |

### `EmbedMediaInput`

| Field | Type     | Description |
| ----- | -------- | ----------- |
| `url` | `Option` |             |

### `EditMessageArgs`

| Field              | Type     | Description |
| ------------------ | -------- | ----------- |
| `channel_id`       | `String` |             |
| `message_id`       | `String` |             |
| `content`          | `Option` |             |
| `embeds`           | `Option` |             |
| `allowed_mentions` | `Option` |             |
| `flags`            | `Option` |             |

### `InteractionResponseArgs`

| Field              | Type     | Description |
| ------------------ | -------- | ----------- |
| `interaction_id`   | `String` |             |
| `token`            | `String` |             |
| `content`          | `Option` |             |
| `embeds`           | `Option` |             |
| `attachments`      | `Option` |             |
| `tts`              | `Option` |             |
| `allowed_mentions` | `Option` |             |
| `ephemeral`        | `Option` |             |

### `SendMessageArgs`

| Field              | Type     | Description |
| ------------------ | -------- | ----------- |
| `channel_id`       | `String` |             |
| `content`          | `Option` |             |
| `embeds`           | `Option` |             |
| `attachments`      | `Option` |             |
| `tts`              | `Option` |             |
| `allowed_mentions` | `Option` |             |
| `flags`            | `Option` |             |
| `message_id`       | `Option` |             |
| `reply_to`         | `Option` |             |

### `UpsertGuildCommandsArgs`

| Field      | Type     | Description |
| ---------- | -------- | ----------- |
| `guild_id` | `String` |             |
| `commands` | `Vec`    |             |

## KV Store Types

Types for the key-value store API.

### `KvKeyInfo`

| Field        | Type     | Description |
| ------------ | -------- | ----------- |
| `name`       | `String` |             |
| `expiration` | `Option` |             |
| `metadata`   | `Option` |             |

### `KvKeyMetadata`

| Field        | Type     | Description |
| ------------ | -------- | ----------- |
| `expiration` | `Option` |             |
| `metadata`   | `Option` |             |

### `ListKeysOptions`

| Field    | Type     | Description |
| -------- | -------- | ----------- |
| `prefix` | `Option` |             |
| `limit`  | `Option` |             |
| `cursor` | `Option` |             |

### `ListKeysResult`

| Field           | Type     | Description |
| --------------- | -------- | ----------- |
| `keys`          | `Vec`    |             |
| `list_complete` | `bool`   |             |
| `cursor`        | `Option` |             |

### `SetOptions`

| Field        | Type     | Description |
| ------------ | -------- | ----------- |
| `expiration` | `Option` |             |
| `metadata`   | `Option` |             |
