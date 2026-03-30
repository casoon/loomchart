#!/bin/bash
set -e

echo "=== Testing Rust Chart Engine (chartcore) ==="
echo ""

cd /Users/jseidel/GitHub/loom

echo "1. Testing Rust build..."
cargo build -p chartcore --features wasm
echo "  ✓ Rust build successful"
echo ""

echo "2. Checking WASM files..."
if [ -f "apps/frontend/public/wasm-chartcore/chartcore_bg.wasm" ]; then
    echo "  ✓ chartcore_bg.wasm exists"
    ls -lh apps/frontend/public/wasm-chartcore/chartcore_bg.wasm
else
    echo "  ✗ WASM file missing!"
    exit 1
fi
echo ""

echo "3. Checking TypeScript definitions..."
if [ -f "apps/frontend/public/wasm-chartcore/chartcore.d.ts" ]; then
    echo "  ✓ chartcore.d.ts exists"
else
    echo "  ✗ TypeScript definitions missing!"
    exit 1
fi
echo ""

echo "4. Checking frontend wrapper..."
if [ -f "apps/frontend/src/lib/rust-chart.ts" ]; then
    echo "  ✓ rust-chart.ts exists"
else
    echo "  ✗ Frontend wrapper missing!"
    exit 1
fi
echo ""

echo "=== ALL TESTS PASSED! ==="
echo ""
echo "Chart Engine is ready!"
echo ""
echo "To start the frontend:"
echo "  cd apps/frontend"
echo "  pnpm dev"
echo ""
echo "Then open http://localhost:4321 and click 'Generate Test Data'"
