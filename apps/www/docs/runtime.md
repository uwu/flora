---
outline: deep
---

# Runtime

## Trusted orchestrator

Flora can run one operator-owned JavaScript deployment as a trusted feature policy. Configure
`orchestrator.operator_user_id` (or `ORCHESTRATOR_OPERATOR_USER_ID`) with the Discord user ID that
may manage it through `/api/orchestrator/deployment`.

The deployment must register one authorization function:

```ts
defineOrchestrator({
  authorizeFeature({ feature, userId }) {
    return feature === 'custom_bots' && ['123456789012345678'].includes(userId)
  }
})
```

Deployments are replaced atomically. If the new script fails to load or does not register
`authorizeFeature`, the previous orchestrator remains active. Missing orchestrators deny feature
access by default.

The custom-bot feature asks the orchestrator with `feature: 'custom_bots'`. Redeploying the
orchestrator immediately reconciles existing User Bot gateways, so removing a user from the
policy stops their bot isolates and Discord clients.

Each account may own one User Bot. Creating another is rejected until the existing bot is
deleted. A guild may likewise configure one Server Custom Bot, and supplying a new token replaces
the previous identity.

Guild-owned bot identities use `feature: 'server_custom_bots'` and include the target guild in
metadata:

```ts
defineOrchestrator({
  authorizeFeature({ feature, userId, metadata }) {
    if (feature !== 'server_custom_bots') return false
    return metadata?.guildId === '123456789012345678' && userId === '234567890123456789'
  }
})
```

A guild administrator supplies a token for a bot application they own. Flora encrypts the token,
binds it to that guild, and runs the guild's existing deployment through that identity. DMs and
events or REST operations for other guilds are rejected. Central `@Flora` stops handling the guild
while the custom identity is active and resumes when the identity is removed or policy is revoked.

## Cron scheduler

The runtime includes a per-worker cron scheduler that fires every second to check for due jobs. Cron jobs registered via `cron()` in scripts are:

- Stored in a per-worker registry keyed by guild ID
- Evaluated using the `croner` crate (POSIX/Vixie-cron compatible)
- Dispatched as synthetic events (`__cron:<name>`) through the same dispatch path as Discord events
- Subject to their own timeout (`cron_timeout_secs`, default 5s)
- Limited per guild (`max_cron_jobs`, default 32)

Cron jobs are cleared automatically when a guild script is redeployed or unloaded.

...document others later
