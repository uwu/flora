---
title: 'Embeds'
description: 'Create rich embeds for Discord messages'
outline: deep
---

# Embeds

Embeds are structured message blocks in Discord. They are useful for status messages, profiles, reports, and anything that reads better with fields or a clear title.

flora provides an `embed()` builder with chainable methods.

## Basic Usage

```typescript
const myEmbed = embed()
  .setTitle('Status')
  .setDescription('All systems nominal')
  .setColor(0x00ff00)
  .toJSON()

await ctx.reply({ embeds: [myEmbed] })
```

## Title and Description

```typescript
const infoEmbed = embed()
  .setTitle('Server information')
  .setDescription('Details about this server')
  .toJSON()
```

## Color

Set the accent color with a hex number or integer:

```typescript
embed().setColor(0x3366ff)
embed().setColor(3368703)
```

Common colors:

| Color           | Value      |
| --------------- | ---------- |
| Green           | `0x00ff00` |
| Red             | `0xff0000` |
| Blue            | `0x0099ff` |
| Orange          | `0xffaa00` |
| Purple          | `0x9900ff` |
| Discord blurple | `0x5865f2` |

## URL

Make the title clickable:

```typescript
const docs = embed().setTitle('Documentation').setUrl('https://flora.dev/docs')
```

## Timestamp

```typescript
const stamped = embed().setTitle('Report').setTimestamp(new Date().toISOString())
```

## Fields

Fields are good for compact structured data.

```typescript
const statsEmbed = embed()
  .setTitle('Server stats')
  .addField('Members', '1,234', true)
  .addField('Online', '567', true)
  .addField('Channels', '42', true)
  .toJSON()
```

The third parameter controls inline layout:

```typescript
embed()
  .addField('Field 1', 'Value 1', true)
  .addField('Field 2', 'Value 2', true)
  .addField('Field 3', 'Value 3', false)
```

:::tip
Discord can fit up to 3 inline fields per row. If a field is too wide, Discord wraps it for you.
:::

### Multiple Fields

```typescript
embed().addFields([
  { name: 'CPU', value: '45%', inline: true },
  { name: 'Memory', value: '2.1 GB', inline: true },
  { name: 'Uptime', value: '5 days', inline: true }
])
```

### Replace Fields

```typescript
const builder = embed().addField('Old', 'Data')

builder.setFields([{ name: 'New', value: 'Data' }])
```

## Images and Thumbnails

Use `setImage()` for a large image at the bottom of the embed:

```typescript
embed().setImage('https://example.com/banner.png')
```

Use `setThumbnail()` for a smaller image in the top-right:

```typescript
embed().setThumbnail('https://example.com/icon.png')
```

Example:

```typescript
const userEmbed = embed()
  .setTitle('User profile')
  .setThumbnail('https://cdn.discord.com/avatars/123/abc.png')
  .addField('Username', 'Alice')
  .addField('Joined', '2 years ago')
  .setImage('https://example.com/banner.png')
  .setColor(0x3366ff)
  .toJSON()
```

## Footer

```typescript
embed().setFooter('flora bot', 'https://example.com/logo.png')
embed().setFooter('Page 1 of 5')
```

## Author

```typescript
embed().setAuthor('Alice', {
  url: 'https://example.com/alice',
  iconUrl: 'https://example.com/alice.png'
})

embed().setAuthor('System')
```

## Complete Example

```typescript
const statusEmbed = embed()
  .setTitle('Bot status')
  .setDescription('All systems operational')
  .setColor(0x00ff00)
  .setThumbnail('https://example.com/icon.png')
  .addField('Uptime', '99.9%', true)
  .addField('Latency', '42ms', true)
  .addField('Version', '1.0.0', true)
  .setFooter('flora runtime')
  .setTimestamp(new Date().toISOString())
  .toJSON()

await ctx.reply({ embeds: [statusEmbed] })
```

## Multiple Embeds

Discord allows up to 10 embeds in one message.

```typescript
const embed1 = embed().setTitle('First').setColor(0xff0000).toJSON()
const embed2 = embed().setTitle('Second').setColor(0x00ff00).toJSON()
const embed3 = embed().setTitle('Third').setColor(0x0000ff).toJSON()

await ctx.reply({
  embeds: [embed1, embed2, embed3]
})
```

## Reusing Embeds

You can initialize a builder with existing embed data:

```typescript
const template = {
  title: 'Template',
  color: 0x3366ff,
  footer: { text: 'flora' }
}

const customEmbed = embed(template).setDescription('Custom description').toJSON()
```

## Type Definitions

```typescript
export class EmbedBuilder {
  setTitle(title: string): this
  setDescription(description: string): this
  setUrl(url: string): this
  setColor(color: number): this
  setTimestamp(timestamp: string): this
  setFooter(text: string, iconUrl?: string): this
  setImage(url: string): this
  setThumbnail(url: string): this
  setAuthor(name?: string, options?: { url?: string; iconUrl?: string }): this
  addField(name: string, value: string, inline?: boolean): this
  addFields(fields: EmbedField[]): this
  setFields(fields: EmbedField[]): this
  toJSON(): Embed
}

export type EmbedField = {
  name: string
  value: string
  inline?: boolean
}

export function embed(initial?: Embed): EmbedBuilder
```

## Limits

| Field       | Limit           |
| ----------- | --------------- |
| Title       | 256 characters  |
| Description | 4096 characters |
| Fields      | 25 fields       |
| Field name  | 256 characters  |
| Field value | 1024 characters |
| Footer text | 2048 characters |
| Author name | 256 characters  |
| Total text  | 6000 characters |

## Tips

- Use colors meaningfully, like green for success and red for errors.
- Keep descriptions short and put structured data in fields.
- Add timestamps when the data is time-sensitive.
- Use inline fields for compact stats.
