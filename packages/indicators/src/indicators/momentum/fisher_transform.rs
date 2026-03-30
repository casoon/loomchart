//! Fisher Transform

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Fisher Transform Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FisherOutput {
    /// Fisher Transform value
    pub fisher: f64,
    /// Fisher Transform signal (previous fisher value)
    pub signal: f64,
}

impl FisherOutput {
    pub fn new(fisher: f64, signal: f64) -> Self {
        Self { fisher, signal }
    }

    /// Check if bullish crossover (fisher crosses above signal)
    pub fn is_bullish_cross(&self) -> bool {
        self.fisher > self.signal && self.fisher > 0.0
    }

    /// Check if bearish crossover (fisher crosses below signal)
    pub fn is_bearish_cross(&self) -> bool {
        self.fisher < self.signal && self.fisher < 0.0
    }
}

/// Fisher Transform
///
/// Developed by John Ehlers, the Fisher Transform converts prices into a
/// Gaussian normal distribution. This makes turning points easier to identify.
///
/// The Fisher Transform is an oscillator that swings between -infinity and +infinity,
/// but typical values are between -3 and +3. Extreme values indicate potential
/// trend reversals.
///
/// # Formula
///
/// ```text
/// 1. Normalize price to range [-1, 1]:
///    Value = (Price - Low(n)) / (High(n) - Low(n)) * 2 - 1
///    Value = clamp(Value, -0.999, 0.999)
///
/// 2. Smooth the value:
///    Value = (Value + 2 * PrevValue) / 3
///
/// 3. Apply Fisher Transform:
///    Fisher = 0.5 * ln((1 + Value) / (1 - Value))
///
/// 4. Apply smoothing:
///    Fisher = 0.5 * Fisher + 0.25 * PrevFisher + 0.25 * PrevPrevFisher
/// ```
///
/// # Interpretation
///
/// - **Fisher > 0**: Bullish bias
/// - **Fisher < 0**: Bearish bias
/// - **Fisher crosses above signal**: Buy signal
/// - **Fisher crosses below signal**: Sell signal
/// - **Extreme values (> 2 or < -2)**: Potential reversal
///
/// # Parameters
///
/// * `period` - Lookback period for high/low (default: 10)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::{FisherTransform, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut fisher = FisherTransform::new(10);
///
/// for candle in candles.iter() {
///     if let Some(output) = fisher.next(candle) {
///         if output.is_bullish_cross() {
///             println!("Bullish crossover at {:.2}", output.fisher);
///         }
///     }
/// }
/// ```
#[derive(Clone)]
pub struct FisherTransform {
    period: usize,
    prices: VecDeque<f64>,
    value: f64,
    prev_value: f64,
    fisher: f64,
    prev_fisher: f64,
    prev_prev_fisher: f64,
    current: Option<FisherOutput>,
}

impl FisherTransform {
    /// Create a new Fisher Transform with the specified period
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");

        Self {
            period,
            prices: VecDeque::with_capacity(period),
            value: 0.0,
            prev_value: 0.0,
            fisher: 0.0,
            prev_fisher: 0.0,
            prev_prev_fisher: 0.0,
            current: None,
        }
    }

    fn highest(&self) -> f64 {
        self.prices
            .iter()
            .fold(f64::NEG_INFINITY, |max, &p| max.max(p))
    }

    fn lowest(&self) -> f64 {
        self.prices.iter().fold(f64::INFINITY, |min, &p| min.min(p))
    }

    fn clamp(value: f64, min: f64, max: f64) -> f64 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}

impl Period for FisherTransform {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<&Ohlcv> for FisherTransform {
    type Output = FisherOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        // Use midpoint of high/low
        let price = (candle.high + candle.low) / 2.0;

        self.prices.push_back(price);
        if self.prices.len() > self.period {
            self.prices.pop_front();
        }

        // Need at least 2 prices to calculate
        if self.prices.len() < 2 {
            return None;
        }

        // Normalize to [-1, 1]
        let highest = self.highest();
        let lowest = self.lowest();
        let range = highest - lowest;

        let mut normalized = if range > 0.0 {
            ((price - lowest) / range) * 2.0 - 1.0
        } else {
            0.0
        };

        // Clamp to prevent log of infinity
        normalized = Self::clamp(normalized, -0.999, 0.999);

        // Smooth the value
        self.prev_value = self.value;
        self.value = (normalized + 2.0 * self.prev_value) / 3.0;

        // Apply Fisher Transform: 0.5 * ln((1 + x) / (1 - x))
        let fisher_raw = 0.5 * ((1.0 + self.value) / (1.0 - self.value)).ln();

        // Smooth the Fisher Transform
        self.prev_prev_fisher = self.prev_fisher;
        self.prev_fisher = self.fisher;
        self.fisher = 0.5 * fisher_raw + 0.25 * self.prev_fisher + 0.25 * self.prev_prev_fisher;

        let output = FisherOutput::new(self.fisher, self.prev_fisher);
        self.current = Some(output);
        Some(output)
    }
}

