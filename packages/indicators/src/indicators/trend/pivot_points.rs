//! Pivot Points

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Pivot Point Type
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PivotType {
    /// Standard/Classic pivot points
    Standard,
    /// Fibonacci pivot points
    Fibonacci,
    /// Woodie's pivot points
    Woodie,
    /// Camarilla pivot points
    Camarilla,
    /// DeMark pivot points
    DeMark,
}

/// Pivot Points Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PivotPointsOutput {
    /// Pivot Point (PP)
    pub pivot: f64,
    /// Resistance 1
    pub r1: f64,
    /// Resistance 2
    pub r2: f64,
    /// Resistance 3
    pub r3: f64,
    /// Support 1
    pub s1: f64,
    /// Support 2
    pub s2: f64,
    /// Support 3
    pub s3: f64,
}

impl PivotPointsOutput {
    pub fn new(pivot: f64, r1: f64, r2: f64, r3: f64, s1: f64, s2: f64, s3: f64) -> Self {
        Self {
            pivot,
            r1,
            r2,
            r3,
            s1,
            s2,
            s3,
        }
    }

    /// Check if price is above pivot
    pub fn is_above_pivot(&self, price: f64) -> bool {
        price > self.pivot
    }

    /// Check if price is below pivot
    pub fn is_below_pivot(&self, price: f64) -> bool {
        price < self.pivot
    }

    /// Get nearest support level
    pub fn nearest_support(&self, price: f64) -> f64 {
        if price >= self.s1 {
            self.s1
        } else if price >= self.s2 {
            self.s2
        } else {
            self.s3
        }
    }

    /// Get nearest resistance level
    pub fn nearest_resistance(&self, price: f64) -> f64 {
        if price <= self.r1 {
            self.r1
        } else if price <= self.r2 {
            self.r2
        } else {
            self.r3
        }
    }
}

/// Pivot Points
///
/// Support and resistance levels calculated from previous period's high, low, and close.
/// Used by traders to identify potential turning points in the market.
///
/// # Types
///
/// - **Standard**: Classic pivot point calculation
/// - **Fibonacci**: Uses Fibonacci ratios (0.382, 0.618)
/// - **Woodie**: Gives more weight to the close price
/// - **Camarilla**: Uses smaller multipliers for tighter levels
/// - **DeMark**: Conditional calculation based on open vs close
///
/// # Standard Formula
///
/// ```text
/// PP = (High + Low + Close) / 3
/// R1 = (2 * PP) - Low
/// R2 = PP + (High - Low)
/// R3 = High + 2 * (PP - Low)
/// S1 = (2 * PP) - High
/// S2 = PP - (High - Low)
/// S3 = Low - 2 * (High - PP)
/// ```
///
/// # Parameters
///
/// * `pivot_type` - Type of pivot calculation (default: Standard)
///
/// # Example
///
/// ```
/// use loom_indicators::indicators::{PivotPoints, PivotType, Next};
/// use loom_indicators::types::Ohlcv;
///
/// let mut pp = PivotPoints::new(PivotType::Standard);
///
/// // Use previous day's data to calculate today's pivots
/// let yesterday = Ohlcv::new(100.0, 105.0, 98.0, 103.0, 10000.0);
/// if let Some(pivots) = pp.next(&yesterday) {
///     println!("Pivot: {:.2}", pivots.pivot);
///     println!("R1: {:.2}, R2: {:.2}, R3: {:.2}", pivots.r1, pivots.r2, pivots.r3);
///     println!("S1: {:.2}, S2: {:.2}, S3: {:.2}", pivots.s1, pivots.s2, pivots.s3);
/// }
/// ```
#[derive(Clone)]
pub struct PivotPoints {
    pivot_type: PivotType,
    current: Option<PivotPointsOutput>,
}

impl PivotPoints {
    /// Create a new PivotPoints calculator with the specified type
    pub fn new(pivot_type: PivotType) -> Self {
        Self {
            pivot_type,
            current: None,
        }
    }

    fn calculate_standard(&self, high: f64, low: f64, close: f64) -> PivotPointsOutput {
        let pivot = (high + low + close) / 3.0;
        let range = high - low;

        let r1 = (2.0 * pivot) - low;
        let r2 = pivot + range;
        let r3 = high + 2.0 * (pivot - low);

        let s1 = (2.0 * pivot) - high;
        let s2 = pivot - range;
        let s3 = low - 2.0 * (high - pivot);

        PivotPointsOutput::new(pivot, r1, r2, r3, s1, s2, s3)
    }

