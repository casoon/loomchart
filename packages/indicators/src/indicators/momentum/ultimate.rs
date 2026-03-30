//! Ultimate Oscillator

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Ultimate Oscillator
///
/// Developed by Larry Williams, the Ultimate Oscillator uses weighted
/// averages of three different time periods to reduce volatility and
/// false signals compared to single timeframe oscillators.
///
/// Formula uses Buying Pressure and True Range over 3 periods.
///
/// Values range from 0 to 100:
/// - > 70: Overbought
/// - < 30: Oversold
///
/// Default periods: 7, 14, 28
#[derive(Clone)]
pub struct UltimateOscillator {
    period1: usize,
    period2: usize,
    period3: usize,
    bp_buffer: Vec<f64>,    // Buying Pressure
    tr_buffer: Vec<f64>,    // True Range
    index: usize,
    count: usize,
    prev_close: Option<f64>,
    current: Option<f64>,
}

impl UltimateOscillator {
    pub fn new(period1: usize, period2: usize, period3: usize) -> Self {
        assert!(period1 > 0 && period2 > period1 && period3 > period2,
                "Periods must be: 0 < period1 < period2 < period3");

        Self {
            period1,
            period2,
            period3,
            bp_buffer: vec![0.0; period3],
            tr_buffer: vec![0.0; period3],
            index: 0,
            count: 0,
            prev_close: None,
            current: None,
        }
    }

    fn sum_range(&self, buffer: &[f64], periods: usize) -> f64 {
        if self.count < periods {
            return 0.0;
        }

        let mut sum = 0.0;
        for i in 0..periods {
            let idx = (self.index + self.period3 - 1 - i) % self.period3;
            sum += buffer[idx];
        }
        sum
    }
}

impl Next<&Ohlcv> for UltimateOscillator {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        let result = if let Some(prev_close) = self.prev_close {
            // Calculate True Range
            let tr = (candle.high - candle.low)
                .max(libm::fabs(candle.high - prev_close))
                .max(libm::fabs(candle.low - prev_close));

            // Calculate Buying Pressure = Close - Min(Low, Previous Close)
            let bp = candle.close - candle.low.min(prev_close);

            // Store values
            self.bp_buffer[self.index] = bp;
            self.tr_buffer[self.index] = tr;

            self.index = (self.index + 1) % self.period3;
            self.count += 1;

            if self.count >= self.period3 {
                let bp1 = self.sum_range(&self.bp_buffer, self.period1);
                let tr1 = self.sum_range(&self.tr_buffer, self.period1);
                let bp2 = self.sum_range(&self.bp_buffer, self.period2);
                let tr2 = self.sum_range(&self.tr_buffer, self.period2);
                let bp3 = self.sum_range(&self.bp_buffer, self.period3);
                let tr3 = self.sum_range(&self.tr_buffer, self.period3);

                let avg1 = if tr1 > 0.0 { bp1 / tr1 } else { 0.0 };
                let avg2 = if tr2 > 0.0 { bp2 / tr2 } else { 0.0 };
                let avg3 = if tr3 > 0.0 { bp3 / tr3 } else { 0.0 };

                // Weights: 4 for shortest, 2 for middle, 1 for longest
                let uo = 100.0 * (4.0 * avg1 + 2.0 * avg2 + avg3) / 7.0;

                self.current = Some(uo);
                Some(uo)
            } else {
                None
            }
        } else {
            None
        };

        self.prev_close = Some(candle.close);
        result
    }
}

impl Reset for UltimateOscillator {
    fn reset(&mut self) {
        self.bp_buffer.fill(0.0);
        self.tr_buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.prev_close = None;
        self.current = None;
    }
}

impl Period for UltimateOscillator {
    fn period(&self) -> usize {
        self.period3
    }
}

impl Current for UltimateOscillator {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for UltimateOscillator {
    fn default() -> Self {
        Self::new(7, 14, 28)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_uo_basic() {
        let mut uo = UltimateOscillator::new(3, 5, 7);

        for i in 0..15 {
            uo.next(&make_candle(100.0 + i as f64, 98.0 + i as f64, 99.0 + i as f64));
        }

        assert!(uo.current().is_some());
        let value = uo.current().unwrap();
        assert!(value >= 0.0 && value <= 100.0);
    }

    #[test]
    fn test_uo_reset() {
        let mut uo = UltimateOscillator::default();

        for i in 0..35 {
            uo.next(&make_candle(100.0 + i as f64, 98.0, 99.0));
        }

        uo.reset();
        assert!(uo.current().is_none());
    }
}
