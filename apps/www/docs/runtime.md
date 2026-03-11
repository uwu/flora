---
outline: deep
---

## Cron scheduler

The runtime includes a per-worker cron scheduler that fires every second to check for due jobs. Cron jobs registered via `cron()` in scripts are:

- Stored in a per-worker registry keyed by guild ID
- Evaluated using the `croner` crate (POSIX/Vixie-cron compatible)
- Dispatched as synthetic events (`__cron:<name>`) through the same dispatch path as Discord events
- Subject to their own timeout (`cron_timeout_secs`, default 5s)
- Limited per guild (`max_cron_jobs`, default 32)

Cron jobs are cleared automatically when a guild script is redeployed or unloaded.

...document others later