    fn calculate_fibonacci(&self, high: f64, low: f64, close: f64) -> PivotPointsOutput {
        let pivot = (high + low + close) / 3.0;
        let range = high - low;

        let r1 = pivot + (0.382 * range);
        let r2 = pivot + (0.618 * range);
        let r3 = pivot + range;

        let s1 = pivot - (0.382 * range);
        let s2 = pivot - (0.618 * range);
        let s3 = pivot - range;

        PivotPointsOutput::new(pivot, r1, r2, r3, s1, s2, s3)
    }

    fn calculate_woodie(&self, high: f64, low: f64, close: f64) -> PivotPointsOutput {
        let pivot = (high + low + (2.0 * close)) / 4.0;
        let range = high - low;

        let r1 = (2.0 * pivot) - low;
        let r2 = pivot + range;
        let r3 = high + 2.0 * (pivot - low);

        let s1 = (2.0 * pivot) - high;
        let s2 = pivot - range;
        let s3 = low - 2.0 * (high - pivot);

        PivotPointsOutput::new(pivot, r1, r2, r3, s1, s2, s3)
    }

    fn calculate_camarilla(&self, high: f64, low: f64, close: f64) -> PivotPointsOutput {
        let pivot = close;
        let range = high - low;

        let r1 = close + (range * 1.1 / 12.0);
        let r2 = close + (range * 1.1 / 6.0);
        let r3 = close + (range * 1.1 / 4.0);

        let s1 = close - (range * 1.1 / 12.0);
        let s2 = close - (range * 1.1 / 6.0);
        let s3 = close - (range * 1.1 / 4.0);

        PivotPointsOutput::new(pivot, r1, r2, r3, s1, s2, s3)
    }

    fn calculate_demark(&self, high: f64, low: f64, close: f64, open: f64) -> PivotPointsOutput {
        let x = if close < open {
            high + (2.0 * low) + close
        } else if close > open {
            (2.0 * high) + low + close
        } else {
            high + low + (2.0 * close)
        };

        let pivot = x / 4.0;

        let r1 = x / 2.0 - low;
        let s1 = x / 2.0 - high;

        // DeMark typically only calculates one level each side
        let r2 = r1 + (r1 - pivot);
        let r3 = r2 + (r1 - pivot);
        let s2 = s1 - (pivot - s1);
        let s3 = s2 - (pivot - s1);

        PivotPointsOutput::new(pivot, r1, r2, r3, s1, s2, s3)
    }
}

impl Period for PivotPoints {
    fn period(&self) -> usize {
        1 // Pivots are calculated from a single period
    }
}

impl Next<&Ohlcv> for PivotPoints {
    type Output = PivotPointsOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<Self::Output> {
        let output = match self.pivot_type {
            PivotType::Standard => self.calculate_standard(candle.high, candle.low, candle.close),
            PivotType::Fibonacci => self.calculate_fibonacci(candle.high, candle.low, candle.close),
            PivotType::Woodie => self.calculate_woodie(candle.high, candle.low, candle.close),
            PivotType::Camarilla => self.calculate_camarilla(candle.high, candle.low, candle.close),
            PivotType::DeMark => {
                self.calculate_demark(candle.high, candle.low, candle.close, candle.open)
            }
        };

        self.current = Some(output);
        Some(output)
    }
}

impl Next<Ohlcv> for PivotPoints {
    type Output = PivotPointsOutput;

    fn next(&mut self, candle: Ohlcv) -> Option<Self::Output> {
        self.next(&candle)
    }
}

impl Current for PivotPoints {
    type Output = PivotPointsOutput;

    fn current(&self) -> Option<Self::Output> {
        self.current
    }
}

impl Reset for PivotPoints {
    fn reset(&mut self) {
        self.current = None;
    }
}

