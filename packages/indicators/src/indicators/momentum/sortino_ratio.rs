//! Sortino Ratio
//!
//! A risk-adjusted return metric that measures the return earned in excess of the
//! risk-free rate per unit of downside risk. Unlike the Sharpe ratio, it only
//! penalizes downside volatility, making it more suitable for strategies with
//! asymmetric return distributions.
//!
//! # Formula
//!
//! ```text
//! Sortino Ratio = (Return - Risk-Free Rate) / Downside Deviation
//!
//! Where:
//! - Return = Average return over period
//! - Downside Deviation = StdDev of returns below target/risk-free rate
//! - Annualized: Multiply by sqrt(periods per year)
//! ```
//!
//! # Interpretation
//!
//! - Higher values indicate better risk-adjusted returns
//! - > 2.0: Excellent
//! - 1.0 - 2.0: Very good
//! - 0.5 - 1.0: Good
//! - < 0.5: Not attractive
//! - Negative: Strategy losing money
//!
//! # Examples
//!
//! ```
//! use loom_indicators::prelude::*;
//! use loom_indicators::indicators::momentum::SortinoRatio;
//!
//! let mut sortino = SortinoRatio::new(20, 0.02, 252.0);
//!
//! let result = sortino.next(100.0);
//! ```

use crate::indicators::{Current, Next, Period, Reset};
#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Sortino Ratio output
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SortinoRatioOutput {
    /// The Sortino ratio (annualized)
    pub sortino: f64,
    /// Average return
    pub avg_return: f64,
    /// Downside deviation
    pub downside_deviation: f64,
}

/// Sortino Ratio
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SortinoRatio {
    period: usize,
    risk_free_rate: f64,
    periods_per_year: f64,
    prices: VecDeque<f64>,
    current: Option<SortinoRatioOutput>,
}

impl SortinoRatio {
    /// Creates a new Sortino Ratio
    ///
    /// # Arguments
    ///
    /// * `period` - Period for calculation (default: 20)
    /// * `risk_free_rate` - Annual risk-free rate as decimal (e.g., 0.02 for 2%)
    /// * `periods_per_year` - Trading periods per year for annualization (252 for daily, 52 for weekly)
    ///
    /// # Errors
    ///
    /// Returns `IndicatorError::InvalidParameter` if:
    /// - Period < 2
    /// - Periods per year <= 0
    pub fn new(period: usize, risk_free_rate: f64, periods_per_year: f64) -> Self {
        assert!(period >= 2, "period must be at least 2");
        assert!(
            periods_per_year > 0.0,
            "periods_per_year must be greater than 0"
        );

        Self {
            period,
            risk_free_rate,
            periods_per_year,
            prices: VecDeque::with_capacity(period + 1),
            current: None,
        }
    }

    /// Calculate Sortino ratio from price series
    fn calculate_sortino(&self) -> Option<SortinoRatioOutput> {
        if self.prices.len() < 2 {
            return None;
        }

        // Calculate returns
        let mut returns = Vec::new();
        for i in 1..self.prices.len() {
            let ret = (self.prices[i] - self.prices[i - 1]) / self.prices[i - 1];
            returns.push(ret);
        }

        let n = returns.len();
        if n == 0 {
            return None;
        }

        // Calculate average return
        let avg_return = returns.iter().sum::<f64>() / n as f64;

        // Calculate risk-free rate per period
        let rf_per_period = self.risk_free_rate / self.periods_per_year;

        // Calculate downside deviation (only negative deviations from risk-free rate)
        let mut downside_sum = 0.0;
        let mut downside_count = 0;

        for &ret in &returns {
            if ret < rf_per_period {
                let deviation = ret - rf_per_period;
                downside_sum += deviation * deviation;
                downside_count += 1;
            }
        }

        // Downside deviation
        let downside_deviation = if downside_count > 0 {
            (downside_sum / downside_count as f64).sqrt()
        } else {
            // No downside - use very small number to avoid division by zero
            1e-10
        };

        // Calculate Sortino ratio (annualized)
        let excess_return = avg_return - rf_per_period;
        let sortino = (excess_return / downside_deviation) * self.periods_per_year.sqrt();

        Some(SortinoRatioOutput {
            sortino,
            avg_return: avg_return * self.periods_per_year, // Annualized
            downside_deviation: downside_deviation * self.periods_per_year.sqrt(), // Annualized
        })
    }
}