impl Next<Ohlcv> for FisherTransform {
    type Output = FisherOutput;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for FisherTransform {
    type Output = FisherOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for FisherTransform {
    fn reset(&mut self) {
        self.prices.clear();
        self.value = 0.0;
        self.prev_value = 0.0;
        self.fisher = 0.0;
        self.prev_fisher = 0.0;
        self.prev_prev_fisher = 0.0;
        self.current = None;
    }
}

impl Default for FisherTransform {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(high: f64, low: f64) -> Ohlcv {
        let mid = (high + low) / 2.0;
        Ohlcv::new(mid, high, low, mid, 1000.0)
    }

    #[test]
    fn test_fisher_transform_new() {
        let fisher = FisherTransform::new(10);
        assert_eq!(fisher.period(), 10);
    }

    #[test]
    #[should_panic]
    fn test_fisher_transform_invalid_period() {
        FisherTransform::new(0);
    }

    #[test]
    fn test_fisher_transform_uptrend() {
        let mut fisher = FisherTransform::new(5);

        // Strong uptrend
        for i in 0..20 {
            let base = 100.0 + i as f64 * 2.0;
            fisher.next(&candle(base + 2.0, base));
        }

        let result = fisher.current().unwrap();
        // In uptrend, Fisher should be positive
        assert!(result.fisher > 0.0, "Fisher should be positive in uptrend");
    }

    #[test]
    fn test_fisher_transform_downtrend() {
        let mut fisher = FisherTransform::new(5);

        // Strong downtrend
        for i in 0..20 {
            let base = 150.0 - i as f64 * 2.0;
            fisher.next(&candle(base, base - 2.0));
        }

        let result = fisher.current().unwrap();
        // In downtrend, Fisher should be negative
        assert!(
            result.fisher < 0.0,
            "Fisher should be negative in downtrend"
        );
    }

    #[test]
    fn test_fisher_transform_oscillation() {
        let mut fisher = FisherTransform::new(10);

        // Build base
        for i in 0..15 {
            let base = 100.0 + (i as f64 % 5.0);
            fisher.next(&candle(base + 2.0, base));
        }

        assert!(fisher.current().is_some());
    }

    #[test]
    fn test_fisher_transform_extreme_values() {
        let mut fisher = FisherTransform::new(5);

        // Values should be bounded even with extreme inputs
        for _ in 0..10 {
            fisher.next(&candle(1000.0, 100.0));
        }

        if let Some(result) = fisher.current() {
            // Fisher values typically stay within reasonable bounds
            assert!(result.fisher.abs() < 10.0, "Fisher value should be bounded");
        }
    }

    #[test]
    fn test_fisher_output_methods() {
        let output1 = FisherOutput::new(1.5, 0.5);
        assert!(output1.is_bullish_cross());
        assert!(!output1.is_bearish_cross());

        let output2 = FisherOutput::new(-1.5, -0.5);
        assert!(!output2.is_bullish_cross());
        assert!(output2.is_bearish_cross());

        let output3 = FisherOutput::new(0.5, 1.5);
        assert!(!output3.is_bullish_cross());
        assert!(!output3.is_bearish_cross());
    }

    #[test]
    fn test_fisher_transform_reset() {
        let mut fisher = FisherTransform::new(10);

        fisher.next(&candle(105.0, 95.0));
        fisher.next(&candle(110.0, 100.0));
        assert!(fisher.current().is_some());

        fisher.reset();

        assert!(fisher.current().is_none());
        assert_eq!(fisher.prices.len(), 0);
        assert_eq!(fisher.value, 0.0);
        assert_eq!(fisher.fisher, 0.0);
    }

    #[test]
    fn test_fisher_transform_default() {
        let fisher = FisherTransform::default();
        assert_eq!(fisher.period(), 10);
    }

    #[test]
    fn test_fisher_transform_insufficient_data() {
        let mut fisher = FisherTransform::new(5);

        assert!(fisher.next(&candle(100.0, 95.0)).is_none());
        // 2nd value should return Some
        assert!(fisher.next(&candle(102.0, 97.0)).is_some());
    }

    #[test]
    fn test_fisher_clamp() {
        assert_eq!(FisherTransform::clamp(0.5, 0.0, 1.0), 0.5);
        assert_eq!(FisherTransform::clamp(-0.5, 0.0, 1.0), 0.0);
        assert_eq!(FisherTransform::clamp(1.5, 0.0, 1.0), 1.0);
    }

    #[test]
    fn test_fisher_transform_signal_lag() {
        let mut fisher = FisherTransform::new(5);

        for i in 0..10 {
            let base = 100.0 + i as f64;
            fisher.next(&candle(base + 2.0, base));
        }

        let result = fisher.current().unwrap();
        // Signal is previous fisher value
        assert_eq!(result.signal, fisher.prev_fisher);
    }

    #[test]
    fn test_fisher_transform_zero_range() {
        let mut fisher = FisherTransform::new(3);

        // All same prices (zero range)
        fisher.next(&candle(100.0, 100.0));
        fisher.next(&candle(100.0, 100.0));
        let result = fisher.next(&candle(100.0, 100.0));

        // Should handle zero range gracefully
        assert!(result.is_some());
        assert!(result.unwrap().fisher.is_finite());
    }

    #[test]
    fn test_fisher_transform_smoothing() {
        let mut fisher = FisherTransform::new(5);

        // Feed some data
        for i in 0..10 {
            let base = 100.0 + (i % 2) as f64 * 5.0;
            fisher.next(&candle(base + 2.0, base));
        }

        // Fisher should smooth out rapid oscillations
        assert!(fisher.current().is_some());
    }

    #[test]
    fn test_fisher_transform_reversal_detection() {
        let mut fisher = FisherTransform::new(5);

        // Uptrend
        for i in 0..10 {
            let base = 100.0 + i as f64 * 2.0;
            fisher.next(&candle(base + 2.0, base));
        }

        let uptrend_fisher = fisher.current().unwrap().fisher;

        // Reversal to downtrend
        for i in 0..10 {
            let base = 120.0 - i as f64 * 2.0;
            fisher.next(&candle(base, base - 2.0));
        }

        let downtrend_fisher = fisher.current().unwrap().fisher;

        // Fisher should change from positive to negative
        assert!(uptrend_fisher > 0.0);
        assert!(downtrend_fisher < 0.0);
    }
}
