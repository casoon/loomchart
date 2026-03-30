//! Permutation Entropy
//!
//! Measures the complexity of time series by analyzing ordinal patterns.
//! More robust to noise than Shannon entropy, focuses on the order of values rather than their magnitudes.
//!
//! Formula: H = -∑(p(π) * log(p(π)))
//! where p(π) is the probability of each ordinal pattern
//!
//! Interpretation:
//! - High entropy (> 0.8): Random, unpredictable price movements
//! - Medium entropy (0.4 - 0.8): Normal market with some structure
//! - Low entropy (< 0.4): Strong ordinal patterns, predictable sequences
//!
//! Use cases:
//! - Detect deterministic vs stochastic behavior
//! - Measure market complexity (robust to outliers)
//! - Early warning for regime changes
//! - Better than Shannon entropy for noisy data

use std::collections::{HashMap, VecDeque};

/// Permutation Entropy indicator
pub struct PermutationEntropy {
    pub period: usize,
    pub dimension: usize,
    pub delay: usize,
    buffer: VecDeque<f64>,
}

impl PermutationEntropy {
    /// Create a new Permutation Entropy indicator
    ///
    /// # Arguments
    /// * `period` - Number of periods to analyze (recommended: 50-200)
    /// * `dimension` - Dimension for ordinal patterns (recommended: 3-7, default: 3)
    /// * `delay` - Time delay for embedding (recommended: 1)
    ///
    /// Note: dimension! should be much less than period
    /// For d=3: 3! = 6 patterns
    /// For d=4: 4! = 24 patterns
    /// For d=5: 5! = 120 patterns
    pub fn new(period: usize, dimension: usize, delay: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(dimension >= 2, "Embedding dimension must be at least 2");
        assert!(dimension <= 7, "Embedding dimension too large (max: 7)");
        assert!(delay > 0, "Delay must be greater than 0");

        // Recommend period to be larger than factorial for better statistics
        // but don't enforce it strictly to allow flexibility
        let factorial = Self::factorial(dimension);
        let min_period = (dimension - 1) * delay + 1;
        assert!(
            period >= min_period,
            "Period must be at least {} for embedding dimension {} and delay {}",
            min_period,
            dimension,
            delay
        );

        if period < factorial * 2 {
            // Warning: not enforced, just a recommendation
            // In production, you might want to log this
        }

        Self {
            period,
            dimension,
            delay,
            buffer: VecDeque::with_capacity(period),
        }
    }

    /// Calculate permutation entropy for the next value
    pub fn next(&mut self, value: f64) -> Option<f64> {
        // Add new value to buffer
        self.buffer.push_back(value);

        // Remove oldest if buffer exceeds period
        if self.buffer.len() > self.period {
            self.buffer.pop_front();
        }

        // Need enough data for embedding
        let min_length = (self.dimension - 1) * self.delay + 1;
        if self.buffer.len() < min_length.max(self.period) {
            return None;
        }

        // Calculate permutation entropy
        Some(self.calculate_permutation_entropy())
    }