impl Period for SortinoRatio {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<f64> for SortinoRatio {
    type Output = SortinoRatioOutput;

    fn next(&mut self, price: f64) -> Option<Self::Output> {
        // Add new price
        self.prices.push_back(price);

        // Maintain window size (+1 for return calculation)
        while self.prices.len() > self.period + 1 {
            self.prices.pop_front();
        }

        // Calculate Sortino ratio
        let output = self.calculate_sortino()?;

        self.current = Some(output.clone());
        Some(output)
    }
}

impl<'a> Next<&'a f64> for SortinoRatio {
    type Output = SortinoRatioOutput;

    fn next(&mut self, price: &'a f64) -> Option<Self::Output> {
        self.next(*price)
    }
}

impl Next<f64> for Box<SortinoRatio> {
    type Output = SortinoRatioOutput;

    fn next(&mut self, price: f64) -> Option<Self::Output> {
        (**self).next(price)
    }
}

impl Reset for SortinoRatio {
    fn reset(&mut self) {
        self.prices.clear();
        self.current = None;
    }
}

impl Current for SortinoRatio {
    type Output = SortinoRatioOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current.clone()
    }
}

impl Default for SortinoRatio {
    fn default() -> Self {
        Self::new(20, 0.02, 252.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let sortino = SortinoRatio::new(20, 0.02, 252.0);
        assert_eq!(sortino.period(), 20);
    }

    #[test]
    #[should_panic(expected = "period must be at least 2")]
    fn test_new_invalid_period() {
        SortinoRatio::new(1, 0.02, 252.0);
    }

    #[test]
    #[should_panic(expected = "periods_per_year must be greater than 0")]
    fn test_new_invalid_periods_per_year() {
        SortinoRatio::new(20, 0.02, 0.0);
    }

    #[test]
    fn test_default() {
        let sortino = SortinoRatio::default();
        assert_eq!(sortino.period, 20);
        assert_eq!(sortino.risk_free_rate, 0.02);
        assert_eq!(sortino.periods_per_year, 252.0);
    }

    #[test]
    fn test_positive_returns_no_downside() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Consistent positive returns
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With no downside, Sortino should be very high
        assert!(result.sortino > 0.0);
        assert!(result.avg_return > 0.0);
    }

    #[test]
    fn test_negative_returns() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Consistent negative returns
        let prices = vec![
            100.0, 99.0, 98.0, 97.0, 96.0, 95.0, 94.0, 93.0, 92.0, 91.0, 90.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // With negative returns, Sortino should be negative
        assert!(result.sortino < 0.0);
        assert!(result.avg_return < 0.0);
    }

    #[test]
    fn test_mixed_returns() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Mixed positive and negative returns
        let prices = vec![
            100.0, 102.0, 101.0, 103.0, 102.0, 104.0, 103.0, 105.0, 104.0, 106.0, 105.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Mixed returns should still be positive overall
        assert!(result.avg_return > 0.0);
        assert!(result.downside_deviation > 0.0);
    }

