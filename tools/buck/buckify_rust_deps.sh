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
THIRD_PARTY_DIR="$ROOT_DIR/third-party/rust"
REINDEER_CONFIG="$ROOT_DIR/third-party/reindeer.toml"

if ! command -v reindeer >/dev/null 2>&1; then
  echo "reindeer not found in PATH" >&2
  echo "Install from source: cargo install --git https://github.com/facebookincubator/reindeer.git reindeer" >&2
  exit 1
fi

if [[ ! -f "$ROOT_DIR/Cargo.toml" ]]; then
  echo "missing $ROOT_DIR/Cargo.toml" >&2
  exit 1
fi

if [[ ! -f "$REINDEER_CONFIG" ]]; then
  echo "missing $REINDEER_CONFIG" >&2
  exit 1
fi

reindeer \
  -c "$REINDEER_CONFIG" \
  vendor
reindeer \
  -c "$REINDEER_CONFIG" \
  buckify

echo "Reindeer vendor + buckify completed in $THIRD_PARTY_DIR"
