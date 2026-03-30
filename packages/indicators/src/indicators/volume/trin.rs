//! Arms Index (TRIN)
//!
//! Also known as the Short-Term Trading Index (TRIN), this indicator measures
//! the relationship between advancing/declining stocks and their volume.
//! It's a market breadth indicator used to gauge overall market sentiment.
//!
//! # Formula
//!
//! ```text
//! Advance Ratio = Advancing Issues / Declining Issues
//! Volume Ratio = Advancing Volume / Declining Volume
//! TRIN = Advance Ratio / Volume Ratio
//!
//! Or equivalently:
//! TRIN = (Advancing Issues / Declining Issues) / (Advancing Volume / Declining Volume)
//! TRIN = (Advancing Issues * Declining Volume) / (Declining Issues * Advancing Volume)
//! ```
//!
//! # Interpretation
//!
//! - TRIN > 1.0: More volume in declining stocks (bearish)
//! - TRIN < 1.0: More volume in advancing stocks (bullish)
//! - TRIN = 1.0: Neutral market
//!
//! # Examples
//!
//! ```
//! use loom_indicators::prelude::*;
//! use loom_indicators::indicators::volume::Trin;
//!
//! let mut trin = Trin::new(10);
//!
//! // (Advancing issues, Declining issues, Advancing volume, Declining volume)
//! let result = trin.next((1500.0, 1000.0, 50000000.0, 30000000.0));
//! ```

use crate::indicators::trend::Sma;
use crate::indicators::{Current, Next, Period, Reset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Arms Index (TRIN) input data
pub type TrinInput = (f64, f64, f64, f64);

/// Arms Index (TRIN) output
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrinOutput {
    /// Current TRIN value
    pub trin: f64,
    /// Moving average of TRIN
    pub trin_ma: f64,
}

/// Arms Index (TRIN)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Trin {
    period: usize,
    ma: Sma,
    current: Option<TrinOutput>,
}

impl Trin {
    /// Creates a new Arms Index (TRIN)
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

    /// Calculate TRIN value from input
    fn calculate_trin(&self, input: TrinInput) -> Option<f64> {
        let (adv_issues, dec_issues, adv_volume, dec_volume) = input;

        // Avoid division by zero
        if dec_issues == 0.0 || adv_volume == 0.0 {
            return None;
        }

        // TRIN = (Advancing Issues * Declining Volume) / (Declining Issues * Advancing Volume)
        let trin = (adv_issues * dec_volume) / (dec_issues * adv_volume);

        Some(trin)
    }
}

impl Period for Trin {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<TrinInput> for Trin {
    type Output = TrinOutput;

    fn next(&mut self, input: TrinInput) -> Option<Self::Output> {
        let trin = self.calculate_trin(input)?;

        // Update moving average
        let trin_ma = self.ma.next(trin)?;

        let output = TrinOutput { trin, trin_ma };

        self.current = Some(output.clone());
        Some(output)
    }
}

impl<'a> Next<&'a TrinInput> for Trin {
    type Output = TrinOutput;

    fn next(&mut self, input: &'a TrinInput) -> Option<Self::Output> {
        self.next(*input)
    }
}

impl Next<TrinInput> for Box<Trin> {
    type Output = TrinOutput;

    fn next(&mut self, input: TrinInput) -> Option<Self::Output> {
        (**self).next(input)
    }
}

impl Reset for Trin {
    fn reset(&mut self) {
        self.ma.reset();
        self.current = None;
    }
}

impl Current for Trin {
    type Output = TrinOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current.clone()
    }
}

