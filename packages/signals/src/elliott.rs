//! Elliott Wave analysis.
//!
//! Implements Elliott Wave Theory for market analysis:
//! - Impulse waves (5-wave pattern)
//! - Corrective waves (3-wave pattern: ABC, zigzag, flat, triangle)
//! - Wave degree classification
//! - Fibonacci relationships
//!
//! ## Elliott Wave Rules
//!
//! ### Impulse Wave Rules (must be followed)
//! 1. Wave 2 cannot retrace more than 100% of Wave 1
//! 2. Wave 3 cannot be the shortest of waves 1, 3, and 5
//! 3. Wave 4 cannot overlap Wave 1's price territory
//!
//! ### Guidelines (usually followed)
//! - Wave 2 typically retraces 50-61.8% of Wave 1
//! - Wave 3 is often the longest and strongest
//! - Wave 4 typically retraces 38.2% of Wave 3
//! - Wave 5 often equals Wave 1 in length
//! - Alternation: If Wave 2 is sharp, Wave 4 is usually sideways

use loom_core::{Candle, OHLCV, Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, string::String};

/// Wave type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WaveType {
    /// Impulse wave (motive, trend direction)
    Impulse,
    /// Corrective wave (counter-trend)
    Corrective,
}

/// Wave degree (timeframe significance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WaveDegree {
    /// Subminuette (minutes)
    Subminuette,
    /// Minuette (hours)
    Minuette,
    /// Minute (days)
    Minute,
    /// Minor (weeks)
    Minor,
    /// Intermediate (months)
    Intermediate,
    /// Primary (years)
    Primary,
    /// Cycle (decades)
    Cycle,
    /// Supercycle (multi-decade)
    Supercycle,
    /// Grand Supercycle (century)
    GrandSupercycle,
}

impl WaveDegree {
    /// Get typical bar count for this degree
    pub fn typical_bars(&self) -> usize {
        match self {
            Self::Subminuette => 5,
            Self::Minuette => 20,
            Self::Minute => 60,
            Self::Minor => 200,
            Self::Intermediate => 500,
            Self::Primary => 1000,
            Self::Cycle => 5000,
            Self::Supercycle => 10000,
            Self::GrandSupercycle => 50000,
        }
    }
}

/// A single wave
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Wave {
    /// Wave label (1, 2, 3, 4, 5 or A, B, C)
    pub label: String,
    /// Wave type
    pub wave_type: WaveType,
    /// Wave degree
    pub degree: WaveDegree,
    /// Start bar index
    pub start_index: usize,
    /// End bar index
    pub end_index: usize,
    /// Start price
    pub start_price: Price,
    /// End price
    pub end_price: Price,
    /// Start timestamp
    pub start_time: Timestamp,
    /// End timestamp
    pub end_time: Timestamp,
    /// Is upward movement
    pub is_up: bool,
    /// Sub-waves (if any)
    pub sub_waves: Vec<Wave>,
    /// Confidence score (0-100)
    pub confidence: f64,
}

impl Wave {
    /// Wave length (price move)
    pub fn length(&self) -> Price {
        (self.end_price - self.start_price).abs()
    }

    /// Wave duration (bars)
    pub fn duration(&self) -> usize {
        self.end_index - self.start_index
    }

    /// Retracement of this wave relative to previous wave
    pub fn retracement_of(&self, previous: &Wave) -> f64 {
        if previous.length() == 0.0 {
            return 0.0;
        }
        self.length() / previous.length()
    }

    /// Extension ratio relative to reference wave
    pub fn extension_of(&self, reference: &Wave) -> f64 {
        if reference.length() == 0.0 {
            return 0.0;
        }
        self.length() / reference.length()
    }
}

/// Complete wave count
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WaveCount {
    /// Primary trend direction
    pub trend_up: bool,
    /// Identified waves
    pub waves: Vec<Wave>,
    /// Current wave being formed
    pub current_wave: Option<Wave>,
    /// Overall confidence
    pub confidence: f64,
    /// Pattern type
    pub pattern: Option<ImpulsePattern>,
}

