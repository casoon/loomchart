//! TRIX - Triple Exponential Average

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Ema;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// TRIX - Triple Exponential Average
///
/// TRIX is a momentum oscillator that shows the percent rate of change
/// of a triple exponentially smoothed moving average. It filters out
/// minor price movements.
///
/// Formula: 100 * (EMA3 - Previous EMA3) / Previous EMA3
/// where EMA3 = EMA(EMA(EMA(price)))
///
/// Signals:
/// - Zero line crossovers indicate trend changes
/// - Divergences with price indicate potential reversals
///
/// Default period: 15
#[derive(Clone)]
pub struct Trix {
    period: usize,
    ema1: Ema,
    ema2: Ema,
    ema3: Ema,
    prev_ema3: Option<f64>,
    current: Option<f64>,
}

impl Trix {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            ema1: Ema::new(period),
            ema2: Ema::new(period),
            ema3: Ema::new(period),
            prev_ema3: None,
            current: None,
        }
    }
}

impl Next<f64> for Trix {
    type Output = f64;

    fn next(&mut self, value: f64) -> Option<f64> {
        let ema1_val = self.ema1.next(value)?;
        let ema2_val = self.ema2.next(ema1_val)?;
        let ema3_val = self.ema3.next(ema2_val)?;

        let result = if let Some(prev) = self.prev_ema3 {
            if prev != 0.0 {
                let trix = 100.0 * (ema3_val - prev) / prev;
                self.current = Some(trix);
                Some(trix)
            } else {
                None
            }
        } else {
            None
        };

        self.prev_ema3 = Some(ema3_val);
        result
    }
}

impl Reset for Trix {
    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema3.reset();
        self.prev_ema3 = None;
        self.current = None;
    }
}

impl Period for Trix {
    fn period(&self) -> usize {
        // Needs 3x periods for the triple smoothing plus one for rate of change
        3 * self.period + 1
    }
}

impl Current for Trix {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Trix {
    fn default() -> Self {
        Self::new(15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trix_uptrend() {
        let mut trix = Trix::new(5);

        // Rising prices
        for i in 0..25 {
            trix.next(100.0 + i as f64);
        }

        assert!(trix.current().is_some());
        // In uptrend, TRIX should be positive
        assert!(trix.current().unwrap() > 0.0);
    }

    #[test]
    fn test_trix_downtrend() {
        let mut trix = Trix::new(5);

        // Falling prices
        for i in 0..25 {
            trix.next(150.0 - i as f64);
        }

        assert!(trix.current().is_some());
        // In downtrend, TRIX should be negative
        assert!(trix.current().unwrap() < 0.0);
    }

    #[test]
    fn test_trix_reset() {
        let mut trix = Trix::default();

        for i in 0..50 {
            trix.next(100.0 + (i % 10) as f64);
        }

        trix.reset();
        assert!(trix.current().is_none());
    }
}
