---
outline: deep
---

# Runtime

Flora Runtime is the Rust service that hosts Discord connectivity, V8 isolates,
and the HTTP API that the CLI uses. It bridges Discord events into a single
JavaScript runtime per guild and exposes Discord operations to scripts via ops.

## Architecture

Core components:

- Discord client (Serenity) for gateway events and REST calls
- Bot runtime for isolate lifecycle and event dispatch
- Deno Core ops for Discord actions, logging, KV, and command registration
- HTTP API for deployments, logs, KV, and token auth
- Storage: Postgres (deployments, tokens), Redis (sessions/cache), Sled (KV)

## Boot flow

At startup the runtime:

1. Loads config and initializes tracing.
2. Connects to Postgres and runs migrations.
3. Connects to Redis for cache/session storage.
4. Initializes V8 once per process.
5. Loads the SDK bundle into the runtime.
6. Restores cached deployments and starts the Discord client + HTTP server.

## Event flow

1. Discord gateway event arrives.
2. Event payload is serialized and routed to the guild isolate.
3. Handlers registered via `on()` execute.
4. Calls to `ctx.reply`, `ctx.edit`, `registerSlashCommands`, and `kv.*` map to
   runtime ops and back to Discord or storage.

## KV store details

KV is scoped per guild and per store name. Stores are backed by Sled on disk
and indexed in Postgres. Runtime constraints:

- Value size max: 1 MB
- Key length max: 512 characters
- Store name max: 64 characters
- List default limit: 100, max 1000
- Optional metadata and expiration per key
- Prefix filtering and cursor-based pagination

## Configuration and services

Required:

- Discord bot token and OAuth client credentials
- Postgres for deployments and tokens
- Redis for cache and auth sessions

Optional:

- KV storage is always available, backed by local disk under `data/kv`

## Logs and metrics

The runtime exposes logs via the HTTP API and supports streaming logs via SSE,
which the CLI can follow in real time.

