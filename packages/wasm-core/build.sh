#!/bin/bash
set -e

echo "Building WASM module..."
echo "Working directory: $(pwd)"

# Run wasm-pack build
wasm-pack build --target web --out-dir ../../apps/frontend/public/wasm

echo "Build complete!"
