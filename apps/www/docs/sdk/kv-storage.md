---
title: 'KV Storage'
description: 'Persist per-guild data with the flora KV API'
outline: deep
---

# KV Storage

flora includes a key-value storage API for bot state. Storage is scoped per guild, and you can organize data into named stores like `settings`, `counters`, or `cache`.

## Basic Usage

```typescript
const store = kv.store('settings')

await store.set('prefix', '!')
const prefixValue = await store.get('prefix')
```

`prefixValue` will be `'!'`, or `null` if the key is missing.

## Creating a Store

Stores are namespaces inside the current guild.

```typescript
const settings = kv.store('settings')
const counters = kv.store('counters')
const cache = kv.store('cache')
```

:::tip
Store names are just strings. Use names that describe the data inside the store.
:::

## Operations

### Get

```typescript
const value = await store.get('key')

if (value === null) {
  console.log('key not found')
} else {
  console.log('value', value)
}
```

### Set

```typescript
await store.set('key', 'value')
```

:::info
KV values are stored as strings. Each value can be up to 1 MB.
:::

### Set with Options

Use options for expiration and metadata.

```typescript
const expiresAt = Math.floor(Date.now() / 1000) + 3600

await store.set('session', JSON.stringify({ userId: '123' }), {
  expiration: expiresAt,
  metadata: { source: 'login', ip: '192.168.1.1' }
})
```

| Option       | Type     | What it does                                            |
| ------------ | -------- | ------------------------------------------------------- |
| `expiration` | `number` | Unix timestamp, in seconds, when the key should expire. |
| `metadata`   | `object` | JSON metadata attached to the key.                      |

### Get with Metadata

```typescript
const result = await store.getWithMetadata('session')

if (result.value === null) {
  console.log('not found')
} else {
  console.log('value', result.value)
  console.log('metadata', result.metadata)
}
```

### Update Metadata

Change metadata without changing the stored value.

```typescript
await store.updateMetadata('session', {
  lastAccessed: new Date().toISOString()
})
```

### Delete

```typescript
await store.delete('key')
```

### List Keys

`list()` returns a page of keys. Use the cursor when there is more to fetch.

```typescript
const page = await store.list({ limit: 100 })

console.log('keys', page.keys)
console.log('complete?', page.list_complete)

if (!page.list_complete) {
  const nextPage = await store.list({ cursor: page.cursor })
}
```

List options:

| Option   | Type     | What it does                                        |
| -------- | -------- | --------------------------------------------------- |
| `prefix` | `string` | Filters keys by prefix, like `user:`.               |
| `limit`  | `number` | Maximum keys per page. Default is 100, max is 1000. |
| `cursor` | `string` | Cursor from the previous page.                      |

## Patterns

### JSON Storage

```typescript
const store = kv.store('data')

const user = { id: '123', name: 'Alice', level: 5 }
await store.set('user:123', JSON.stringify(user))

const raw = await store.get('user:123')

if (raw) {
  const user = JSON.parse(raw)
  console.log(user.name)
}
```

### Counter

```typescript
const store = kv.store('counters')

async function increment(key: string): Promise<number> {
  const current = await store.get(key)
  const count = current ? parseInt(current, 10) : 0
  const newCount = count + 1

  await store.set(key, String(newCount))
  return newCount
}

const count = await increment('messages')
await ctx.reply(`Message count: ${count}`)
```

### Temporary Cache

```typescript
const cache = kv.store('cache')

async function cachedFetch(url: string): Promise<string> {
  const cached = await cache.get(url)
  if (cached) return cached

  const response = await fetch(url)
  const data = await response.text()
  const expiresAt = Math.floor(Date.now() / 1000) + 300

  await cache.set(url, data, { expiration: expiresAt })
  return data
}
```

### User Preferences

```typescript
const prefs = kv.store('preferences')

async function setPreference(userId: string, key: string, value: string) {
  const prefKey = `${userId}:${key}`
  await prefs.set(prefKey, value)
}

async function getPreference(userId: string, key: string): Promise<string | null> {
  const prefKey = `${userId}:${key}`
  return await prefs.get(prefKey)
}

await setPreference('123', 'timezone', 'America/New_York')
const timezone = await getPreference('123', 'timezone')
```

### List by Prefix

```typescript
const store = kv.store('users')

await store.set('user:123', JSON.stringify({ name: 'Alice' }))
await store.set('user:456', JSON.stringify({ name: 'Bob' }))
await store.set('admin:789', JSON.stringify({ name: 'Eve' }))

const result = await store.list({ prefix: 'user:' })
console.log(result.keys)
```

## Complete Example

```typescript
const counter = slash({
  name: 'counter',
  description: 'A simple counter using KV storage',
  subcommands: [
    {
      name: 'get',
      description: 'Get current count',
      run: async (ctx) => {
        const store = kv.store('counters')
        const count = await store.get('main')
        await ctx.reply(`Current count: ${count || 0}`)
      }
    },
    {
      name: 'increment',
      description: 'Increment the counter',
      run: async (ctx) => {
        const store = kv.store('counters')
        const current = parseInt((await store.get('main')) || '0', 10)
        const newCount = current + 1
        await store.set('main', String(newCount))
        await ctx.reply(`Count is now: ${newCount}`)
      }
    },
    {
      name: 'reset',
      description: 'Reset the counter',
      run: async (ctx) => {
        const store = kv.store('counters')
        await store.set('main', '0')
        await ctx.reply('Counter reset to 0')
      }
    }
  ]
})

createBot({ slashCommands: [counter] })
```

## Type Definitions

```typescript
export class KvStore {
  async get(key: string): Promise<string | null>

  async getWithMetadata(key: string): Promise<{
    value: string | null
    metadata?: Record<string, unknown>
  }>

  async set(
    key: string,
    value: string,
    options?: {
      expiration?: number
      metadata?: JsonValue
    }
  ): Promise<void>

  async updateMetadata(key: string, metadata: JsonValue | undefined): Promise<void>

  async delete(key: string): Promise<void>

  async list(options?: { prefix?: string; limit?: number; cursor?: string }): Promise<{
    keys: RawKvKeyInfo[]
    list_complete: boolean
    cursor?: string
  }>
}

export function store(name: string): KvStore

export const kv = { store }
```

## Limits

| Limit              | Value          |
| ------------------ | -------------- |
| Value size         | 1 MB per value |
| Keys per list call | 1000           |

:::info
There is no hard total storage limit per guild yet, but large datasets still need careful key design and pagination.
:::

## Tips

- Prefix keys by type, like `user:123`, `channel:456`, or `config:timezone`.
- Store objects with `JSON.stringify()` and read them with `JSON.parse()`.
- Use `expiration` for temporary cache entries and sessions.
- Always handle `null` when reading keys.
