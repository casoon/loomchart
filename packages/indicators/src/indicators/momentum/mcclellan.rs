//! McClellan Oscillator
//!
//! A market breadth indicator based on the difference between advancing and declining issues.
//! It uses two exponential moving averages (19-period and 39-period) of the advance-decline data.
//!
//! # Formula
//!
//! ```text
//! Net Advances = Advancing Issues - Declining Issues
//! Fast EMA = EMA(Net Advances, 19)
//! Slow EMA = EMA(Net Advances, 39)
//! McClellan Oscillator = Fast EMA - Slow EMA
//! McClellan Summation = Cumulative sum of McClellan Oscillator
//! ```
//!
//! # Examples
//!
//! ```
//! use loom_indicators::prelude::*;
//! use loom_indicators::indicators::momentum::McClellan;
//!
//! let mut mcclellan = McClellan::new(19, 39);
//!
//! // Advancing issues, Declining issues
//! let result1 = mcclellan.next((1500.0, 1000.0));
//! let result2 = mcclellan.next((1600.0, 900.0));
//! ```

use crate::indicators::trend::Ema;
use crate::indicators::{Current, Next, Period, Reset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// McClellan Oscillator output
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct McClellanOutput {
    /// The McClellan Oscillator value (Fast EMA - Slow EMA)
    pub oscillator: f64,
    /// The McClellan Summation Index (cumulative sum)
    pub summation: f64,
    /// Fast EMA value
    pub fast_ema: f64,
    /// Slow EMA value
    pub slow_ema: f64,
}

/// McClellan Oscillator
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct McClellan {
    #[allow(dead_code)]
    fast_period: usize,
    slow_period: usize,
    fast_ema: Ema,
    slow_ema: Ema,
    summation: f64,
    current: Option<McClellanOutput>,
}

impl McClellan {
    /// Creates a new McClellan Oscillator
    ///
    /// # Arguments
    ///
    /// * `fast_period` - Fast EMA period (default: 19)
    /// * `slow_period` - Slow EMA period (default: 39)
    ///
    /// # Errors
    ///
    /// Returns `IndicatorError::InvalidParameter` if:
    /// - Any period is 0
    /// - Fast period >= Slow period
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        assert!(
            fast_period > 0 && slow_period > 0,
            "periods must be greater than 0"
        );
        assert!(
            fast_period < slow_period,
            "fast_period must be less than slow_period"
        );

        Self {
            fast_period,
            slow_period,
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            summation: 0.0,
            current: None,
        }
    }
}

impl Period for McClellan {
    fn period(&self) -> usize {
        self.slow_period
    }
}

impl Next<(f64, f64)> for McClellan {
    type Output = McClellanOutput;

    fn next(&mut self, (advancing, declining): (f64, f64)) -> Option<Self::Output> {
        // Calculate net advances
        let net_advances = advancing - declining;

        // Update EMAs
        let fast_ema = self.fast_ema.next(net_advances)?;
        let slow_ema = self.slow_ema.next(net_advances)?;

        // Calculate oscillator
        let oscillator = fast_ema - slow_ema;

        // Update summation
        self.summation += oscillator;

        let output = McClellanOutput {
            oscillator,
            summation: self.summation,
            fast_ema,
            slow_ema,
        };

        self.current = Some(output.clone());
        Some(output)
    }
}

impl<'a> Next<&'a (f64, f64)> for McClellan {
    type Output = McClellanOutput;

    fn next(&mut self, input: &'a (f64, f64)) -> Option<Self::Output> {
        self.next(*input)
    }
}

impl Next<(f64, f64)> for Box<McClellan> {
    type Output = McClellanOutput;

    fn next(&mut self, input: (f64, f64)) -> Option<Self::Output> {
        (**self).next(input)
    }
}

impl Reset for McClellan {
    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.summation = 0.0;
        self.current = None;
    }
}

impl Current for McClellan {
    type Output = McClellanOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current.clone()
    }
}

impl Default for McClellan {
    fn default() -> Self {
        Self::new(19, 39)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mcclellan = McClellan::new(19, 39);
        assert_eq!(mcclellan.period(), 39);
    }

    #[test]
    #[should_panic(expected = "periods must be greater than 0")]
    fn test_new_invalid_zero_period() {
        McClellan::new(0, 10);
    }

    #[test]
    #[should_panic(expected = "fast_period must be less than slow_period")]
    fn test_new_invalid_fast_slow() {
        McClellan::new(39, 19);
    }

    #[test]
    fn test_default() {
        let mcclellan = McClellan::default();
        assert_eq!(mcclellan.fast_period, 19);
        assert_eq!(mcclellan.slow_period, 39);
    }

