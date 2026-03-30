//! Signal DSL - Simple functions for trading rules.
//!
//! Provides stateless and stateful signal detection utilities.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Check if series A crossed over series B (bullish)
///
/// Returns true if A was below B and is now above B
#[inline]
pub fn cross_over(a: &[f64], b: &[f64]) -> bool {
    if a.len() < 2 || b.len() < 2 {
        return false;
    }
    let len = a.len().min(b.len());
    a[len - 2] <= b[len - 2] && a[len - 1] > b[len - 1]
}

/// Check if series A crossed under series B (bearish)
///
/// Returns true if A was above B and is now below B
#[inline]
pub fn cross_under(a: &[f64], b: &[f64]) -> bool {
    if a.len() < 2 || b.len() < 2 {
        return false;
    }
    let len = a.len().min(b.len());
    a[len - 2] >= b[len - 2] && a[len - 1] < b[len - 1]
}

/// Check if series crossed over a fixed value
#[inline]
pub fn cross_over_value(series: &[f64], value: f64) -> bool {
    if series.len() < 2 {
        return false;
    }
    series[series.len() - 2] <= value && series[series.len() - 1] > value
}

/// Check if series crossed under a fixed value
#[inline]
pub fn cross_under_value(series: &[f64], value: f64) -> bool {
    if series.len() < 2 {
        return false;
    }
    series[series.len() - 2] >= value && series[series.len() - 1] < value
}

/// Check if A is above B (current values)
#[inline]
pub fn above(a: &[f64], b: &[f64]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a[a.len() - 1] > b[b.len() - 1]
}

/// Check if A is below B (current values)
#[inline]
pub fn below(a: &[f64], b: &[f64]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a[a.len() - 1] < b[b.len() - 1]
}

/// Check if A has been above B for N bars
pub fn above_for(a: &[f64], b: &[f64], n: usize) -> bool {
    if a.len() < n || b.len() < n {
        return false;
    }
    let len = a.len().min(b.len());
    (0..n).all(|i| a[len - 1 - i] > b[len - 1 - i])
}

/// Check if A has been below B for N bars
pub fn below_for(a: &[f64], b: &[f64], n: usize) -> bool {
    if a.len() < n || b.len() < n {
        return false;
    }
    let len = a.len().min(b.len());
    (0..n).all(|i| a[len - 1 - i] < b[len - 1 - i])
}

/// Calculate slope over N periods (rate of change)
pub fn slope(series: &[f64], periods: usize) -> Option<f64> {
    if series.len() < periods + 1 || periods == 0 {
        return None;
    }
    let current = series[series.len() - 1];
    let past = series[series.len() - 1 - periods];
    Some((current - past) / periods as f64)
}

/// Calculate acceleration (change in slope)
pub fn acceleration(series: &[f64], periods: usize) -> Option<f64> {
    if series.len() < periods * 2 + 1 {
        return None;
    }
    let slope_current = slope(series, periods)?;
    let slope_past = slope(&series[..series.len() - periods], periods)?;
    Some(slope_current - slope_past)
}

/// Calculate momentum (current - past)
#[inline]
pub fn momentum(series: &[f64], periods: usize) -> Option<f64> {
    if series.len() <= periods {
        return None;
    }
    Some(series[series.len() - 1] - series[series.len() - 1 - periods])
}

/// Detect regular divergence between price and indicator
///
/// Bullish: Price makes lower low, indicator makes higher low
/// Bearish: Price makes higher high, indicator makes lower high
pub fn divergence(price: &[f64], indicator: &[f64], lookback: usize) -> Option<DivergenceType> {
    if price.len() < lookback + 1 || indicator.len() < lookback + 1 {
        return None;
    }

    let p_len = price.len();
    let i_len = indicator.len();

    // Find swing points in lookback period
    let price_current = price[p_len - 1];
    let indicator_current = indicator[i_len - 1];

    // Find lowest/highest in lookback
    let mut price_low = price_current;
    let mut price_high = price_current;
    let mut ind_at_low = indicator_current;
    let mut ind_at_high = indicator_current;

    for i in 1..=lookback {
        let p = price[p_len - 1 - i];
        let ind = indicator[i_len - 1 - i];

        if p < price_low {
            price_low = p;
            ind_at_low = ind;
        }
        if p > price_high {
            price_high = p;
            ind_at_high = ind;
        }
    }

    // Bullish divergence: price lower low, indicator higher low
    if price_current <= price_low && indicator_current > ind_at_low {
        return Some(DivergenceType::Bullish);
    }

    // Bearish divergence: price higher high, indicator lower high
    if price_current >= price_high && indicator_current < ind_at_high {
        return Some(DivergenceType::Bearish);
    }

    None
}

