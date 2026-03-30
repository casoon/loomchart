# chartcore

Lightweight trading chart engine in Rust - A modern alternative to TradingView Lightweight Charts.

Built from the ground up in Rust for performance, type safety, and portability.

## Features

- **Pure Rust Core**: All indicator calculations and plugin system logic in Rust
- **Plugin Architecture**: Extensible system for both built-in and external indicators
- **WebAssembly Support**: External developers can create plugins that compile to WASM
- **Testable**: Plugins can be tested in pure Rust without WASM compilation
- **Technical Analysis Library**: Comprehensive TA calculation primitives
- **Incremental Calculation**: O(1) updates for real-time data (e.g., EMA)
- **Type Safety**: Compile-time type checking

## Architecture

The crate is organized into five main modules:

### 1. Core (`core`)

Chart data structures and state management:
- **Candle**: OHLCV data with helper methods (typical_price, is_bullish, etc.)
- **Chart**: Main chart structure with data buffer
- **ChartBuffer**: Ring buffer for efficient candle storage (default 2000)
- **ChartConfig**: Chart configuration and settings
- **Point**: Time/price point on chart
- **Timeframe**: Enumeration of all supported timeframes (1s to 1M)
- **CandleGenerator**: Realistic market data simulator for backtesting
  - Multiple market types: Stock, Forex, Crypto, Futures, Commodities
  - Configurable trends: Bullish, Bearish, Sideways
  - Volatility regimes: Low, Normal, High, Extreme
  - Respects market hours and gaps (weekends, holidays)
  - Reproducible with seeds

### 2. Technical Analysis Library (`ta`)

Reusable calculation functions for technical indicators:

**Moving Averages:**
- SMA (Simple Moving Average)
- EMA (Exponential Moving Average)
- WMA (Weighted Moving Average)
- RMA (Wilder's Smoothed Moving Average)
- DEMA, TEMA, HMA, VWMA

**Momentum Indicators:**
- RSI (Relative Strength Index)
- Stochastic %K
- CCI (Commodity Channel Index)
- Williams %R

**Statistical Functions:**
- Standard Deviation
- Highest/Lowest
- Sum, Change, ROC
- Correlation, Linear Regression

### 3. Indicator System (`indicators`)

Core plugin trait and infrastructure:

```rust
pub trait IndicatorPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn category(&self) -> IndicatorCategory;
    fn inputs(&self) -> Vec<InputConfig>;
    fn plots(&self) -> Vec<PlotConfig>;
    fn calculate(&self, context: &CalculationContext) -> IndicatorResult;
    
    // Optional: for real-time optimization
    fn calculate_incremental(
        &self,
        context: &CalculationContext,
        new_candle: &Candle,
        previous_result: &IndicatorResult,
    ) -> Option<IndicatorResult>;
}
```

**Built-in Plugins:**
- RSI (Relative Strength Index)
- EMA (Exponential Moving Average)
- SMA (Simple Moving Average)

### 4. Primitives (`primitives`)

Drawing primitives for visualization:
- **Color**: RGBA color with predefined trading colors
- **LineStyle**: Solid, Dashed, Dotted
- **PlotConfig**: Configuration for indicator plots

### 5. Plugins (`plugins`)

Chart analysis plugins (future):
- Framework for chart patterns and analysis tools
- Examples: Pivots, Wyckoff, SMC, Elliott Waves
- To be migrated from loom-chart

### 6. Renderers (`renderers`)

Output adapters for different platforms (future):
- Canvas renderer for WebAssembly
- SVG renderer
- WebGL renderer for performance
- No-op renderer for headless testing

## Indicator Registry

Manages both built-in and external (WASM) plugins:

```rust
let registry = PluginRegistry::default(); // Auto-registers built-ins

// Use a plugin
let context = CalculationContext::new(&candles)
    .with_input("length", InputValue::Int(14));
    
let result = registry.calculate("rsi", &context).unwrap();
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
chartcore = { path = "../crates/chartcore" }
# Or from crates.io (when published):
# chartcore = "0.1"
```

Example:

```rust
use chartcore::prelude::*;

// Or import specific types:
// use chartcore::{Chart, Candle, IndicatorRegistry, CalculationContext, InputValue};

// Create some sample candles
let candles: Vec<Candle> = vec![
    Candle { time: 0, o: 100.0, h: 101.0, l: 99.0, c: 100.5, v: 1000.0 },
    // ... more candles
];

// Initialize registry with built-in plugins
let registry = PluginRegistry::default();

// Calculate RSI
let context = CalculationContext::new(&candles)
    .with_input("length", InputValue::Int(14))
    .with_input("source", InputValue::Source(SourceType::Close));

let result = registry.calculate("rsi", &context).unwrap();
let rsi_values = &result.plots["rsi"];
```

### Generate Realistic Market Data

```rust
use chartcore::prelude::*;

// Crypto bull market with high volatility
let config = GeneratorConfig::crypto()
    .with_trend(Trend::BullishStrong)
    .with_regime(VolatilityRegime::High)
    .with_seed(42); // Reproducible

let mut generator = CandleGenerator::new(config);
let candles = generator.generate(1000);

// Stock market with normal trading hours and weekends
let config = GeneratorConfig::stock()
    .with_trend(Trend::Sideways)
    .with_volatility(1.5);

let mut generator = CandleGenerator::new(config);
let candles = generator.generate(500);

// Forex with low volatility
let config = GeneratorConfig::forex()
    .with_trend(Trend::BearishMild)
    .with_regime(VolatilityRegime::Low);

let mut generator = CandleGenerator::new(config);
let candles = generator.generate(200);
```

## Development Workflow

### Testing Plugins (Pure Rust)

During development, plugins are pure Rust and can be tested without WASM:

```rust
#[test]
fn test_my_indicator() {
    let indicator = MyIndicator::default();
    let candles = vec![/* ... */];
    let context = CalculationContext::new(&candles);
    
    let result = indicator.calculate(&context);
    assert!(result.plots.contains_key("output"));
}
```

### Building for WASM

For external plugins (future):

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Future: External Plugin API

A separate `chartcore-plugin-api` crate will allow external developers to create custom indicators that integrate seamlessly with the plugin system.

## License

This project is dual-licensed:

- **AGPL-3.0-or-later** for open-source and non-commercial use
- **Commercial License** for proprietary or commercial use

If you use this crate in a closed-source application, SaaS, or any
commercial product, you must obtain a commercial license.

Contact: licensing@casoon.de
