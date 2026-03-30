//! Average True Range (ATR)

use crate::indicators::{Current, Next, Period, Reset};
use crate::math::range::{true_range, true_range_first};
use crate::types::Ohlcv;

/// Average True Range (ATR)
///
/// ATR measures market volatility by decomposing the entire range of an
/// asset price for that period.
///
/// True Range = max(High - Low, |High - Prev Close|, |Low - Prev Close|)
/// ATR = Wilder's Smoothed Average of True Range
///
/// Default period: 14
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Atr, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut atr = Atr::new(14);
///
/// for candle in candles.iter() {
///     if let Some(value) = atr.next(candle) {
///         println!("ATR: {:.4}", value);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Atr {
    period: usize,
    prev_close: Option<f64>,
    sum_tr: f64,
    count: usize,
    is_initialized: bool,
    current: Option<f64>,
}

impl Atr {
    /// Create a new ATR with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            prev_close: None,
            sum_tr: 0.0,
            count: 0,
            is_initialized: false,
            current: None,
        }
    }

    /// Get the last calculated True Range value.
    pub fn last_true_range(&self, candle: &Ohlcv) -> f64 {
        match self.prev_close {
            Some(prev) => true_range(candle.high, candle.low, prev),
            None => true_range_first(candle.high, candle.low),
        }
    }
}

impl Next<&Ohlcv> for Atr {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        // Calculate True Range
        let tr = match self.prev_close {
            Some(prev) => true_range(candle.high, candle.low, prev),
            None => true_range_first(candle.high, candle.low),
        };

        self.prev_close = Some(candle.close);

        let result = if !self.is_initialized {
            // Accumulating for first SMA
            self.sum_tr += tr;
            self.count += 1;

            if self.count >= self.period {
                // First ATR: use SMA of True Range
                let atr_val = self.sum_tr / self.period as f64;
                self.current = Some(atr_val);
                self.is_initialized = true;
                Some(atr_val)
            } else {
                None
            }
        } else {
            // Wilder's smoothing: ATR = ((prev_ATR * (n-1)) + TR) / n
            let prev_atr = self.current.unwrap();
            let p = self.period as f64;
            let atr_val = (prev_atr * (p - 1.0) + tr) / p;
            self.current = Some(atr_val);
            Some(atr_val)
        };

        result
    }
}

impl Reset for Atr {
    fn reset(&mut self) {
        self.prev_close = None;
        self.sum_tr = 0.0;
        self.count = 0;
        self.is_initialized = false;
        self.current = None;
    }
}

impl Period for Atr {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Atr {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Atr {
    fn default() -> Self {
        Self::new(14)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_atr_warmup() {
        let mut atr = Atr::new(3);

        assert!(atr.next(&make_candle(110.0, 100.0, 105.0)).is_none());
        assert!(atr.next(&make_candle(115.0, 103.0, 110.0)).is_none());
        assert!(atr.next(&make_candle(112.0, 105.0, 108.0)).is_some());
    }

    #[test]
    fn test_atr_constant_range() {
        let mut atr = Atr::new(3);

        // All candles have range of 10
        atr.next(&make_candle(110.0, 100.0, 105.0));
        atr.next(&make_candle(115.0, 105.0, 110.0));
        let val = atr.next(&make_candle(120.0, 110.0, 115.0)).unwrap();

        // ATR should be around 10 (TR = 10 for each)
        assert_approx_eq!(val, 10.0, 0.1);
    }

    #[test]
    fn test_atr_with_gap() {
        let mut atr = Atr::new(3);

        // First candle
        atr.next(&make_candle(110.0, 100.0, 105.0));

        // Gap up: prev close was 105, new low is 115
        // TR = max(120-115=5, |120-105|=15, |115-105|=10) = 15
        atr.next(&make_candle(120.0, 115.0, 118.0));

        // Continue
        let val = atr.next(&make_candle(125.0, 116.0, 122.0)).unwrap();

        // First ATR will be average of the three TRs
        assert!(val > 0.0);
    }

    #[test]
    fn test_atr_reset() {
        let mut atr = Atr::new(3);

        atr.next(&make_candle(110.0, 100.0, 105.0));
        atr.next(&make_candle(115.0, 105.0, 110.0));
        atr.next(&make_candle(120.0, 110.0, 115.0));

        atr.reset();

        assert!(atr.current().is_none());
        assert!(!atr.is_initialized);
    }
}
