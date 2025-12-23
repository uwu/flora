# flora

> **⚠️Work in progress:** This is an early, very alpha project; expect breaking changes and rough edges while the foundations solidify. Here be dragons!

A single-runtime Discord bot engine that lets server administrators to run TypeScript on a fast, isolated runtime.

## Prerequisites

- Rust toolchain (edition 2024)
- Bun (for SDK bundling)
- Optional: Postgres + Redis (dev script uses ports 5433/5434)

## Run the bot (local)

1. Create `.env` with at least `DISCORD_TOKEN=<your token>`. Optional overrides:
   - `DATABASE_URL` (default: `postgres://user:pass@localhost:5433/flora`)
   - `VALKEY_URL` (default: `redis://127.0.0.1:5434/0`)
   - `API_ADDR` (default: `0.0.0.0:3000`)
2. Start supporting services (optional): `./dev.sh` (Postgres/Redis).
3. Run: `cargo run`\
   Logging defaults are already set in `.envrc`: `RUST_LOG=flora=debug,flora::runtime=trace,serenity=info`.
   If you are not using direnv, export that before running.

## Developing

- Format/lint: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`.
- Tests: `cargo test`.
- Rebuild SDK bundle: `bun run sdk/build.ts` (run from repo root; dependencies via `bun install`).