/// Detect hidden divergence (trend continuation)
///
/// Bullish: Price makes higher low, indicator makes lower low
/// Bearish: Price makes lower high, indicator makes higher high
pub fn hidden_divergence(price: &[f64], indicator: &[f64], lookback: usize) -> Option<DivergenceType> {
    if price.len() < lookback + 1 || indicator.len() < lookback + 1 {
        return None;
    }

    let p_len = price.len();
    let i_len = indicator.len();

    let price_current = price[p_len - 1];
    let indicator_current = indicator[i_len - 1];

    // Find reference points
    let mut price_low = price_current;
    let mut price_high = price_current;
    let mut ind_at_low = indicator_current;
    let mut ind_at_high = indicator_current;

    for i in 1..=lookback {
        let p = price[p_len - 1 - i];
        let ind = indicator[i_len - 1 - i];

        if p < price_low {
            price_low = p;
            ind_at_low = ind;
        }
        if p > price_high {
            price_high = p;
            ind_at_high = ind;
        }
    }

    // Hidden bullish: price higher low, indicator lower low
    if price_current > price_low && indicator_current < ind_at_low {
        return Some(DivergenceType::HiddenBullish);
    }

    // Hidden bearish: price lower high, indicator higher high
    if price_current < price_high && indicator_current > ind_at_high {
        return Some(DivergenceType::HiddenBearish);
    }

    None
}

/// Divergence type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DivergenceType {
    /// Regular bullish divergence (reversal signal)
    Bullish,
    /// Regular bearish divergence (reversal signal)
    Bearish,
    /// Hidden bullish divergence (continuation signal)
    HiddenBullish,
    /// Hidden bearish divergence (continuation signal)
    HiddenBearish,
}

impl DivergenceType {
    pub fn is_bullish(&self) -> bool {
        matches!(self, Self::Bullish | Self::HiddenBullish)
    }

    pub fn is_bearish(&self) -> bool {
        matches!(self, Self::Bearish | Self::HiddenBearish)
    }

    pub fn is_hidden(&self) -> bool {
        matches!(self, Self::HiddenBullish | Self::HiddenBearish)
    }
}

/// Signal state for stateful tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalState {
    /// No signal
    None,
    /// Bullish signal
    Bullish,
    /// Bearish signal
    Bearish,
}

impl SignalState {
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Stateful signal buffer for tracking crossovers
pub struct SignalBuffer {
    prev_a: f64,
    prev_b: f64,
    initialized: bool,
}

impl SignalBuffer {
    pub fn new() -> Self {
        Self {
            prev_a: 0.0,
            prev_b: 0.0,
            initialized: false,
        }
    }

    /// Update with new values and return signal state
    pub fn update(&mut self, a: f64, b: f64) -> SignalState {
        if !self.initialized {
            self.prev_a = a;
            self.prev_b = b;
            self.initialized = true;
            return SignalState::None;
        }

        let signal = if self.prev_a <= self.prev_b && a > b {
            SignalState::Bullish
        } else if self.prev_a >= self.prev_b && a < b {
            SignalState::Bearish
        } else {
            SignalState::None
        };

        self.prev_a = a;
        self.prev_b = b;
        signal
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.initialized = false;
    }
}

impl Default for SignalBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_over() {
        let fast = [10.0, 11.0, 12.0];
        let slow = [10.5, 11.2, 11.0];
        assert!(cross_over(&fast, &slow));
    }

    #[test]
    fn test_cross_under() {
        let fast = [12.0, 11.0, 10.0];
        let slow = [10.5, 10.8, 11.0];
        assert!(cross_under(&fast, &slow));
    }

    #[test]
    fn test_slope() {
        let series = [100.0, 102.0, 104.0, 106.0, 108.0];
        assert_eq!(slope(&series, 2), Some(2.0));
    }

    #[test]
    fn test_signal_buffer() {
        let mut buf = SignalBuffer::new();
        assert_eq!(buf.update(10.0, 11.0), SignalState::None); // First update
        assert_eq!(buf.update(11.0, 10.5), SignalState::Bullish); // Crossover
        assert_eq!(buf.update(11.5, 10.0), SignalState::None); // Stays above
        assert_eq!(buf.update(9.0, 10.0), SignalState::Bearish); // Cross under
    }
}
