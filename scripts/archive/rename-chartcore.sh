#!/bin/bash
set -e

echo "=== Renaming chartcore-indicators to chartcore ==="

cd /Users/jseidel/GitHub/loom

# Check if directories exist
if [ ! -d "crates/chartcore-indicators" ]; then
    echo "ERROR: crates/chartcore-indicators does not exist!"
    exit 1
fi

echo "Step 1: Removing empty crates/chartcore directory (if exists)..."
if [ -d "crates/chartcore" ]; then
    rm -rf crates/chartcore
    echo "  ✓ Removed crates/chartcore"
fi

echo "Step 2: Renaming crates/chartcore-indicators to crates/chartcore..."
mv crates/chartcore-indicators crates/chartcore
echo "  ✓ Renamed to crates/chartcore"

echo "Step 3: Verifying..."
if [ -d "crates/chartcore" ] && [ -f "crates/chartcore/Cargo.toml" ]; then
    echo "  ✓ crates/chartcore exists and has Cargo.toml"
else
    echo "  ✗ ERROR: Something went wrong!"
    exit 1
fi

if [ -d "crates/chartcore-indicators" ]; then
    echo "  ✗ WARNING: crates/chartcore-indicators still exists!"
else
    echo "  ✓ crates/chartcore-indicators removed"
fi

echo ""
echo "=== SUCCESS! ==="
echo "Directory renamed from crates/chartcore-indicators to crates/chartcore"
echo ""
echo "Next steps:"
echo "1. Test build: cargo build -p chartcore --features wasm"
echo "2. Build WASM: cd crates/chartcore && wasm-pack build --target web --out-dir ../../apps/frontend/public/wasm-chartcore --features wasm"
