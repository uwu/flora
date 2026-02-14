# Buck2 (Rust/V8)

This repository includes a Buck2 setup for Rust builds of:

- `apps/runtime` (`flora` lib + bin)
- `apps/cli` (`flora-cli` bin)
- `crates/flora_config`
- `crates/flora_macros`
- `crates/flora_typegen`

## Prerequisites

- `buck2`
- Rust toolchain (`cargo`, `rustc`)
- For V8 source-mode builds: `gn`, `ninja`, `python3`, `clang/libclang` (for bindgen)

## Dependency Snapshot Sync

Update Cargo metadata snapshot used by Buck2 tooling:

```bash
buck2 run //tools/buck:sync_rust_deps
# or directly:
tools/buck/sync_rust_deps.sh
```

## Build Targets

```bash
# runtime (default V8 mode)
buck2 build //apps/runtime:flora_bin

# runtime library
buck2 build //apps/runtime:flora_lib

# runtime with source V8 build path
buck2 build //apps/runtime:flora_bin_v8_source

# cli
buck2 build //apps/cli:flora_cli

# support crates
buck2 build //crates/flora_config:flora_config
buck2 build //crates/flora_macros:flora_macros
buck2 build //crates/flora_typegen:flora_typegen

# aggregate
buck2 build //:build_all_rust
```

## Check/Test Targets

```bash
buck2 build //:check_rust
buck2 build //:test_rust
```

## V8 Source Mode

Use `//apps/runtime:flora_bin_v8_source` when changing `submodules/rusty_v8`.

Common env before building:

```bash
export V8_FROM_SOURCE=1
export LIBCLANG_PATH=/usr/lib/llvm-19/lib
export GN=/usr/bin/gn
export NINJA=/usr/bin/ninja
export PYTHON=python3
```

The `flora_bin_v8_source` target already forces `V8_FROM_SOURCE=1` for that build action.

## Notes

- Buck2 here is local/dev focused; CI integration is intentionally out of scope.
- Cargo remains source-of-truth for dependency resolution and lockfile state.