/// Impulse pattern variations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImpulsePattern {
    /// Standard impulse
    Standard,
    /// Extended Wave 3
    Extended3,
    /// Extended Wave 5
    Extended5,
    /// Diagonal (wedge)
    Diagonal,
    /// Ending diagonal
    EndingDiagonal,
}

/// Corrective pattern variations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CorrectivePattern {
    /// Simple zigzag (5-3-5)
    Zigzag,
    /// Double zigzag
    DoubleZigzag,
    /// Triple zigzag
    TripleZigzag,
    /// Flat (3-3-5)
    Flat,
    /// Expanded flat
    ExpandedFlat,
    /// Running flat
    RunningFlat,
    /// Triangle
    Triangle,
    /// Expanding triangle
    ExpandingTriangle,
    /// Complex combination
    Complex,
}

/// Common Fibonacci ratios for Elliott Waves
pub mod fibonacci {
    pub const RATIO_236: f64 = 0.236;
    pub const RATIO_382: f64 = 0.382;
    pub const RATIO_500: f64 = 0.500;
    pub const RATIO_618: f64 = 0.618;
    pub const RATIO_786: f64 = 0.786;
    pub const RATIO_886: f64 = 0.886;
    pub const RATIO_100: f64 = 1.000;
    pub const RATIO_127: f64 = 1.272;
    pub const RATIO_161: f64 = 1.618;
    pub const RATIO_200: f64 = 2.000;
    pub const RATIO_261: f64 = 2.618;
    pub const RATIO_361: f64 = 3.618;
    pub const RATIO_423: f64 = 4.236;

    /// Check if value is near a Fibonacci ratio
    pub fn is_near_fib(value: f64, tolerance: f64) -> Option<f64> {
        let fibs = [
            RATIO_236, RATIO_382, RATIO_500, RATIO_618, RATIO_786,
            RATIO_886, RATIO_100, RATIO_127, RATIO_161, RATIO_200,
            RATIO_261, RATIO_361, RATIO_423,
        ];

        for &fib in &fibs {
            if (value - fib).abs() < tolerance {
                return Some(fib);
            }
        }
        None
    }

    /// Calculate Fibonacci retracement levels
    pub fn retracement_levels(high: f64, low: f64) -> Vec<(f64, f64)> {
        let range = high - low;
        vec![
            (RATIO_236, high - range * RATIO_236),
            (RATIO_382, high - range * RATIO_382),
            (RATIO_500, high - range * RATIO_500),
            (RATIO_618, high - range * RATIO_618),
            (RATIO_786, high - range * RATIO_786),
        ]
    }

    /// Calculate Fibonacci extension levels
    pub fn extension_levels(start: f64, end: f64, retracement: f64) -> Vec<(f64, f64)> {
        let range = (end - start).abs();
        let direction = if end > start { 1.0 } else { -1.0 };

        vec![
            (RATIO_100, retracement + range * RATIO_100 * direction),
            (RATIO_127, retracement + range * RATIO_127 * direction),
            (RATIO_161, retracement + range * RATIO_161 * direction),
            (RATIO_200, retracement + range * RATIO_200 * direction),
            (RATIO_261, retracement + range * RATIO_261 * direction),
        ]
    }
}

/// Elliott Wave analyzer
pub struct ElliottAnalyzer {
    /// Minimum wave size (percentage)
    min_wave_size: f64,
    /// Swing detection lookback
    swing_lookback: usize,
    /// Tolerance for Fibonacci ratios
    fib_tolerance: f64,
}

impl ElliottAnalyzer {
    pub fn new() -> Self {
        Self {
            min_wave_size: 0.01, // 1% minimum
            swing_lookback: 5,
            fib_tolerance: 0.05, // 5% tolerance
        }
    }

    pub fn with_min_wave_size(mut self, size: f64) -> Self {
        self.min_wave_size = size;
        self
    }

    pub fn with_swing_lookback(mut self, lookback: usize) -> Self {
        self.swing_lookback = lookback;
        self
    }

