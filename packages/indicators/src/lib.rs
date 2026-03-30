//! # loom-indicators
//!
//! A pure Rust technical analysis library for trading applications.
//!
//! ## Features
//!
//! - **`no_std` compatible**: Works in embedded systems and WASM
//! - **Modular**: Enable only the indicators you need via feature flags
//! - **Two-layer design**: Stateless math functions + stateful indicators
//!
//! ## Architecture
//!
//! The library is split into two main modules:
//!
//! ### `math` - Stateless Mathematical Functions
//!
//! Pure functions for calculations that can be reused across indicators:
//!
//! ```rust
//! use loom_indicators::math;
//!
//! // Moving averages
//! let sma = math::sma(&[1.0, 2.0, 3.0, 4.0, 5.0]);
//! let k = math::ema_multiplier(21);
//! let ema = math::ema_next(105.0, 100.0, k);
//!
//! // Statistics
//! let variance = math::variance(&[1.0, 2.0, 3.0]);
//! let stddev = math::stddev(&[1.0, 2.0, 3.0]);
//!
//! // Range calculations
//! let tr = math::true_range(110.0, 100.0, 105.0);
//! let highest = math::highest(&[1.0, 5.0, 3.0, 2.0]);
//!
//! // Momentum calculations
//! let rsi = math::rsi(1.5, 0.5); // avg_gain, avg_loss
//! ```
//!
//! ### `indicators` - Stateful Indicators
//!
//! Streaming indicators that maintain internal state:
//!
//! ```rust
//! use loom_indicators::indicators::{Ema, Rsi, Next};
//!
//! // Create indicators
//! let mut ema = Ema::new(21);
//! let mut rsi = Rsi::new(14);
//!
//! // Feed data
//! for price in [100.0, 101.0, 99.0, 102.0, 103.0].iter() {
//!     if let Some(value) = ema.next(*price) {
//!         println!("EMA: {:.2}", value);
//!     }
//! }
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `std` | Enable standard library (default) |
//! | `full` | Enable all indicator groups |
//! | `trend` | Trend indicators (EMA, SMA, MACD) |
//! | `momentum` | Momentum indicators (RSI, MFI, Stochastic) |
//! | `volatility` | Volatility indicators (ATR, Bollinger) |
//! | `volume` | Volume indicators (OBV, VWAP) |
//! | `patterns` | Candlestick pattern recognition |
//! | `serde` | Serialization support |

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

pub mod math;
pub mod types;

#[cfg(any(feature = "trend", feature = "momentum", feature = "volatility", feature = "volume", feature = "full"))]
pub mod indicators;

#[cfg(any(feature = "patterns", feature = "full"))]
pub mod patterns;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::math;
    pub use crate::types::{AsOhlcv, IndicatorValue, Ohlcv, RingBuffer};

    #[cfg(any(feature = "trend", feature = "momentum", feature = "volatility", feature = "volume", feature = "full"))]
    pub use crate::indicators::{Current, Indicator, Next, Period, Reset};

    #[cfg(any(feature = "trend", feature = "full"))]
    pub use crate::indicators::trend::{Dema, Ema, Macd, MacdOutput, Sma, Tema};

    #[cfg(any(feature = "momentum", feature = "full"))]
    pub use crate::indicators::momentum::{Mfi, Rsi, Stochastic, StochasticOutput};

    #[cfg(any(feature = "volatility", feature = "full"))]
    pub use crate::indicators::volatility::{Atr, BollingerBands, BollingerOutput};

    #[cfg(any(feature = "volume", feature = "full"))]
    pub use crate::indicators::volume::{Obv, Vwap};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_basic_workflow() {
        // Math functions
        let prices = [100.0, 101.0, 102.0, 103.0, 104.0];
        let sma = math::sma(&prices);
        assert!((sma - 102.0).abs() < 1e-10);

        // OHLCV
        let candle = Ohlcv::new(100.0, 105.0, 98.0, 103.0, 1000.0);
        assert!(candle.is_bullish());
        assert!((candle.typical_price() - 102.0).abs() < 1e-10);
    }

    #[cfg(feature = "trend")]
    #[test]
    fn test_ema_indicator() {
        let mut ema = Ema::new(3);

        assert_eq!(ema.next(100.0), None);
        assert_eq!(ema.next(101.0), None);
        assert!(ema.next(102.0).is_some());
    }

    #[cfg(feature = "momentum")]
    #[test]
    fn test_rsi_indicator() {
        let mut rsi = Rsi::new(3);

        // Need period + 1 values
        for i in 0..4 {
            rsi.next(100.0 + i as f64);
        }

        assert!(rsi.current().is_some());
    }

    #[cfg(feature = "volatility")]
    #[test]
    fn test_bollinger_indicator() {
        let mut bb = BollingerBands::new(3, 2.0);

        bb.next(100.0);
        bb.next(101.0);
        let out = bb.next(102.0).unwrap();

        assert!(out.upper > out.middle);
        assert!(out.middle > out.lower);
    }
}
