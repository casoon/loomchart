//! Simple Moving Average (SMA) Indicator

use crate::indicators::{Current, Next, Period, Reset};
use crate::types::Ohlcv;

/// Simple Moving Average (SMA)
///
/// SMA = sum(prices) / period
///
/// Uses a rolling window to efficiently calculate the average without
/// re-summing all values on each update.
///
/// # Example
/// ```
/// use loom_indicators::indicators::Sma;
/// use loom_indicators::indicators::Next;
///
/// let mut sma = Sma::new(3);
///
/// assert_eq!(sma.next(1.0), None);
/// assert_eq!(sma.next(2.0), None);
/// assert_eq!(sma.next(3.0), Some(2.0)); // (1+2+3)/3 = 2
/// assert_eq!(sma.next(4.0), Some(3.0)); // (2+3+4)/3 = 3
/// ```
#[derive(Debug, Clone)]
pub struct Sma {
    period: usize,
    buffer: Vec<f64>,
    index: usize,
    count: usize,
    sum: f64,
}

impl Sma {
    /// Create a new SMA with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            buffer: vec![0.0; period],
            index: 0,
            count: 0,
            sum: 0.0,
        }
    }
}

impl Next<f64> for Sma {
    type Output = f64;

    fn next(&mut self, value: f64) -> Option<f64> {
        // Subtract the value being replaced
        if self.count >= self.period {
            self.sum -= self.buffer[self.index];
        }

        // Add the new value
        self.buffer[self.index] = value;
        self.sum += value;

        // Advance index
        self.index = (self.index + 1) % self.period;

        // Track count
        if self.count < self.period {
            self.count += 1;
        }

        // Return SMA if we have enough data
        if self.count >= self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

impl Next<&Ohlcv> for Sma {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Sma {
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.sum = 0.0;
    }
}

impl Period for Sma {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Sma {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        if self.count >= self.period {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }
}

impl Default for Sma {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    #[test]
    fn test_sma() {
        let mut sma = Sma::new(3);

        assert_eq!(sma.next(1.0), None);
        assert_eq!(sma.next(2.0), None);
        assert_approx_eq!(sma.next(3.0).unwrap(), 2.0);
        assert_approx_eq!(sma.next(4.0).unwrap(), 3.0);
        assert_approx_eq!(sma.next(5.0).unwrap(), 4.0);
    }

    #[test]
    fn test_sma_reset() {
        let mut sma = Sma::new(3);

        sma.next(1.0);
        sma.next(2.0);
        sma.next(3.0);

        sma.reset();

        assert_eq!(sma.next(10.0), None);
        assert_eq!(sma.next(20.0), None);
        assert_approx_eq!(sma.next(30.0).unwrap(), 20.0);
    }

    #[test]
    fn test_sma_current() {
        let mut sma = Sma::new(3);

        assert_eq!(sma.current(), None);
        sma.next(1.0);
        sma.next(2.0);
        sma.next(3.0);
        assert_approx_eq!(sma.current().unwrap(), 2.0);
    }
}
