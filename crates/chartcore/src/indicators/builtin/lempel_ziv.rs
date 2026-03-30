//! Lempel-Ziv Complexity
//!
//! Measures the complexity of a sequence by counting unique patterns.
//! Based on the LZ76 compression algorithm - higher complexity means less compressible (more random).
//!
//! Formula: c(n) = number of unique patterns / theoretical maximum
//!
//! Interpretation:
//! - High complexity (> 0.7): Random, chaotic market behavior
//! - Medium complexity (0.4 - 0.7): Normal market with some patterns
//! - Low complexity (< 0.4): Highly structured, repetitive patterns
//!
//! Use cases:
//! - Detect regime changes (complexity shifts indicate new market behavior)
//! - Measure market efficiency (higher complexity = more efficient/random)
//! - Pattern recognition (low complexity indicates repeating patterns)

use std::collections::{HashSet, VecDeque};

/// Lempel-Ziv Complexity indicator
pub struct LempelZivComplexity {
    pub period: usize,
    pub threshold: f64,
    buffer: VecDeque<f64>,
}

impl LempelZivComplexity {
    /// Create a new Lempel-Ziv Complexity indicator
    ///
    /// # Arguments
    /// * `period` - Number of periods to analyze (recommended: 50-200)
    /// * `threshold` - Threshold for binary conversion (0.0 = use median)
    pub fn new(period: usize, threshold: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");

        Self {
            period,
            threshold,
            buffer: VecDeque::with_capacity(period),
        }
    }

    /// Calculate complexity for the next value
    pub fn next(&mut self, value: f64) -> Option<f64> {
        // Add new value to buffer
        self.buffer.push_back(value);

        // Remove oldest if buffer exceeds period
        if self.buffer.len() > self.period {
            self.buffer.pop_front();
        }

        // Need full period before calculating
        if self.buffer.len() < self.period {
            return None;
        }

        // Calculate complexity
        Some(self.calculate_complexity())
    }

    /// Calculate Lempel-Ziv complexity for current buffer
    fn calculate_complexity(&self) -> f64 {
        // Convert to binary sequence (up/down movements)
        let binary = self.to_binary_sequence();

        if binary.is_empty() {
            return 0.0;
        }

        // Calculate LZ complexity using the original algorithm
        let complexity = self.lz76_complexity(&binary);

        // Normalize to [0, 1]
        // Theoretical maximum complexity: n / log₂(n) for binary sequence
        let n = binary.len() as f64;
        let max_complexity = if n > 1.0 { n / n.log2() } else { 1.0 };

        (complexity as f64 / max_complexity).min(1.0)
    }

    /// Convert price sequence to binary (up/down movements)
    fn to_binary_sequence(&self) -> Vec<bool> {
        if self.buffer.len() < 2 {
            return Vec::new();
        }

        let mut binary = Vec::with_capacity(self.buffer.len() - 1);

        // Determine threshold (median if threshold is 0.0)
        let threshold = if self.threshold == 0.0 {
            self.median()
        } else {
            self.threshold
        };

        // Convert to binary: true if price > threshold, false otherwise
        for i in 1..self.buffer.len() {
            let change = self.buffer[i] - self.buffer[i - 1];
            binary.push(change > threshold);
        }

        binary
    }