    /// Analyze candles for Elliott Wave patterns
    pub fn analyze(&self, candles: &[Candle], degree: WaveDegree) -> Option<WaveCount> {
        if candles.len() < 20 {
            return None;
        }

        // Find swing points
        let swings = self.find_swings(candles);
        if swings.len() < 5 {
            return None;
        }

        // Try to identify impulse pattern
        if let Some(impulse) = self.find_impulse(&swings, candles, degree) {
            return Some(impulse);
        }

        // Try corrective pattern
        if let Some(corrective) = self.find_corrective(&swings, candles, degree) {
            return Some(corrective);
        }

        None
    }

    /// Find swing points
    fn find_swings(&self, candles: &[Candle]) -> Vec<(usize, Price, bool)> {
        let mut swings = Vec::new();
        let lb = self.swing_lookback;

        if candles.len() < lb * 2 + 1 {
            return swings;
        }

        for i in lb..candles.len() - lb {
            let is_high = (0..lb).all(|j| candles[i].high >= candles[i - j - 1].high)
                && (0..lb).all(|j| candles[i].high >= candles[i + j + 1].high);

            let is_low = (0..lb).all(|j| candles[i].low <= candles[i - j - 1].low)
                && (0..lb).all(|j| candles[i].low <= candles[i + j + 1].low);

            if is_high && !is_low {
                swings.push((i, candles[i].high, true));
            } else if is_low && !is_high {
                swings.push((i, candles[i].low, false));
            }
        }

        // Filter out insignificant swings
        self.filter_swings(swings, candles)
    }

    /// Filter out minor swings
    fn filter_swings(&self, swings: Vec<(usize, Price, bool)>, candles: &[Candle]) -> Vec<(usize, Price, bool)> {
        if swings.len() < 2 {
            return swings;
        }

        let avg_price = candles.iter().map(|c| c.close).sum::<f64>() / candles.len() as f64;
        let min_move = avg_price * self.min_wave_size;

        let mut filtered = vec![swings[0]];

        for i in 1..swings.len() {
            let (_, last_price, _) = filtered.last().unwrap();
            let (idx, price, is_high) = swings[i];

            if (price - last_price).abs() >= min_move {
                filtered.push((idx, price, is_high));
            }
        }

        filtered
    }

    /// Try to find an impulse wave pattern
    fn find_impulse(
        &self,
        swings: &[(usize, Price, bool)],
        candles: &[Candle],
        degree: WaveDegree,
    ) -> Option<WaveCount> {
        if swings.len() < 6 {
            return None;
        }

        // Try to find 5-wave structure
        for start in 0..swings.len().saturating_sub(5) {
            if let Some(waves) = self.validate_impulse(&swings[start..start + 6], candles, degree) {
                let trend_up = waves[0].is_up;
                let confidence = self.calculate_impulse_confidence(&waves);

                return Some(WaveCount {
                    trend_up,
                    waves,
                    current_wave: None,
                    confidence,
                    pattern: Some(self.classify_impulse_pattern(&swings[start..start + 6])),
                });
            }
        }

        None
    }