impl Default for PivotPoints {
    fn default() -> Self {
        Self::new(PivotType::Standard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(open: f64, high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(open, high, low, close, 1000.0)
    }

    #[test]
    fn test_pivot_points_standard() {
        let mut pp = PivotPoints::new(PivotType::Standard);

        // High: 110, Low: 90, Close: 100
        let result = pp.next(&candle(95.0, 110.0, 90.0, 100.0)).unwrap();

        // PP = (110 + 90 + 100) / 3 = 100
        assert!((result.pivot - 100.0).abs() < 1e-10);

        // R1 = (2 * 100) - 90 = 110
        assert!((result.r1 - 110.0).abs() < 1e-10);

        // S1 = (2 * 100) - 110 = 90
        assert!((result.s1 - 90.0).abs() < 1e-10);

        // R2 = 100 + (110 - 90) = 120
        assert!((result.r2 - 120.0).abs() < 1e-10);

        // S2 = 100 - (110 - 90) = 80
        assert!((result.s2 - 80.0).abs() < 1e-10);
    }

    #[test]
    fn test_pivot_points_fibonacci() {
        let mut pp = PivotPoints::new(PivotType::Fibonacci);

        let result = pp.next(&candle(95.0, 110.0, 90.0, 100.0)).unwrap();

        // PP = (110 + 90 + 100) / 3 = 100
        // Range = 20
        // R1 = 100 + (0.382 * 20) = 107.64
        assert!((result.r1 - 107.64).abs() < 0.01);

        // R2 = 100 + (0.618 * 20) = 112.36
        assert!((result.r2 - 112.36).abs() < 0.01);
    }

    #[test]
    fn test_pivot_points_woodie() {
        let mut pp = PivotPoints::new(PivotType::Woodie);

        let result = pp.next(&candle(95.0, 110.0, 90.0, 100.0)).unwrap();

        // PP = (110 + 90 + 2*100) / 4 = 100
        assert!((result.pivot - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_pivot_points_camarilla() {
        let mut pp = PivotPoints::new(PivotType::Camarilla);

        let result = pp.next(&candle(95.0, 110.0, 90.0, 100.0)).unwrap();

        // PP = close = 100
        assert_eq!(result.pivot, 100.0);

        // Camarilla uses tighter levels
        assert!(result.r1 < result.r2);
        assert!(result.r2 < result.r3);
        assert!(result.s1 > result.s2);
        assert!(result.s2 > result.s3);
    }

    #[test]
    fn test_pivot_points_demark() {
        let mut pp = PivotPoints::new(PivotType::DeMark);

        // Close > Open (bullish)
        let result = pp.next(&candle(95.0, 110.0, 90.0, 105.0)).unwrap();

        // X = 2*110 + 90 + 105 = 415
        // PP = 415 / 4 = 103.75
        assert!((result.pivot - 103.75).abs() < 0.01);
    }

    #[test]
    fn test_pivot_output_methods() {
        let output = PivotPointsOutput::new(100.0, 110.0, 120.0, 130.0, 90.0, 80.0, 70.0);

        assert!(output.is_above_pivot(105.0));
        assert!(output.is_below_pivot(95.0));

        assert_eq!(output.nearest_support(105.0), 90.0);
        assert_eq!(output.nearest_support(85.0), 80.0);
        assert_eq!(output.nearest_support(75.0), 70.0);

        assert_eq!(output.nearest_resistance(95.0), 110.0);
        assert_eq!(output.nearest_resistance(115.0), 120.0);
        assert_eq!(output.nearest_resistance(125.0), 130.0);
    }

    #[test]
    fn test_pivot_points_reset() {
        let mut pp = PivotPoints::new(PivotType::Standard);

        pp.next(&candle(95.0, 110.0, 90.0, 100.0));
        assert!(pp.current().is_some());

        pp.reset();
        assert!(pp.current().is_none());
    }

    #[test]
    fn test_pivot_points_default() {
        let pp = PivotPoints::default();
        assert_eq!(pp.pivot_type, PivotType::Standard);
    }

    #[test]
    fn test_pivot_points_levels_order() {
        let mut pp = PivotPoints::new(PivotType::Standard);

        let result = pp.next(&candle(95.0, 110.0, 90.0, 100.0)).unwrap();

        // Resistance levels should be ascending
        assert!(result.r1 < result.r2);
        assert!(result.r2 < result.r3);

        // Support levels should be descending
        assert!(result.s1 > result.s2);
        assert!(result.s2 > result.s3);

        // Pivot should be between S1 and R1
        assert!(result.pivot > result.s1);
        assert!(result.pivot < result.r1);
    }

    #[test]
    fn test_pivot_points_symmetry() {
        let mut pp = PivotPoints::new(PivotType::Standard);

        // Symmetric candle (close at midpoint)
        let result = pp.next(&candle(100.0, 110.0, 90.0, 100.0)).unwrap();

        // For standard pivots with symmetric data, distances should be equal
        let r1_distance = result.r1 - result.pivot;
        let s1_distance = result.pivot - result.s1;

        assert!((r1_distance - s1_distance).abs() < 1e-10);
    }

    #[test]
    fn test_pivot_types_all_return_values() {
        let types = vec![
            PivotType::Standard,
            PivotType::Fibonacci,
            PivotType::Woodie,
            PivotType::Camarilla,
            PivotType::DeMark,
        ];

        for pivot_type in types {
            let mut pp = PivotPoints::new(pivot_type);
            let result = pp.next(&candle(95.0, 110.0, 90.0, 100.0));
            assert!(
                result.is_some(),
                "Pivot type {:?} should return a value",
                pivot_type
            );
        }
    }
}
