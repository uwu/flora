---
outline: deep
---

# Limitations

This page documents known limitations and design trade-offs in the Flora runtime.

## Cron Jobs

### No Persistence

Cron jobs are **ephemeral** — they exist only in memory for the lifetime of the worker process. The runtime does not persist cron job state to a database.

**What this means:**

| State             | Persisted? | Behavior on restart               |
| ----------------- | ---------- | --------------------------------- |
| Job definitions   | No         | Re-registered when scripts reload |
| `next_run` time   | No         | Recalculated from current time    |
| `is_running` flag | No         | Reset to `false`                  |
| Execution history | No         | No audit trail                    |

### Why This Is Usually Fine

1. **Scripts reload on startup.** When the runtime starts, it loads all deployments from the database and executes them. Your `cron()` calls run again, re-registering all jobs.

2. **Schedules resume, not catch up.** After a restart, cron jobs calculate their next run from "now" rather than trying to execute missed runs. For most Discord bot use cases (reminders, periodic cleanups, status updates), this is the expected behavior.

3. **Redeployment clears old jobs.** When you deploy a new version of your script, the runtime clears all cron jobs for that guild before loading the new script. This prevents stale jobs from lingering.

### Edge Cases to Be Aware Of

**Duplicate execution on crash:** If the runtime crashes while a cron job is executing, the `is_running` flag is lost. On restart, the job may run again if it's due. Use `skipIfRunning: true` and design handlers to be idempotent where possible.

```ts
cron('daily-cleanup', '0 0 * * *', async () => {
  // This handler should be safe to run twice
  await cleanupOldMessages()
}, { skipIfRunning: true })
```

**Schedule drift:** If the runtime is down for an extended period, jobs won't "catch up" on missed executions. A job scheduled for midnight won't run at 2 AM if the bot was down at midnight — it will wait for the next midnight.

### When You Need More

If your use case requires:

- **Exactly-once execution** with audit logs
- **Catch-up runs** after downtime
- **Non-idempotent side effects** (billing, one-time notifications)

Consider implementing your own persistence layer using the [KV store](/sdk#kv-store) to track execution state:

```ts
cron('critical-job', '0 * * * *', async () => {
  const lastRun = await kv.get<string>('critical-job:last-run')
  const now = new Date().toISOString()

  // Check if we already ran this hour
  if (lastRun && isSameHour(lastRun, now)) {
    return
  }

  await performCriticalWork()
  await kv.set('critical-job:last-run', now)
})
```

## Guild-Only Scope

Flora is designed for **guild contexts only**. Direct messages and global commands are not supported. All events without a `guild_id` are dropped by the runtime.

## Isolate Limits

Each guild runs in its own V8 isolate with enforced limits:

| Resource                | Default Limit |
| ----------------------- | ------------- |
| Script size             | 1 MB          |
| Dispatch timeout        | 30 seconds    |
| Cron timeout            | 5 seconds     |
| Boot timeout            | 10 seconds    |
| Max cron jobs per guild | 32            |

These can be adjusted via runtime configuration but are not exposed to end users.
