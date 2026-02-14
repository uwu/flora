#!/usr/bin/env bash
set -euo pipefail

resolve_root_dir() {
  if command -v git >/dev/null 2>&1; then
    local git_root
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
    if [[ -n "$git_root" && -f "$git_root/Cargo.toml" ]]; then
      echo "$git_root"
      return
    fi
  fi

  local script_root
  script_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  if [[ -f "$script_root/Cargo.toml" ]]; then
    echo "$script_root"
    return
  fi

  echo "failed to locate workspace root (Cargo.toml not found)" >&2
  exit 1
}

ROOT_DIR="${ROOT_DIR:-$(resolve_root_dir)}"
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
