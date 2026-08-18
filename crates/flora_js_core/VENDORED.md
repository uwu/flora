# Vendored Deno core

This directory contains the runtime portions of Deno's `deno_core` and
`deno_v8` crates copied from:

- Repository: <https://github.com/denoland/deno>
- Revision: `89f33cbef296a2b287f323d42de54c871fa69c77`
- Revision date: 2026-08-14

The upstream benches, examples, test suites, and testdata are intentionally
omitted. The library test, doctest, and bench targets are disabled in the
vendored manifests.

Flora carries a local shared-isolate patch so `deno_core::JsRuntime` can use
the mainline `v8::SharedIsolate` and `v8::Locker` APIs and migrate between
runtime worker threads.

## Flora patches

- `JsRuntime` can own either an `OwnedIsolate` or `SharedIsolate`.
  `RuntimeOptions::use_v8_locker`, `V8Guard`, guarded cleanup, and
  `is_idle_for_migration()` preserve Flora's cross-worker runtime migration.
- Flora enables `deno_core/unsafe_use_unprotected_platform`. Rusty V8 requires
  thread-isolated allocations to be disabled when an isolate can be created on
  one thread and used or disposed on another. Using the protected platform
  caused concurrent migration and isolate teardown to crash in V8's
  `JSDispatchTable` allocator.
- Rusty V8's public aligned context-data setter creates a private
  `ContextAnnex` containing a `v8::Weak`, and live weak handles prevent
  `OwnedIsolate::try_into_shared()`. The vendored core therefore calls V8's
  aligned-pointer C ABI directly, initializes rusty_v8's annex slot to null,
  and maps Deno's logical context slots after rusty_v8's two internal slots.
  This depends on the `v8 152.1.0` context-slot layout and must be re-audited
  whenever rusty_v8 changes.
- Runtime bootstrap loading follows current Deno core's lazy
  `core.loadExtScript(...)` behavior. The obsolete `--no-validate-asm` V8 flag
  was removed during the upgrade.

The local Cargo manifests retain the upstream package names and versions so
the workspace's `[patch.crates-io]` entries replace only `deno_core` and
`deno_v8`; mainline `v8` continues to come from crates.io.
