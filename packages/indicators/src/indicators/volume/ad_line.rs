//! Accumulation/Distribution Line

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;
use crate::math::momentum::money_flow_multiplier;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Accumulation/Distribution Line (A/D Line)
///
/// Developed by Marc Chaikin, the A/D Line measures cumulative money flow.
/// It uses the relationship of close to the high-low range, weighted by volume.
///
/// Formula:
/// Money Flow Multiplier = ((Close - Low) - (High - Close)) / (High - Low)
/// Money Flow Volume = MFM * Volume
/// A/D = Previous A/D + Money Flow Volume
///
/// Interpretation:
/// - Rising A/D with rising price: Confirms uptrend
/// - Falling A/D with falling price: Confirms downtrend
/// - Divergence: Potential trend reversal
#[derive(Clone)]
pub struct AdLine {
    ad: f64,
    count: usize,
    current: Option<f64>,
}

impl AdLine {
    pub fn new() -> Self {
        Self {
            ad: 0.0,
            count: 0,
            current: None,
        }
    }
}

impl Next<&Ohlcv> for AdLine {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.count += 1;

        let mfm = money_flow_multiplier(candle.high, candle.low, candle.close);
        let mfv = mfm * candle.volume;
        self.ad += mfv;

        self.current = Some(self.ad);
        Some(self.ad)
    }
}

impl Reset for AdLine {
    fn reset(&mut self) {
        self.ad = 0.0;
        self.count = 0;
        self.current = None;
    }
}

impl Period for AdLine {
    fn period(&self) -> usize {
        1 // No warmup needed
    }
}

impl Current for AdLine {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for AdLine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, volume)
    }

    #[test]
    fn test_ad_line_close_at_high() {
        let mut ad = AdLine::new();

        // Close at high = MFM of +1
        let result = ad.next(&make_candle(110.0, 100.0, 110.0, 1000.0));
        assert!(result.is_some());
        assert!((result.unwrap() - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_ad_line_close_at_low() {
        let mut ad = AdLine::new();

        // Close at low = MFM of -1
        let result = ad.next(&make_candle(110.0, 100.0, 100.0, 1000.0));
        assert!(result.is_some());
        assert!((result.unwrap() - (-1000.0)).abs() < 0.01);
    }

    #[test]
    fn test_ad_line_cumulative() {
        let mut ad = AdLine::new();

        ad.next(&make_candle(110.0, 100.0, 110.0, 1000.0)); // +1000
        ad.next(&make_candle(110.0, 100.0, 110.0, 500.0));  // +500

        let value = ad.current().unwrap();
        assert!((value - 1500.0).abs() < 0.01);
    }

    #[test]
    fn test_ad_line_reset() {
        let mut ad = AdLine::new();

        for _ in 0..10 {
            ad.next(&make_candle(110.0, 100.0, 105.0, 1000.0));
        }

        ad.reset();
        assert!(ad.current().is_none());
    }
}
