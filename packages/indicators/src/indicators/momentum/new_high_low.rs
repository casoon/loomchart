//! New High/Low Index
//!
//! A market breadth indicator that measures the number of stocks making new highs
//! versus new lows over a specified period. It helps identify market momentum and
//! potential reversals.
//!
//! # Formula
//!
//! ```text
//! New High-Low Index = (New Highs - New Lows) / (New Highs + New Lows) * 100
//!
//! Or as absolute difference:
//! Net New Highs = New Highs - New Lows
//! ```
//!
//! # Interpretation
//!
//! - Index > 0: More new highs than lows (bullish)
//! - Index < 0: More new lows than highs (bearish)
//! - Index near 0: Neutral market
//! - Extreme values (+/- 100) indicate strong trends
//!
//! # Examples
//!
//! ```
//! use loom_indicators::prelude::*;
//! use loom_indicators::indicators::momentum::NewHighLow;
//!
//! let mut nhl = NewHighLow::new(10);
//!
//! // (New highs, New lows)
//! let result = nhl.next((150.0, 50.0));
//! ```

use crate::indicators::trend::Sma;
use crate::indicators::{Current, Next, Period, Reset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// New High/Low Index output
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NewHighLowOutput {
    /// The New High-Low Index (-100 to +100)
    pub index: f64,
    /// Net new highs (New Highs - New Lows)
    pub net_new_highs: f64,
    /// Moving average of the index
    pub index_ma: f64,
}

/// New High/Low Index
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NewHighLow {
    period: usize,
    ma: Sma,
    current: Option<NewHighLowOutput>,
}

impl NewHighLow {
    /// Creates a new New High/Low Index
    ///
    /// # Arguments
    ///
    /// * `period` - Period for the moving average (default: 10)
    ///
    /// # Errors
    ///
    /// Returns `IndicatorError::InvalidParameter` if period is 0
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be greater than 0");

        Self {
            period,
            ma: Sma::new(period),
            current: None,
        }
    }

    /// Calculate the New High-Low Index
    fn calculate_index(&self, new_highs: f64, new_lows: f64) -> (f64, f64) {
        let net_new_highs = new_highs - new_lows;
        let total = new_highs + new_lows;

        let index = if total > 0.0 {
            (net_new_highs / total) * 100.0
        } else {
            0.0 // If no new highs or lows, index is 0
        };

        (index, net_new_highs)
    }
}

impl Period for NewHighLow {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<(f64, f64)> for NewHighLow {
    type Output = NewHighLowOutput;

    fn next(&mut self, (new_highs, new_lows): (f64, f64)) -> Option<Self::Output> {
        let (index, net_new_highs) = self.calculate_index(new_highs, new_lows);

        // Update moving average
        let index_ma = self.ma.next(index)?;

        let output = NewHighLowOutput {
            index,
            net_new_highs,
            index_ma,
        };

        self.current = Some(output.clone());
        Some(output)
    }
}

impl<'a> Next<&'a (f64, f64)> for NewHighLow {
    type Output = NewHighLowOutput;

    fn next(&mut self, input: &'a (f64, f64)) -> Option<Self::Output> {
        self.next(*input)
    }
}

impl Next<(f64, f64)> for Box<NewHighLow> {
    type Output = NewHighLowOutput;

    fn next(&mut self, input: (f64, f64)) -> Option<Self::Output> {
        (**self).next(input)
    }
}

impl Reset for NewHighLow {
    fn reset(&mut self) {
        self.ma.reset();
        self.current = None;
    }
}

impl Current for NewHighLow {
    type Output = NewHighLowOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current.clone()
    }
}

