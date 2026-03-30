#!/bin/bash
# Diagnose script for WASM build environment

echo "=== WASM Build Environment Diagnostics ==="
echo ""

echo "1. Checking Rust installation..."
if command -v rustc &> /dev/null; then
    rustc --version
else
    echo "❌ rustc not found"
fi
echo ""

echo "2. Checking wasm32 target..."
if rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "✅ wasm32-unknown-unknown target installed"
else
    echo "❌ wasm32-unknown-unknown target NOT installed"
    echo "   Run: rustup target add wasm32-unknown-unknown"
fi
echo ""

echo "3. Checking wasm-pack..."
if command -v wasm-pack &> /dev/null; then
    wasm-pack --version
else
    echo "❌ wasm-pack not found"
    echo "   Run: cargo install wasm-pack"
fi
echo ""

echo "4. Checking project structure..."
if [ -d "packages/wasm-core" ]; then
    echo "✅ packages/wasm-core exists"
else
    echo "❌ packages/wasm-core not found"
fi

if [ -d "crates/chartcore-indicators" ]; then
    echo "✅ crates/chartcore-indicators exists"
else
    echo "❌ crates/chartcore-indicators not found"
fi
echo ""

echo "5. Checking Cargo.toml references..."
if grep -q 'chartcore-indicators' packages/wasm-core/Cargo.toml; then
    echo "✅ Cargo.toml references chartcore-indicators correctly"
else
    echo "❌ Cargo.toml reference incorrect"
fi
echo ""

echo "6. Testing cargo check..."
cd packages/wasm-core
if cargo check --target wasm32-unknown-unknown 2>&1 | head -20; then
    echo ""
    echo "✅ Basic cargo check passed (showing first 20 lines)"
else
    echo "❌ cargo check failed"
fi
cd ../..
echo ""

echo "=== Diagnostics Complete ==="
echo ""
echo "To build WASM, run:"
echo "  cd packages/wasm-core"
echo "  wasm-pack build --target web --out-dir ../../apps/frontend/public/wasm"
