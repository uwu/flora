# Rust Third-Party Snapshot

This directory is updated by `tools/buck/sync_rust_deps.sh`.

Current Buck2 setup in this repo uses Cargo-driven build actions. The files here are a deterministic
snapshot of `Cargo.lock` and `cargo metadata` used for dependency auditing and future Buck-native
dependency generation.

Regenerate:

```bash
tools/buck/sync_rust_deps.sh
```
