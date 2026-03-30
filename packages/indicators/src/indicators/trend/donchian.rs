//! Donchian Channel

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Donchian Channel Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DonchianOutput {
    /// Upper band (highest high)
    pub upper: f64,
    /// Lower band (lowest low)
    pub lower: f64,
    /// Middle band (average of upper and lower)
    pub middle: f64,
    /// Channel width
    pub width: f64,
}

impl DonchianOutput {
    pub fn new(upper: f64, lower: f64) -> Self {
        Self {
            upper,
            lower,
            middle: (upper + lower) / 2.0,
            width: upper - lower,
        }
    }

    /// Check if price is at upper band (breakout potential)
    pub fn is_at_upper(&self, price: f64) -> bool {
        (price - self.upper).abs() < f64::EPSILON
    }

    /// Check if price is at lower band (breakdown potential)
    pub fn is_at_lower(&self, price: f64) -> bool {
        (price - self.lower).abs() < f64::EPSILON
    }
}

/// Donchian Channel
///
/// Created by Richard Donchian, this indicator shows the highest high
/// and lowest low over a period, creating a channel around price.
///
/// Used in:
/// - Turtle Trading system
/// - Breakout strategies
/// - Trend identification
///
/// Default period: 20
#[derive(Clone)]
pub struct DonchianChannel {
    period: usize,
    high_buffer: Vec<f64>,
    low_buffer: Vec<f64>,
    index: usize,
    count: usize,
    current: Option<DonchianOutput>,
}

impl DonchianChannel {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            high_buffer: vec![f64::NEG_INFINITY; period],
            low_buffer: vec![f64::INFINITY; period],
            index: 0,
            count: 0,
            current: None,
        }
    }

    fn find_highest(&self) -> f64 {
        let mut max = f64::NEG_INFINITY;
        for i in 0..self.count.min(self.period) {
            if self.high_buffer[i] > max {
                max = self.high_buffer[i];
            }
        }
        max
    }

    fn find_lowest(&self) -> f64 {
        let mut min = f64::INFINITY;
        for i in 0..self.count.min(self.period) {
            if self.low_buffer[i] < min {
                min = self.low_buffer[i];
            }
        }
        min
    }
}

impl Next<&Ohlcv> for DonchianChannel {
    type Output = DonchianOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<DonchianOutput> {
        self.high_buffer[self.index] = candle.high;
        self.low_buffer[self.index] = candle.low;

        self.index = (self.index + 1) % self.period;
        if self.count < self.period {
            self.count += 1;
        }

        if self.count >= self.period {
            let upper = self.find_highest();
            let lower = self.find_lowest();
            let output = DonchianOutput::new(upper, lower);
            self.current = Some(output);
            Some(output)
        } else {
            None
        }
    }
}

impl Reset for DonchianChannel {
    fn reset(&mut self) {
        self.high_buffer.fill(f64::NEG_INFINITY);
        self.low_buffer.fill(f64::INFINITY);
        self.index = 0;
        self.count = 0;
        self.current = None;
    }
}

impl Period for DonchianChannel {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for DonchianChannel {
    type Output = DonchianOutput;

    fn current(&self) -> Option<DonchianOutput> {
        self.current
    }
}

impl Default for DonchianChannel {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_donchian_basic() {
        let mut dc = DonchianChannel::new(5);

        // Build up period
        dc.next(&make_candle(102.0, 98.0, 100.0));
        dc.next(&make_candle(105.0, 99.0, 103.0));
        dc.next(&make_candle(103.0, 97.0, 101.0));
        dc.next(&make_candle(108.0, 100.0, 106.0));
        let out = dc.next(&make_candle(104.0, 96.0, 102.0));

        assert!(out.is_some());
        let output = out.unwrap();
        assert!((output.upper - 108.0).abs() < 0.01);
        assert!((output.lower - 96.0).abs() < 0.01);
        assert!((output.middle - 102.0).abs() < 0.01);
    }

    #[test]
    fn test_donchian_reset() {
        let mut dc = DonchianChannel::new(5);

        for i in 0..10 {
            dc.next(&make_candle(100.0 + i as f64, 95.0, 98.0));
        }

        dc.reset();
        assert!(dc.current().is_none());
    }
}
