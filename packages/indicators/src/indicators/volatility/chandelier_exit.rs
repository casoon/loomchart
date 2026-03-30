//! Chandelier Exit

use crate::indicators::volatility::Atr;
use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Chandelier Exit Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ChandelierExitOutput {
    /// Long exit level (trailing stop for long positions)
    pub long_exit: f64,
    /// Short exit level (trailing stop for short positions)
    pub short_exit: f64,
}

impl ChandelierExitOutput {
    pub fn new(long_exit: f64, short_exit: f64) -> Self {
        Self {
            long_exit,
            short_exit,
        }
    }
}

/// Chandelier Exit
///
/// A volatility-based trailing stop indicator that "hangs" from the highest high
/// (for longs) or lowest low (for shorts) like a chandelier from a ceiling.
///
/// Developed by Chuck LeBeau, it uses ATR to set stops at a multiple of
/// volatility away from the high/low.
///
/// # Formula
///
/// ```text
/// Long Exit = Highest High(n) - ATR(n) * Multiplier
/// Short Exit = Lowest Low(n) + ATR(n) * Multiplier
/// ```
///
/// Where:
/// - Highest High(n) = Maximum high over n periods
/// - Lowest Low(n) = Minimum low over n periods
/// - ATR(n) = Average True Range over n periods
/// - Multiplier = Typically 3.0
///
/// # Interpretation
///
/// - **Long positions**: Exit when price closes below Long Exit
/// - **Short positions**: Exit when price closes above Short Exit
/// - **Higher multiplier**: Wider stops, fewer exits
/// - **Lower multiplier**: Tighter stops, more exits
///
/// # Trading Signals
///
/// - Price above Long Exit: Hold long position
/// - Price below Long Exit: Exit long (potential trend reversal)
/// - Price below Short Exit: Hold short position
/// - Price above Short Exit: Exit short (potential trend reversal)
///
/// # Parameters
///
/// * `period` - Lookback period for ATR and highs/lows (default: 22)
/// * `multiplier` - ATR multiplier (default: 3.0)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::{ChandelierExit, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut ce = ChandelierExit::new(22, 3.0);
///
/// for candle in candles.iter() {
///     if let Some(output) = ce.next(candle) {
///         println!("Long Exit: {:.2}, Short Exit: {:.2}",
///                  output.long_exit, output.short_exit);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct ChandelierExit {
    period: usize,
    multiplier: f64,
    atr: Atr,
    highs: VecDeque<f64>,
    lows: VecDeque<f64>,
    current: Option<ChandelierExitOutput>,
}

impl ChandelierExit {
    /// Create a new Chandelier Exit indicator
    ///
    /// # Arguments
    ///
    /// * `period` - Lookback period for ATR and highs/lows
    /// * `multiplier` - ATR multiplier for exit levels
    pub fn new(period: usize, multiplier: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(multiplier > 0.0, "Multiplier must be greater than 0");

        Self {
            period,
            multiplier,
            atr: Atr::new(period),
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            current: None,
        }
    }

    /// Find the highest high in the period
    fn highest_high(&self) -> f64 {
        self.highs
            .iter()
            .fold(f64::NEG_INFINITY, |max, &h| max.max(h))
    }

    /// Find the lowest low in the period
    fn lowest_low(&self) -> f64 {
        self.lows.iter().fold(f64::INFINITY, |min, &l| min.min(l))
    }
}

impl Period for ChandelierExit {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<&Ohlcv> for ChandelierExit {
    type Output = ChandelierExitOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        // Update highs and lows
        self.highs.push_back(candle.high);
        self.lows.push_back(candle.low);

        if self.highs.len() > self.period {
            self.highs.pop_front();
        }
        if self.lows.len() > self.period {
            self.lows.pop_front();
        }

        // Calculate ATR
        let atr_opt = self.atr.next(candle);

        // Need both ATR and enough data
        if let Some(atr) = atr_opt {
            if self.highs.len() >= self.period && self.lows.len() >= self.period {
                let highest = self.highest_high();
                let lowest = self.lowest_low();

                let long_exit = highest - (atr * self.multiplier);
                let short_exit = lowest + (atr * self.multiplier);

                let output = ChandelierExitOutput::new(long_exit, short_exit);
                self.current = Some(output);
                return Some(output);
            }
        }

        None
    }
}

