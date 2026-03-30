//! Beta
//!
//! Measures the volatility (systematic risk) of a security or portfolio compared to
//! the market as a whole. Beta is calculated as the covariance of the asset's returns
//! with the market's returns divided by the variance of the market's returns.
//!
//! # Formula
//!
//! ```text
//! Beta = Covariance(Asset Returns, Market Returns) / Variance(Market Returns)
//! Beta = Covariance(Asset, Market) / (StdDev(Market)²)
//! ```
//!
//! # Interpretation
//!
//! - Beta = 1.0: Asset moves in line with the market
//! - Beta > 1.0: Asset is more volatile than the market
//! - Beta < 1.0: Asset is less volatile than the market
//! - Beta < 0.0: Asset moves inversely to the market
//!
//! # Examples
//!
//! ```
//! use loom_indicators::prelude::*;
//! use loom_indicators::indicators::trend::Beta;
//!
//! let mut beta = Beta::new(20);
//!
//! // (Asset price, Market price)
//! let result1 = beta.next((100.0, 4000.0));
//! let result2 = beta.next((101.0, 4010.0));
//! ```

use crate::indicators::{Current, Next, Period, Reset};
#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Beta indicator
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Beta {
    period: usize,
    asset_prices: VecDeque<f64>,
    market_prices: VecDeque<f64>,
    current: Option<f64>,
}

impl Beta {
    /// Creates a new Beta indicator
    ///
    /// # Arguments
    ///
    /// * `period` - Period for beta calculation (default: 20)
    ///
    /// # Errors
    ///
    /// Returns `IndicatorError::InvalidParameter` if period < 2
    pub fn new(period: usize) -> Self {
        assert!(period >= 2, "period must be at least 2");

        Self {
            period,
            asset_prices: VecDeque::with_capacity(period + 1),
            market_prices: VecDeque::with_capacity(period + 1),
            current: None,
        }
    }

    /// Calculate beta from price series
    fn calculate_beta(&self) -> Option<f64> {
        if self.asset_prices.len() < 2 || self.market_prices.len() < 2 {
            return None;
        }

        // Calculate returns
        let mut asset_returns = Vec::new();
        let mut market_returns = Vec::new();

        for i in 1..self.asset_prices.len() {
            let asset_ret =
                (self.asset_prices[i] - self.asset_prices[i - 1]) / self.asset_prices[i - 1];
            let market_ret =
                (self.market_prices[i] - self.market_prices[i - 1]) / self.market_prices[i - 1];

            asset_returns.push(asset_ret);
            market_returns.push(market_ret);
        }

        let n = asset_returns.len();
        if n == 0 {
            return None;
        }

        // Calculate means
        let asset_mean = asset_returns.iter().sum::<f64>() / n as f64;
        let market_mean = market_returns.iter().sum::<f64>() / n as f64;

        // Calculate covariance and variance
        let mut covariance = 0.0;
        let mut market_variance = 0.0;

        for i in 0..n {
            let asset_dev = asset_returns[i] - asset_mean;
            let market_dev = market_returns[i] - market_mean;

            covariance += asset_dev * market_dev;
            market_variance += market_dev * market_dev;
        }

        covariance /= n as f64;
        market_variance /= n as f64;

        // Avoid division by zero
        if market_variance == 0.0 {
            return None;
        }

        Some(covariance / market_variance)
    }
}

impl Period for Beta {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<(f64, f64)> for Beta {
    type Output = f64;

    fn next(&mut self, (asset_price, market_price): (f64, f64)) -> Option<Self::Output> {
        // Add new prices
        self.asset_prices.push_back(asset_price);
        self.market_prices.push_back(market_price);

        // Maintain window size (+1 for return calculation)
        while self.asset_prices.len() > self.period + 1 {
            self.asset_prices.pop_front();
        }
        while self.market_prices.len() > self.period + 1 {
            self.market_prices.pop_front();
        }

        // Calculate beta
        let beta = self.calculate_beta()?;

        self.current = Some(beta);
        Some(beta)
    }
}

impl<'a> Next<&'a (f64, f64)> for Beta {
    type Output = f64;

    fn next(&mut self, input: &'a (f64, f64)) -> Option<Self::Output> {
        self.next(*input)
    }
}

impl Next<(f64, f64)> for Box<Beta> {
    type Output = f64;

    fn next(&mut self, input: (f64, f64)) -> Option<Self::Output> {
        (**self).next(input)
    }
}

impl Reset for Beta {
    fn reset(&mut self) {
        self.asset_prices.clear();
        self.market_prices.clear();
        self.current = None;
    }
}

impl Current for Beta {
    type Output = f64;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Default for Beta {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let beta = Beta::new(20);
        assert_eq!(beta.period(), 20);
    }

