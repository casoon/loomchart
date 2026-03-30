# CandleGenerator Test Guide

This document explains how to test the chartcore `CandleGenerator` in the Loom frontend.

## Quick Start

### 1. Build the WASM Module

```bash
# Make the build script executable
chmod +x build-wasm.sh

# Build WASM
./build-wasm.sh
```

Or manually:

```bash
cd packages/wasm-core
wasm-pack build --target web --out-dir ../../packages/chart/public/wasm
cd ../..
```

### 2. Test with the HTML Test Page

```bash
# Serve the test page (use any static server)
python3 -m http.server 8080

# Or with Node.js
npx serve .
```

Then open http://localhost:8080/test-generator.html in your browser.

### 3. Use the Generator

The test page provides controls for:

- **Market Type**: crypto, stock, forex, futures, commodities
- **Trend**: bullish_strong, bullish_mild, sideways, bearish_mild, bearish_strong
- **Volatility**: low, normal, high, extreme
- **Candle Count**: 10-2000 candles

Click "Generate & Load Candles" to create realistic test data.

## WASM API

The `load_test_data()` function is exported from the WASM module:

```javascript
// Import WASM module
import init, { load_test_data } from './wasm/trading_ui.js';
await init();

// Generate test data
load_test_data(
    "crypto",           // market_type
    "bullish_strong",   // trend
    "high",            // volatility
    1000               // count
);
```

### Market Types

- **crypto**: 24/7 trading, high volatility, no gaps
- **stock**: 9:30-16:00 EST, weekend gaps, holiday gaps
- **forex**: 24/5 trading, weekend gaps only
- **futures**: Nearly 24/5, minimal gaps
- **commodities**: Variable hours, commodity-specific patterns

### Trends

- **bullish_strong**: +0.2% per candle drift
- **bullish_mild**: +0.05% per candle drift
- **sideways**: No drift, mean-reverting
- **bearish_mild**: -0.05% per candle drift
- **bearish_strong**: -0.2% per candle drift

### Volatility Regimes

- **low**: 0.3% volatility
- **normal**: 0.8% volatility (default)
- **high**: 2.0% volatility
- **extreme**: 5.0% volatility

## Integration with Frontend

To integrate the generator into your Astro/Alpine.js frontend:

```javascript
// In your chart component
import { init, load_test_data } from '@loom/wasm-core';

// Initialize WASM
await init(JSON.stringify({ ws_url: "..." }));

// Load test data
function loadTestData() {
    load_test_data("crypto", "sideways", "normal", 500);
}
```

## Candle Format

Generated candles follow the WASM Candle format:

```typescript
interface Candle {
    source: string;      // "generator"
    symbol: string;      // "TEST_CRYPTO", "TEST_STOCK", etc.
    tf: string;          // "5m"
    ts: string;          // ISO 8601 timestamp
    o: number;           // Open price
    h: number;           // High price
    l: number;           // Low price
    c: number;           // Close price
    v: number;           // Volume
    is_final: boolean;   // Always true for generated data
}
```

## Features

The generator creates realistic market data with:

- **Geometric Brownian Motion**: Realistic price movements
- **Market Hours**: Trading hours and gaps based on market type
- **Weekend Gaps**: Automatic weekend detection and gap simulation
- **Volatility Clustering**: Periods of high/low volatility
- **Realistic Wicks**: Based on liquidity and volatility
- **Volume Correlation**: Higher volume during volatile periods
- **Trends**: Configurable directional bias
- **Reproducibility**: Seeded random number generation (seed=42)

## Troubleshooting

### WASM Module Not Found

Ensure the WASM module is built and in the correct location:

```bash
ls -la packages/chart/public/wasm/
```

You should see:
- `trading_ui.js`
- `trading_ui_bg.wasm`
- `trading_ui.d.ts`

### Build Errors

If you get compilation errors:

```bash
# Update Rust and wasm-pack
rustup update
cargo install wasm-pack

# Clean and rebuild
cd packages/wasm-core
cargo clean
wasm-pack build --target web --out-dir ../../packages/chart/public/wasm
```

### Browser Console Errors

Check the browser console (F12) for detailed error messages. Common issues:

- CORS errors: Serve from a proper HTTP server, not `file://`
- Module import errors: Check the import path matches your directory structure
- WASM initialization errors: Ensure `init()` completes before calling other functions

## Example Usage Scenarios

### Backtesting Indicators

```javascript
// Load historical-style data
load_test_data("stock", "bullish_mild", "normal", 2000);

// Enable indicators
toggle_indicator("ema", JSON.stringify({ length: 20 }), true);
toggle_indicator("rsi", JSON.stringify({ length: 14 }), true);
```

### Stress Testing Chart Performance

```javascript
// Maximum candles with high volatility
load_test_data("crypto", "sideways", "extreme", 2000);
```

### Simulating Market Conditions

```javascript
// Bear market crash
load_test_data("stock", "bearish_strong", "extreme", 1000);

// Forex ranging market
load_test_data("forex", "sideways", "low", 500);
```

## Next Steps

After testing the generator, you can:

1. Integrate it into your main frontend UI
2. Add controls for switching between live and test data
3. Use it for indicator backtesting
4. Create predefined scenarios (crash, rally, consolidation)
5. Add export functionality to save generated datasets
