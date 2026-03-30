# Changelog

All notable changes to chartcore will be documented in this file.

## [Unreleased]

### Added
- **Core Module**: Complete chart infrastructure
  - `Candle` type with helper methods (is_bullish, typical_price, range, etc.)
  - `Chart` struct for managing candle data
  - `ChartBuffer` ring buffer (default 2000 candles)
  - `ChartConfig` for chart configuration
  - `Timeframe` enum (1s to 1M)
  - `CandleGenerator` for realistic market simulation
    - Stock market: Trading hours 9:30-16:00, weekends/holidays closed
    - Forex: 24/5 trading with weekend gaps
    - Crypto: 24/7 trading, high volatility
    - Futures: Nearly 24/5 with session gaps
    - Commodities: Restricted trading hours
    - Configurable trends (Bullish/Bearish/Sideways)
    - Volatility regimes (Low/Normal/High/Extreme)
    - Reproducible with seeds
    - Realistic price action using Geometric Brownian Motion
    - Market hours and gap simulation

- **Indicator System**: Complete technical analysis framework
  - IndicatorPlugin trait for extensibility
  - IndicatorRegistry with auto-registration
  - Built-in indicators: RSI, EMA, SMA
  - TA library: 20+ calculation functions
    - Moving Averages: SMA, EMA, WMA, RMA, DEMA, TEMA, HMA, VWMA
    - Momentum: RSI, Stochastic, CCI, Williams %R
    - Statistics: StdDev, Highest, Lowest, Change, ROC, Correlation, LinReg
  - Incremental calculation support (O(1) updates)
  - Input validation
  - WASM plugin loader (API defined, implementation pending)

- **Primitives Module**: Drawing and visualization types
  - Color with predefined trading colors
  - LineStyle (Solid, Dashed, Dotted)
  - PlotConfig for indicator visualization

- **Plugins Module**: Framework for chart analysis plugins (placeholder)
  - ChartPlugin trait
  - Future: Pivots, Wyckoff, SMC, Elliott Waves

- **Renderers Module**: Platform abstraction (placeholder)
  - Renderer trait
  - NoopRenderer for headless testing
  - Future: Canvas, WebGL, SVG renderers

- **Examples**:
  - `generator.rs`: Comprehensive candle generation examples

- **Documentation**:
  - Complete README with architecture overview
  - Usage examples for all major features
  - API documentation

### Changed
- Renamed from `chartcore-indicators` to `chartcore`
- Reorganized module structure for clarity
- Updated all dependencies in workspace

### Technical
- Zero external dependencies (except serde for serialization)
- Full test coverage for TA library and core types
- Modular architecture for future extensibility

## Notes

This is the initial release establishing chartcore as a standalone chart engine.
The goal is to create a Rust-native alternative to TradingView Lightweight Charts.

Future releases will focus on:
1. Migrating chart analysis plugins from loom-chart
2. Implementing renderer backends (Canvas, WebGL, SVG)
3. WASM plugin loader implementation
4. Additional built-in indicators (MACD, Bollinger Bands, etc.)
5. Performance optimizations
6. External publication to crates.io
