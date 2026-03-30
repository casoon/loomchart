//! Parabolic SAR (Stop and Reverse)

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Parabolic SAR Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SarOutput {
    /// SAR value
    pub sar: f64,
    /// Is currently in uptrend
    pub is_uptrend: bool,
}

impl SarOutput {
    pub fn new(sar: f64, is_uptrend: bool) -> Self {
        Self { sar, is_uptrend }
    }
}

/// Parabolic SAR (Stop and Reverse)
///
/// Developed by J. Welles Wilder, Parabolic SAR provides potential entry
/// and exit points. The indicator trails price as the trend extends.
///
/// Parameters:
/// - af_start: Initial acceleration factor (default 0.02)
/// - af_step: AF increment (default 0.02)
/// - af_max: Maximum AF (default 0.20)
///
/// Trading signals:
/// - SAR below price: Uptrend, potential long
/// - SAR above price: Downtrend, potential short
/// - SAR flip: Trend reversal signal
#[derive(Clone)]
pub struct ParabolicSar {
    af_start: f64,
    af_step: f64,
    af_max: f64,
    is_uptrend: bool,
    sar: f64,
    ep: f64,  // Extreme point
    af: f64,  // Current acceleration factor
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    count: usize,
    current: Option<SarOutput>,
}

impl ParabolicSar {
    pub fn new(af_start: f64, af_step: f64, af_max: f64) -> Self {
        assert!(af_start > 0.0 && af_start < 1.0, "af_start must be between 0 and 1");
        assert!(af_step > 0.0 && af_step < 1.0, "af_step must be between 0 and 1");
        assert!(af_max > af_start && af_max <= 1.0, "af_max must be > af_start and <= 1");

        Self {
            af_start,
            af_step,
            af_max,
            is_uptrend: true,
            sar: 0.0,
            ep: 0.0,
            af: af_start,
            prev_high: None,
            prev_low: None,
            count: 0,
            current: None,
        }
    }

    fn update_af(&mut self, new_ep: f64) {
        if (self.is_uptrend && new_ep > self.ep) || (!self.is_uptrend && new_ep < self.ep) {
            self.af = (self.af + self.af_step).min(self.af_max);
            self.ep = new_ep;
        }
    }
}

impl Next<&Ohlcv> for ParabolicSar {
    type Output = SarOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<SarOutput> {
        self.count += 1;

        if self.count == 1 {
            // Initialize on first candle
            self.prev_high = Some(candle.high);
            self.prev_low = Some(candle.low);
            return None;
        }

        if self.count == 2 {
            // Initialize trend direction based on first two candles
            let prev_high = self.prev_high.unwrap();
            let prev_low = self.prev_low.unwrap();

            if candle.close > (prev_high + prev_low) / 2.0 {
                self.is_uptrend = true;
                self.sar = prev_low;
                self.ep = candle.high;
            } else {
                self.is_uptrend = false;
                self.sar = prev_high;
                self.ep = candle.low;
            }

            self.af = self.af_start;
            self.prev_high = Some(candle.high);
            self.prev_low = Some(candle.low);

            let output = SarOutput::new(self.sar, self.is_uptrend);
            self.current = Some(output);
            return Some(output);
        }

        let prev_high = self.prev_high.unwrap();
        let prev_low = self.prev_low.unwrap();

        // Calculate new SAR
        let mut new_sar = self.sar + self.af * (self.ep - self.sar);

        if self.is_uptrend {
            // SAR cannot be above the prior two lows
            new_sar = new_sar.min(prev_low).min(
                if self.count > 2 { prev_low } else { candle.low }
            );

            // Check for reversal
            if candle.low < new_sar {
                // Switch to downtrend
                self.is_uptrend = false;
                new_sar = self.ep; // SAR becomes the EP
                self.ep = candle.low;
                self.af = self.af_start;
            } else {
                // Continue uptrend
                self.update_af(candle.high);
            }
        } else {
            // SAR cannot be below the prior two highs
            new_sar = new_sar.max(prev_high).max(
                if self.count > 2 { prev_high } else { candle.high }
            );

            // Check for reversal
            if candle.high > new_sar {
                // Switch to uptrend
                self.is_uptrend = true;
                new_sar = self.ep; // SAR becomes the EP
                self.ep = candle.high;
                self.af = self.af_start;
            } else {
                // Continue downtrend
                self.update_af(candle.low);
            }
        }

        self.sar = new_sar;
        self.prev_high = Some(candle.high);
        self.prev_low = Some(candle.low);

        let output = SarOutput::new(self.sar, self.is_uptrend);
        self.current = Some(output);
        Some(output)
    }
}

impl Reset for ParabolicSar {
    fn reset(&mut self) {
        self.is_uptrend = true;
        self.sar = 0.0;
        self.ep = 0.0;
        self.af = self.af_start;
        self.prev_high = None;
        self.prev_low = None;
        self.count = 0;
        self.current = None;
    }
}

impl Period for ParabolicSar {
    fn period(&self) -> usize {
        2 // Needs 2 candles to start
    }
}

impl Current for ParabolicSar {
    type Output = SarOutput;

    fn current(&self) -> Option<SarOutput> {
        self.current
    }
}

impl Default for ParabolicSar {
    fn default() -> Self {
        Self::new(0.02, 0.02, 0.20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_sar_uptrend() {
        let mut sar = ParabolicSar::default();

        // First two candles establish trend
        sar.next(&make_candle(102.0, 98.0, 100.0));
        let out = sar.next(&make_candle(105.0, 99.0, 104.0));

        assert!(out.is_some());
        let output = out.unwrap();
        assert!(output.is_uptrend);
    }

    #[test]
    fn test_sar_reversal() {
        let mut sar = ParabolicSar::default();

        // Establish uptrend
        sar.next(&make_candle(100.0, 95.0, 98.0));
        sar.next(&make_candle(105.0, 99.0, 104.0));

        // Continue uptrend
        for _ in 0..5 {
            sar.next(&make_candle(110.0, 105.0, 108.0));
        }

        // Force reversal with sharp drop
        let output = sar.next(&make_candle(102.0, 90.0, 91.0));
        assert!(output.is_some());
    }

    #[test]
    fn test_sar_reset() {
        let mut sar = ParabolicSar::default();

        for i in 0..10 {
            sar.next(&make_candle(100.0 + i as f64, 95.0, 98.0));
        }

        sar.reset();
        assert!(sar.current().is_none());
    }
}
