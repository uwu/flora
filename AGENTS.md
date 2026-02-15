# Repository Guidelines

This guide is for AI agents working in the flora codebase.

## Quick Reference

- Project name: `flora` (lowercase, do not correct to `Flora`)
- Scope: **guild-only** (no DMs, global commands, or non-guild contexts)
- Be extremely concise in conversations; sacrifice grammar for the sake of concision.
- At the end of each plan, give me a list of unresolved questions to answer, if any. Make the questions extremely concise.
- Prefer runtime workflows via `./x` commands.

## Runtime Commands (`./x`)

Use these from repo root:

```bash
# Build runtime (dev) via Cargo
./x build-dev

# Build runtime (release) via Buck2 target //apps/runtime:flora_bin_release
./x build-release

# Run runtime (dev)
./x run-dev

# Run runtime (release): build + execute release binary
./x run-release
```

Notes:

- `./x run-release --help` still performs release build first, then forwards args.
- `./x` normalizes `BINDGEN_EXTRA_CLANG_ARGS` for nix clang wrapper environments.
- `./x run-*` warns if required runtime env vars are missing.

## Build, Test & Lint Commands

### Rust

```bash
# Build entire workspace
cargo build

# Build runtime (dev) [preferred wrapper]
./x build-dev

# Run runtime (dev) [preferred wrapper]
./x run-dev

# Build runtime (release) [preferred wrapper]
./x build-release

# Run runtime (release) [preferred wrapper]
./x run-release

# Build CLI tool
cargo run --package flora-cli

# Run all tests
cargo test

# Run single test (by name)
cargo test test_name

# Run tests in specific package
cargo test --package flora
cargo test --package flora_config

# Run tests in specific file/module
cargo test --test integration_test_name

# Lint (must pass with zero warnings)
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# Type checking only (no build)
cargo check
```

### Buck2 vs Cargo

- Buck2 entrypoints are in `BUCK2` and package `BUCK2` files.
- Buck release runtime target is `//apps/runtime:flora_bin_release`.
- Buck actions currently invoke Cargo through `tools/buck/cargo_action.py`.
- Cargo remains the Rust build backend and owns dependency resolution/lockfile.
- Artifacts/caches:
  - Cargo: `target/`
  - Buck2: `buck-out/v2/`

### TypeScript/SDK

```bash
# Install dependencies (from root)
pnpm install

# Build SDK
pnpm --filter sdk run build

# Run SDK tests
pnpm test
# or specifically in SDK directory:
pnpm --filter sdk test

# Run single test file
pnpm --filter sdk test src/sdk/embed.test.ts

# Format TypeScript/JSON/YAML/TOML/Markdown
dprint fmt

# Typecheck SDK
pnpm --filter sdk run typecheck
```

### Development Environment

```bash
# Start PostgreSQL (port 5433) and Redis/Valkey (port 5434)
./dev.sh

# Environment variables printed by dev.sh:
# DATABASE_URL=postgres://user:pass@localhost:5433/flora
# REDIS_URL=redis://localhost:5434
```

## Configuration Files

### Clippy (`.cargo/clippy.toml`)

- Pedantic lints enabled
- Allows: `module_name_repetitions`, `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`, `similar_names`, `too_many_lines`, `struct_excessive_bools`
- Denies: `inefficient_to_string`, `unnecessary_collect`, `large_enum_variant`
- Tests may use `unwrap()` and `expect()`

### Rustfmt (`.cargo/rustfmt.toml`)

- `max_width=140`, `chain_width=100`, `fn_call_width=100`
- `imports_granularity=Crate`
- `group_imports=StdExternalCrate` (std → external → crate imports)
- `wrap_comments=true`, `comment_width=100`
- `format_strings=true`, `format_macro_matchers=true`, `format_macro_bodies=true`
- `reorder_impl_items=true`, `use_field_init_shorthand=true`

### dprint (`.dprint.json`)

- TypeScript: `quoteStyle=preferSingle`, `semiColons=asi`, `trailingCommas=never`, `lineWidth=100`
- JSON: `indentWidth=2`
- YAML: `quotes=preferSingle`

## Rust Code Style

### Imports

- Edition 2024
- Group imports: std → external crates → internal crates
- Use `imports_granularity=Crate` (merge imports from same crate)
- Common patterns:

  ```rust
  use std::{cell::RefCell, rc::Rc, sync::Arc};

  use deno_core::{OpState, op2};
  use serenity::all::CommandOptionType;
  use tracing::{error, info, warn};

  use super::components::parse_components;
  use crate::state::AppState;
  ```

### Formatting & Structure

- 4-space indentation
- Max line width: 140 characters
- Use early returns for error cases
- Prefer `?` operator over manual error handling
- Doc comments (`///`) for public APIs; explain module purpose at top with `//!`

