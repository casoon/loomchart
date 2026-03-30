//! On-Balance Volume (OBV)

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;

/// On-Balance Volume (OBV)
///
/// OBV is a cumulative indicator that adds volume on up days and
/// subtracts volume on down days.
///
/// - If close > prev_close: OBV = prev_OBV + volume
/// - If close < prev_close: OBV = prev_OBV - volume
/// - If close = prev_close: OBV = prev_OBV
///
/// OBV is used to confirm price trends and identify divergences.
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Obv, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut obv = Obv::new();
///
/// for candle in candles.iter() {
///     let value = obv.next(candle);
///     println!("OBV: {:.0}", value);
/// }
/// ```
#[derive(Clone)]
pub struct Obv {
    prev_close: Option<f64>,
    current: f64,
}

impl Obv {
    /// Create a new OBV indicator.
    pub fn new() -> Self {
        Self {
            prev_close: None,
            current: 0.0,
        }
    }

    /// Check if OBV is rising (confirms uptrend).
    pub fn is_rising(&self, periods: &[f64]) -> bool {
        if periods.len() < 2 {
            return false;
        }
        periods.windows(2).all(|w| w[1] > w[0])
    }

    /// Check if OBV is falling (confirms downtrend).
    pub fn is_falling(&self, periods: &[f64]) -> bool {
        if periods.len() < 2 {
            return false;
        }
        periods.windows(2).all(|w| w[1] < w[0])
    }
}

impl Next<&Ohlcv> for Obv {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        if let Some(prev) = self.prev_close {
            if candle.close > prev {
                self.current += candle.volume;
            } else if candle.close < prev {
                self.current -= candle.volume;
            }
            // If equal, OBV stays the same
        }

        self.prev_close = Some(candle.close);
        Some(self.current)
    }
}

impl Reset for Obv {
    fn reset(&mut self) {
        self.prev_close = None;
        self.current = 0.0;
    }
}

impl Period for Obv {
    fn period(&self) -> usize {
        1 // OBV has no warmup period
    }
}

impl Current for Obv {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        Some(self.current)
    }
}

impl Default for Obv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    fn make_candle(close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, close + 1.0, close - 1.0, close, volume)
    }

    #[test]
    fn test_obv_up_day() {
        let mut obv = Obv::new();

        obv.next(&make_candle(100.0, 1000.0));
        let val = obv.next(&make_candle(102.0, 1500.0)).unwrap();

        // Price up -> add volume
        assert_approx_eq!(val, 1500.0);
    }

    #[test]
    fn test_obv_down_day() {
        let mut obv = Obv::new();

        obv.next(&make_candle(100.0, 1000.0));
        let val = obv.next(&make_candle(98.0, 1500.0)).unwrap();

        // Price down -> subtract volume
        assert_approx_eq!(val, -1500.0);
    }

    #[test]
    fn test_obv_unchanged() {
        let mut obv = Obv::new();

        obv.next(&make_candle(100.0, 1000.0));
        obv.next(&make_candle(102.0, 1500.0)); // +1500
        let val = obv.next(&make_candle(102.0, 2000.0)).unwrap(); // unchanged

        // Price unchanged -> OBV stays same
        assert_approx_eq!(val, 1500.0);
    }

    #[test]
    fn test_obv_cumulative() {
        let mut obv = Obv::new();

        obv.next(&make_candle(100.0, 1000.0)); // First, OBV = 0
        obv.next(&make_candle(102.0, 1000.0)); // Up, OBV = 1000
        obv.next(&make_candle(101.0, 500.0)); // Down, OBV = 500
        let val = obv.next(&make_candle(103.0, 2000.0)).unwrap(); // Up, OBV = 2500

        assert_approx_eq!(val, 2500.0);
    }
}
