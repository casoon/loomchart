//! Correlation Coefficient

use crate::indicators::{Current, Next, Period, Reset};
use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Correlation Coefficient
///
/// Measures the strength and direction of the linear relationship between
/// two price series. Values range from -1 to +1.
///
/// # Interpretation
///
/// - **+1.0**: Perfect positive correlation (move together)
/// - **0.0**: No correlation (independent movement)
/// - **-1.0**: Perfect negative correlation (move opposite)
/// - **> 0.7**: Strong positive correlation
/// - **< -0.7**: Strong negative correlation
/// - **-0.3 to 0.3**: Weak or no correlation
///
/// # Formula
///
/// ```text
/// Correlation = Covariance(X, Y) / (StdDev(X) * StdDev(Y))
///
/// Where:
/// Covariance(X, Y) = Sum((X - Mean(X)) * (Y - Mean(Y))) / N
/// StdDev(X) = sqrt(Sum((X - Mean(X))^2) / N)
/// ```
///
/// # Parameters
///
/// * `period` - Lookback period (default: 20)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::Correlation;
/// use loom_indicators::traits::Next;
///
/// let mut corr = Correlation::new(20);
///
/// // Compare two price series
/// for (price1, price2) in price_pairs {
///     if let Some(correlation) = corr.next((price1, price2)) {
///         println!("Correlation: {:.3}", correlation);
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Correlation {
    period: usize,
    x_values: VecDeque<f64>,
    y_values: VecDeque<f64>,
    current: Option<f64>,
}

impl Correlation {
    /// Create a new Correlation indicator with the specified period
    pub fn new(period: usize) -> Self {
        assert!(period > 1, "Period must be greater than 1");

        Self {
            period,
            x_values: VecDeque::with_capacity(period),
            y_values: VecDeque::with_capacity(period),
            current: None,
        }
    }

    fn mean(values: &VecDeque<f64>) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    fn calculate(&self) -> f64 {
        if self.x_values.len() < 2 {
            return 0.0;
        }

        let x_mean = Self::mean(&self.x_values);
        let y_mean = Self::mean(&self.y_values);

        let n = self.x_values.len() as f64;

        // Calculate covariance
        let covariance: f64 = self
            .x_values
            .iter()
            .zip(self.y_values.iter())
            .map(|(x, y)| (x - x_mean) * (y - y_mean))
            .sum::<f64>()
            / n;

        // Calculate standard deviations
        let x_variance: f64 = self
            .x_values
            .iter()
            .map(|x| (x - x_mean).powi(2))
            .sum::<f64>()
            / n;

        let y_variance: f64 = self
            .y_values
            .iter()
            .map(|y| (y - y_mean).powi(2))
            .sum::<f64>()
            / n;

        let x_stddev = x_variance.sqrt();
        let y_stddev = y_variance.sqrt();

        // Avoid division by zero
        if x_stddev == 0.0 || y_stddev == 0.0 {
            return 0.0;
        }

        // Calculate correlation
        covariance / (x_stddev * y_stddev)
    }
}

impl Period for Correlation {
    fn period(&self) -> usize {
        self.period
    }
}

impl Next<(f64, f64)> for Correlation {
    type Output = f64;

    fn next(&mut self, (x, y): (f64, f64)) -> Option<Self::Output> {
        self.x_values.push_back(x);
        self.y_values.push_back(y);

        if self.x_values.len() > self.period {
            self.x_values.pop_front();
            self.y_values.pop_front();
        }

        if self.x_values.len() >= self.period {
            let correlation = self.calculate();
            self.current = Some(correlation);
            Some(correlation)
        } else {
            None
        }
    }
}

impl Current for Correlation {
    type Output = f64;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for Correlation {
    fn reset(&mut self) {
        self.x_values.clear();
        self.y_values.clear();
        self.current = None;
    }
}

impl Default for Correlation {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_new() {
        let corr = Correlation::new(20);
        assert_eq!(corr.period(), 20);
    }

    #[test]
    #[should_panic]
    fn test_correlation_invalid_period() {
        Correlation::new(1);
    }

    #[test]
    fn test_correlation_perfect_positive() {
        let mut corr = Correlation::new(10);

        // Perfect positive correlation (same values)
        for i in 0..15 {
            let val = 100.0 + i as f64;
            corr.next((val, val));
        }

        let result = corr.current().unwrap();
        assert!((result - 1.0).abs() < 0.01, "Should be close to 1.0");
    }

    #[test]
    fn test_correlation_perfect_negative() {
        let mut corr = Correlation::new(10);

        // Perfect negative correlation (inverse)
        for i in 0..15 {
            let x = 100.0 + i as f64;
            let y = 200.0 - i as f64;
            corr.next((x, y));
        }

        let result = corr.current().unwrap();
        assert!((result + 1.0).abs() < 0.01, "Should be close to -1.0");
    }

