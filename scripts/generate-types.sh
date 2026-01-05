#!/bin/bash
# Generate TypeScript types from Rust structs using ts-rs
#
# This script runs cargo test to trigger ts-rs type generation,
# then copies the generated files to sdk/src/generated/
#
# Usage: ./scripts/generate-types.sh

set -e

echo "Generating TypeScript types from Rust..."

# Run cargo test to trigger ts-rs export
cargo test --quiet

# ts-rs exports to bindings/ by default, copy to sdk/
if [ -d "bindings/sdk/src/generated" ]; then
  echo "Copying generated types to sdk/src/generated/..."
  cp -r bindings/sdk/src/generated/* sdk/src/generated/
fi

# Copy serde_json types if generated
if [ -d "bindings/serde_json" ]; then
  mkdir -p sdk/src/generated/serde_json
  cp -r bindings/serde_json/* sdk/src/generated/serde_json/
fi

# Fix import paths in generated files (ts-rs uses relative paths from bindings/)
echo "Fixing import paths..."
sed -i 's|"../../../serde_json/JsonValue"|"./serde_json/JsonValue"|g' sdk/src/generated/*.ts 2>/dev/null || true

echo "TypeScript types generated successfully!"
echo ""
echo "Generated files:"
ls -la sdk/src/generated/
