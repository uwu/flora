# Rust Third-Party Snapshot

This directory is updated by:

- `tools/buck/sync_rust_deps.sh` for Cargo lock/metadata snapshots.
- `tools/buck/buckify_rust_deps.sh` for Reindeer vendor/buckify output (`vendor/` + generated `BUCK2`).

Current Buck2 setup in this repo uses Cargo-driven build actions. The files here are a deterministic
snapshot of `Cargo.lock` and `cargo metadata` used for dependency auditing and future Buck-native
dependency generation.

Regenerate:

```bash
tools/buck/sync_rust_deps.sh
```

Buckify third-party Rust deps with Reindeer:

```bash
./x buckify-rust-deps
```

Reindeer config lives at `third-party/reindeer.toml`.
