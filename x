#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

normalize_bindgen_args() {
  if [[ -n "${BINDGEN_EXTRA_CLANG_ARGS:-}" ]]; then
    BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS//--resource-dir=/-resource-dir=}"
    export BINDGEN_EXTRA_CLANG_ARGS
  fi
}

build_runtime_release() {
  cargo build --release --package flora >/dev/null
  printf '%s\n' "$ROOT_DIR/target/release/flora"
}

build_frontend_production() {
  pnpm --filter frontend run build
}

usage() {
  cat <<'EOF'
usage: ./x <command>

commands:
  build-dev     build runtime in dev mode (cargo build --package flora)
  build-release build frontend + runtime release with cargo, print binary path
  run-dev       run runtime in dev mode (cargo run --package flora)
  run-release   build frontend + runtime release with cargo, then run it
  help     show this help
EOF
}

cmd="${1:-help}"
case "$cmd" in
  build-dev)
    normalize_bindgen_args
    exec cargo build --package flora
    ;;
  build-release)
    normalize_bindgen_args
    build_frontend_production
    build_runtime_release
    ;;
  run-dev)
    normalize_bindgen_args
    exec cargo run --package flora
    ;;
  run-release)
    shift || true
    normalize_bindgen_args
    build_frontend_production
    BIN_PATH="$(build_runtime_release)"
    exec "$BIN_PATH" "$@"
    ;;
  help | -h | --help)
    usage
    ;;
  *)
    echo "unknown command: $cmd" >&2
    echo >&2
    usage >&2
    exit 1
    ;;
esac
