//! Supertrend Indicator

use crate::indicators::volatility::Atr;
use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Supertrend Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SupertrendOutput {
    /// Supertrend value
    pub value: f64,
    /// Current trend direction (1 = up, -1 = down)
    pub trend: i8,
    /// Upper band
    pub upper_band: f64,
    /// Lower band
    pub lower_band: f64,
}

impl SupertrendOutput {
    pub fn new(value: f64, trend: i8, upper_band: f64, lower_band: f64) -> Self {
        Self {
            value,
            trend,
            upper_band,
            lower_band,
        }
    }

    /// Check if in uptrend
    pub fn is_uptrend(&self) -> bool {
        self.trend > 0
    }

    /// Check if in downtrend
    pub fn is_downtrend(&self) -> bool {
        self.trend < 0
    }
}

/// Supertrend Indicator
///
/// A trend-following indicator based on Average True Range (ATR).
/// Shows buy/sell signals and acts as dynamic support/resistance.
///
/// Formula:
/// - Basic Upper Band = (High + Low) / 2 + (Multiplier * ATR)
/// - Basic Lower Band = (High + Low) / 2 - (Multiplier * ATR)
/// - Final Bands adjust based on trend direction and don't cross price
///
/// Parameters:
/// - period: ATR period (default 10)
/// - multiplier: ATR multiplier (default 3.0)
///
/// Trading signals:
/// - Supertrend below price: Uptrend, potential long
/// - Supertrend above price: Downtrend, potential short
/// - Supertrend flip: Trend reversal signal
#[derive(Clone)]
pub struct Supertrend {
    period: usize,
    multiplier: f64,
    atr: Atr,
    trend: i8,
    upper_band: f64,
    lower_band: f64,
    supertrend: f64,
    prev_upper: f64,
    prev_lower: f64,
    prev_close: f64,
    count: usize,
    current: Option<SupertrendOutput>,
}

impl Supertrend {
    pub fn new(period: usize, multiplier: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(multiplier > 0.0, "Multiplier must be greater than 0");

        Self {
            period,
            multiplier,
            atr: Atr::new(period),
            trend: 1,
            upper_band: 0.0,
            lower_band: 0.0,
            supertrend: 0.0,
            prev_upper: 0.0,
            prev_lower: 0.0,
            prev_close: 0.0,
            count: 0,
            current: None,
        }
    }

    /// Get current Supertrend value
    pub fn value(&self) -> Option<f64> {
        self.current.map(|o| o.value)
    }

    /// Get current trend direction
    pub fn trend_direction(&self) -> Option<i8> {
        self.current.map(|o| o.trend)
    }

    /// Check if in uptrend
    pub fn is_uptrend(&self) -> bool {
        self.current.map(|o| o.is_uptrend()).unwrap_or(false)
    }

    /// Check if in downtrend
    pub fn is_downtrend(&self) -> bool {
        self.current.map(|o| o.is_downtrend()).unwrap_or(false)
    }

    fn calculate_bands(&mut self, high: f64, low: f64, atr: f64) -> (f64, f64) {
        let hl_avg = (high + low) / 2.0;
        let basic_upper = hl_avg + (self.multiplier * atr);
        let basic_lower = hl_avg - (self.multiplier * atr);

        // Adjust bands to not cross previous close
        let final_upper = if basic_upper < self.prev_upper || self.prev_close > self.prev_upper {
            basic_upper
        } else {
            self.prev_upper
        };

        let final_lower = if basic_lower > self.prev_lower || self.prev_close < self.prev_lower {
            basic_lower
        } else {
            self.prev_lower
        };

        (final_upper, final_lower)
    }
}

impl Period for Supertrend {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<&Ohlcv> for Supertrend {
    type Output = SupertrendOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        self.count += 1;

        // Calculate ATR
        let atr_opt = self.atr.next(candle);

        // Need ATR to be ready
        if atr_opt.is_none() {
            return None;
        }

        let atr = atr_opt.unwrap();

        if self.count == self.period {
            // Initialize on first valid ATR
            let hl_avg = (candle.high + candle.low) / 2.0;
            self.upper_band = hl_avg + (self.multiplier * atr);
            self.lower_band = hl_avg - (self.multiplier * atr);

            // Initial trend based on close position
            self.trend = if candle.close <= self.lower_band {
                -1
            } else {
                1
            };

            self.supertrend = if self.trend > 0 {
                self.lower_band
            } else {
                self.upper_band
            };

            self.prev_upper = self.upper_band;
            self.prev_lower = self.lower_band;
            self.prev_close = candle.close;

            let output = SupertrendOutput::new(
                self.supertrend,
                self.trend,
                self.upper_band,
                self.lower_band,
            );
            self.current = Some(output);
            return Some(output);
        }