impl Default for Trin {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let trin = Trin::new(10);
        assert_eq!(trin.period(), 10);
    }

    #[test]
    #[should_panic(expected = "period must be greater than 0")]
    fn test_new_invalid_zero_period() {
        Trin::new(0);
    }

    #[test]
    fn test_default() {
        let trin = Trin::default();
        assert_eq!(trin.period, 10);
    }

    #[test]
    fn test_bullish_trin() {
        let mut trin = Trin::new(5);

        // Strong bullish: more advancing issues with higher volume
        // (Advancing issues, Declining issues, Advancing volume, Declining volume)
        let data = vec![
            (2000.0, 500.0, 100000000.0, 20000000.0),
            (2100.0, 400.0, 110000000.0, 18000000.0),
            (2200.0, 300.0, 120000000.0, 15000000.0),
            (2300.0, 200.0, 130000000.0, 12000000.0),
            (2400.0, 100.0, 140000000.0, 10000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Bullish TRIN should be < 1.0
        assert!(result.trin < 1.0);
    }

    #[test]
    fn test_bearish_trin() {
        let mut trin = Trin::new(5);

        // Strong bearish: more declining issues with higher volume
        // (Advancing issues, Declining issues, Advancing volume, Declining volume)
        let data = vec![
            (500.0, 2000.0, 20000000.0, 100000000.0),
            (400.0, 2100.0, 18000000.0, 110000000.0),
            (300.0, 2200.0, 15000000.0, 120000000.0),
            (200.0, 2300.0, 12000000.0, 130000000.0),
            (100.0, 2400.0, 10000000.0, 140000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Bearish TRIN should be > 1.0
        assert!(result.trin > 1.0);
    }

    #[test]
    fn test_neutral_trin() {
        let mut trin = Trin::new(5);

        // Neutral: balanced advancing/declining with proportional volume
        // (Advancing issues, Declining issues, Advancing volume, Declining volume)
        let data = vec![
            (1500.0, 1500.0, 60000000.0, 60000000.0),
            (1500.0, 1500.0, 60000000.0, 60000000.0),
            (1500.0, 1500.0, 60000000.0, 60000000.0),
            (1500.0, 1500.0, 60000000.0, 60000000.0),
            (1500.0, 1500.0, 60000000.0, 60000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Neutral TRIN should be ~1.0
        assert!((result.trin - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_trin_calculation() {
        let mut trin = Trin::new(5);

        // Manual calculation test
        // (2000, 1000, 100M, 40M)
        // TRIN = (2000 * 40M) / (1000 * 100M) = 80B / 100B = 0.8
        let data = vec![
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2000.0, 1000.0, 100000000.0, 40000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        let result = last_result.unwrap();
        assert!((result.trin - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_zero_declining_issues() {
        let mut trin = Trin::new(5);

        // Zero declining issues should return None (division by zero)
        let result = trin.next((2000.0, 0.0, 100000000.0, 10000000.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_zero_advancing_volume() {
        let mut trin = Trin::new(5);

        // Zero advancing volume should return None (division by zero)
        let result = trin.next((2000.0, 1000.0, 0.0, 10000000.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_moving_average() {
        let mut trin = Trin::new(3);

        let data = vec![
            (2000.0, 1000.0, 100000000.0, 40000000.0), // TRIN = 0.8
            (1800.0, 1200.0, 90000000.0, 48000000.0),  // TRIN = 0.96
            (1600.0, 1400.0, 80000000.0, 56000000.0),  // TRIN = 1.12
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        let result = last_result.unwrap();

        // MA should be average of 0.8, 0.96, 1.12 = 2.88 / 3 = 0.96
        assert!((result.trin_ma - 0.96).abs() < 0.01);
    }

    #[test]
    fn test_extreme_bullish() {
        let mut trin = Trin::new(5);

        // Very bullish: 10:1 advancing with heavy volume
        let data = vec![
            (3000.0, 300.0, 150000000.0, 10000000.0),
            (3100.0, 200.0, 160000000.0, 8000000.0),
            (3200.0, 100.0, 170000000.0, 5000000.0),
            (3300.0, 50.0, 180000000.0, 3000000.0),
            (3400.0, 25.0, 190000000.0, 2000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        let result = last_result.unwrap();

        // Should be very low (significantly < 1.0)
        assert!(result.trin < 0.5);
    }

    #[test]
    fn test_extreme_bearish() {
        let mut trin = Trin::new(5);

        // Very bearish: 10:1 declining with heavy volume
        let data = vec![
            (300.0, 3000.0, 10000000.0, 150000000.0),
            (200.0, 3100.0, 8000000.0, 160000000.0),
            (100.0, 3200.0, 5000000.0, 170000000.0),
            (50.0, 3300.0, 3000000.0, 180000000.0),
            (25.0, 3400.0, 2000000.0, 190000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        let result = last_result.unwrap();

        // Should be very high (significantly > 1.0)
        assert!(result.trin > 2.0);
    }

    #[test]
    fn test_reset() {
        let mut trin = Trin::new(5);

        let data = vec![
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2100.0, 900.0, 110000000.0, 35000000.0),
            (2200.0, 800.0, 120000000.0, 30000000.0),
            (2300.0, 700.0, 130000000.0, 25000000.0),
            (2400.0, 600.0, 140000000.0, 20000000.0),
        ];

        for point in &data {
            trin.next(*point);
        }

        let result_before = trin.current().unwrap();

        trin.reset();
        assert!(trin.current().is_none());

        for point in &data {
            trin.next(*point);
        }

        let result_after = trin.current().unwrap();
        assert_eq!(result_before.trin, result_after.trin);
        assert_eq!(result_before.trin_ma, result_after.trin_ma);
    }

    #[test]
    fn test_current() {
        let mut trin = Trin::new(5);
        assert!(trin.current().is_none());

        let data = vec![
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2100.0, 900.0, 110000000.0, 35000000.0),
            (2200.0, 800.0, 120000000.0, 30000000.0),
            (2300.0, 700.0, 130000000.0, 25000000.0),
            (2400.0, 600.0, 140000000.0, 20000000.0),
        ];

        for point in &data {
            trin.next(*point);
        }

        assert!(trin.current().is_some());
    }

    #[test]
    fn test_sentiment_shift() {
        let mut trin = Trin::new(5);

        // Start bullish
        let bullish_data = vec![
            (2500.0, 500.0, 120000000.0, 20000000.0),
            (2600.0, 400.0, 130000000.0, 18000000.0),
            (2700.0, 300.0, 140000000.0, 15000000.0),
            (2800.0, 200.0, 150000000.0, 12000000.0),
            (2900.0, 100.0, 160000000.0, 10000000.0),
        ];

        for point in &bullish_data {
            trin.next(*point);
        }

        let bullish_result = trin.current().unwrap();
        assert!(bullish_result.trin < 1.0);

        // Shift to bearish
        let bearish_data = vec![
            (500.0, 2500.0, 20000000.0, 120000000.0),
            (400.0, 2600.0, 18000000.0, 130000000.0),
            (300.0, 2700.0, 15000000.0, 140000000.0),
            (200.0, 2800.0, 12000000.0, 150000000.0),
            (100.0, 2900.0, 10000000.0, 160000000.0),
        ];

        for point in &bearish_data {
            trin.next(*point);
        }

        let bearish_result = trin.current().unwrap();
        assert!(bearish_result.trin > 1.0);
    }

    #[test]
    fn test_divergence_detection_setup() {
        let mut trin = Trin::new(3);

        // Declining TRIN values (improving sentiment)
        let data = vec![
            (1200.0, 1800.0, 50000000.0, 100000000.0), // Bearish
            (1500.0, 1500.0, 60000000.0, 60000000.0),  // Neutral
            (1800.0, 1200.0, 100000000.0, 50000000.0), // Bullish
        ];

        let mut trin_values = Vec::new();
        for point in &data {
            if let Some(result) = trin.next(*point) {
                trin_values.push(result.trin);
            }
        }

        // TRIN should decline from bearish to bullish
        assert!(trin_values.len() == 3);
        assert!(trin_values[0] > trin_values[1]);
        assert!(trin_values[1] > trin_values[2]);
    }

    #[test]
    fn test_reference_input() {
        let mut trin = Trin::new(5);

        let data = vec![
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2100.0, 900.0, 110000000.0, 35000000.0),
            (2200.0, 800.0, 120000000.0, 30000000.0),
            (2300.0, 700.0, 130000000.0, 25000000.0),
            (2400.0, 600.0, 140000000.0, 20000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_boxed() {
        let mut trin: Box<Trin> = Box::new(Trin::new(5));

        let data = vec![
            (2000.0, 1000.0, 100000000.0, 40000000.0),
            (2100.0, 900.0, 110000000.0, 35000000.0),
            (2200.0, 800.0, 120000000.0, 30000000.0),
            (2300.0, 700.0, 130000000.0, 25000000.0),
            (2400.0, 600.0, 140000000.0, 20000000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_trin_ma_smoothing() {
        let mut trin = Trin::new(5);

        // Consistent TRIN values
        let data = vec![
            (2000.0, 1000.0, 100000000.0, 40000000.0), // TRIN = 0.8
            (2000.0, 1000.0, 100000000.0, 40000000.0), // TRIN = 0.8
            (2000.0, 1000.0, 100000000.0, 40000000.0), // TRIN = 0.8
            (2000.0, 1000.0, 100000000.0, 40000000.0), // TRIN = 0.8
            (2000.0, 1000.0, 100000000.0, 40000000.0), // TRIN = 0.8
        ];

        let mut last_result = None;
        for point in &data {
            last_result = trin.next(*point);
        }

        let result = last_result.unwrap();

        // With constant TRIN, MA should equal TRIN
        assert!((result.trin - result.trin_ma).abs() < 0.01);
    }

    #[test]
    fn test_trin_volatility() {
        let mut trin = Trin::new(5);

        // Alternating bullish/bearish
        let data = vec![
            (2500.0, 500.0, 120000000.0, 20000000.0), // Bullish
            (500.0, 2500.0, 20000000.0, 120000000.0), // Bearish
            (2500.0, 500.0, 120000000.0, 20000000.0), // Bullish
            (500.0, 2500.0, 20000000.0, 120000000.0), // Bearish
            (2500.0, 500.0, 120000000.0, 20000000.0), // Bullish
        ];

        let mut trin_values = Vec::new();
        for point in &data {
            if let Some(result) = trin.next(*point) {
                trin_values.push(result.trin);
            }
        }

        // Should alternate between low and high values
        assert!(trin_values.len() == 5);
        assert!(trin_values[0] < 1.0);
        assert!(trin_values[1] > 1.0);
        assert!(trin_values[2] < 1.0);
        assert!(trin_values[3] > 1.0);
        assert!(trin_values[4] < 1.0);
    }
}
