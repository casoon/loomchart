//! Stochastic Oscillator

use crate::indicators::{Current, Next, Period, Reset};
use crate::indicators::trend::Sma;
use crate::math::range::{highest, lowest, percent_in_range};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Stochastic output containing %K and %D.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StochasticOutput {
    /// %K (fast line)
    pub k: f64,
    /// %D (slow line, SMA of %K)
    pub d: f64,
}

impl StochasticOutput {
    pub fn new(k: f64, d: f64) -> Self {
        Self { k, d }
    }
}

impl From<StochasticOutput> for (f64, f64) {
    fn from(s: StochasticOutput) -> Self {
        (s.k, s.d)
    }
}

/// Stochastic Oscillator
///
/// The Stochastic Oscillator compares a security's closing price to its
/// price range over a given period.
///
/// %K = (Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// %D = SMA(%K)
///
/// Values:
/// - 0-20: Oversold
/// - 80-100: Overbought
///
/// Default parameters: (14, 3, 3) for %K period, %K slowing, and %D period.
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Stochastic, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut stoch = Stochastic::new(14, 3, 3);
///
/// for candle in candles.iter() {
///     if let Some(output) = stoch.next(candle) {
///         println!("%K: {:.2}, %D: {:.2}", output.k, output.d);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Stochastic {
    k_period: usize,
    k_slowing: usize,
    d_period: usize,
    high_buffer: Vec<f64>,
    low_buffer: Vec<f64>,
    index: usize,
    count: usize,
    k_sma: Sma, // For %K slowing
    d_sma: Sma, // For %D (SMA of slowed %K)
    current: Option<StochasticOutput>,
}

impl Stochastic {
    /// Create a new Stochastic with the specified parameters.
    ///
    /// # Arguments
    /// - `k_period`: Lookback period for highest high / lowest low
    /// - `k_slowing`: Smoothing period for raw %K (typically 3)
    /// - `d_period`: Period for %D (SMA of smoothed %K)
    pub fn new(k_period: usize, k_slowing: usize, d_period: usize) -> Self {
        assert!(k_period > 0, "K period must be greater than 0");
        assert!(k_slowing > 0, "K slowing must be greater than 0");
        assert!(d_period > 0, "D period must be greater than 0");

        Self {
            k_period,
            k_slowing,
            d_period,
            high_buffer: vec![0.0; k_period],
            low_buffer: vec![f64::MAX; k_period],
            index: 0,
            count: 0,
            k_sma: Sma::new(k_slowing),
            d_sma: Sma::new(d_period),
            current: None,
        }
    }

    /// Create a fast Stochastic (no slowing, %K period only).
    pub fn fast(k_period: usize, d_period: usize) -> Self {
        Self::new(k_period, 1, d_period)
    }

    /// Check if Stochastic indicates overbought (%K > 80).
    pub fn is_overbought(&self) -> bool {
        self.current.map_or(false, |s| s.k > 80.0)
    }

    /// Check if Stochastic indicates oversold (%K < 20).
    pub fn is_oversold(&self) -> bool {
        self.current.map_or(false, |s| s.k < 20.0)
    }

    /// Check for bullish crossover (%K crosses above %D).
    pub fn is_bullish_crossover(&self, prev: Option<StochasticOutput>) -> bool {
        match (prev, self.current) {
            (Some(p), Some(c)) => p.k <= p.d && c.k > c.d,
            _ => false,
        }
    }

    /// Check for bearish crossover (%K crosses below %D).
    pub fn is_bearish_crossover(&self, prev: Option<StochasticOutput>) -> bool {
        match (prev, self.current) {
            (Some(p), Some(c)) => p.k >= p.d && c.k < c.d,
            _ => false,
        }
    }
}

impl Next<&Ohlcv> for Stochastic {
    type Output = StochasticOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<StochasticOutput> {
        // Update buffers
        self.high_buffer[self.index] = candle.high;
        self.low_buffer[self.index] = candle.low;

        // Advance
        self.index = (self.index + 1) % self.k_period;
        if self.count < self.k_period {
            self.count += 1;
        }

        // Need full K period
        if self.count < self.k_period {
            return None;
        }

        // Calculate raw %K
        let hh = highest(&self.high_buffer);
        let ll = lowest(&self.low_buffer);
        let raw_k = percent_in_range(candle.close, ll, hh);

        // Apply slowing (SMA of raw %K)
        let smoothed_k = self.k_sma.next(raw_k)?;

        // Calculate %D (SMA of smoothed %K)
        let d = self.d_sma.next(smoothed_k)?;

        let output = StochasticOutput::new(smoothed_k, d);
        self.current = Some(output);
        Some(output)
    }
}

impl Reset for Stochastic {
    fn reset(&mut self) {
        self.high_buffer.fill(0.0);
        self.low_buffer.fill(f64::MAX);
        self.index = 0;
        self.count = 0;
        self.k_sma.reset();
        self.d_sma.reset();
        self.current = None;
    }
}

impl Period for Stochastic {
    fn period(&self) -> usize {
        self.k_period + self.k_slowing + self.d_period - 2
    }
}

impl Current for Stochastic {
    type Output = StochasticOutput;

    fn current(&self) -> Option<StochasticOutput> {
        self.current
    }
}

impl Default for Stochastic {
    fn default() -> Self {
        Self::new(14, 3, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(open, high, low, close, 1000.0)
    }

    #[test]
    fn test_stochastic_at_high() {
        let mut stoch = Stochastic::new(3, 1, 1);

        // Build up period
        stoch.next(&make_candle(100.0, 110.0, 90.0, 105.0));
        stoch.next(&make_candle(105.0, 115.0, 95.0, 110.0));
        let out = stoch.next(&make_candle(110.0, 120.0, 100.0, 120.0)); // Close at high

        // Close at highest high -> %K = 100
        let out = out.unwrap();
        assert_approx_eq!(out.k, 100.0);
    }

    #[test]
    fn test_stochastic_at_low() {
        let mut stoch = Stochastic::new(3, 1, 1);

        // Build up period
        stoch.next(&make_candle(100.0, 110.0, 90.0, 105.0));
        stoch.next(&make_candle(105.0, 115.0, 95.0, 100.0));
        let out = stoch.next(&make_candle(100.0, 105.0, 85.0, 85.0)); // Close at low

        // Close at lowest low -> %K = 0
        let out = out.unwrap();
        assert_approx_eq!(out.k, 0.0);
    }

    #[test]
    fn test_stochastic_reset() {
        let mut stoch = Stochastic::new(3, 1, 1);

        stoch.next(&make_candle(100.0, 110.0, 90.0, 105.0));
        stoch.next(&make_candle(105.0, 115.0, 95.0, 110.0));
        stoch.next(&make_candle(110.0, 120.0, 100.0, 115.0));

        stoch.reset();

        assert!(stoch.current().is_none());
        assert_eq!(stoch.count, 0);
    }
}