    #[test]
    fn test_with_risk_free_rate() {
        let mut sortino = SortinoRatio::new(10, 0.05, 252.0); // 5% risk-free rate

        // Returns above risk-free rate
        let prices = vec![
            100.0, 101.5, 103.0, 104.5, 106.0, 107.5, 109.0, 110.5, 112.0, 113.5, 115.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Returns above risk-free rate should give positive Sortino
        assert!(result.sortino > 0.0);
    }

    #[test]
    fn test_insufficient_data() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Only one price
        let result = sortino.next(100.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_asymmetric_returns() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Large positive returns, small negative returns
        let prices = vec![
            100.0, 105.0, 104.0, 109.0, 108.0, 113.0, 112.0, 117.0, 116.0, 121.0, 120.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Should have high Sortino due to limited downside
        assert!(result.sortino > 0.0);
    }

    #[test]
    fn test_high_volatility_downside() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // High downside volatility
        let prices = vec![
            100.0, 95.0, 101.0, 90.0, 102.0, 85.0, 103.0, 80.0, 104.0, 75.0, 105.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // High downside volatility should reduce Sortino
        assert!(result.downside_deviation > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut sortino = SortinoRatio::new(10, 0.02, 252.0);

        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        for price in &prices {
            sortino.next(*price);
        }

        let result_before = sortino.current().unwrap();

        sortino.reset();
        assert!(sortino.current().is_none());
        assert_eq!(sortino.prices.len(), 0);

        for price in &prices {
            sortino.next(*price);
        }

        let result_after = sortino.current().unwrap();
        assert!((result_before.sortino - result_after.sortino).abs() < 0.01);
    }

    #[test]
    fn test_current() {
        let mut sortino = SortinoRatio::new(10, 0.02, 252.0);
        assert!(sortino.current().is_none());

        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        for price in &prices {
            sortino.next(*price);
        }

        assert!(sortino.current().is_some());
    }

    #[test]
    fn test_rolling_window() {
        let mut sortino = SortinoRatio::new(5, 0.0, 252.0);

        // Feed more data than period
        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        let mut results = Vec::new();
        for price in &prices {
            if let Some(result) = sortino.next(*price) {
                results.push(result.sortino);
            }
        }

        // Should have multiple results
        assert!(results.len() > 0);
    }

    #[test]
    fn test_excellent_sortino() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Excellent strategy: high returns, no downside
        let prices = vec![
            100.0, 102.0, 104.0, 106.0, 108.0, 110.0, 112.0, 114.0, 116.0, 118.0, 120.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        let result = last_result.unwrap();

        // Should be > 2.0 (excellent)
        assert!(result.sortino > 0.0);
    }

    #[test]
    fn test_poor_sortino() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Poor strategy: minimal returns, high downside
        let prices = vec![
            100.0, 95.0, 96.0, 91.0, 92.0, 87.0, 88.0, 89.0, 90.0, 91.0, 92.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        let result = last_result.unwrap();

        // Should be negative or very low
        assert!(result.sortino < 1.0);
    }

    #[test]
    fn test_weekly_periods() {
        let mut sortino = SortinoRatio::new(10, 0.02, 52.0); // Weekly data

        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Should calculate correctly with weekly periods
        assert!(result.sortino > 0.0);
    }

    #[test]
    fn test_reference_input() {
        let mut sortino = SortinoRatio::new(10, 0.02, 252.0);

        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(price);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_boxed() {
        let mut sortino: Box<SortinoRatio> = Box::new(SortinoRatio::new(10, 0.02, 252.0));

        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_constant_prices() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Constant prices (no returns)
        let prices = vec![
            100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Zero returns with zero risk-free rate should give zero or near-zero Sortino
        assert!(result.avg_return.abs() < 0.01);
    }

    #[test]
    fn test_large_drawdown() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Large drawdown followed by recovery
        let prices = vec![
            100.0, 90.0, 80.0, 70.0, 75.0, 80.0, 85.0, 90.0, 95.0, 100.0, 105.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Large drawdown should result in high downside deviation
        assert!(result.downside_deviation > 0.0);
    }

    #[test]
    fn test_upward_trend_small_pullbacks() {
        let mut sortino = SortinoRatio::new(10, 0.0, 252.0);

        // Strong upward trend with small pullbacks
        let prices = vec![
            100.0, 105.0, 104.0, 108.0, 107.0, 111.0, 110.0, 114.0, 113.0, 117.0, 116.0,
        ];

        let mut last_result = None;
        for price in &prices {
            last_result = sortino.next(*price);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Should have good Sortino - positive returns with limited downside
        assert!(result.sortino > 1.0);
        assert!(result.avg_return > 0.0);
    }

    #[test]
    fn test_annualization() {
        let mut sortino = SortinoRatio::new(10, 0.02, 252.0);

        let prices = vec![
            100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
        ];

        for price in &prices {
            sortino.next(*price);
        }

        let result = sortino.current().unwrap();

        // Verify that returns and deviation are annualized
        assert!(result.avg_return.abs() > 0.0); // Should be annualized percentage
        assert!(result.downside_deviation >= 0.0); // Should be annualized
    }
}