    #[test]
    #[should_panic(expected = "period must be at least 2")]
    fn test_new_invalid_period() {
        Beta::new(1);
    }

    #[test]
    fn test_default() {
        let beta = Beta::default();
        assert_eq!(beta.period, 20);
    }

    #[test]
    fn test_beta_equal_movement() {
        let mut beta = Beta::new(10);

        // Asset and market move in perfect sync (beta should be ~1.0)
        let data = vec![
            (100.0, 4000.0),
            (101.0, 4040.0), // Both up 1%
            (102.0, 4080.0), // Both up ~1%
            (103.0, 4120.0),
            (104.0, 4160.0),
            (105.0, 4200.0),
            (106.0, 4240.0),
            (107.0, 4280.0),
            (108.0, 4320.0),
            (109.0, 4360.0),
            (110.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Beta should be close to 1.0 for equal percentage movements
        assert!((result - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_beta_more_volatile() {
        let mut beta = Beta::new(10);

        // Asset moves 2x the market (beta should be ~2.0)
        let data = vec![
            (100.0, 4000.0),
            (102.0, 4040.0), // Asset up 2%, market up 1%
            (104.0, 4080.0),
            (106.0, 4120.0),
            (108.0, 4160.0),
            (110.0, 4200.0),
            (112.0, 4240.0),
            (114.0, 4280.0),
            (116.0, 4320.0),
            (118.0, 4360.0),
            (120.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Beta should be close to 2.0
        assert!(result > 1.5);
    }

    #[test]
    fn test_beta_less_volatile() {
        let mut beta = Beta::new(10);

        // Asset moves 0.5x the market (beta should be ~0.5)
        let data = vec![
            (100.0, 4000.0),
            (100.5, 4040.0), // Asset up 0.5%, market up 1%
            (101.0, 4080.0),
            (101.5, 4120.0),
            (102.0, 4160.0),
            (102.5, 4200.0),
            (103.0, 4240.0),
            (103.5, 4280.0),
            (104.0, 4320.0),
            (104.5, 4360.0),
            (105.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Beta should be close to 0.5
        assert!(result < 1.0);
        assert!(result > 0.0);
    }

    #[test]
    fn test_beta_negative() {
        let mut beta = Beta::new(10);

        // Asset moves inverse to market (beta should be negative)
        let data = vec![
            (100.0, 4000.0),
            (99.0, 4040.0), // Asset down 1%, market up 1%
            (98.0, 4080.0),
            (97.0, 4120.0),
            (96.0, 4160.0),
            (95.0, 4200.0),
            (94.0, 4240.0),
            (93.0, 4280.0),
            (92.0, 4320.0),
            (91.0, 4360.0),
            (90.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Beta should be negative
        assert!(result < 0.0);
    }

    #[test]
    fn test_beta_zero_correlation() {
        let mut beta = Beta::new(10);

        // Asset price constant while market moves (beta should be ~0)
        let data = vec![
            (100.0, 4000.0),
            (100.0, 4040.0),
            (100.0, 4080.0),
            (100.0, 4120.0),
            (100.0, 4160.0),
            (100.0, 4200.0),
            (100.0, 4240.0),
            (100.0, 4280.0),
            (100.0, 4320.0),
            (100.0, 4360.0),
            (100.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        assert!(last_result.is_some());
        let result = last_result.unwrap();

        // Beta should be close to 0
        assert!(result.abs() < 0.1);
    }

    #[test]
    fn test_constant_market() {
        let mut beta = Beta::new(10);

        // Market constant while asset moves (should return None - division by zero)
        let data = vec![
            (100.0, 4000.0),
            (101.0, 4000.0),
            (102.0, 4000.0),
            (103.0, 4000.0),
            (104.0, 4000.0),
            (105.0, 4000.0),
            (106.0, 4000.0),
            (107.0, 4000.0),
            (108.0, 4000.0),
            (109.0, 4000.0),
            (110.0, 4000.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        // Should return None due to zero market variance
        assert!(last_result.is_none());
    }

    #[test]
    fn test_insufficient_data() {
        let mut beta = Beta::new(10);

        // Only one data point
        let result = beta.next((100.0, 4000.0));
        assert!(result.is_none());
    }

    #[test]
    fn test_rolling_window() {
        let mut beta = Beta::new(5);

        // Feed more data than period
        let data = vec![
            (100.0, 4000.0),
            (101.0, 4040.0),
            (102.0, 4080.0),
            (103.0, 4120.0),
            (104.0, 4160.0),
            (105.0, 4200.0),
            (106.0, 4240.0),
            (107.0, 4280.0),
            (108.0, 4320.0),
            (109.0, 4360.0),
        ];

        let mut results = Vec::new();
        for point in &data {
            if let Some(result) = beta.next(*point) {
                results.push(result);
            }
        }

        // Should have calculated beta for each point after the first
        assert!(results.len() > 0);

        // Beta should remain relatively stable for consistent movements
        assert!(results.last().unwrap().abs() > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut beta = Beta::new(10);

        let data = vec![
            (100.0, 4000.0),
            (101.0, 4040.0),
            (102.0, 4080.0),
            (103.0, 4120.0),
            (104.0, 4160.0),
            (105.0, 4200.0),
            (106.0, 4240.0),
            (107.0, 4280.0),
            (108.0, 4320.0),
            (109.0, 4360.0),
            (110.0, 4400.0),
        ];

        for point in &data {
            beta.next(*point);
        }

        let result_before = beta.current().unwrap();

        beta.reset();
        assert!(beta.current().is_none());
        assert_eq!(beta.asset_prices.len(), 0);
        assert_eq!(beta.market_prices.len(), 0);

        for point in &data {
            beta.next(*point);
        }

        let result_after = beta.current().unwrap();
        assert!((result_before - result_after).abs() < 0.01);
    }

    #[test]
    fn test_current() {
        let mut beta = Beta::new(10);
        assert!(beta.current().is_none());

        let data = vec![
            (100.0, 4000.0),
            (101.0, 4040.0),
            (102.0, 4080.0),
            (103.0, 4120.0),
            (104.0, 4160.0),
            (105.0, 4200.0),
            (106.0, 4240.0),
            (107.0, 4280.0),
            (108.0, 4320.0),
            (109.0, 4360.0),
            (110.0, 4400.0),
        ];

        for point in &data {
            beta.next(*point);
        }

        assert!(beta.current().is_some());
    }

    #[test]
    fn test_mixed_movements() {
        let mut beta = Beta::new(10);

        // Mixed up and down movements
        let data = vec![
            (100.0, 4000.0),
            (102.0, 4040.0), // Both up
            (101.0, 4080.0), // Asset down, market up
            (103.0, 4120.0), // Both up
            (102.0, 4100.0), // Asset down, market down
            (104.0, 4140.0), // Both up
            (103.0, 4160.0), // Asset down, market up
            (105.0, 4180.0), // Both up
            (104.0, 4160.0), // Asset down, market down
            (106.0, 4200.0), // Both up
            (105.0, 4220.0), // Asset down, market up
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        assert!(last_result.is_some());
        // Beta should be positive but less than perfect correlation
        let result = last_result.unwrap();
        assert!(result > 0.0);
    }

    #[test]
    fn test_reference_input() {
        let mut beta = Beta::new(10);

        let data = vec![
            (100.0, 4000.0),
            (101.0, 4040.0),
            (102.0, 4080.0),
            (103.0, 4120.0),
            (104.0, 4160.0),
            (105.0, 4200.0),
            (106.0, 4240.0),
            (107.0, 4280.0),
            (108.0, 4320.0),
            (109.0, 4360.0),
            (110.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_boxed() {
        let mut beta: Box<Beta> = Box::new(Beta::new(10));

        let data = vec![
            (100.0, 4000.0),
            (101.0, 4040.0),
            (102.0, 4080.0),
            (103.0, 4120.0),
            (104.0, 4160.0),
            (105.0, 4200.0),
            (106.0, 4240.0),
            (107.0, 4280.0),
            (108.0, 4320.0),
            (109.0, 4360.0),
            (110.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        assert!(last_result.is_some());
    }

    #[test]
    fn test_high_beta_stock() {
        let mut beta = Beta::new(10);

        // Tech stock scenario - amplified movements
        let data = vec![
            (100.0, 4000.0),
            (103.0, 4040.0), // Asset up 3%, market up 1%
            (106.0, 4080.0),
            (109.0, 4120.0),
            (112.0, 4160.0),
            (115.0, 4200.0),
            (118.0, 4240.0),
            (121.0, 4280.0),
            (124.0, 4320.0),
            (127.0, 4360.0),
            (130.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        let result = last_result.unwrap();

        // Beta should be significantly > 1.0 (around 3.0)
        assert!(result > 2.0);
    }

    #[test]
    fn test_defensive_stock() {
        let mut beta = Beta::new(10);

        // Defensive stock - minimal movements
        let data = vec![
            (100.0, 4000.0),
            (100.2, 4040.0), // Asset up 0.2%, market up 1%
            (100.4, 4080.0),
            (100.6, 4120.0),
            (100.8, 4160.0),
            (101.0, 4200.0),
            (101.2, 4240.0),
            (101.4, 4280.0),
            (101.6, 4320.0),
            (101.8, 4360.0),
            (102.0, 4400.0),
        ];

        let mut last_result = None;
        for point in &data {
            last_result = beta.next(*point);
        }

        let result = last_result.unwrap();

        // Beta should be significantly < 1.0 (around 0.2)
        assert!(result < 0.5);
        assert!(result > 0.0);
    }
}