### Naming Conventions

- Types: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Lifetimes: descriptive names, not single chars

### Error Handling

- Use `tracing` macros (`error!`, `warn!`, `info!`, `debug!`, `trace!`) for logging
- Prefer `eyre::Result` for application errors (with context)
- Use `JsErrorBox` for errors crossing JS/Rust boundary
- Return errors early; avoid deep nesting
- Example:

  ```rust
  use eyre::{Context, Result};
  use tracing::error;

  fn process() -> Result<()> {
      let data = load_data()
          .context("failed to load data")?;

      if !data.is_valid() {
          error!("invalid data");
          return Err(eyre!("data validation failed"));
      }

      Ok(())
  }
  ```

### Testing

- Unit tests: inline with `#[cfg(test)]` modules
- Integration tests: in `tests/` directory
- Cover event dispatching and op error paths
- Tests may use `unwrap()` and `expect()`
- Use descriptive test names: `test_<what>_<scenario>`

## TypeScript/SDK Code Style

### Module System

- ESM modules with named exports
- No imports needed (global namespace SDK)
- File names: `lower_snake_case.ts`

### Formatting

- Single quotes preferred
- ASI (Automatic Semicolon Insertion) - no semicolons
- No trailing commas
- 100 character line width

### Command Definitions

```typescript
// Prefix commands
const myCmd = prefix({
  name: 'ping',
  description: 'Pong!',
  handler: async (msg, args) => {
    await msg.reply('Pong!')
  }
})

// Slash commands
const mySlash = slash({
  name: 'greet',
  description: 'Say hello',
  handler: async (interaction) => {
    await interaction.reply({ content: 'Hello!' })
  }
})

// Bot creation
createBot({
  prefix: '!',
  commands: [myCmd],
  slashCommands: [mySlash]
})
```

### Testing

- Use Vitest framework: `import { describe, expect, it } from 'vitest'`
- Test files: `*.test.ts`
- Focus on SDK API surface and builder patterns

## Type Generation

- Rust structs with `#[expose_input]` macro (from `flora_macros`) auto-generate TypeScript types
- Types exported to `sdk/src/generated.ts` via `cargo run -p flora_typegen`
- Run `cargo test --package flora_config` to regenerate config template

## Deno Core / Ops Patterns

- Use `#[op2(async)]` for async operations
- Extract state via `Rc<RefCell<OpState>>`
- Input structs use `#[expose_input]` (auto-derives `serde::Deserialize`, `ts_rs::TS`, camelCase rename)
- Extensions configured via `deno_core::extension!` macro
- Example:

  ```rust
  #[expose_input]
  pub struct SendMessageInput {
      pub channel_id: String,
      pub content: String,
  }

  #[op2(async)]
  pub async fn op_send_message(
      state: Rc<RefCell<OpState>>,
      #[serde] input: SendMessageInput,
  ) -> Result<String, JsErrorBox> {
      // ...
  }
  ```

## Runtime Configuration

- Modify `crates/flora_config` for runtime options
- Uses `confique` crate with `#[config]` derive
- Environment variables override `config.toml` values, which override defaults
- Config files `.env`, `testbotenv` must NOT be committed

## Git Workflow

### Commits

- Use Conventional Commits: `feat:`, `fix:`, `chore:`, `refactor:`, `test:`, `docs:`
- Keep scope small and focused
- Use imperative mood: "add feature" not "added feature"
- Examples:
  - `feat: add js dispatch logging`
  - `fix: handle null guild_id in logs`
  - `refactor: extract auth middleware`

### Pull Requests

- Include brief summary of changes
- Link related issues
- Note manual verification: commands run, env vars used
- If behavior changes Discord interactions, include expected output or logs

## Common Patterns

### State Management

- `AppState` (Arc) shared across handlers
- `OpState` (Rc<RefCell>) in Deno ops
- Use `Arc` for thread-safe sharing, `Rc` for single-threaded isolate

### Serenity Discord Patterns

- Builders: `CreateCommand`, `CreateInteractionResponse`, `CreateEmbed`
- IDs: strongly-typed snowflakes (`GuildId`, `ChannelId`, `UserId`)
- HTTP client: `Arc<Http>` passed to ops

### Database & Storage

- PostgreSQL via `sqlx` with compile-time checked queries
- Redis/Valkey via `fred` for sessions
- Sled for per-guild key-value storage

## Pre-commit Checklist

Before committing or submitting PR:

1. `cargo fmt` (format Rust)
2. `dprint fmt` (format TS/JSON/etc)
3. `cargo clippy --all-targets -- -D warnings` (lint with zero warnings)
4. `cargo test` (all tests pass)
5. `pnpm test` (SDK tests pass, if applicable)
6. Verify no secrets in changed files (`.env`, credentials)
