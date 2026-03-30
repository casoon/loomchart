//! Awesome Oscillator

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Sma;
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Awesome Oscillator
///
/// Developed by Bill Williams, the Awesome Oscillator is a momentum
/// indicator that shows the difference between a 34-period and 5-period
/// SMA of the median price (H+L)/2.
///
/// Signals:
/// - Zero line cross: Trend change
/// - Saucer: Three-bar pattern above/below zero for continuation
/// - Twin peaks: Divergence pattern for reversals
///
/// Default periods: 5, 34
#[derive(Clone)]
pub struct AwesomeOscillator {
    #[allow(dead_code)]
    fast_period: usize,
    slow_period: usize,
    fast_sma: Sma,
    slow_sma: Sma,
    current: Option<f64>,
}

impl AwesomeOscillator {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period > 0 && slow_period > fast_period,
                "fast_period must be > 0 and slow_period > fast_period");

        Self {
            fast_period,
            slow_period,
            fast_sma: Sma::new(fast_period),
            slow_sma: Sma::new(slow_period),
            current: None,
        }
    }
}

impl Next<&Ohlcv> for AwesomeOscillator {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let median = (candle.high + candle.low) / 2.0;
        self.next(median)
    }
}

impl Next<f64> for AwesomeOscillator {
    type Output = f64;

    fn next(&mut self, median_price: f64) -> Option<f64> {
        let fast = self.fast_sma.next(median_price);
        let slow = self.slow_sma.next(median_price);

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let ao = f - s;
                self.current = Some(ao);
                Some(ao)
            }
            _ => None,
        }
    }
}

impl Reset for AwesomeOscillator {
    fn reset(&mut self) {
        self.fast_sma.reset();
        self.slow_sma.reset();
        self.current = None;
    }
}

impl Period for AwesomeOscillator {
    fn period(&self) -> usize {
        self.slow_period
    }
}

impl Current for AwesomeOscillator {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for AwesomeOscillator {
    fn default() -> Self {
        Self::new(5, 34)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_ao_uptrend() {
        let mut ao = AwesomeOscillator::new(3, 10);

        // Rising prices
        for i in 0..20 {
            ao.next(&make_candle(102.0 + i as f64 * 2.0, 98.0 + i as f64 * 2.0, 100.0 + i as f64 * 2.0));
        }

        assert!(ao.current().is_some());
        // In uptrend, AO should be positive (fast > slow)
        assert!(ao.current().unwrap() > 0.0);
    }

    #[test]
    fn test_ao_downtrend() {
        let mut ao = AwesomeOscillator::new(3, 10);

        // Falling prices
        for i in 0..20 {
            ao.next(&make_candle(102.0 - i as f64 * 2.0, 98.0 - i as f64 * 2.0, 100.0 - i as f64 * 2.0));
        }

        assert!(ao.current().is_some());
        // In downtrend, AO should be negative (fast < slow)
        assert!(ao.current().unwrap() < 0.0);
    }

    #[test]
    fn test_ao_reset() {
        let mut ao = AwesomeOscillator::default();

        for i in 0..40 {
            ao.next(&make_candle(102.0 + i as f64, 98.0, 100.0));
        }

        ao.reset();
        assert!(ao.current().is_none());
    }
}
