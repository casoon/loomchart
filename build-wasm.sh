#!/bin/bash
# Build WASM core module

set -e

echo "Building WASM core..."
cd packages/wasm-core
wasm-pack build --target web --out-dir ../../apps/frontend/public/wasm

echo "WASM build complete!"
echo "Output: apps/frontend/public/wasm/"
