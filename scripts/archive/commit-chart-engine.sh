#!/bin/bash
set -e

cd /Users/jseidel/GitHub/loom

echo "=== Committing Rust Chart Engine ==="

# Stage all changes
git add -A

# Commit
git commit -m "feat: Complete Rust/WASM chart engine implementation

✅ Core Features:
- Rust chart engine with viewport, rendering, and event handling
- WASM compilation with wasm-pack
- TypeScript wrapper for browser integration
- Canvas 2D renderer with proper scaling

✅ Fixed Issues:
- Timestamp conversion from milliseconds to seconds
- Timeframe propagation to viewport for correct bar width calculation
- Canvas size initialization on attach
- Double initialization preventing candles from rendering
- Render loop now runs continuously

✅ Working Features:
- Grid/raster rendering
- Crosshair/fadenkreuz display
- Candlestick chart with bullish/bearish colors
- API data loading (BTCUSDT 1h from Binance)
- Test data generation

⏳ Known Issues (TODO):
- ResizeObserver temporarily disabled (causes infinite loop)
- Crosshair offset due to pixel ratio not accounted in mouse events
- No axis labels (time/price scales) yet

Technical Details:
- Canvas buffer size: 3456x3456 (with pixel ratio 2.0)
- Canvas CSS size: 1728x1728
- Viewport correctly calculates visible_bars and bar_width
- Candles render at correct coordinates with proper colors"

# Push to remote
git push origin HEAD

echo "✓ Changes committed and pushed!"
