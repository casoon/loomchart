//! Moving Average Convergence Divergence (MACD)

use crate::indicators::{Current, Next, Period, Reset};
use crate::indicators::trend::Ema;
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// MACD output containing all three components.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MacdOutput {
    /// MACD line (fast EMA - slow EMA)
    pub macd: f64,
    /// Signal line (EMA of MACD)
    pub signal: f64,
    /// Histogram (MACD - Signal)
    pub histogram: f64,
}

impl MacdOutput {
    pub fn new(macd: f64, signal: f64) -> Self {
        Self {
            macd,
            signal,
            histogram: macd - signal,
        }
    }
}

impl From<MacdOutput> for (f64, f64, f64) {
    fn from(m: MacdOutput) -> Self {
        (m.macd, m.signal, m.histogram)
    }
}

/// Moving Average Convergence Divergence (MACD)
///
/// MACD is a trend-following momentum indicator that shows the relationship
/// between two EMAs of a security's price.
///
/// - MACD Line = Fast EMA - Slow EMA
/// - Signal Line = EMA of MACD Line
/// - Histogram = MACD Line - Signal Line
///
/// Default parameters: (12, 26, 9)
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Macd, Next};
///
/// let mut macd = Macd::new(12, 26, 9);
///
/// // Feed price data
/// for price in prices.iter() {
///     if let Some(output) = macd.next(*price) {
///         println!("MACD: {:.4}, Signal: {:.4}, Hist: {:.4}",
///                  output.macd, output.signal, output.histogram);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Macd {
    #[allow(dead_code)]
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
    current: Option<MacdOutput>,
}

impl Macd {
    /// Create a new MACD with the specified periods.
    ///
    /// # Arguments
    /// - `fast_period`: Period for fast EMA (typically 12)
    /// - `slow_period`: Period for slow EMA (typically 26)
    /// - `signal_period`: Period for signal line EMA (typically 9)
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        assert!(fast_period < slow_period, "Fast period must be less than slow period");
        Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            signal_ema: Ema::new(signal_period),
            current: None,
        }
    }

    /// Get the current MACD line value (fast EMA - slow EMA).
    pub fn macd_line(&self) -> Option<f64> {
        self.current.map(|o| o.macd)
    }

    /// Get the current signal line value.
    pub fn signal_line(&self) -> Option<f64> {
        self.current.map(|o| o.signal)
    }

    /// Get the current histogram value.
    pub fn histogram(&self) -> Option<f64> {
        self.current.map(|o| o.histogram)
    }

    /// Check if MACD is above signal (bullish).
    pub fn is_bullish(&self) -> bool {
        self.current.map_or(false, |o| o.histogram > 0.0)
    }

    /// Check if MACD is below signal (bearish).
    pub fn is_bearish(&self) -> bool {
        self.current.map_or(false, |o| o.histogram < 0.0)
    }
}

impl Next<f64> for Macd {
    type Output = MacdOutput;

    fn next(&mut self, price: f64) -> Option<MacdOutput> {
        let fast = self.fast_ema.next(price);
        let slow = self.slow_ema.next(price);

        match (fast, slow) {
            (Some(fast_val), Some(slow_val)) => {
                let macd_line = fast_val - slow_val;

                // Signal line needs MACD values
                if let Some(signal) = self.signal_ema.next(macd_line) {
                    let output = MacdOutput::new(macd_line, signal);
                    self.current = Some(output);
                    Some(output)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Next<&Ohlcv> for Macd {
    type Output = MacdOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<MacdOutput> {
        self.next(candle.close)
    }
}

impl Reset for Macd {
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.current = None;
    }
}

impl Period for Macd {
    fn period(&self) -> usize {
        // Warmup period is slow_period + signal_period - 1
        self.slow_period + self.signal_period - 1
    }
}

impl Current for Macd {
    type Output = MacdOutput;

    fn current(&self) -> Option<MacdOutput> {
        self.current
    }
}

impl Default for Macd {
    fn default() -> Self {
        Self::new(12, 26, 9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    #[test]
    fn test_macd_warmup() {
        let mut macd = Macd::new(3, 5, 3);

        // Need slow_period + signal_period - 1 = 5 + 3 - 1 = 7 values
        for i in 0..6 {
            let result = macd.next(100.0 + i as f64);
            assert!(result.is_none(), "Should not produce value at index {}", i);
        }

        // 7th value should produce output
        let result = macd.next(106.0);
        assert!(result.is_some());
    }

    #[test]
    fn test_macd_flat_prices() {
        let mut macd = Macd::new(3, 5, 3);

        // With flat prices, MACD line should be 0 (fast EMA = slow EMA)
        for _ in 0..20 {
            macd.next(100.0);
        }

        let output = macd.current().unwrap();
        assert_approx_eq!(output.macd, 0.0, 0.0001);
        assert_approx_eq!(output.signal, 0.0, 0.0001);
        assert_approx_eq!(output.histogram, 0.0, 0.0001);
    }

    #[test]
    fn test_macd_trending() {
        let mut macd = Macd::new(3, 5, 3);

        // Rising prices should produce positive MACD
        for i in 0..20 {
            macd.next(100.0 + i as f64);
        }

        let output = macd.current().unwrap();
        assert!(output.macd > 0.0, "MACD should be positive in uptrend");
    }

    #[test]
    fn test_macd_reset() {
        let mut macd = Macd::new(3, 5, 3);

        for _ in 0..20 {
            macd.next(100.0);
        }

        macd.reset();

        assert!(macd.current().is_none());
        assert!(macd.next(100.0).is_none());
    }
}
