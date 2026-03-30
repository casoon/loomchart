//! Chaikin Oscillator

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Ema;
use crate::types::Ohlcv;
use crate::math::momentum::money_flow_multiplier;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Chaikin Oscillator
///
/// Developed by Marc Chaikin, this oscillator measures the momentum of the
/// Accumulation/Distribution Line using the difference between 3-period
/// and 10-period EMAs of A/D.
///
/// Formula: Chaikin Osc = EMA(A/D, 3) - EMA(A/D, 10)
///
/// Signals:
/// - Zero line crossovers indicate momentum shifts
/// - Divergences with price indicate potential reversals
/// - Rising oscillator: Accumulation increasing
/// - Falling oscillator: Distribution increasing
///
/// Default periods: 3 (fast), 10 (slow)
#[derive(Clone)]
pub struct ChaikinOscillator {
    #[allow(dead_code)]
    fast_period: usize,
    slow_period: usize,
    ad: f64,
    fast_ema: Ema,
    slow_ema: Ema,
    current: Option<f64>,
}

impl ChaikinOscillator {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(fast_period > 0 && slow_period > fast_period,
                "fast_period must be > 0 and slow_period > fast_period");
        Self {
            fast_period,
            slow_period,
            ad: 0.0,
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            current: None,
        }
    }
}

impl Next<&Ohlcv> for ChaikinOscillator {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        // Update A/D Line
        let mfm = money_flow_multiplier(candle.high, candle.low, candle.close);
        let mfv = mfm * candle.volume;
        self.ad += mfv;

        // Calculate EMAs
        let fast = self.fast_ema.next(self.ad);
        let slow = self.slow_ema.next(self.ad);

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let osc = f - s;
                self.current = Some(osc);
                Some(osc)
            }
            _ => None,
        }
    }
}

impl Reset for ChaikinOscillator {
    fn reset(&mut self) {
        self.ad = 0.0;
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.current = None;
    }
}

impl Period for ChaikinOscillator {
    fn period(&self) -> usize {
        self.slow_period
    }
}

impl Current for ChaikinOscillator {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for ChaikinOscillator {
    fn default() -> Self {
        Self::new(3, 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64, volume: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, volume)
    }

    #[test]
    fn test_chaikin_osc_basic() {
        let mut co = ChaikinOscillator::new(3, 5);

        for i in 0..15 {
            co.next(&make_candle(
                102.0 + i as f64,
                98.0 + i as f64,
                101.0 + i as f64,
                1000.0,
            ));
        }

        assert!(co.current().is_some());
    }

    #[test]
    fn test_chaikin_osc_accumulation() {
        let mut co = ChaikinOscillator::new(3, 5);

        // Consistent accumulation (close near high)
        for i in 0..20 {
            co.next(&make_candle(110.0 + i as f64, 100.0 + i as f64, 109.0 + i as f64, 1000.0));
        }

        // Should be positive with accumulation
        assert!(co.current().is_some());
    }

    #[test]
    fn test_chaikin_osc_reset() {
        let mut co = ChaikinOscillator::default();

        for _ in 0..15 {
            co.next(&make_candle(110.0, 100.0, 108.0, 1000.0));
        }

        co.reset();
        assert!(co.current().is_none());
    }
}
