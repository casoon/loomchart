//! Ease of Movement (EoM)

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Sma;
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Ease of Movement (EoM / EMV)
///
/// Developed by Richard Arms, Ease of Movement relates price change
/// to volume, showing how easily prices move.
///
/// Formula:
/// Distance = ((High + Low) / 2) - ((Prev High + Prev Low) / 2)
/// Box Ratio = (Volume / scale) / (High - Low)
/// EoM = Distance / Box Ratio
///
/// Smoothed with SMA for trading signals.
///
/// Interpretation:
/// - Positive: Prices moving up with low volume (easy upward movement)
/// - Negative: Prices moving down with low volume (easy downward movement)
/// - Near zero: Price movement difficult or sideways
///
/// Default period: 14, scale: 1,000,000
#[derive(Clone)]
pub struct EaseOfMovement {
    period: usize,
    scale: f64,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    sma: Sma,
    current: Option<f64>,
}

impl EaseOfMovement {
    pub fn new(period: usize, scale: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(scale > 0.0, "Scale must be greater than 0");
        Self {
            period,
            scale,
            prev_high: None,
            prev_low: None,
            sma: Sma::new(period),
            current: None,
        }
    }
}

impl Next<&Ohlcv> for EaseOfMovement {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let result = if let (Some(prev_high), Some(prev_low)) = (self.prev_high, self.prev_low) {
            let distance = ((candle.high + candle.low) / 2.0) - ((prev_high + prev_low) / 2.0);
            let hl_range = candle.high - candle.low;

            let raw_eom = if hl_range > 0.0 && candle.volume > 0.0 {
                let box_ratio = (candle.volume / self.scale) / hl_range;
                distance / box_ratio
            } else {
                0.0
            };

            let smoothed = self.sma.next(raw_eom);
            if smoothed.is_some() {
                self.current = smoothed;
            }
            smoothed
        } else {
            None
        };

        self.prev_high = Some(candle.high);
        self.prev_low = Some(candle.low);
        result
    }
}

impl Reset for EaseOfMovement {
    fn reset(&mut self) {
        self.prev_high = None;
        self.prev_low = None;
        self.sma.reset();
        self.current = None;
    }
}

impl Period for EaseOfMovement {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for EaseOfMovement {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for EaseOfMovement {
    fn default() -> Self {
        Self::new(14, 1_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, volume)
    }

    #[test]
    fn test_eom_basic() {
        let mut eom = EaseOfMovement::new(5, 1000.0);

        for i in 0..10 {
            eom.next(&make_candle(
                102.0 + i as f64,
                98.0 + i as f64,
                100.0 + i as f64,
                100.0,
            ));
        }

        assert!(eom.current().is_some());
    }

    #[test]
    fn test_eom_reset() {
        let mut eom = EaseOfMovement::default();

        for i in 0..20 {
            eom.next(&make_candle(102.0 + i as f64, 98.0, 100.0, 1000.0));
        }

        eom.reset();
        assert!(eom.current().is_none());
    }
}
