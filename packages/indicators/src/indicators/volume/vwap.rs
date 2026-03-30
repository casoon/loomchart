//! Volume Weighted Average Price (VWAP)

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;

/// Volume Weighted Average Price (VWAP)
///
/// VWAP is the cumulative average price weighted by volume, typically
/// calculated from the start of a trading session.
///
/// VWAP = Cumulative(Typical Price * Volume) / Cumulative(Volume)
/// Typical Price = (High + Low + Close) / 3
///
/// VWAP is used as a benchmark for trade execution quality and as
/// support/resistance.
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Vwap, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut vwap = Vwap::new();
///
/// for candle in session_candles.iter() {
///     let value = vwap.next(candle);
///     println!("VWAP: {:.4}", value);
/// }
///
/// // Reset at session start
/// vwap.reset();
/// ```
#[derive(Clone)]
pub struct Vwap {
    cumulative_tp_vol: f64,
    cumulative_vol: f64,
    current: Option<f64>,
    count: usize,
}

impl Vwap {
    /// Create a new VWAP indicator.
    pub fn new() -> Self {
        Self {
            cumulative_tp_vol: 0.0,
            cumulative_vol: 0.0,
            current: None,
            count: 0,
        }
    }

    /// Get cumulative volume.
    pub fn cumulative_volume(&self) -> f64 {
        self.cumulative_vol
    }

    /// Get number of periods processed.
    pub fn periods(&self) -> usize {
        self.count
    }

    /// Check if price is above VWAP (bullish).
    pub fn is_above(&self, price: f64) -> bool {
        self.current.map_or(false, |vwap| price > vwap)
    }

    /// Check if price is below VWAP (bearish).
    pub fn is_below(&self, price: f64) -> bool {
        self.current.map_or(false, |vwap| price < vwap)
    }
}

impl Next<&Ohlcv> for Vwap {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let tp = candle.typical_price();

        self.cumulative_tp_vol += tp * candle.volume;
        self.cumulative_vol += candle.volume;
        self.count += 1;

        if self.cumulative_vol.abs() < f64::EPSILON {
            return None;
        }

        let vwap = self.cumulative_tp_vol / self.cumulative_vol;
        self.current = Some(vwap);
        Some(vwap)
    }
}

impl Reset for Vwap {
    fn reset(&mut self) {
        self.cumulative_tp_vol = 0.0;
        self.cumulative_vol = 0.0;
        self.current = None;
        self.count = 0;
    }
}

impl Period for Vwap {
    fn period(&self) -> usize {
        1 // VWAP has no warmup period
    }
}

impl Current for Vwap {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Vwap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    fn make_candle(high: f64, low: f64, close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, volume)
    }

    #[test]
    fn test_vwap_single() {
        let mut vwap = Vwap::new();

        // TP = (110 + 100 + 105) / 3 = 105
        let val = vwap.next(&make_candle(110.0, 100.0, 105.0, 1000.0)).unwrap();

        assert_approx_eq!(val, 105.0);
    }

    #[test]
    fn test_vwap_weighted() {
        let mut vwap = Vwap::new();

        // First candle: TP = 100, Volume = 1000
        vwap.next(&make_candle(101.0, 99.0, 100.0, 1000.0));

        // Second candle: TP = 110, Volume = 2000
        let val = vwap.next(&make_candle(111.0, 109.0, 110.0, 2000.0)).unwrap();

        // VWAP = (100*1000 + 110*2000) / (1000 + 2000)
        //      = (100000 + 220000) / 3000 = 320000 / 3000 = 106.666...
        assert_approx_eq!(val, 106.666667, 0.001);
    }

    #[test]
    fn test_vwap_reset() {
        let mut vwap = Vwap::new();

        vwap.next(&make_candle(110.0, 100.0, 105.0, 1000.0));
        vwap.next(&make_candle(115.0, 105.0, 110.0, 1000.0));

        vwap.reset();

        assert!(vwap.current().is_none());
        assert_eq!(vwap.cumulative_volume(), 0.0);
        assert_eq!(vwap.periods(), 0);
    }
}
