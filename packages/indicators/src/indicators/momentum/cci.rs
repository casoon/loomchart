//! Commodity Channel Index (CCI)

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Commodity Channel Index (CCI)
///
/// Developed by Donald Lambert, CCI measures the variation of a security's
/// price from its statistical mean. High values show that prices are
/// unusually high compared to average prices.
///
/// Formula: CCI = (Typical Price - SMA(TP)) / (0.015 * Mean Deviation)
///
/// Values:
/// - > +100: Overbought
/// - < -100: Oversold
/// - 0: Near average
///
/// Default period: 20
#[derive(Clone)]
pub struct Cci {
    period: usize,
    buffer: Vec<f64>,
    index: usize,
    count: usize,
    sum: f64,
    current: Option<f64>,
}

impl Cci {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            buffer: vec![0.0; period],
            index: 0,
            count: 0,
            sum: 0.0,
            current: None,
        }
    }

    fn mean_deviation(&self, mean: f64) -> f64 {
        let n = self.count.min(self.period);
        if n == 0 {
            return 0.0;
        }

        let mut sum = 0.0;
        for i in 0..n {
            sum += libm::fabs(self.buffer[i] - mean);
        }
        sum / n as f64
    }
}

impl Next<&Ohlcv> for Cci {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let tp = candle.typical_price();

        // Update buffer
        if self.count >= self.period {
            self.sum -= self.buffer[self.index];
        }
        self.buffer[self.index] = tp;
        self.sum += tp;

        self.index = (self.index + 1) % self.period;
        if self.count < self.period {
            self.count += 1;
        }

        if self.count >= self.period {
            let mean = self.sum / self.period as f64;
            let md = self.mean_deviation(mean);

            let cci = if md > 0.0 {
                (tp - mean) / (0.015 * md)
            } else {
                0.0
            };

            self.current = Some(cci);
            Some(cci)
        } else {
            None
        }
    }
}

impl Reset for Cci {
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.sum = 0.0;
        self.current = None;
    }
}

impl Period for Cci {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Cci {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Cci {
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
    fn test_cci_basic() {
        let mut cci = Cci::new(5);

        for i in 0..10 {
            cci.next(&make_candle(100.0 + i as f64 * 2.0, 98.0 + i as f64, 99.0 + i as f64));
        }

        assert!(cci.current().is_some());
    }

    #[test]
    fn test_cci_overbought() {
        let mut cci = Cci::new(5);

        // Stable prices first
        for _ in 0..5 {
            cci.next(&make_candle(100.0, 99.0, 99.5));
        }

        // Strong move up
        let result = cci.next(&make_candle(120.0, 115.0, 118.0));
        assert!(result.is_some());
        // After a big up move, CCI should be positive
    }

    #[test]
    fn test_cci_reset() {
        let mut cci = Cci::default();

        for i in 0..30 {
            cci.next(&make_candle(100.0 + i as f64, 98.0, 99.0));
        }

        cci.reset();
        assert!(cci.current().is_none());
    }
}