    /// Calculate permutation entropy for current buffer
    fn calculate_permutation_entropy(&self) -> f64 {
        // Extract ordinal patterns
        let patterns = self.extract_ordinal_patterns();

        if patterns.is_empty() {
            return 0.0;
        }

        // Count pattern frequencies
        let mut pattern_counts: HashMap<Vec<usize>, usize> = HashMap::new();
        for pattern in &patterns {
            *pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
        }

        // Calculate probabilities and entropy
        let total = patterns.len() as f64;
        let mut entropy = 0.0;

        for count in pattern_counts.values() {
            if *count > 0 {
                let probability = *count as f64 / total;
                // H = -∑(p * log(p))
                entropy -= probability * probability.ln();
            }
        }

        // Normalize to [0, 1]
        // Maximum entropy = ln(d!) where d is embedding dimension
        let max_entropy = (Self::factorial(self.dimension) as f64).ln();

        if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        }
    }

    /// Extract ordinal patterns from buffer
    ///
    /// For each subsequence of length dimension,
    /// create a permutation that represents the ordering of values.
    ///
    /// Example: [3.2, 1.5, 4.1] -> [1, 0, 2] (middle is smallest, left is middle, right is largest)
    fn extract_ordinal_patterns(&self) -> Vec<Vec<usize>> {
        let mut patterns = Vec::new();

        // Number of patterns we can extract
        let max_index = self.buffer.len() - (self.dimension - 1) * self.delay;

        for i in 0..max_index {
            // Extract embedding vector
            let mut embedding = Vec::with_capacity(self.dimension);
            for j in 0..self.dimension {
                let index = i + j * self.delay;
                if index < self.buffer.len() {
                    embedding.push((index, self.buffer[index]));
                }
            }

            // Convert to ordinal pattern (argsort)
            let pattern = self.argsort(&embedding);
            patterns.push(pattern);
        }

        patterns
    }

    /// Convert values to ordinal pattern (rank order)
    ///
    /// Returns the indices that would sort the array
    fn argsort(&self, embedding: &[(usize, f64)]) -> Vec<usize> {
        let mut indexed: Vec<(usize, f64)> = embedding
            .iter()
            .enumerate()
            .map(|(i, &(_, value))| (i, value))
            .collect();

        // Sort by value, using index as tiebreaker for stability
        indexed.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        // Return rank indices
        indexed.iter().map(|(i, _)| *i).collect()
    }

    /// Calculate factorial
    fn factorial(n: usize) -> usize {
        (1..=n).product()
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

/// Calculate Permutation Entropy for a slice of values
///
/// # Arguments
/// * `values` - Price data
/// * `period` - Window size
/// * `dimension` - Dimension for ordinal patterns (default: 3)
/// * `delay` - Time delay for embedding (default: 1)
///
/// # Returns
/// Vector of permutation entropy values (NaN for insufficient data)
pub fn permutation_entropy(
    values: &[f64],
    period: usize,
    dimension: usize,
    delay: usize,
) -> Vec<f64> {
    let mut indicator = PermutationEntropy::new(period, dimension, delay);
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
    fn test_permutation_entropy_monotonic() {
        // Monotonic increasing sequence should have very low entropy
        let values: Vec<f64> = (1..=50).map(|x| x as f64).collect();
        let mut indicator = PermutationEntropy::new(50, 3, 1);

        let mut entropy = None;
        for value in values {
            entropy = indicator.next(value);
        }

        // Monotonic sequence has only one pattern -> zero entropy
        assert!(entropy.unwrap() < 0.1);
    }

    #[test]
    fn test_permutation_entropy_alternating() {
        // Alternating pattern should have specific entropy
        let values = vec![
            1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0,
            2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0,
        ];
        let mut indicator = PermutationEntropy::new(30, 3, 1);

        let mut entropy = None;
        for value in values {
            entropy = indicator.next(value);
        }

        // Alternating pattern has limited ordinal patterns
        let ent = entropy.unwrap();
        assert!(ent > 0.0);
        assert!(ent < 0.8); // Not maximum entropy
    }

    #[test]
    fn test_permutation_entropy_random() {
        // Random-ish sequence should have higher entropy
        let values = vec![
            3.2, 1.5, 4.8, 2.1, 5.3, 1.9, 4.2, 2.7, 5.1, 1.8, 3.5, 2.3, 4.9, 1.7, 5.2, 2.5, 4.1,
            3.1, 5.5, 2.9, 3.8, 1.6, 4.5, 2.2, 5.4, 1.4, 4.7, 2.8, 5.0, 2.0,
        ];
        let mut indicator = PermutationEntropy::new(30, 3, 1);

        let mut entropy = None;
        for value in values {
            entropy = indicator.next(value);
        }

        // Random sequence should have high entropy
        assert!(entropy.unwrap() > 0.6);
    }

    #[test]
    fn test_permutation_entropy_incremental() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut indicator = PermutationEntropy::new(3, 3, 1);

        // Need at least dimension values
        assert!(indicator.next(1.0).is_none());
        assert!(indicator.next(2.0).is_none());
        assert!(indicator.next(3.0).is_some()); // First valid result
        assert!(indicator.next(4.0).is_some());
        assert!(indicator.next(5.0).is_some());
    }

    #[test]
    fn test_permutation_entropy_reset() {
        let mut indicator = PermutationEntropy::new(10, 3, 1);

        for i in 1..=10 {
            indicator.next(i as f64);
        }

        assert_eq!(indicator.len(), 10);

        indicator.reset();
        assert_eq!(indicator.len(), 0);
        assert!(indicator.is_empty());
    }

    #[test]
    fn test_permutation_entropy_function() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = permutation_entropy(&values, 5, 3, 1);

        assert_eq!(result.len(), 10);

        // First few values should be NaN
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
    fn test_entropy_normalization() {
        let values = vec![
            1.0, 3.0, 2.0, 5.0, 4.0, 7.0, 6.0, 9.0, 8.0, 10.0, 2.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0,
            10.0, 9.0, 11.0,
        ];
        let mut indicator = PermutationEntropy::new(20, 3, 1);

        for value in values {
            if let Some(entropy) = indicator.next(value) {
                assert!(
                    entropy >= 0.0 && entropy <= 1.0,
                    "Entropy should be in [0, 1]: {}",
                    entropy
                );
            }
        }
    }

    #[test]
    fn test_factorial() {
        assert_eq!(PermutationEntropy::factorial(3), 6);
        assert_eq!(PermutationEntropy::factorial(4), 24);
        assert_eq!(PermutationEntropy::factorial(5), 120);
    }

    #[test]
    fn test_argsort() {
        let indicator = PermutationEntropy::new(10, 3, 1);

        // Test argsort
        let embedding = vec![(0, 3.0), (1, 1.0), (2, 2.0)];
        let pattern = indicator.argsort(&embedding);

        // Should be [1, 2, 0] because 1.0 < 2.0 < 3.0
        assert_eq!(pattern, vec![1, 2, 0]);
    }

    #[test]
    fn test_different_dimensions() {
        let values: Vec<f64> = (1..=100).map(|x| (x as f64 * 1.5).sin()).collect();

        // Test d=3
        let result3 = permutation_entropy(&values, 50, 3, 1);
        assert!(result3[60..].iter().all(|&x| !x.is_nan()));

        // Test d=4
        let result4 = permutation_entropy(&values, 50, 4, 1);
        assert!(result4[60..].iter().all(|&x| !x.is_nan()));

        // Higher dimension should potentially capture more complexity
        // but this is data-dependent
    }
}
