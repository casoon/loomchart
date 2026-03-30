//! Bollinger Bands

use crate::indicators::{Current, Next, Period, Reset};
use crate::math::stats::stddev_with_mean;
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Bollinger Bands output.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BollingerOutput {
    /// Middle band (SMA)
    pub middle: f64,
    /// Upper band (SMA + k * StdDev)
    pub upper: f64,
    /// Lower band (SMA - k * StdDev)
    pub lower: f64,
    /// Bandwidth: (upper - lower) / middle * 100
    pub bandwidth: f64,
    /// %B: (price - lower) / (upper - lower)
    pub percent_b: f64,
}

impl BollingerOutput {
    pub fn new(middle: f64, upper: f64, lower: f64, price: f64) -> Self {
        let bandwidth = if middle.abs() > f64::EPSILON {
            (upper - lower) / middle * 100.0
        } else {
            0.0
        };

        let range = upper - lower;
        let percent_b = if range.abs() > f64::EPSILON {
            (price - lower) / range
        } else {
            0.5
        };

        Self {
            middle,
            upper,
            lower,
            bandwidth,
            percent_b,
        }
    }
}

/// Bollinger Bands
///
/// Bollinger Bands consist of a middle band (SMA) with upper and lower bands
/// at k standard deviations above and below.
///
/// - Middle Band = SMA(period)
/// - Upper Band = Middle + (k * StdDev)
/// - Lower Band = Middle - (k * StdDev)
///
/// Default: 20 period, 2 standard deviations
///
/// Trading signals:
/// - Price near upper band: potentially overbought
/// - Price near lower band: potentially oversold
/// - Bandwidth squeeze: low volatility, potential breakout
///
/// # Example
/// ```
/// use loom_indicators::indicators::{BollingerBands, Next};
///
/// let mut bb = BollingerBands::new(20, 2.0);
///
/// for price in prices.iter() {
///     if let Some(output) = bb.next(*price) {
///         println!("Upper: {:.2}, Middle: {:.2}, Lower: {:.2}",
///                  output.upper, output.middle, output.lower);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct BollingerBands {
    period: usize,
    multiplier: f64,
    buffer: Vec<f64>,
    index: usize,
    count: usize,
    sum: f64,
    current: Option<BollingerOutput>,
}

impl BollingerBands {
    /// Create new Bollinger Bands with specified period and standard deviation multiplier.
    pub fn new(period: usize, multiplier: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(multiplier > 0.0, "Multiplier must be positive");

        Self {
            period,
            multiplier,
            buffer: vec![0.0; period],
            index: 0,
            count: 0,
            sum: 0.0,
            current: None,
        }
    }

    /// Get the standard deviation multiplier.
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    /// Check if price is above upper band.
    pub fn is_above_upper(&self, price: f64) -> bool {
        self.current.map_or(false, |bb| price > bb.upper)
    }

    /// Check if price is below lower band.
    pub fn is_below_lower(&self, price: f64) -> bool {
        self.current.map_or(false, |bb| price < bb.lower)
    }

    /// Check if we're in a squeeze (low bandwidth).
    pub fn is_squeeze(&self, threshold: f64) -> bool {
        self.current.map_or(false, |bb| bb.bandwidth < threshold)
    }
}

impl Next<f64> for BollingerBands {
    type Output = BollingerOutput;

    fn next(&mut self, price: f64) -> Option<BollingerOutput> {
        // Subtract old value if buffer is full
        if self.count >= self.period {
            self.sum -= self.buffer[self.index];
        }

        // Add new value
        self.buffer[self.index] = price;
        self.sum += price;

        // Advance
        self.index = (self.index + 1) % self.period;
        if self.count < self.period {
            self.count += 1;
        }

        // Need full period
        if self.count < self.period {
            return None;
        }

        // Calculate SMA and StdDev
        let sma = self.sum / self.period as f64;
        let std = stddev_with_mean(&self.buffer, sma);

        // Calculate bands
        let upper = sma + self.multiplier * std;
        let lower = sma - self.multiplier * std;

        let output = BollingerOutput::new(sma, upper, lower, price);
        self.current = Some(output);
        Some(output)
    }
}

impl Next<&Ohlcv> for BollingerBands {
    type Output = BollingerOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<BollingerOutput> {
        self.next(candle.close)
    }
}

impl Reset for BollingerBands {
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.sum = 0.0;
        self.current = None;
    }
}

impl Period for BollingerBands {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for BollingerBands {
    type Output = BollingerOutput;

    fn current(&self) -> Option<BollingerOutput> {
        self.current
    }
}

impl Default for BollingerBands {
    fn default() -> Self {
        Self::new(20, 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    #[test]
    fn test_bollinger_warmup() {
        let mut bb = BollingerBands::new(3, 2.0);

        assert!(bb.next(100.0).is_none());
        assert!(bb.next(101.0).is_none());
        assert!(bb.next(102.0).is_some());
    }

    #[test]
    fn test_bollinger_flat_prices() {
        let mut bb = BollingerBands::new(3, 2.0);

        bb.next(100.0);
        bb.next(100.0);
        let out = bb.next(100.0).unwrap();

        // Flat prices -> stddev = 0 -> bands collapse to middle
        assert_approx_eq!(out.middle, 100.0);
        assert_approx_eq!(out.upper, 100.0);
        assert_approx_eq!(out.lower, 100.0);
        assert_approx_eq!(out.bandwidth, 0.0);
        assert_approx_eq!(out.percent_b, 0.5); // At middle
    }

    #[test]
    fn test_bollinger_percent_b() {
        let mut bb = BollingerBands::new(3, 2.0);

        bb.next(100.0);
        bb.next(110.0);
        let out = bb.next(105.0).unwrap();

        // %B should be 0 at lower, 1 at upper, 0.5 at middle
        assert!(out.percent_b >= 0.0 && out.percent_b <= 1.0);
    }

    #[test]
    fn test_bollinger_reset() {
        let mut bb = BollingerBands::new(3, 2.0);

        bb.next(100.0);
        bb.next(101.0);
        bb.next(102.0);

        bb.reset();

        assert!(bb.current().is_none());
        assert_eq!(bb.count, 0);
    }
}