    /// Validate impulse wave rules
    fn validate_impulse(
        &self,
        swings: &[(usize, Price, bool)],
        candles: &[Candle],
        degree: WaveDegree,
    ) -> Option<Vec<Wave>> {
        if swings.len() < 6 {
            return None;
        }

        // Determine if this is an upward or downward impulse
        let (_, start_price, _) = swings[0];
        let (_, end_price, _) = swings[5];
        let is_up = end_price > start_price;

        // Create wave objects
        let mut waves = Vec::new();

        for i in 0..5 {
            let (start_idx, start_p, _) = swings[i];
            let (end_idx, end_p, _) = swings[i + 1];

            let label = match i {
                0 => "1",
                1 => "2",
                2 => "3",
                3 => "4",
                4 => "5",
                _ => "?",
            };

            let wave_type = if i % 2 == 0 {
                WaveType::Impulse
            } else {
                WaveType::Corrective
            };

            waves.push(Wave {
                label: String::from(label),
                wave_type,
                degree,
                start_index: start_idx,
                end_index: end_idx,
                start_price: start_p,
                end_price: end_p,
                start_time: candles[start_idx].time,
                end_time: candles[end_idx].time,
                is_up: end_p > start_p,
                sub_waves: Vec::new(),
                confidence: 0.0,
            });
        }

        // Validate Elliott Wave rules
        // Rule 1: Wave 2 cannot retrace more than 100% of Wave 1
        if waves[1].length() >= waves[0].length() {
            return None;
        }

        // Rule 2: Wave 3 cannot be the shortest
        let w1_len = waves[0].length();
        let w3_len = waves[2].length();
        let w5_len = waves[4].length();

        if w3_len < w1_len && w3_len < w5_len {
            return None;
        }

        // Rule 3: Wave 4 cannot overlap Wave 1 territory
        if is_up {
            // For upward impulse: Wave 4 low cannot go below Wave 1 high
            let wave1_high = waves[0].end_price;
            let wave4_low = waves[3].end_price.min(waves[3].start_price);
            if wave4_low < wave1_high {
                return None;
            }
        } else {
            // For downward impulse: Wave 4 high cannot go above Wave 1 low
            let wave1_low = waves[0].end_price;
            let wave4_high = waves[3].end_price.max(waves[3].start_price);
            if wave4_high > wave1_low {
                return None;
            }
        }

        Some(waves)
    }

    /// Calculate confidence score for impulse
    fn calculate_impulse_confidence(&self, waves: &[Wave]) -> f64 {
        let mut score = 50.0; // Base score for valid pattern

        // Wave 2 retracement (ideal: 50-61.8%)
        let w2_ret = waves[1].retracement_of(&waves[0]);
        if w2_ret >= 0.5 && w2_ret <= 0.618 {
            score += 15.0;
        } else if w2_ret >= 0.382 && w2_ret <= 0.786 {
            score += 8.0;
        }

        // Wave 3 is longest (ideal case)
        let w1_len = waves[0].length();
        let w3_len = waves[2].length();
        let w5_len = waves[4].length();

        if w3_len > w1_len && w3_len > w5_len {
            score += 15.0;
        }

        // Wave 3 extension (ideal: 161.8% of Wave 1)
        let w3_ext = waves[2].extension_of(&waves[0]);
        if let Some(_) = fibonacci::is_near_fib(w3_ext, self.fib_tolerance) {
            score += 10.0;
        }

        // Wave 4 retracement (ideal: 38.2%)
        let w4_ret = waves[3].retracement_of(&waves[2]);
        if (w4_ret - 0.382).abs() < self.fib_tolerance {
            score += 10.0;
        }

        score.min(100.0)
    }

    /// Classify impulse pattern type
    fn classify_impulse_pattern(&self, swings: &[(usize, Price, bool)]) -> ImpulsePattern {
        if swings.len() < 6 {
            return ImpulsePattern::Standard;
        }

        let w1_len = (swings[1].1 - swings[0].1).abs();
        let w3_len = (swings[3].1 - swings[2].1).abs();
        let w5_len = (swings[5].1 - swings[4].1).abs();

        // Extended Wave 3
        if w3_len > w1_len * 1.618 && w3_len > w5_len * 1.618 {
            return ImpulsePattern::Extended3;
        }

        // Extended Wave 5
        if w5_len > w1_len * 1.618 && w5_len > w3_len {
            return ImpulsePattern::Extended5;
        }

        ImpulsePattern::Standard
    }

    /// Try to find corrective wave pattern
    fn find_corrective(
        &self,
        swings: &[(usize, Price, bool)],
        candles: &[Candle],
        degree: WaveDegree,
    ) -> Option<WaveCount> {
        if swings.len() < 4 {
            return None;
        }

        // Try to find ABC pattern
        for start in 0..swings.len().saturating_sub(3) {
            if let Some(waves) = self.validate_abc(&swings[start..start + 4], candles, degree) {
                let trend_up = !waves[0].is_up; // Corrective is against main trend
                let confidence = self.calculate_abc_confidence(&waves);

                return Some(WaveCount {
                    trend_up,
                    waves,
                    current_wave: None,
                    confidence,
                    pattern: None,
                });
            }
        }

        None
    }

