//! Chaikin Money Flow (CMF)

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;
use crate::math::momentum::money_flow_multiplier;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Chaikin Money Flow (CMF)
///
/// Developed by Marc Chaikin, CMF measures the amount of money flow
/// volume over a specific period. It's a bounded oscillator between -1 and +1.
///
/// Formula:
/// CMF = Sum(Money Flow Volume, n) / Sum(Volume, n)
/// where Money Flow Volume = Money Flow Multiplier * Volume
///
/// Values:
/// - > 0: Buying pressure
/// - < 0: Selling pressure
/// - > 0.25: Strong buying
/// - < -0.25: Strong selling
///
/// Default period: 20
#[derive(Clone)]
pub struct Cmf {
    period: usize,
    mfv_buffer: Vec<f64>,
    vol_buffer: Vec<f64>,
    index: usize,
    count: usize,
    mfv_sum: f64,
    vol_sum: f64,
    current: Option<f64>,
}

impl Cmf {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            mfv_buffer: vec![0.0; period],
            vol_buffer: vec![0.0; period],
            index: 0,
            count: 0,
            mfv_sum: 0.0,
            vol_sum: 0.0,
            current: None,
        }
    }
}

impl Next<&Ohlcv> for Cmf {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let mfm = money_flow_multiplier(candle.high, candle.low, candle.close);
        let mfv = mfm * candle.volume;

        // Remove old values
        if self.count >= self.period {
            self.mfv_sum -= self.mfv_buffer[self.index];
            self.vol_sum -= self.vol_buffer[self.index];
        }

        // Store new values
        self.mfv_buffer[self.index] = mfv;
        self.vol_buffer[self.index] = candle.volume;
        self.mfv_sum += mfv;
        self.vol_sum += candle.volume;

        self.index = (self.index + 1) % self.period;
        if self.count < self.period {
            self.count += 1;
        }

        if self.count >= self.period {
            let cmf = if self.vol_sum > 0.0 {
                self.mfv_sum / self.vol_sum
            } else {
                0.0
            };
            self.current = Some(cmf);
            Some(cmf)
        } else {
            None
        }
    }
}

impl Reset for Cmf {
    fn reset(&mut self) {
        self.mfv_buffer.fill(0.0);
        self.vol_buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.mfv_sum = 0.0;
        self.vol_sum = 0.0;
        self.current = None;
    }
}

impl Period for Cmf {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Cmf {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Cmf {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, volume)
    }

    #[test]
    fn test_cmf_all_buying() {
        let mut cmf = Cmf::new(5);

        // All closes at high = all buying
        for _ in 0..5 {
            cmf.next(&make_candle(110.0, 100.0, 110.0, 1000.0));
        }

        let value = cmf.current().unwrap();
        assert!((value - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cmf_all_selling() {
        let mut cmf = Cmf::new(5);

        // All closes at low = all selling
        for _ in 0..5 {
            cmf.next(&make_candle(110.0, 100.0, 100.0, 1000.0));
        }

        let value = cmf.current().unwrap();
        assert!((value - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_cmf_neutral() {
        let mut cmf = Cmf::new(5);

        // Close at midpoint = neutral
        for _ in 0..5 {
            cmf.next(&make_candle(110.0, 100.0, 105.0, 1000.0));
        }

        let value = cmf.current().unwrap();
        assert!(value.abs() < 0.01);
    }

    #[test]
    fn test_cmf_reset() {
        let mut cmf = Cmf::default();

        for _ in 0..25 {
            cmf.next(&make_candle(110.0, 100.0, 108.0, 1000.0));
        }

        cmf.reset();
        assert!(cmf.current().is_none());
    }
}
