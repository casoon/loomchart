//! Technical Indicators
//!
//! Stateful indicators that maintain internal state for streaming data processing.
//!
//! # Indicator Traits
//!
//! All indicators implement one or more of these traits:
//!
//! - [`Next<f64>`] - Process a single price value
//! - [`Next<&Ohlcv>`] - Process an OHLCV candle
//! - [`Reset`] - Reset internal state
//!
//! # Categories
//!
//! - **Trend**: Moving averages and trend-following (EMA, SMA, MACD)
//! - **Momentum**: Oscillators and momentum (RSI, Stochastic, MFI)
//! - **Volatility**: Volatility measures (ATR, Bollinger Bands)
//! - **Volume**: Volume-based indicators (OBV, VWAP)

pub mod momentum;
pub mod trend;
pub mod volatility;
pub mod volume;

use crate::types::IndicatorValue;

/// Trait for indicators that process a value and optionally return a result.
///
/// The input type `T` is typically `f64` for price-based indicators or
/// `&Ohlcv` for candle-based indicators.
///
/// Returns `Some(value)` when enough data has been processed, `None` otherwise.
pub trait Next<T> {
    /// The output type of the indicator.
    type Output;

    /// Process the next value and return the indicator result.
    fn next(&mut self, input: T) -> Option<Self::Output>;
}

/// Trait for resetting indicator state.
pub trait Reset {
    /// Reset the indicator to its initial state.
    fn reset(&mut self);
}

/// Trait for indicators that have a warmup period.
pub trait Period {
    /// The number of values needed before the indicator produces output.
    fn period(&self) -> usize;
}

/// Trait for indicators that can report their current value without advancing.
pub trait Current {
    /// The output type.
    type Output;

    /// Get the current indicator value without advancing state.
    fn current(&self) -> Option<Self::Output>;
}

/// Combined trait for common indicator operations.
pub trait Indicator<T>: Next<T> + Reset + Period {}

// Blanket implementation
impl<T, I> Indicator<T> for I where I: Next<T> + Reset + Period {}

/// A wrapper that converts indicator output to IndicatorValue.
pub struct IntoIndicatorValue<I> {
    inner: I,
}

impl<I> IntoIndicatorValue<I> {
    pub fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I, T> Next<T> for IntoIndicatorValue<I>
where
    I: Next<T>,
    I::Output: Into<IndicatorValue>,
{
    type Output = IndicatorValue;

    fn next(&mut self, input: T) -> Option<Self::Output> {
        self.inner.next(input).map(Into::into)
    }
}

impl<I: Reset> Reset for IntoIndicatorValue<I> {
    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl<I: Period> Period for IntoIndicatorValue<I> {
    fn period(&self) -> usize {
        self.inner.period()
    }
}

// Re-exports of common indicators
#[cfg(feature = "trend")]
pub use trend::{Dema, Ema, Macd, Sma, Tema};

#[cfg(feature = "momentum")]
pub use momentum::{Mfi, Rsi, Stochastic};

#[cfg(feature = "volatility")]
pub use volatility::{Atr, BollingerBands};

#[cfg(feature = "volume")]
pub use volume::{Obv, Vwap};

/// Helper macro for creating indicator tests.
#[cfg(test)]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr) => {
        assert_approx_eq!($left, $right, 1e-6)
    };
    ($left:expr, $right:expr, $epsilon:expr) => {
        let left = $left;
        let right = $right;
        if (left - right).abs() > $epsilon {
            panic!(
                "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`",
                left, right
            );
        }
    };
}

#[cfg(test)]
pub(crate) use assert_approx_eq;