    /// Validate ABC corrective pattern
    fn validate_abc(
        &self,
        swings: &[(usize, Price, bool)],
        candles: &[Candle],
        degree: WaveDegree,
    ) -> Option<Vec<Wave>> {
        if swings.len() < 4 {
            return None;
        }

        let mut waves = Vec::new();
        let labels = ["A", "B", "C"];

        for i in 0..3 {
            let (start_idx, start_p, _) = swings[i];
            let (end_idx, end_p, _) = swings[i + 1];

            waves.push(Wave {
                label: String::from(labels[i]),
                wave_type: if i == 1 { WaveType::Corrective } else { WaveType::Impulse },
                degree,
                start_index: start_idx,
                end_index: end_idx,
                start_price: start_p,
                end_price: end_p,
                start_time: candles[start_idx].time,
                end_time: candles[end_idx].time,
                is_up: end_p > start_p,
                sub_waves: Vec::new(),
                confidence: 0.0,
            });
        }

        // Basic ABC validation:
        // Wave B should not retrace more than 100% of Wave A
        if waves[1].length() >= waves[0].length() {
            return None;
        }

        Some(waves)
    }

    /// Calculate confidence for ABC pattern
    fn calculate_abc_confidence(&self, waves: &[Wave]) -> f64 {
        let mut score = 40.0;

        // Wave B retracement (ideal: 50-61.8% of A)
        let b_ret = waves[1].retracement_of(&waves[0]);
        if b_ret >= 0.5 && b_ret <= 0.618 {
            score += 20.0;
        }

        // Wave C often equals Wave A
        let c_ext = waves[2].extension_of(&waves[0]);
        if (c_ext - 1.0).abs() < self.fib_tolerance {
            score += 20.0;
        } else if let Some(_) = fibonacci::is_near_fib(c_ext, self.fib_tolerance) {
            score += 10.0;
        }

        score.min(100.0)
    }

    /// Get projected targets for current wave
    pub fn project_targets(&self, wave_count: &WaveCount) -> Vec<(String, Price)> {
        let mut targets = Vec::new();

        if wave_count.waves.is_empty() {
            return targets;
        }

        let waves = &wave_count.waves;

        // Based on pattern progress, project next moves
        match waves.len() {
            2 => {
                // After Wave 2, project Wave 3 targets
                let w1 = &waves[0];
                let w2 = &waves[1];

                let extensions = fibonacci::extension_levels(
                    w1.start_price,
                    w1.end_price,
                    w2.end_price,
                );

                for (ratio, price) in extensions {
                    targets.push((format!("W3 {:.1}%", ratio * 100.0), price));
                }
            }
            4 => {
                // After Wave 4, project Wave 5 targets
                let w1 = &waves[0];
                let w4 = &waves[3];

                // Wave 5 often equals Wave 1
                let w5_equal = w4.end_price + w1.length() * if wave_count.trend_up { 1.0 } else { -1.0 };
                targets.push((String::from("W5 = W1"), w5_equal));

                // Wave 5 at 61.8% of Waves 1-3
                let w1_to_w3 = (waves[2].end_price - waves[0].start_price).abs();
                let w5_fib = w4.end_price + w1_to_w3 * 0.618 * if wave_count.trend_up { 1.0 } else { -1.0 };
                targets.push((String::from("W5 61.8%"), w5_fib));
            }
            _ => {}
        }

        targets
    }
}

impl Default for ElliottAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_levels() {
        let levels = fibonacci::retracement_levels(100.0, 0.0);
        assert_eq!(levels.len(), 5);
        assert!((levels[2].1 - 50.0).abs() < 0.001); // 50% level
    }

    #[test]
    fn test_fib_near() {
        assert!(fibonacci::is_near_fib(0.62, 0.05).is_some());
        assert!(fibonacci::is_near_fib(0.38, 0.02).is_some());
        assert!(fibonacci::is_near_fib(0.75, 0.02).is_none());
    }
}
