//! Fibonacci Retracement and Extension Levels

use crate::indicators::{Current, Next, Period, Reset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Fibonacci Levels Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FibonacciOutput {
    /// 0% level (start point)
    pub level_0: f64,
    /// 23.6% retracement
    pub level_236: f64,
    /// 38.2% retracement
    pub level_382: f64,
    /// 50% retracement
    pub level_500: f64,
    /// 61.8% retracement (golden ratio)
    pub level_618: f64,
    /// 78.6% retracement
    pub level_786: f64,
    /// 100% level (end point)
    pub level_100: f64,
    /// 161.8% extension
    pub extension_1618: f64,
    /// 261.8% extension
    pub extension_2618: f64,
    /// 423.6% extension
    pub extension_4236: f64,
}

impl FibonacciOutput {
    pub fn new(
        level_0: f64,
        level_236: f64,
        level_382: f64,
        level_500: f64,
        level_618: f64,
        level_786: f64,
        level_100: f64,
        extension_1618: f64,
        extension_2618: f64,
        extension_4236: f64,
    ) -> Self {
        Self {
            level_0,
            level_236,
            level_382,
            level_500,
            level_618,
            level_786,
            level_100,
            extension_1618,
            extension_2618,
            extension_4236,
        }
    }

    /// Get the nearest Fibonacci level to a given price
    pub fn nearest_level(&self, price: f64) -> f64 {
        let levels = vec![
            self.level_0,
            self.level_236,
            self.level_382,
            self.level_500,
            self.level_618,
            self.level_786,
            self.level_100,
            self.extension_1618,
            self.extension_2618,
            self.extension_4236,
        ];

        levels
            .iter()
            .min_by(|a, b| {
                let dist_a = (price - **a).abs();
                let dist_b = (price - **b).abs();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
            .unwrap_or(price)
    }

    /// Check if price is near a Fibonacci level (within tolerance percentage)
    pub fn is_near_level(&self, price: f64, tolerance_pct: f64) -> Option<f64> {
        let nearest = self.nearest_level(price);
        let distance_pct = ((price - nearest).abs() / nearest) * 100.0;

        if distance_pct <= tolerance_pct {
            Some(nearest)
        } else {
            None
        }
    }
}

/// Fibonacci Retracement and Extension Levels
///
/// Calculates Fibonacci retracement and extension levels based on a high and low point.
/// These levels are commonly used to identify potential support/resistance levels
/// and profit targets.
///
/// # Retracement Levels
///
/// - 23.6% - Minor retracement
/// - 38.2% - Common retracement
/// - 50.0% - Psychological level (not a Fibonacci ratio but commonly used)
/// - 61.8% - Golden ratio retracement (most significant)
/// - 78.6% - Deep retracement
///
/// # Extension Levels
///
/// - 161.8% - Common profit target
/// - 261.8% - Extended profit target
/// - 423.6% - Extreme extension
///
/// # Formula
///
/// For uptrend (low to high):
/// ```text
/// Retracement = High - (Range * Ratio)
/// Extension = High + (Range * (Ratio - 1.0))
/// ```
///
/// For downtrend (high to low):
/// ```text
/// Retracement = Low + (Range * Ratio)
/// Extension = Low - (Range * (Ratio - 1.0))
/// ```
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::Fibonacci;
///
/// // Calculate Fibonacci levels for an uptrend from 100 to 150
/// let fib = Fibonacci::calculate(100.0, 150.0, true);
///
/// println!("61.8% retracement: {:.2}", fib.level_618);
/// println!("161.8% extension: {:.2}", fib.extension_1618);
/// ```
#[derive(Clone)]
pub struct Fibonacci {
    is_uptrend: bool,
    current: Option<FibonacciOutput>,
}

impl Fibonacci {
    /// Create a new Fibonacci calculator
    ///
    /// # Arguments
    ///
    /// * `is_uptrend` - true for uptrend (low to high), false for downtrend (high to low)
    pub fn new(is_uptrend: bool) -> Self {
        Self {
            is_uptrend,
            current: None,
        }
    }

    /// Calculate Fibonacci levels from two price points
    ///
    /// # Arguments
    ///
    /// * `start` - Starting price (low for uptrend, high for downtrend)
    /// * `end` - Ending price (high for uptrend, low for downtrend)
    /// * `is_uptrend` - Direction of the move
    pub fn calculate(start: f64, end: f64, is_uptrend: bool) -> FibonacciOutput {
        let range = (end - start).abs();

        if is_uptrend {
            // Uptrend: start is low, end is high
            let level_0 = end;
            let level_100 = start;

            let level_236 = end - (range * 0.236);
            let level_382 = end - (range * 0.382);
            let level_500 = end - (range * 0.500);
            let level_618 = end - (range * 0.618);
            let level_786 = end - (range * 0.786);

            let extension_1618 = end + (range * 0.618);
            let extension_2618 = end + (range * 1.618);
            let extension_4236 = end + (range * 3.236);

            FibonacciOutput::new(
                level_0,
                level_236,
                level_382,
                level_500,
                level_618,
                level_786,
                level_100,
                extension_1618,
                extension_2618,
                extension_4236,
            )
        } else {
            // Downtrend: start is high, end is low
            let level_0 = end;
            let level_100 = start;

            let level_236 = end + (range * 0.236);
            let level_382 = end + (range * 0.382);
            let level_500 = end + (range * 0.500);
            let level_618 = end + (range * 0.618);
            let level_786 = end + (range * 0.786);

            let extension_1618 = end - (range * 0.618);
            let extension_2618 = end - (range * 1.618);
            let extension_4236 = end - (range * 3.236);

            FibonacciOutput::new(
                level_0,
                level_236,
                level_382,
                level_500,
                level_618,
                level_786,
                level_100,
                extension_1618,
                extension_2618,
                extension_4236,
            )
        }
    }
}

impl Period for Fibonacci {
    fn period(&self) -> usize {
        2 // Requires two points
    }
}

impl Next<(f64, f64)> for Fibonacci {
    type Output = FibonacciOutput;

    fn next(&mut self, (start, end): (f64, f64)) -> Option<Self::Output> {
        let output = Self::calculate(start, end, self.is_uptrend);
        self.current = Some(output);
        Some(output)
    }
}

impl Current for Fibonacci {
    type Output = FibonacciOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for Fibonacci {
    fn reset(&mut self) {
        self.current = None;
    }
}

impl Default for Fibonacci {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_uptrend() {
        let fib = Fibonacci::calculate(100.0, 150.0, true);

        // Range = 50
        // 0% = 150 (high)
        assert_eq!(fib.level_0, 150.0);

        // 100% = 100 (low)
        assert_eq!(fib.level_100, 100.0);

        // 23.6% = 150 - (50 * 0.236) = 138.2
        assert!((fib.level_236 - 138.2).abs() < 0.01);

        // 38.2% = 150 - (50 * 0.382) = 130.9
        assert!((fib.level_382 - 130.9).abs() < 0.01);

        // 50% = 150 - (50 * 0.5) = 125
        assert_eq!(fib.level_500, 125.0);

        // 61.8% = 150 - (50 * 0.618) = 119.1
        assert!((fib.level_618 - 119.1).abs() < 0.01);

        // 78.6% = 150 - (50 * 0.786) = 110.7
        assert!((fib.level_786 - 110.7).abs() < 0.01);

        // 161.8% extension = 150 + (50 * 0.618) = 180.9
        assert!((fib.extension_1618 - 180.9).abs() < 0.01);
    }

    #[test]
    fn test_fibonacci_downtrend() {
        let fib = Fibonacci::calculate(150.0, 100.0, false);

        // Range = 50
        // 0% = 100 (low)
        assert_eq!(fib.level_0, 100.0);

        // 100% = 150 (high)
        assert_eq!(fib.level_100, 150.0);

        // 23.6% = 100 + (50 * 0.236) = 111.8
        assert!((fib.level_236 - 111.8).abs() < 0.01);

        // 61.8% = 100 + (50 * 0.618) = 130.9
        assert!((fib.level_618 - 130.9).abs() < 0.01);

        // 161.8% extension = 100 - (50 * 0.618) = 69.1
        assert!((fib.extension_1618 - 69.1).abs() < 0.01);
    }

    #[test]
    fn test_fibonacci_next() {
        let mut fib = Fibonacci::new(true);

        let result = fib.next((100.0, 150.0)).unwrap();

        assert_eq!(result.level_0, 150.0);
        assert_eq!(result.level_100, 100.0);
        assert_eq!(result.level_500, 125.0);
    }

    #[test]
    fn test_fibonacci_golden_ratio() {
        let fib = Fibonacci::calculate(100.0, 200.0, true);

        // Range = 100
        // Golden ratio (61.8%) = 200 - (100 * 0.618) = 138.2
        assert!((fib.level_618 - 138.2).abs() < 0.01);
    }

    #[test]
    fn test_fibonacci_nearest_level() {
        let fib = Fibonacci::calculate(100.0, 150.0, true);

        // Price at 125 should be nearest to 50% level
        let nearest = fib.nearest_level(125.0);
        assert_eq!(nearest, 125.0);

        // Price at 140 should be nearest to 23.6% level (138.2)
        let nearest = fib.nearest_level(140.0);
        assert!((nearest - 138.2).abs() < 0.01);
    }

    #[test]
    fn test_fibonacci_is_near_level() {
        let fib = Fibonacci::calculate(100.0, 150.0, true);

        // Price at 125.5 should be near 50% level (125) with 1% tolerance
        let near = fib.is_near_level(125.5, 1.0);
        assert!(near.is_some());
        assert_eq!(near.unwrap(), 125.0);

        // Price at 140 should not be near 50% level (125) with 1% tolerance
        let near = fib.is_near_level(140.0, 1.0);
        assert_ne!(near, Some(125.0));
    }

    #[test]
    fn test_fibonacci_extensions() {
        let fib = Fibonacci::calculate(100.0, 200.0, true);

        // Range = 100
        // 161.8% = 200 + (100 * 0.618) = 261.8
        assert!((fib.extension_1618 - 261.8).abs() < 0.01);

        // 261.8% = 200 + (100 * 1.618) = 361.8
        assert!((fib.extension_2618 - 361.8).abs() < 0.01);

        // 423.6% = 200 + (100 * 3.236) = 523.6
        assert!((fib.extension_4236 - 523.6).abs() < 0.01);
    }

    #[test]
    fn test_fibonacci_reset() {
        let mut fib = Fibonacci::new(true);

        fib.next((100.0, 150.0));
        assert!(fib.current().is_some());

        fib.reset();
        assert!(fib.current().is_none());
    }

    #[test]
    fn test_fibonacci_default() {
        let fib = Fibonacci::default();
        assert!(fib.is_uptrend);
    }

    #[test]
    fn test_fibonacci_symmetry() {
        // Uptrend from 100 to 150
        let fib_up = Fibonacci::calculate(100.0, 150.0, true);

        // Downtrend from 150 to 100 should give mirrored levels
        let fib_down = Fibonacci::calculate(150.0, 100.0, false);

        // The range midpoint should be the same for both
        let mid_up = (fib_up.level_0 + fib_up.level_100) / 2.0;
        let mid_down = (fib_down.level_0 + fib_down.level_100) / 2.0;
        assert!((mid_up - mid_down).abs() < 0.01);

        // 38.2% retracement should be equidistant from the midpoint
        let dist_up = (fib_up.level_382 - mid_up).abs();
        let dist_down = (fib_down.level_382 - mid_down).abs();
        assert!((dist_up - dist_down).abs() < 0.01);
    }

    #[test]
    fn test_fibonacci_small_range() {
        let fib = Fibonacci::calculate(100.0, 101.0, true);

        // Range = 1
        // Should still calculate correctly
        assert_eq!(fib.level_0, 101.0);
        assert_eq!(fib.level_100, 100.0);
        assert!((fib.level_500 - 100.5).abs() < 0.01);
    }

    #[test]
    fn test_fibonacci_large_range() {
        let fib = Fibonacci::calculate(1000.0, 5000.0, true);

        // Range = 4000
        // 61.8% = 5000 - (4000 * 0.618) = 2528
        assert!((fib.level_618 - 2528.0).abs() < 1.0);
    }

    #[test]
    fn test_fibonacci_zero_start() {
        let fib = Fibonacci::calculate(0.0, 100.0, true);

        assert_eq!(fib.level_0, 100.0);
        assert_eq!(fib.level_100, 0.0);
        assert_eq!(fib.level_500, 50.0);
    }

    #[test]
    fn test_fibonacci_levels_order_uptrend() {
        let fib = Fibonacci::calculate(100.0, 200.0, true);

        // Levels should be in descending order for uptrend
        assert!(fib.level_0 > fib.level_236);
        assert!(fib.level_236 > fib.level_382);
        assert!(fib.level_382 > fib.level_500);
        assert!(fib.level_500 > fib.level_618);
        assert!(fib.level_618 > fib.level_786);
        assert!(fib.level_786 > fib.level_100);
    }

    #[test]
    fn test_fibonacci_levels_order_downtrend() {
        let fib = Fibonacci::calculate(200.0, 100.0, false);

        // Levels should be in ascending order for downtrend
        assert!(fib.level_0 < fib.level_236);
        assert!(fib.level_236 < fib.level_382);
        assert!(fib.level_382 < fib.level_500);
        assert!(fib.level_500 < fib.level_618);
        assert!(fib.level_618 < fib.level_786);
        assert!(fib.level_786 < fib.level_100);
    }
}
