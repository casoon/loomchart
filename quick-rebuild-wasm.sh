#!/bin/bash
set -e

echo "=== Quick WASM Rebuild ==="
cd /Users/jseidel/GitHub/loom/crates/chartcore
wasm-pack build --target web --out-dir ../../apps/frontend/src/wasm --features wasm
echo "✓ WASM rebuilt! Refresh your browser."
