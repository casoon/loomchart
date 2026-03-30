//! Keltner Channel

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Ema;
use crate::indicators::volatility::Atr;
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Keltner Channel Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeltnerOutput {
    /// Upper band (EMA + multiplier * ATR)
    pub upper: f64,
    /// Middle band (EMA)
    pub middle: f64,
    /// Lower band (EMA - multiplier * ATR)
    pub lower: f64,
}

impl KeltnerOutput {
    pub fn new(upper: f64, middle: f64, lower: f64) -> Self {
        Self { upper, middle, lower }
    }

    /// Check if price is above upper band (overbought/breakout)
    pub fn is_above_upper(&self, price: f64) -> bool {
        price > self.upper
    }

    /// Check if price is below lower band (oversold/breakdown)
    pub fn is_below_lower(&self, price: f64) -> bool {
        price < self.lower
    }

    /// Get position within channel (0 = lower, 1 = upper)
    pub fn percent_b(&self, price: f64) -> f64 {
        let width = self.upper - self.lower;
        if width > 0.0 {
            (price - self.lower) / width
        } else {
            0.5
        }
    }
}

/// Keltner Channel
///
/// A volatility-based envelope set above and below an EMA.
/// Uses ATR to set channel width, making it adaptive to volatility.
///
/// Calculation:
/// - Middle = EMA(close, period)
/// - Upper = Middle + multiplier * ATR(period)
/// - Lower = Middle - multiplier * ATR(period)
///
/// Trading signals:
/// - Price above upper: Strong uptrend or overbought
/// - Price below lower: Strong downtrend or oversold
/// - Narrow channel: Low volatility, breakout expected
///
/// Default: period=20, multiplier=2.0
#[derive(Clone)]
pub struct KeltnerChannel {
    period: usize,
    multiplier: f64,
    ema: Ema,
    atr: Atr,
    current: Option<KeltnerOutput>,
}

impl KeltnerChannel {
    pub fn new(period: usize, multiplier: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(multiplier > 0.0, "Multiplier must be greater than 0");

        Self {
            period,
            multiplier,
            ema: Ema::new(period),
            atr: Atr::new(period),
            current: None,
        }
    }
}

impl Next<&Ohlcv> for KeltnerChannel {
    type Output = KeltnerOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<KeltnerOutput> {
        let ema_val = self.ema.next(candle.close);
        let atr_val = self.atr.next(candle);

        match (ema_val, atr_val) {
            (Some(middle), Some(atr)) => {
                let offset = self.multiplier * atr;
                let output = KeltnerOutput::new(
                    middle + offset,
                    middle,
                    middle - offset,
                );
                self.current = Some(output);
                Some(output)
            }
            _ => None,
        }
    }
}

impl Reset for KeltnerChannel {
    fn reset(&mut self) {
        self.ema.reset();
        self.atr.reset();
        self.current = None;
    }
}

impl Period for KeltnerChannel {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for KeltnerChannel {
    type Output = KeltnerOutput;

    fn current(&self) -> Option<KeltnerOutput> {
        self.current
    }
}

impl Default for KeltnerChannel {
    fn default() -> Self {
        Self::new(20, 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(open, high, low, close, 1000.0)
    }

    #[test]
    fn test_keltner_basic() {
        let mut kc = KeltnerChannel::new(5, 2.0);

        // Need enough data for both EMA and ATR
        for i in 0..10 {
            kc.next(&make_candle(
                100.0 + i as f64,
                102.0 + i as f64,
                98.0 + i as f64,
                101.0 + i as f64,
            ));
        }

        let output = kc.current().unwrap();
        assert!(output.upper > output.middle);
        assert!(output.middle > output.lower);
    }

    #[test]
    fn test_keltner_percent_b() {
        let output = KeltnerOutput::new(110.0, 100.0, 90.0);

        assert!((output.percent_b(100.0) - 0.5).abs() < 0.01);
        assert!((output.percent_b(90.0) - 0.0).abs() < 0.01);
        assert!((output.percent_b(110.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_keltner_reset() {
        let mut kc = KeltnerChannel::new(5, 2.0);

        for i in 0..10 {
            kc.next(&make_candle(100.0, 102.0 + i as f64, 98.0, 101.0));
        }

        kc.reset();
        assert!(kc.current().is_none());
    }
}
