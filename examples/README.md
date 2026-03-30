# Loom Examples

This directory contains example strategies and usage patterns for the Loom trading platform.

## Quick Start

```bash
# Run any example
cargo run --example simple_sma

# With release mode for backtesting
cargo run --release --example backtest_sma
```

## Examples

### Indicators

- **simple_indicators.rs** - Basic indicator usage (SMA, EMA, RSI)
- **streaming_indicators.rs** - Streaming indicator pattern with candles

### Strategies

- **simple_sma.rs** - Simple moving average crossover strategy
- **rsi_strategy.rs** - RSI overbought/oversold strategy
- **macd_strategy.rs** - MACD signal line crossover

### Backtesting

- **backtest_sma.rs** - Full backtest of SMA crossover
- **backtest_multi.rs** - Running multiple strategies in parallel
- **optimization.rs** - Parameter optimization

### Chart Plugins

- **custom_plugin.rs** - Creating a custom chart plugin
- **indicator_overlay.rs** - Drawing indicators on chart

### Data

- **csv_import.rs** - Loading data from CSV
- **mock_data.rs** - Generating test data
