//! Force Index

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Ema;
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Force Index
///
/// Developed by Alexander Elder, the Force Index measures the force
/// of bulls during up days and bears during down days.
///
/// Formula: Force = (Close - Previous Close) * Volume
/// Smoothed with EMA for less noise.
///
/// Interpretation:
/// - Positive: Bulls in control
/// - Negative: Bears in control
/// - Zero crossovers: Potential trend changes
///
/// Default period: 13 (for smoothing EMA)
#[derive(Clone)]
pub struct ForceIndex {
    period: usize,
    prev_close: Option<f64>,
    ema: Ema,
    current: Option<f64>,
}

impl ForceIndex {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            prev_close: None,
            ema: Ema::new(period),
            current: None,
        }
    }

    /// Create a 1-period (raw) Force Index
    pub fn raw() -> Self {
        Self::new(1)
    }
}

impl Next<&Ohlcv> for ForceIndex {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let result = if let Some(prev) = self.prev_close {
            let raw_force = (candle.close - prev) * candle.volume;

            if self.period == 1 {
                self.current = Some(raw_force);
                Some(raw_force)
            } else {
                let smoothed = self.ema.next(raw_force);
                if smoothed.is_some() {
                    self.current = smoothed;
                }
                smoothed
            }
        } else {
            None
        };

        self.prev_close = Some(candle.close);
        result
    }
}

impl Reset for ForceIndex {
    fn reset(&mut self) {
        self.prev_close = None;
        self.ema.reset();
        self.current = None;
    }
}

impl Period for ForceIndex {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for ForceIndex {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for ForceIndex {
    fn default() -> Self {
        Self::new(13)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, close + 1.0, close - 1.0, close, volume)
    }

    #[test]
    fn test_force_index_raw_up() {
        let mut fi = ForceIndex::raw();

        fi.next(&make_candle(100.0, 1000.0));
        let result = fi.next(&make_candle(102.0, 1500.0));

        assert!(result.is_some());
        // (102 - 100) * 1500 = 3000
        assert!((result.unwrap() - 3000.0).abs() < 0.01);
    }

    #[test]
    fn test_force_index_raw_down() {
        let mut fi = ForceIndex::raw();

        fi.next(&make_candle(100.0, 1000.0));
        let result = fi.next(&make_candle(98.0, 2000.0));

        assert!(result.is_some());
        // (98 - 100) * 2000 = -4000
        assert!((result.unwrap() - (-4000.0)).abs() < 0.01);
    }

    #[test]
    fn test_force_index_smoothed() {
        let mut fi = ForceIndex::new(5);

        for i in 0..10 {
            fi.next(&make_candle(100.0 + i as f64, 1000.0));
        }

        assert!(fi.current().is_some());
    }

    #[test]
    fn test_force_index_reset() {
        let mut fi = ForceIndex::default();

        for i in 0..20 {
            fi.next(&make_candle(100.0 + i as f64, 1000.0));
        }

        fi.reset();
        assert!(fi.current().is_none());
    }
}
