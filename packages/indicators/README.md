# loom-indicators

A pure Rust technical analysis library with no external dependencies (except `libm` for `no_std`).

## Features

- **`no_std` compatible** - Works in embedded systems and WASM
- **Zero-copy where possible** - Efficient memory usage
- **Modular** - Enable only the indicators you need
- **Well-tested** - Comprehensive test coverage

## Architecture

```
loom-indicators/
├── math/           # Reusable mathematical functions
│   ├── average     # SMA, EMA, WMA, VWAP calculations
│   ├── stats       # Variance, StdDev, Correlation
│   ├── range       # True Range, Highest/Lowest
│   └── momentum    # ROC, Momentum, Gain/Loss
│
├── indicators/     # Technical indicators using math functions
│   ├── trend/      # EMA, SMA, MACD, ADX
│   ├── momentum/   # RSI, Stochastic, MFI, CCI
│   ├── volatility/ # ATR, Bollinger Bands, Keltner
│   └── volume/     # OBV, MFI, VWAP, A/D Line
│
└── types           # OHLCV, Price, common types
```

## Usage

```rust
use loom_indicators::prelude::*;

// Using mathematical functions directly
let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
let sma = math::sma(&prices);  // Simple Moving Average

// Using indicators with streaming data
let mut ema = Ema::new(21);
for price in prices {
    if let Some(value) = ema.next(price) {
        println!("EMA: {}", value);
    }
}

// Using with OHLCV data
let mut rsi = Rsi::new(14);
for candle in candles {
    if let Some(value) = rsi.next_ohlcv(&candle) {
        println!("RSI: {}", value);
    }
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `std` | Enable standard library (default) |
| `full` | Enable all indicator groups |
| `trend` | Trend indicators (EMA, MACD, ADX) |
| `momentum` | Momentum indicators (RSI, Stoch, MFI) |
| `volatility` | Volatility indicators (ATR, BB) |
| `volume` | Volume indicators (OBV, VWAP) |
| `serde` | Serialization support |
| `wasm` | WASM compatibility |

## Mathematical Functions vs Indicators

### Math Functions (Stateless)
Pure functions that take data and return results:
```rust
// These are reusable building blocks
math::sma(&prices)           // Simple moving average
math::ema_next(price, prev, k)  // Single EMA step
math::true_range(high, low, prev_close)
math::stddev(&prices)
```

### Indicators (Stateful)
Maintain internal state for streaming data:
```rust
// These track history and state
let mut rsi = Rsi::new(14);
rsi.next(price)  // Updates internal state, returns value
rsi.reset()      // Clear state
```

## License

MIT OR Apache-2.0