        if self.count > self.period {
            // Calculate new bands
            let (upper, lower) = self.calculate_bands(candle.high, candle.low, atr);
            self.upper_band = upper;
            self.lower_band = lower;

            // Determine trend
            let prev_trend = self.trend;

            if prev_trend > 0 {
                // Was in uptrend
                if candle.close <= self.lower_band {
                    self.trend = -1; // Switch to downtrend
                }
            } else {
                // Was in downtrend
                if candle.close >= self.upper_band {
                    self.trend = 1; // Switch to uptrend
                }
            }

            // Set Supertrend value based on trend
            self.supertrend = if self.trend > 0 {
                self.lower_band
            } else {
                self.upper_band
            };

            // Update previous values
            self.prev_upper = self.upper_band;
            self.prev_lower = self.lower_band;
            self.prev_close = candle.close;

            let output = SupertrendOutput::new(
                self.supertrend,
                self.trend,
                self.upper_band,
                self.lower_band,
            );
            self.current = Some(output);
            return Some(output);
        }

        None
    }
}

impl Next<Ohlcv> for Supertrend {
    type Output = SupertrendOutput;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for Supertrend {
    type Output = SupertrendOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for Supertrend {
    fn reset(&mut self) {
        self.atr.reset();
        self.trend = 1;
        self.upper_band = 0.0;
        self.lower_band = 0.0;
        self.supertrend = 0.0;
        self.prev_upper = 0.0;
        self.prev_lower = 0.0;
        self.prev_close = 0.0;
        self.count = 0;
        self.current = None;
    }
}

impl Default for Supertrend {
    fn default() -> Self {
        Self::new(10, 3.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ohlcv(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_supertrend_new() {
        let st = Supertrend::new(10, 3.0);
        assert_eq!(st.period(), 10);
    }

    #[test]
    fn test_supertrend_uptrend() {
        let mut st = Supertrend::new(10, 3.0);

        // Strong uptrend
        for i in 1..=30 {
            let price = 100.0 + i as f64;
            st.next(&ohlcv(price + 2.0, price - 2.0, price));
        }

        // Should be in uptrend
        assert!(st.is_uptrend());
        assert!(st.value().unwrap() > 0.0);
    }

    #[test]
    fn test_supertrend_downtrend() {
        let mut st = Supertrend::new(10, 3.0);

        // Strong downtrend
        for i in 1..=30 {
            let price = 150.0 - i as f64;
            st.next(&ohlcv(price + 2.0, price - 2.0, price));
        }

        // Should be in downtrend
        assert!(st.is_downtrend());
    }

    #[test]
    fn test_supertrend_reversal() {
        let mut st = Supertrend::new(10, 3.0);

        // Uptrend
        for i in 1..=20 {
            let price = 100.0 + i as f64;
            st.next(&ohlcv(price + 2.0, price - 2.0, price));
        }

        let was_uptrend = st.is_uptrend();

        // Sharp reversal
        for i in 1..=20 {
            let price = 120.0 - i as f64 * 2.0;
            st.next(&ohlcv(price + 2.0, price - 2.0, price));
        }

        // Trend should have changed
        assert_ne!(was_uptrend, st.is_uptrend());
    }

    #[test]
    fn test_supertrend_bands() {
        let mut st = Supertrend::new(10, 3.0);

        for i in 1..=20 {
            let price = 100.0 + i as f64;
            st.next(&ohlcv(price + 2.0, price - 2.0, price));
        }

        if let Some(output) = st.current() {
            // Bands should be non-zero
            assert!(output.upper_band > 0.0);
            assert!(output.lower_band > 0.0);
            // Upper band should be above lower band
            assert!(output.upper_band > output.lower_band);
        }
    }

    #[test]
    fn test_supertrend_reset() {
        let mut st = Supertrend::new(10, 3.0);

        st.next(&ohlcv(105.0, 95.0, 100.0));
        st.next(&ohlcv(110.0, 98.0, 105.0));

        st.reset();

        assert!(st.value().is_none());
        assert_eq!(st.count, 0);
    }

    #[test]
    fn test_supertrend_output() {
        let output = SupertrendOutput::new(100.0, 1, 110.0, 95.0);

        assert_eq!(output.value, 100.0);
        assert_eq!(output.trend, 1);
        assert!(output.is_uptrend());
        assert!(!output.is_downtrend());
    }

    #[test]
    fn test_supertrend_initialization() {
        let mut st = Supertrend::new(10, 3.0);

        // First few values should be None
        for i in 0..9 {
            let price = 100.0 + i as f64;
            let result = st.next(&ohlcv(price + 2.0, price - 2.0, price));
            assert!(result.is_none(), "Should be None before period is complete");
        }

        // Period-th value should return Some
        let result = st.next(&ohlcv(112.0, 108.0, 110.0));
        assert!(result.is_some(), "Should return Some at period");
    }
}
