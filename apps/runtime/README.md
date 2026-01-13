# Flora Runtime

The core Rust application powering Flora's Discord bot engine with a multi-threaded V8 JavaScript runtime.

## Architecture

A single-process application that bridges Discord events to isolated JavaScript runtimes:

- **Discord client integration** (Serenity) receives gateway events and serializes them for JavaScript
- **Multi-threaded worker pool** manages per-guild V8 isolates for isolation and performance
- **Deno Core extensions** expose Discord operations (messages, interactions, KV storage) to JavaScript code
- **HTTP API layer** (Axum + utoipa) provides REST endpoints with OpenAPI documentation
- **Authentication** supports Discord OAuth session management and long-lived API tokens for CLI auth
- **Persistence** spans PostgreSQL (deployments, tokens), Redis (sessions, cache), and Sled (embedded KV stores)

## Key Components

- `runtime.rs` — Worker pool architecture managing isolates and event dispatch
- `discord_handler.rs` — Event serialization and routing to JavaScript handlers
- `ops/` — Deno Core extensions exposing Discord operations (messages, interactions, KV, TLS)
- `handlers/` — REST API endpoints grouped by feature (auth, deployments, KV, metrics, logs)
- `bundler.rs` — Custom TypeScript bundler using oxc with source map generation
- `auth.rs` — Discord OAuth flows, HMAC-signed sessions, token refresh via Redis
- `deployments.rs` — Guild script storage in Postgres with Redis caching
- `kv.rs` — Per-guild key-value store service backed by Sled
- `tokens.rs` — API token management for CLI authentication (SHA256-hashed)
- `metrics.rs` — Prometheus-style metrics for isolate health and dispatch latency
- `log_sink.rs` — Circular log buffer with SSE streaming support