    #[test]
    fn test_bullish_breadth() {
        let mut mcclellan = McClellan::new(5, 10);

        // Strong bullish breadth (more advancing than declining)
        let data = vec![
            (2000.0, 500.0),
            (2100.0, 400.0),
            (2200.0, 300.0),
            (2300.0, 200.0),
            (2400.0, 100.0),
            (2500.0, 50.0),
            (2600.0, 40.0),
            (2700.0, 30.0),
            (2800.0, 20.0),
            (2900.0, 10.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = mcclellan.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With strong bullish breadth, oscillator should be positive
        assert!(result.oscillator > 0.0);
        // Summation should also be positive
        assert!(result.summation > 0.0);
    }

    #[test]
    fn test_bearish_breadth() {
        let mut mcclellan = McClellan::new(5, 10);

        // Strong bearish breadth (more declining than advancing)
        let data = vec![
            (500.0, 2000.0),
            (400.0, 2100.0),
            (300.0, 2200.0),
            (200.0, 2300.0),
            (100.0, 2400.0),
            (50.0, 2500.0),
            (40.0, 2600.0),
            (30.0, 2700.0),
            (20.0, 2800.0),
            (10.0, 2900.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = mcclellan.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With strong bearish breadth, oscillator should be negative
        assert!(result.oscillator < 0.0);
        // Summation should also be negative
        assert!(result.summation < 0.0);
    }

    #[test]
    fn test_neutral_breadth() {
        let mut mcclellan = McClellan::new(5, 10);

        // Neutral breadth (equal advancing and declining)
        let data = vec![
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
            (1500.0, 1500.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = mcclellan.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With neutral breadth, oscillator should be near zero
        assert!(result.oscillator.abs() < 0.01);
        // Summation should also be near zero
        assert!(result.summation.abs() < 0.1);
    }

    #[test]
    fn test_breadth_reversal() {
        let mut mcclellan = McClellan::new(5, 10);

        // Start bullish
        let bullish_data = vec![
            (2000.0, 500.0),
            (2100.0, 400.0),
            (2200.0, 300.0),
            (2300.0, 200.0),
            (2400.0, 100.0),
            (2500.0, 50.0),
            (2600.0, 40.0),
            (2700.0, 30.0),
            (2800.0, 20.0),
            (2900.0, 10.0),
        ];

        for point in &bullish_data {
            mcclellan.next(*point);
        }

        let bullish_result = mcclellan.current().unwrap();
        assert!(bullish_result.oscillator > 0.0);

        // Then turn bearish
        let bearish_data = vec![
            (500.0, 2000.0),
            (400.0, 2100.0),
            (300.0, 2200.0),
            (200.0, 2300.0),
            (100.0, 2400.0),
        ];

        for point in &bearish_data {
            mcclellan.next(*point);
        }

        let bearish_result = mcclellan.current().unwrap();
        assert!(bearish_result.oscillator < 0.0);
    }

    #[test]
    fn test_summation_accumulation() {
        let mut mcclellan = McClellan::new(5, 10);

        // Consistent bullish breadth
        let data = vec![
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
            (2000.0, 1000.0),
        ];

        let mut summations = Vec::new();
        for point in &data {
            if let Some(result) = mcclellan.next(*point) {
                summations.push(result.summation);
            }
        }

        // Summation should be cumulative and increasing
        assert!(summations.len() > 2);
        for i in 1..summations.len() {
            assert!(summations[i] > summations[i - 1]);
        }
    }

    #[test]
    fn test_oscillator_calculation() {
        let mut mcclellan = McClellan::new(5, 10);

        let data = vec![
            (2000.0, 1000.0),
            (2100.0, 900.0),
            (2200.0, 800.0),
            (2300.0, 700.0),
            (2400.0, 600.0),
            (2500.0, 500.0),
            (2600.0, 400.0),
            (2700.0, 300.0),
            (2800.0, 200.0),
            (2900.0, 100.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = mcclellan.next(*point);
        }

        let result = last_result.unwrap();

        // Verify oscillator is fast_ema - slow_ema
        assert!((result.oscillator - (result.fast_ema - result.slow_ema)).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let mut mcclellan = McClellan::new(5, 10);

        let data = vec![
            (2000.0, 1000.0),
            (2100.0, 900.0),
            (2200.0, 800.0),
            (2300.0, 700.0),
            (2400.0, 600.0),
            (2500.0, 500.0),
            (2600.0, 400.0),
            (2700.0, 300.0),
            (2800.0, 200.0),
            (2900.0, 100.0),
        ];

        for point in &data {
            mcclellan.next(*point);
        }

        let result_before = mcclellan.current().unwrap();

        mcclellan.reset();
        assert!(mcclellan.current().is_none());
        assert_eq!(mcclellan.summation, 0.0);

        for point in &data {
            mcclellan.next(*point);
        }

        let result_after = mcclellan.current().unwrap();
        assert_eq!(result_before.oscillator, result_after.oscillator);
        assert_eq!(result_before.summation, result_after.summation);
    }

    #[test]
    fn test_current() {
        let mut mcclellan = McClellan::new(5, 10);
        assert!(mcclellan.current().is_none());

        let data = vec![
            (2000.0, 1000.0),
            (2100.0, 900.0),
            (2200.0, 800.0),
            (2300.0, 700.0),
            (2400.0, 600.0),
            (2500.0, 500.0),
            (2600.0, 400.0),
            (2700.0, 300.0),
            (2800.0, 200.0),
            (2900.0, 100.0),
        ];

        for point in &data {
            mcclellan.next(*point);
        }

        assert!(mcclellan.current().is_some());
    }

    #[test]
    fn test_zero_values() {
        let mut mcclellan = McClellan::new(5, 10);

        let data = vec![
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
            (0.0, 0.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = mcclellan.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With all zeros, everything should be zero
        assert_eq!(result.oscillator, 0.0);
        assert_eq!(result.summation, 0.0);
        assert_eq!(result.fast_ema, 0.0);
        assert_eq!(result.slow_ema, 0.0);
    }

    #[test]
    fn test_divergence_setup() {
        let mut mcclellan = McClellan::new(5, 10);

        // Declining breadth (bearish)
        let data = vec![
            (2000.0, 1000.0),
            (1900.0, 1100.0),
            (1800.0, 1200.0),
            (1700.0, 1300.0),
            (1600.0, 1400.0),
            (1500.0, 1500.0),
            (1400.0, 1600.0),
            (1300.0, 1700.0),
            (1200.0, 1800.0),
            (1100.0, 1900.0),
        ];

        let mut oscillator_values = Vec::new();
        for point in &data {
            if let Some(result) = mcclellan.next(*point) {
                oscillator_values.push(result.oscillator);
            }
        }

        // Oscillator should decline from positive to negative
        assert!(oscillator_values.len() > 2);
        let first_osc = oscillator_values[0];
        let last_osc = *oscillator_values.last().unwrap();

        assert!(first_osc > 0.0);
        assert!(last_osc < 0.0);
    }

    #[test]
    fn test_reference_input() {
        let mut mcclellan = McClellan::new(5, 10);

        let data = vec![
            (2000.0, 1000.0),
            (2100.0, 900.0),
            (2200.0, 800.0),
            (2300.0, 700.0),
            (2400.0, 600.0),
            (2500.0, 500.0),
            (2600.0, 400.0),
            (2700.0, 300.0),
            (2800.0, 200.0),
            (2900.0, 100.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = mcclellan.next(point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_boxed() {
        let mut mcclellan: Box<McClellan> = Box::new(McClellan::new(5, 10));

        let data = vec![
            (2000.0, 1000.0),
            (2100.0, 900.0),
            (2200.0, 800.0),
            (2300.0, 700.0),
            (2400.0, 600.0),
            (2500.0, 500.0),
            (2600.0, 400.0),
            (2700.0, 300.0),
            (2800.0, 200.0),
            (2900.0, 100.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = mcclellan.next(*point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_fast_ema_responsiveness() {
        let mut mcclellan = McClellan::new(5, 10);

        // Feed same data to establish baseline
        for _ in 0..10 {
            mcclellan.next((2000.0, 1000.0));
        }

        let baseline = mcclellan.current().unwrap();

        // Then feed very different data
        mcclellan.next((3000.0, 500.0));
        let new_result = mcclellan.current().unwrap();

        // Fast EMA should change more than slow EMA
        let fast_change = (new_result.fast_ema - baseline.fast_ema).abs();
        let slow_change = (new_result.slow_ema - baseline.slow_ema).abs();

        assert!(fast_change > slow_change);
    }

    #[test]
    fn test_summation_negative_oscillator() {
        let mut mcclellan = McClellan::new(5, 10);

        // Consistent bearish breadth
        let data = vec![
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
            (1000.0, 2000.0),
        ];

        let mut summations = Vec::new();
        for point in &data {
            if let Some(result) = mcclellan.next(*point) {
                summations.push(result.summation);
            }
        }

        // Summation should be cumulative and decreasing (becoming more negative)
        assert!(summations.len() > 2);
        for i in 1..summations.len() {
            assert!(summations[i] < summations[i - 1]);
        }
    }
}