impl Default for NewHighLow {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let nhl = NewHighLow::new(10);
        assert_eq!(nhl.period(), 10);
    }

    #[test]
    #[should_panic(expected = "period must be greater than 0")]
    fn test_new_invalid_zero_period() {
        NewHighLow::new(0);
    }

    #[test]
    fn test_default() {
        let nhl = NewHighLow::default();
        assert_eq!(nhl.period, 10);
    }

    #[test]
    fn test_bullish_index() {
        let mut nhl = NewHighLow::new(5);

        // Strong bullish: more new highs than lows
        let data = vec![
            (150.0, 50.0),
            (160.0, 40.0),
            (170.0, 30.0),
            (180.0, 20.0),
            (190.0, 10.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Bullish index should be positive
        assert!(result.index > 0.0);
        assert!(result.net_new_highs > 0.0);
    }

    #[test]
    fn test_bearish_index() {
        let mut nhl = NewHighLow::new(5);

        // Strong bearish: more new lows than highs
        let data = vec![
            (50.0, 150.0),
            (40.0, 160.0),
            (30.0, 170.0),
            (20.0, 180.0),
            (10.0, 190.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Bearish index should be negative
        assert!(result.index < 0.0);
        assert!(result.net_new_highs < 0.0);
    }

    #[test]
    fn test_neutral_index() {
        let mut nhl = NewHighLow::new(5);

        // Neutral: equal new highs and lows
        let data = vec![
            (100.0, 100.0),
            (100.0, 100.0),
            (100.0, 100.0),
            (100.0, 100.0),
            (100.0, 100.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Neutral index should be 0
        assert_eq!(result.index, 0.0);
        assert_eq!(result.net_new_highs, 0.0);
    }

    #[test]
    fn test_extreme_bullish() {
        let mut nhl = NewHighLow::new(5);

        // Extreme bullish: only new highs, no lows
        let data = vec![
            (200.0, 0.0),
            (200.0, 0.0),
            (200.0, 0.0),
            (200.0, 0.0),
            (200.0, 0.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Index should be +100 (maximum bullish)
        assert!((result.index - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_extreme_bearish() {
        let mut nhl = NewHighLow::new(5);

        // Extreme bearish: only new lows, no highs
        let data = vec![
            (0.0, 200.0),
            (0.0, 200.0),
            (0.0, 200.0),
            (0.0, 200.0),
            (0.0, 200.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Index should be -100 (maximum bearish)
        assert!((result.index + 100.0).abs() < 0.01);
    }

    #[test]
    fn test_index_calculation() {
        let mut nhl = NewHighLow::new(5);

        // Manual calculation: (150 - 50) / (150 + 50) * 100 = 100 / 200 * 100 = 50
        let data = vec![
            (150.0, 50.0),
            (150.0, 50.0),
            (150.0, 50.0),
            (150.0, 50.0),
            (150.0, 50.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        let result = last_result.unwrap();
        assert!((result.index - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_zero_values() {
        let mut nhl = NewHighLow::new(5);

        // No new highs or lows
        let data = vec![(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Index should be 0
        assert_eq!(result.index, 0.0);
        assert_eq!(result.net_new_highs, 0.0);
    }

    #[test]
    fn test_moving_average() {
        let mut nhl = NewHighLow::new(3);

        let data = vec![
            (150.0, 50.0), // Index = 50
            (140.0, 60.0), // Index = 40
            (130.0, 70.0), // Index = 30
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        let result = last_result.unwrap();

        // MA should be (50 + 40 + 30) / 3 = 40
        assert!((result.index_ma - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_trend_reversal() {
        let mut nhl = NewHighLow::new(5);

        // Start bullish
        let bullish_data = vec![
            (200.0, 50.0),
            (210.0, 40.0),
            (220.0, 30.0),
            (230.0, 20.0),
            (240.0, 10.0),
        ];

        for point in &bullish_data {
            nhl.next(*point);
        }

        let bullish_result = nhl.current().unwrap();
        assert!(bullish_result.index > 0.0);

        // Reverse to bearish
        let bearish_data = vec![
            (50.0, 200.0),
            (40.0, 210.0),
            (30.0, 220.0),
            (20.0, 230.0),
            (10.0, 240.0),
        ];

        for point in &bearish_data {
            nhl.next(*point);
        }

        let bearish_result = nhl.current().unwrap();
        assert!(bearish_result.index < 0.0);
    }

    #[test]
    fn test_net_new_highs() {
        let mut nhl = NewHighLow::new(5);

        let data = vec![
            (180.0, 120.0), // Net = 60
            (170.0, 130.0), // Net = 40
            (160.0, 140.0), // Net = 20
            (150.0, 150.0), // Net = 0
            (140.0, 160.0), // Net = -20
        ];

        let mut net_values = Vec::new();
        for point in &data {
            if let Some(result) = nhl.next(*point) {
                net_values.push(result.net_new_highs);
            }
        }

        assert_eq!(net_values.len(), 5);
        assert!((net_values[0] - 60.0).abs() < 0.01);
        assert!((net_values[1] - 40.0).abs() < 0.01);
        assert!((net_values[2] - 20.0).abs() < 0.01);
        assert!((net_values[3] - 0.0).abs() < 0.01);
        assert!((net_values[4] + 20.0).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut nhl = NewHighLow::new(5);

        let data = vec![
            (150.0, 50.0),
            (160.0, 40.0),
            (170.0, 30.0),
            (180.0, 20.0),
            (190.0, 10.0),
        ];

        for point in &data {
            nhl.next(*point);
        }

        let result_before = nhl.current().unwrap();

        nhl.reset();
        assert!(nhl.current().is_none());

        for point in &data {
            nhl.next(*point);
        }

        let result_after = nhl.current().unwrap();
        assert_eq!(result_before.index, result_after.index);
        assert_eq!(result_before.net_new_highs, result_after.net_new_highs);
        assert_eq!(result_before.index_ma, result_after.index_ma);
    }

    #[test]
    fn test_current() {
        let mut nhl = NewHighLow::new(5);
        assert!(nhl.current().is_none());

        let data = vec![
            (150.0, 50.0),
            (160.0, 40.0),
            (170.0, 30.0),
            (180.0, 20.0),
            (190.0, 10.0),
        ];

        for point in &data {
            nhl.next(*point);
        }

        assert!(nhl.current().is_some());
    }

    #[test]
    fn test_divergence_setup() {
        let mut nhl = NewHighLow::new(5);

        // Weakening bullish momentum
        let data = vec![
            (200.0, 50.0),  // Very bullish
            (180.0, 70.0),  // Still bullish but weakening
            (160.0, 90.0),  // Weak bullish
            (140.0, 110.0), // Barely bullish
            (120.0, 130.0), // Turning bearish
        ];

        let mut index_values = Vec::new();
        for point in &data {
            if let Some(result) = nhl.next(*point) {
                index_values.push(result.index);
            }
        }

        // Index should decline over time
        assert!(index_values.len() == 5);
        for i in 1..index_values.len() {
            assert!(index_values[i] < index_values[i - 1]);
        }
    }

    #[test]
    fn test_reference_input() {
        let mut nhl = NewHighLow::new(5);

        let data = vec![
            (150.0, 50.0),
            (160.0, 40.0),
            (170.0, 30.0),
            (180.0, 20.0),
            (190.0, 10.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_boxed() {
        let mut nhl: Box<NewHighLow> = Box::new(NewHighLow::new(5));

        let data = vec![
            (150.0, 50.0),
            (160.0, 40.0),
            (170.0, 30.0),
            (180.0, 20.0),
            (190.0, 10.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_smoothing_effect() {
        let mut nhl = NewHighLow::new(5);

        // Volatile data
        let data = vec![
            (200.0, 0.0), // Index = 100
            (0.0, 200.0), // Index = -100
            (200.0, 0.0), // Index = 100
            (0.0, 200.0), // Index = -100
            (200.0, 0.0), // Index = 100
        ];

        for point in &data {
            nhl.next(*point);
        }

        let result = nhl.current().unwrap();

        // MA should smooth out the volatility
        // Average of [100, -100, 100, -100, 100] = 20
        assert!((result.index_ma - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_asymmetric_values() {
        let mut nhl = NewHighLow::new(5);

        // Slightly more highs than lows
        let data = vec![
            (110.0, 90.0),
            (110.0, 90.0),
            (110.0, 90.0),
            (110.0, 90.0),
            (110.0, 90.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = nhl.next(*point);
        }

        let result = last_result.unwrap();

        // Index should be (110 - 90) / (110 + 90) * 100 = 20 / 200 * 100 = 10
        assert!((result.index - 10.0).abs() < 0.01);
    }
}
