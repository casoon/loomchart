//! Shannon Entropy
//!
//! Measures the information content and unpredictability of price movements.
//! Higher entropy indicates more randomness, lower entropy indicates more structure.
//!
//! Formula: H = -∑(pk * log₂(pk))
//! where pk is the probability of each price bin
//!
//! Interpretation:
//! - High entropy (> 0.8): Market is random, avoid trend-following strategies
//! - Medium entropy (0.4 - 0.8): Normal market behavior
//! - Low entropy (< 0.4): Strong patterns, good for trend-following
//!
//! Use cases:
//! - Regime detection (trending vs ranging)
//! - Risk assessment (higher entropy = higher uncertainty)
//! - Strategy selection (use different strategies for different entropy levels)

use std::collections::VecDeque;

/// Shannon Entropy indicator
pub struct ShannonEntropy {
    pub period: usize,
    pub bins: usize,
    buffer: VecDeque<f64>,
}

impl ShannonEntropy {
    /// Create a new Shannon Entropy indicator
    ///
    /// # Arguments
    /// * `period` - Number of periods to analyze (recommended: 14-50)
    /// * `bins` - Number of histogram bins (recommended: 10-20)
    pub fn new(period: usize, bins: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(bins > 1, "Bins must be greater than 1");

        Self {
            period,
            bins,
            buffer: VecDeque::with_capacity(period),
        }
    }

    /// Calculate entropy for the current window
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

        // Calculate entropy
        Some(self.calculate_entropy())
    }

    /// Calculate Shannon entropy for current buffer
    fn calculate_entropy(&self) -> f64 {
        // Find min and max for binning
        let min = self.buffer.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = self.buffer.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Handle edge case: all values are the same
        if (max - min).abs() < f64::EPSILON {
            return 0.0; // Zero entropy (complete order)
        }

        // Create histogram bins
        let mut histogram = vec![0usize; self.bins];
        let bin_width = (max - min) / self.bins as f64;

        // Populate histogram
        for &value in &self.buffer {
            let bin_index = ((value - min) / bin_width)
                .floor()
                .min((self.bins - 1) as f64) as usize;
            histogram[bin_index] += 1;
        }

        // Calculate probabilities and entropy
        let total = self.buffer.len() as f64;
        let mut entropy = 0.0;

        for &count in &histogram {
            if count > 0 {
                let probability = count as f64 / total;
                // H = -∑(pk * log₂(pk))
                entropy -= probability * probability.log2();
            }
        }

        // Normalize to [0, 1] range
        // Maximum entropy is log₂(bins)
        let max_entropy = (self.bins as f64).log2();
        entropy / max_entropy
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

/// Calculate Shannon Entropy for a slice of values
///
/// # Arguments
/// * `values` - Price data
/// * `period` - Window size
/// * `bins` - Number of histogram bins
///
/// # Returns
/// Vector of entropy values (NaN for insufficient data)
pub fn shannon_entropy(values: &[f64], period: usize, bins: usize) -> Vec<f64> {
    let mut indicator = ShannonEntropy::new(period, bins);
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
    fn test_shannon_entropy_uniform() {
        // Uniform distribution should have high entropy
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let mut indicator = ShannonEntropy::new(10, 10);

        let mut entropy = None;
        for value in values {
            entropy = indicator.next(value);
        }

        // Uniform distribution should have high entropy (close to 1.0)
        assert!(entropy.unwrap() > 0.95);
    }

    #[test]
    fn test_shannon_entropy_constant() {
        // Constant values should have zero entropy
        let values = vec![5.0; 20];
        let mut indicator = ShannonEntropy::new(10, 10);

        let mut entropy = None;
        for value in values {
            entropy = indicator.next(value);
        }

        // Constant values should have zero entropy
        assert_eq!(entropy.unwrap(), 0.0);
    }

    #[test]
    fn test_shannon_entropy_binary() {
        // Binary distribution should have lower entropy
        let values = vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0, 1.0, 2.0];
        let mut indicator = ShannonEntropy::new(10, 10);

        let mut entropy = None;
        for value in values {
            entropy = indicator.next(value);
        }

        // Binary distribution should have lower entropy than uniform
        let ent = entropy.unwrap();
        assert!(ent > 0.0);
        assert!(ent < 0.5); // Much less than maximum
    }

    #[test]
    fn test_shannon_entropy_incremental() {
        // Test incremental updates
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut indicator = ShannonEntropy::new(3, 5);

        assert!(indicator.next(1.0).is_none()); // Not enough data
        assert!(indicator.next(2.0).is_none()); // Not enough data
        assert!(indicator.next(3.0).is_some()); // First full window
        assert!(indicator.next(4.0).is_some()); // Window slides
        assert!(indicator.next(5.0).is_some()); // Window slides
    }

    #[test]
    fn test_shannon_entropy_reset() {
        let mut indicator = ShannonEntropy::new(5, 10);

        for i in 1..=5 {
            indicator.next(i as f64);
        }

        assert_eq!(indicator.len(), 5);

        indicator.reset();
        assert_eq!(indicator.len(), 0);
        assert!(indicator.is_empty());
    }

    #[test]
    fn test_shannon_entropy_function() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = shannon_entropy(&values, 5, 10);

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
    fn test_entropy_normalization() {
        // Test that entropy is properly normalized to [0, 1]
        let values = vec![1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9];
        let mut indicator = ShannonEntropy::new(10, 5);

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
}
