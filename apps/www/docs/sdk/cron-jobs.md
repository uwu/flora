---
title: 'Cron Jobs'
description: 'Schedule recurring tasks with cron expressions'
outline: deep
---

# Cron Jobs

flora can run scheduled work with `cron()`. Use it for cleanup, reminders, periodic reports, and other small recurring tasks. All schedules are evaluated in UTC.

## Basic Usage

```typescript
cron('heartbeat', '* * * * *', (ctx) => {
  console.log(`heartbeat at ${ctx.scheduledAt}`)
})
```

## Cron Expression Format

flora uses standard 5-field cron expressions:

```text
minute hour day-of-month month day-of-week
```

| Field          | Values                          |
| -------------- | ------------------------------- |
| `minute`       | `0-59`                          |
| `hour`         | `0-23` in UTC                   |
| `day-of-month` | `1-31`                          |
| `month`        | `1-12`                          |
| `day-of-week`  | `0-7`, where 0 and 7 are Sunday |

Special characters:

| Character | Meaning         | Example                                           |
| --------- | --------------- | ------------------------------------------------- |
| `*`       | Any value       | `* * * * *` runs every minute.                    |
| `,`       | List of values  | `0,30 * * * *` runs at minute 0 and 30.           |
| `-`       | Range of values | `0 9-17 * * *` runs hourly from 9 AM to 5 PM UTC. |
| `/`       | Step values     | `*/5 * * * *` runs every 5 minutes.               |

## Common Patterns

Every minute:

```typescript
cron('tick', '* * * * *', (ctx) => {
  console.log('tick')
})
```

Every 5 minutes:

```typescript
cron('frequent', '*/5 * * * *', (ctx) => {
  console.log('every 5 minutes')
})
```

Every hour:

```typescript
cron('hourly', '0 * * * *', (ctx) => {
  console.log('top of the hour')
})
```

Daily at midnight UTC:

```typescript
cron('daily-cleanup', '0 0 * * *', async (ctx) => {
  console.log(`running ${ctx.name}`)
})
```

Weekdays at 9 AM UTC:

```typescript
cron('weekday-reminder', '0 9 * * 1-5', (ctx) => {
  console.log('good morning')
})
```

First day of each month:

```typescript
cron('monthly-report', '0 0 1 * *', (ctx) => {
  console.log('monthly report')
})
```

## Cron Context

Each cron handler receives:

```typescript
type CronContext = {
  name: string
  scheduledAt: string
}
```

Example:

```typescript
cron('logger', '0 * * * *', (ctx) => {
  console.log(`job: ${ctx.name}`)
  console.log(`scheduled for: ${ctx.scheduledAt}`)
  console.log(`actual time: ${new Date().toISOString()}`)
})
```

## Options

### Skip If Running

Use `skipIfRunning` to avoid overlapping executions of the same job.

```typescript
cron(
  'long-task',
  '*/5 * * * *',
  async (ctx) => {
    await someLongRunningTask()
  },
  { skipIfRunning: true }
)
```

:::info
If the previous execution is still running when the next one is due, flora skips the new execution.
:::

## Complete Example

```typescript
cron('daily-cleanup', '0 0 * * *', async (ctx) => {
  const store = kv.store('cache')
  const result = await store.list({ prefix: 'temp:' })

  for (const key of result.keys) {
    await store.delete(key.name)
  }

  console.log(`cleaned up ${result.keys.length} temporary entries`)
})

cron('hourly-stats', '0 * * * *', async (ctx) => {
  const store = kv.store('stats')
  const count = await store.get('message_count')

  console.log(`messages this hour: ${count || 0}`)
  await store.set('message_count', '0')
})

cron('weekday-reminder', '0 9 * * 1-5', async (ctx) => {
  console.log('time for daily standup')
})

cron(
  'sync-data',
  '*/10 * * * *',
  async (ctx) => {
    await syncExternalData()
  },
  { skipIfRunning: true }
)
```

## Behavior Notes

### Ephemeral State

Cron jobs live in memory:

- Jobs are registered when your script loads.
- Jobs are cleared when you redeploy.
- Jobs are lost if the runtime restarts.

### Restart Behavior

After a restart:

1. Scripts reload automatically, so your `cron()` calls run again.
2. Schedules resume from the current time. Missed runs do not catch up.
3. Redeployment replaces old cron jobs for that guild.

### Crash Recovery

If the runtime crashes while a cron job is running:

- The running state is lost.
- On restart, the job may run again if it is due.
- Use `skipIfRunning: true` and design handlers to be idempotent.

:::tip
An idempotent handler can run more than once without causing problems. For example, setting a counter to `0` is idempotent, but incrementing a counter is not.
:::

## Limits

| Limit                   | Value     |
| ----------------------- | --------- |
| Max cron jobs per guild | 32        |
| Handler timeout         | 5 seconds |

:::info
These limits are configurable in the runtime, but your handlers should still stay fast and predictable.
:::

## Type Definitions

```typescript
export interface CronContext {
  name: string
  scheduledAt: string
}

export interface CronOptions {
  skipIfRunning?: boolean
}

export type CronHandler = (ctx: CronContext) => void | Promise<void>

declare function cron(
  name: string,
  cronExpr: string,
  handler: CronHandler,
  options?: CronOptions
): void
```

## More Examples

### Cleanup Old Data

```typescript
cron('cleanup-old-logs', '0 2 * * *', async (ctx) => {
  const store = kv.store('logs')
  const cutoff = Date.now() - 7 * 24 * 60 * 60 * 1000
  const result = await store.list({ prefix: 'log:' })

  for (const key of result.keys) {
    const entry = await store.getWithMetadata(key.name)

    if (entry.metadata?.timestamp < cutoff) {
      await store.delete(key.name)
    }
  }
})
```

### Periodic Status Updates

```typescript
cron('status-update', '0 */6 * * *', async (ctx) => {
  const store = kv.store('metrics')
  const uptime = await store.get('uptime')

  console.log(`status update: uptime ${uptime}h`)
})
```

### Rate Limit Reset

```typescript
cron('reset-limits', '0 0 * * *', async (ctx) => {
  const store = kv.store('rate_limits')
  const result = await store.list({ prefix: 'limit:' })

  for (const key of result.keys) {
    await store.set(key.name, '0')
  }

  console.log('rate limits reset')
})
```

## Tips

- Name jobs clearly, like `daily-backup` instead of `job1`.
- Test cron expressions before deploying.
- Keep handlers inside the timeout.
- Make handlers idempotent when they update state.
- Log when jobs run so you can debug schedules later.