    #[test]
    fn test_correlation_no_correlation() {
        let mut corr = Correlation::new(10);

        // No correlation (constant vs varying)
        for i in 0..15 {
            let x = 100.0; // Constant
            let y = 100.0 + i as f64; // Varying
            corr.next((x, y));
        }

        let result = corr.current().unwrap();
        // Correlation should be 0 (or undefined, we return 0)
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_correlation_positive() {
        let mut corr = Correlation::new(10);

        // Positive correlation
        for i in 0..15 {
            let x = 100.0 + i as f64;
            let y = 150.0 + i as f64 * 2.0;
            corr.next((x, y));
        }

        let result = corr.current().unwrap();
        assert!(result > 0.9, "Should have strong positive correlation");
    }

    #[test]
    fn test_correlation_negative() {
        let mut corr = Correlation::new(10);

        // Negative correlation
        for i in 0..15 {
            let x = 100.0 + i as f64;
            let y = 200.0 - i as f64 * 2.0;
            corr.next((x, y));
        }

        let result = corr.current().unwrap();
        assert!(result < -0.9, "Should have strong negative correlation");
    }

    #[test]
    fn test_correlation_weak() {
        let mut corr = Correlation::new(10);

        // Weak correlation (random-ish)
        let pairs = vec![
            (100.0, 150.0),
            (101.0, 148.0),
            (102.0, 151.0),
            (103.0, 149.0),
            (104.0, 152.0),
            (105.0, 150.0),
            (106.0, 153.0),
            (107.0, 151.0),
            (108.0, 154.0),
            (109.0, 152.0),
        ];

        for (x, y) in pairs {
            corr.next((x, y));
        }

        let result = corr.current();
        assert!(result.is_some());
        // Correlation exists but may not be perfect
    }

    #[test]
    fn test_correlation_calculation() {
        let mut corr = Correlation::new(5);

        // Known correlation
        let pairs = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0), (4.0, 8.0), (5.0, 10.0)];

        for (x, y) in pairs {
            corr.next((x, y));
        }

        let result = corr.current().unwrap();
        // Perfect linear relationship Y = 2X
        assert!((result - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_correlation_reset() {
        let mut corr = Correlation::new(10);

        for i in 0..15 {
            corr.next((100.0 + i as f64, 150.0 + i as f64));
        }

        assert!(corr.current().is_some());

        corr.reset();

        assert!(corr.current().is_none());
        assert_eq!(corr.x_values.len(), 0);
        assert_eq!(corr.y_values.len(), 0);
    }

    #[test]
    fn test_correlation_default() {
        let corr = Correlation::default();
        assert_eq!(corr.period(), 20);
    }

    #[test]
    fn test_correlation_insufficient_data() {
        let mut corr = Correlation::new(10);

        for i in 0..9 {
            let result = corr.next((100.0 + i as f64, 150.0 + i as f64));
            assert!(result.is_none(), "Should be None before period complete");
        }

        // 10th value should return Some
        let result = corr.next((109.0, 159.0));
        assert!(result.is_some(), "Should return Some at period");
    }

    #[test]
    fn test_correlation_rolling_window() {
        let mut corr = Correlation::new(5);

        for i in 0..10 {
            corr.next((100.0 + i as f64, 150.0 + i as f64));
        }

        // Should only keep last 5 values
        assert_eq!(corr.x_values.len(), 5);
        assert_eq!(corr.y_values.len(), 5);
    }

    #[test]
    fn test_correlation_mean() {
        let mut values = VecDeque::new();
        values.push_back(1.0);
        values.push_back(2.0);
        values.push_back(3.0);
        values.push_back(4.0);
        values.push_back(5.0);

        let mean = Correlation::mean(&values);
        assert_eq!(mean, 3.0);
    }

    #[test]
    fn test_correlation_zero_stddev() {
        let mut corr = Correlation::new(5);

        // All same values (zero standard deviation)
        for _ in 0..10 {
            corr.next((100.0, 100.0));
        }

        let result = corr.current().unwrap();
        // Should handle gracefully and return 0
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_correlation_alternating() {
        let mut corr = Correlation::new(10);

        // Alternating values
        for i in 0..15 {
            let x = if i % 2 == 0 { 100.0 } else { 110.0 };
            let y = if i % 2 == 0 { 150.0 } else { 160.0 };
            corr.next((x, y));
        }

        let result = corr.current().unwrap();
        // Should have perfect positive correlation
        assert!((result - 1.0).abs() < 0.01);
    }
}
