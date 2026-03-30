//! Williams %R

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Williams %R
///
/// Developed by Larry Williams, this momentum indicator is similar to
/// the Stochastic oscillator but inverted and not smoothed.
///
/// Formula: %R = (Highest High - Close) / (Highest High - Lowest Low) * -100
///
/// Values range from -100 to 0:
/// - -20 to 0: Overbought
/// - -80 to -100: Oversold
///
/// Default period: 14
#[derive(Clone)]
pub struct WilliamsR {
    period: usize,
    high_buffer: Vec<f64>,
    low_buffer: Vec<f64>,
    index: usize,
    count: usize,
    current: Option<f64>,
}

impl WilliamsR {
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

    fn highest(&self) -> f64 {
        let mut max = f64::NEG_INFINITY;
        for i in 0..self.count.min(self.period) {
            if self.high_buffer[i] > max {
                max = self.high_buffer[i];
            }
        }
        max
    }

    fn lowest(&self) -> f64 {
        let mut min = f64::INFINITY;
        for i in 0..self.count.min(self.period) {
            if self.low_buffer[i] < min {
                min = self.low_buffer[i];
            }
        }
        min
    }
}

impl Next<&Ohlcv> for WilliamsR {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.high_buffer[self.index] = candle.high;
        self.low_buffer[self.index] = candle.low;

        self.index = (self.index + 1) % self.period;
        if self.count < self.period {
            self.count += 1;
        }

        if self.count >= self.period {
            let hh = self.highest();
            let ll = self.lowest();
            let range = hh - ll;

            let wr = if range > 0.0 {
                ((hh - candle.close) / range) * -100.0
            } else {
                -50.0 // Middle value when no range
            };

            self.current = Some(wr);
            Some(wr)
        } else {
            None
        }
    }
}

impl Reset for WilliamsR {
    fn reset(&mut self) {
        self.high_buffer.fill(f64::NEG_INFINITY);
        self.low_buffer.fill(f64::INFINITY);
        self.index = 0;
        self.count = 0;
        self.current = None;
    }
}

impl Period for WilliamsR {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for WilliamsR {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for WilliamsR {
    fn default() -> Self {
        Self::new(14)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_williams_r_at_high() {
        let mut wr = WilliamsR::new(5);

        for _ in 0..4 {
            wr.next(&make_candle(100.0, 90.0, 95.0));
        }
        // Close at the high
        let result = wr.next(&make_candle(100.0, 90.0, 100.0));

        assert!(result.is_some());
        let value = result.unwrap();
        assert!((value - 0.0).abs() < 0.01); // At high = 0
    }

    #[test]
    fn test_williams_r_at_low() {
        let mut wr = WilliamsR::new(5);

        for _ in 0..4 {
            wr.next(&make_candle(100.0, 90.0, 95.0));
        }
        // Close at the low
        let result = wr.next(&make_candle(100.0, 90.0, 90.0));

        assert!(result.is_some());
        let value = result.unwrap();
        assert!((value - (-100.0)).abs() < 0.01); // At low = -100
    }

    #[test]
    fn test_williams_r_reset() {
        let mut wr = WilliamsR::default();

        for i in 0..20 {
            wr.next(&make_candle(100.0 + i as f64, 90.0, 95.0));
        }

        wr.reset();
        assert!(wr.current().is_none());
    }
}
