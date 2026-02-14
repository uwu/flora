#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/third-party/rust"

mkdir -p "$OUTPUT_DIR"

cargo metadata --manifest-path "$ROOT_DIR/Cargo.toml" --format-version 1 > "$OUTPUT_DIR/cargo-metadata.json"
cp "$ROOT_DIR/Cargo.lock" "$OUTPUT_DIR/Cargo.lock.snapshot"

cat > "$OUTPUT_DIR/BUCK2" <<'BUCKEOF'
filegroup(
    name = "cargo_lock",
    srcs = ["Cargo.lock.snapshot"],
)

filegroup(
    name = "cargo_metadata",
    srcs = ["cargo-metadata.json"],
)
BUCKEOF

echo "Updated $OUTPUT_DIR/cargo-metadata.json and Cargo.lock.snapshot"