impl Next<Ohlcv> for ChandelierExit {
    type Output = ChandelierExitOutput;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for ChandelierExit {
    type Output = ChandelierExitOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for ChandelierExit {
    fn reset(&mut self) {
        self.atr.reset();
        self.highs.clear();
        self.lows.clear();
        self.current = None;
    }
}

impl Default for ChandelierExit {
    fn default() -> Self {
        Self::new(22, 3.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_chandelier_exit_new() {
        let ce = ChandelierExit::new(22, 3.0);
        assert_eq!(ce.period(), 22);
        assert_eq!(ce.multiplier, 3.0);
    }

    #[test]
    #[should_panic]
    fn test_chandelier_exit_invalid_period() {
        ChandelierExit::new(0, 3.0);
    }

    #[test]
    #[should_panic]
    fn test_chandelier_exit_invalid_multiplier() {
        ChandelierExit::new(22, 0.0);
    }

    #[test]
    fn test_chandelier_exit_uptrend() {
        let mut ce = ChandelierExit::new(10, 3.0);

        // Uptrend - increasing highs
        for i in 0..25 {
            let base = 100.0 + i as f64;
            ce.next(&candle(base + 5.0, base, base + 2.0));
        }

        let output = ce.current().unwrap();

        // Long exit should trail below price
        assert!(output.long_exit > 0.0);
        assert!(output.long_exit < 125.0); // Below recent prices

        // Short exit should be above
        assert!(output.short_exit > output.long_exit);
    }

    #[test]
    fn test_chandelier_exit_downtrend() {
        let mut ce = ChandelierExit::new(10, 3.0);

        // Downtrend - decreasing lows
        for i in 0..25 {
            let base = 150.0 - i as f64;
            ce.next(&candle(base, base - 5.0, base - 2.0));
        }

        let output = ce.current().unwrap();

        // Short exit should trail above price
        assert!(output.short_exit > 0.0);

        // Long exit should be below short exit
        assert!(output.long_exit < output.short_exit);
    }

    #[test]
    fn test_chandelier_exit_calculation() {
        let mut ce = ChandelierExit::new(5, 2.0);

        // Known sequence
        let candles = vec![
            candle(105.0, 95.0, 100.0),
            candle(110.0, 98.0, 105.0),
            candle(108.0, 96.0, 102.0),
            candle(112.0, 100.0, 108.0),
            candle(115.0, 102.0, 110.0),
        ];

        let mut result = None;
        for c in candles {
            result = ce.next(&c);
        }

        assert!(result.is_some());
        let output = result.unwrap();

        // Highest high = 115.0
        // Lowest low = 95.0
        // ATR will be calculated based on true ranges
        // Long Exit = 115.0 - (ATR * 2.0)
        // Short Exit = 95.0 + (ATR * 2.0)

        assert!(output.long_exit < 115.0);
        assert!(output.short_exit > 95.0);
        assert!(output.long_exit < output.short_exit);
    }

    #[test]
    fn test_chandelier_exit_multiplier_effect() {
        let mut ce_tight = ChandelierExit::new(10, 1.0);
        let mut ce_wide = ChandelierExit::new(10, 5.0);

        let candles: Vec<Ohlcv> = (0..25)
            .map(|i| {
                let base = 100.0 + i as f64;
                candle(base + 5.0, base, base + 2.0)
            })
            .collect();

        for c in candles {
            ce_tight.next(&c);
            ce_wide.next(&c);
        }

        let tight = ce_tight.current().unwrap();
        let wide = ce_wide.current().unwrap();

        // Wider multiplier should give wider stops
        // Long exit should be lower (further from high)
        assert!(wide.long_exit < tight.long_exit);

        // Short exit should be higher (further from low)
        assert!(wide.short_exit > tight.short_exit);
    }

    #[test]
    fn test_chandelier_exit_range() {
        let mut ce = ChandelierExit::new(10, 3.0);

        // Sideways range
        for _ in 0..25 {
            ce.next(&candle(105.0, 95.0, 100.0));
        }

        let output = ce.current().unwrap();

        // In a tight range, exits should be relatively close
        let range = output.short_exit - output.long_exit;
        assert!(range > 0.0);
    }

    #[test]
    fn test_chandelier_exit_highest_lowest() {
        let ce = ChandelierExit::new(5, 3.0);

        let mut test_ce = ce.clone();
        test_ce.highs.push_back(100.0);
        test_ce.highs.push_back(105.0);
        test_ce.highs.push_back(103.0);
        test_ce.lows.push_back(90.0);
        test_ce.lows.push_back(88.0);
        test_ce.lows.push_back(92.0);

        assert_eq!(test_ce.highest_high(), 105.0);
        assert_eq!(test_ce.lowest_low(), 88.0);
    }

    #[test]
    fn test_chandelier_exit_reset() {
        let mut ce = ChandelierExit::new(10, 3.0);

        for i in 0..25 {
            let base = 100.0 + i as f64;
            ce.next(&candle(base + 5.0, base, base + 2.0));
        }

        ce.reset();

        assert!(ce.current().is_none());
        assert_eq!(ce.highs.len(), 0);
        assert_eq!(ce.lows.len(), 0);
    }

    #[test]
    fn test_chandelier_exit_default() {
        let ce = ChandelierExit::default();
        assert_eq!(ce.period(), 22);
        assert_eq!(ce.multiplier, 3.0);
    }

    #[test]
    fn test_chandelier_exit_insufficient_data() {
        let mut ce = ChandelierExit::new(10, 3.0);

        for i in 0..9 {
            let base = 100.0 + i as f64;
            let result = ce.next(&candle(base + 5.0, base, base + 2.0));
            assert!(result.is_none(), "Should be None before period complete");
        }

        // 10th value should return Some
        let result = ce.next(&candle(115.0, 110.0, 112.0));
        assert!(result.is_some(), "Should return Some at period");
    }

    #[test]
    fn test_chandelier_exit_trailing() {
        let mut ce = ChandelierExit::new(10, 3.0);

        // Build up initial data
        for i in 0..15 {
            let base = 100.0 + i as f64;
            ce.next(&candle(base + 5.0, base, base + 2.0));
        }

        let before = ce.current().unwrap();

        // Continue uptrend
        for i in 15..20 {
            let base = 100.0 + i as f64;
            ce.next(&candle(base + 5.0, base, base + 2.0));
        }

        let after = ce.current().unwrap();

        // Long exit should trail up in uptrend
        assert!(after.long_exit > before.long_exit);
    }

    #[test]
    fn test_chandelier_exit_rolling_window() {
        let mut ce = ChandelierExit::new(5, 3.0);

        for i in 0..10 {
            let base = 100.0 + i as f64;
            ce.next(&candle(base + 5.0, base, base + 2.0));
        }

        // Should only keep last 5 highs/lows
        assert_eq!(ce.highs.len(), 5);
        assert_eq!(ce.lows.len(), 5);
    }

    #[test]
    fn test_chandelier_exit_output() {
        let output = ChandelierExitOutput::new(95.0, 105.0);
        assert_eq!(output.long_exit, 95.0);
        assert_eq!(output.short_exit, 105.0);
    }
}