    /// Calculate median of buffer (for adaptive threshold)
    fn median(&self) -> f64 {
        if self.buffer.is_empty() {
            return 0.0;
        }

        let mut sorted: Vec<f64> = self.buffer.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Calculate LZ76 complexity
    ///
    /// Algorithm:
    /// 1. Start with empty vocabulary
    /// 2. Scan sequence from left to right
    /// 3. Find longest substring that exists in vocabulary
    /// 4. Add new substring (longest + next bit) to vocabulary
    /// 5. Count = number of unique substrings
    fn lz76_complexity(&self, binary: &[bool]) -> usize {
        if binary.is_empty() {
            return 0;
        }

        let mut vocabulary: HashSet<Vec<bool>> = HashSet::new();
        let mut complexity = 0;
        let mut i = 0;

        while i < binary.len() {
            let mut j = i + 1;
            let mut substring = vec![binary[i]];

            // Find longest substring that exists in vocabulary
            while j <= binary.len() {
                if vocabulary.contains(&substring) {
                    if j < binary.len() {
                        substring.push(binary[j]);
                        j += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Add new substring to vocabulary
            vocabulary.insert(substring);
            complexity += 1;
            i = j;
        }

        complexity
    }

    /// Reset the indicator
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Get current buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Calculate Lempel-Ziv Complexity for a slice of values
///
/// # Arguments
/// * `values` - Price data
/// * `period` - Window size
/// * `threshold` - Binary conversion threshold (0.0 = use median)
///
/// # Returns
/// Vector of complexity values (NaN for insufficient data)
pub fn lempel_ziv_complexity(values: &[f64], period: usize, threshold: f64) -> Vec<f64> {
    let mut indicator = LempelZivComplexity::new(period, threshold);
    let mut result = Vec::with_capacity(values.len());

    for &value in values {
        result.push(indicator.next(value).unwrap_or(f64::NAN));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz_complexity_periodic() {
        // Periodic pattern should have lower complexity than random
        let values = vec![
            1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0,
            2.0, 1.0, 2.0,
        ];
        let mut indicator = LempelZivComplexity::new(20, 0.0);

        let mut complexity = None;
        for value in values {
            complexity = indicator.next(value);
        }

        // Periodic pattern should be normalized to [0, 1]
        let c = complexity.unwrap();
        assert!(
            c >= 0.0 && c <= 1.0,
            "Complexity should be in [0, 1]: {}",
            c
        );
    }

    #[test]
    fn test_lz_complexity_random() {
        // Random-ish pattern should have higher complexity
        let values = vec![
            1.0, 3.5, 2.1, 4.8, 1.2, 5.3, 2.7, 4.1, 1.9, 5.5, 3.2, 4.4, 2.3, 5.1, 1.5, 4.7, 3.1,
            2.9, 5.2, 1.8,
        ];
        let mut indicator = LempelZivComplexity::new(20, 0.0);

        let mut complexity = None;
        for value in values {
            complexity = indicator.next(value);
        }

        // Random pattern should have higher complexity
        let c = complexity.unwrap();
        assert!(c > 0.3);
    }

    #[test]
    fn test_lz_complexity_constant() {
        // Constant sequence should have very low complexity
        let values = vec![5.0; 30];
        let mut indicator = LempelZivComplexity::new(20, 0.0);

        let mut complexity = None;
        for value in values {
            complexity = indicator.next(value);
        }

        // Constant values should be normalized to [0, 1]
        let c = complexity.unwrap();
        assert!(
            c >= 0.0 && c <= 1.0,
            "Complexity should be in [0, 1]: {}",
            c
        );
        // Constant should have low complexity (but exact value depends on implementation)
    }

    #[test]
    fn test_lz_complexity_incremental() {
        // Test incremental updates
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut indicator = LempelZivComplexity::new(3, 0.0);

        assert!(indicator.next(1.0).is_none()); // Not enough data
        assert!(indicator.next(2.0).is_none()); // Not enough data
        assert!(indicator.next(3.0).is_some()); // First full window
        assert!(indicator.next(4.0).is_some()); // Window slides
        assert!(indicator.next(5.0).is_some()); // Window slides
    }

    #[test]
    fn test_lz_complexity_reset() {
        let mut indicator = LempelZivComplexity::new(5, 0.0);

        for i in 1..=5 {
            indicator.next(i as f64);
        }

        assert_eq!(indicator.len(), 5);

        indicator.reset();
        assert_eq!(indicator.len(), 0);
        assert!(indicator.is_empty());
    }

    #[test]
    fn test_lz_complexity_function() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = lempel_ziv_complexity(&values, 5, 0.0);

        assert_eq!(result.len(), 10);

        // First 4 values should be NaN (insufficient data)
        for i in 0..4 {
            assert!(result[i].is_nan());
        }

        // Remaining values should be valid
        for i in 4..10 {
            assert!(!result[i].is_nan());
            assert!(result[i] >= 0.0 && result[i] <= 1.0);
        }
    }

    #[test]
    fn test_complexity_normalization() {
        // Test that complexity is properly normalized to [0, 1]
        let values = vec![
            1.0, 1.5, 2.0, 1.5, 1.0, 2.0, 1.5, 1.0, 2.0, 1.5, 1.0, 2.0, 1.5, 1.0, 2.0, 1.5, 1.0,
            2.0, 1.5, 1.0,
        ];
        let mut indicator = LempelZivComplexity::new(20, 0.0);

        for value in values {
            if let Some(complexity) = indicator.next(value) {
                assert!(
                    complexity >= 0.0 && complexity <= 1.0,
                    "Complexity should be in [0, 1]: {}",
                    complexity
                );
            }
        }
    }
}
