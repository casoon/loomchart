//! Money Flow Index (MFI)

use crate::indicators::{Current, Next, Period, Reset};
use crate::math::momentum::{mfi as calc_mfi, raw_money_flow, typical_price};
use crate::types::Ohlcv;

/// Money Flow Index (MFI)
///
/// MFI is a volume-weighted RSI that uses both price and volume to measure
/// buying and selling pressure.
///
/// Calculation:
/// 1. Typical Price = (High + Low + Close) / 3
/// 2. Raw Money Flow = Typical Price * Volume
/// 3. Positive/Negative Money Flow based on TP change
/// 4. Money Ratio = Positive MF / Negative MF
/// 5. MFI = 100 - (100 / (1 + Money Ratio))
///
/// Values:
/// - 0-20: Oversold
/// - 20-80: Neutral
/// - 80-100: Overbought
///
/// Default period: 14
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Mfi, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut mfi = Mfi::new(14);
///
/// for candle in candles.iter() {
///     if let Some(value) = mfi.next(candle) {
///         println!("MFI: {:.2}", value);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Mfi {
    period: usize,
    prev_tp: Option<f64>,
    pos_mf_buffer: Vec<f64>,
    neg_mf_buffer: Vec<f64>,
    index: usize,
    count: usize,
    pos_mf_sum: f64,
    neg_mf_sum: f64,
    current: Option<f64>,
}

impl Mfi {
    /// Create a new MFI with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            prev_tp: None,
            pos_mf_buffer: vec![0.0; period],
            neg_mf_buffer: vec![0.0; period],
            index: 0,
            count: 0,
            pos_mf_sum: 0.0,
            neg_mf_sum: 0.0,
            current: None,
        }
    }

    /// Check if MFI indicates overbought (> 80).
    pub fn is_overbought(&self) -> bool {
        self.current.map_or(false, |v| v > 80.0)
    }

    /// Check if MFI indicates oversold (< 20).
    pub fn is_oversold(&self) -> bool {
        self.current.map_or(false, |v| v < 20.0)
    }
}

impl Next<&Ohlcv> for Mfi {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let tp = typical_price(candle.high, candle.low, candle.close);
        let raw_mf = raw_money_flow(candle.high, candle.low, candle.close, candle.volume);

        let result = if let Some(prev) = self.prev_tp {
            // Determine positive or negative money flow
            let (pos_mf, neg_mf) = if tp > prev {
                (raw_mf, 0.0)
            } else if tp < prev {
                (0.0, raw_mf)
            } else {
                (0.0, 0.0)
            };

            // Subtract old values being replaced
            if self.count >= self.period {
                self.pos_mf_sum -= self.pos_mf_buffer[self.index];
                self.neg_mf_sum -= self.neg_mf_buffer[self.index];
            }

            // Add new values
            self.pos_mf_buffer[self.index] = pos_mf;
            self.neg_mf_buffer[self.index] = neg_mf;
            self.pos_mf_sum += pos_mf;
            self.neg_mf_sum += neg_mf;

            // Advance index
            self.index = (self.index + 1) % self.period;

            if self.count < self.period {
                self.count += 1;
            }

            // Calculate MFI if we have enough data
            if self.count >= self.period {
                let mfi_val = calc_mfi(self.pos_mf_sum, self.neg_mf_sum);
                self.current = Some(mfi_val);
                Some(mfi_val)
            } else {
                None
            }
        } else {
            // First candle, no MF calculation yet
            None
        };

        self.prev_tp = Some(tp);
        result
    }
}

impl Reset for Mfi {
    fn reset(&mut self) {
        self.prev_tp = None;
        self.pos_mf_buffer.fill(0.0);
        self.neg_mf_buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.pos_mf_sum = 0.0;
        self.neg_mf_sum = 0.0;
        self.current = None;
    }
}

impl Period for Mfi {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Mfi {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Mfi {
    fn default() -> Self {
        Self::new(14)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    fn make_candle(high: f64, low: f64, close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(low, high, low, close, volume)
    }

    #[test]
    fn test_mfi_warmup() {
        let mut mfi = Mfi::new(3);

        // Need period + 1 candles
        assert!(mfi.next(&make_candle(102.0, 100.0, 101.0, 1000.0)).is_none());
        assert!(mfi.next(&make_candle(104.0, 101.0, 103.0, 1000.0)).is_none());
        assert!(mfi.next(&make_candle(105.0, 102.0, 104.0, 1000.0)).is_none());
        assert!(mfi.next(&make_candle(106.0, 103.0, 105.0, 1000.0)).is_some());
    }

    #[test]
    fn test_mfi_all_positive() {
        let mut mfi = Mfi::new(3);

        // All rising typical prices -> all positive money flow
        mfi.next(&make_candle(102.0, 100.0, 101.0, 1000.0));
        mfi.next(&make_candle(104.0, 102.0, 103.0, 1000.0));
        mfi.next(&make_candle(106.0, 104.0, 105.0, 1000.0));
        let val = mfi.next(&make_candle(108.0, 106.0, 107.0, 1000.0)).unwrap();

        // All positive -> MFI = 100
        assert_approx_eq!(val, 100.0);
    }

    #[test]
    fn test_mfi_all_negative() {
        let mut mfi = Mfi::new(3);

        // All falling typical prices -> all negative money flow
        mfi.next(&make_candle(108.0, 106.0, 107.0, 1000.0));
        mfi.next(&make_candle(106.0, 104.0, 105.0, 1000.0));
        mfi.next(&make_candle(104.0, 102.0, 103.0, 1000.0));
        let val = mfi.next(&make_candle(102.0, 100.0, 101.0, 1000.0)).unwrap();

        // All negative -> MFI = 0
        assert_approx_eq!(val, 0.0);
    }
}
