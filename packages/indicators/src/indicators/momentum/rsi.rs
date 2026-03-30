//! Relative Strength Index (RSI)

use crate::indicators::{Current, Next, Period, Reset};
use crate::math::momentum::{gain, loss, rsi as calc_rsi};
use crate::types::Ohlcv;

/// Relative Strength Index (RSI)
///
/// RSI is a momentum oscillator that measures the speed and magnitude of
/// recent price changes to evaluate overbought or oversold conditions.
///
/// RSI = 100 - (100 / (1 + RS))
/// RS = Average Gain / Average Loss
///
/// Values:
/// - 0-30: Oversold (potential buy signal)
/// - 30-70: Neutral
/// - 70-100: Overbought (potential sell signal)
///
/// Default period: 14
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Rsi, Next};
///
/// let mut rsi = Rsi::new(14);
///
/// for price in prices.iter() {
///     if let Some(value) = rsi.next(*price) {
///         if value > 70.0 {
///             println!("Overbought: {:.2}", value);
///         } else if value < 30.0 {
///             println!("Oversold: {:.2}", value);
///         }
///     }
/// }
/// ```
#[derive(Clone)]
pub struct Rsi {
    period: usize,
    prev_price: Option<f64>,
    avg_gain: f64,
    avg_loss: f64,
    count: usize,
    is_initialized: bool,
    current: Option<f64>,
}

impl Rsi {
    /// Create a new RSI with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            prev_price: None,
            avg_gain: 0.0,
            avg_loss: 0.0,
            count: 0,
            is_initialized: false,
            current: None,
        }
    }

    /// Check if RSI indicates overbought (> 70).
    pub fn is_overbought(&self) -> bool {
        self.current.map_or(false, |v| v > 70.0)
    }

    /// Check if RSI indicates oversold (< 30).
    pub fn is_oversold(&self) -> bool {
        self.current.map_or(false, |v| v < 30.0)
    }

    /// Check if RSI is in neutral zone (30-70).
    pub fn is_neutral(&self) -> bool {
        self.current.map_or(false, |v| v >= 30.0 && v <= 70.0)
    }
}

impl Next<f64> for Rsi {
    type Output = f64;

    fn next(&mut self, price: f64) -> Option<f64> {
        let result = if let Some(prev) = self.prev_price {
            let g = gain(price, prev);
            let l = loss(price, prev);

            if !self.is_initialized {
                // Accumulating for first SMA
                self.avg_gain += g;
                self.avg_loss += l;
                self.count += 1;

                if self.count >= self.period {
                    // First RSI: use SMA of gains/losses
                    self.avg_gain /= self.period as f64;
                    self.avg_loss /= self.period as f64;
                    self.is_initialized = true;

                    let rsi_val = calc_rsi(self.avg_gain, self.avg_loss);
                    self.current = Some(rsi_val);
                    Some(rsi_val)
                } else {
                    None
                }
            } else {
                // Smoothed RSI: use Wilder's smoothing (SMMA)
                let p = self.period as f64;
                self.avg_gain = (self.avg_gain * (p - 1.0) + g) / p;
                self.avg_loss = (self.avg_loss * (p - 1.0) + l) / p;

                let rsi_val = calc_rsi(self.avg_gain, self.avg_loss);
                self.current = Some(rsi_val);
                Some(rsi_val)
            }
        } else {
            // First price, no calculation yet
            None
        };

        self.prev_price = Some(price);
        result
    }
}

impl Next<&Ohlcv> for Rsi {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Rsi {
    fn reset(&mut self) {
        self.prev_price = None;
        self.avg_gain = 0.0;
        self.avg_loss = 0.0;
        self.count = 0;
        self.is_initialized = false;
        self.current = None;
    }
}

impl Period for Rsi {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Rsi {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Rsi {
    fn default() -> Self {
        Self::new(14)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    #[test]
    fn test_rsi_warmup() {
        let mut rsi = Rsi::new(3);

        // Need period + 1 values for first RSI
        assert_eq!(rsi.next(100.0), None);
        assert_eq!(rsi.next(101.0), None);
        assert_eq!(rsi.next(102.0), None);
        assert!(rsi.next(103.0).is_some()); // 4th value
    }

    #[test]
    fn test_rsi_all_gains() {
        let mut rsi = Rsi::new(3);

        rsi.next(100.0);
        rsi.next(101.0);
        rsi.next(102.0);
        let val = rsi.next(103.0).unwrap();

        // All gains, no losses -> RSI = 100
        assert_approx_eq!(val, 100.0);
    }

    #[test]
    fn test_rsi_all_losses() {
        let mut rsi = Rsi::new(3);

        rsi.next(100.0);
        rsi.next(99.0);
        rsi.next(98.0);
        let val = rsi.next(97.0).unwrap();

        // All losses, no gains -> RSI = 0
        assert_approx_eq!(val, 0.0);
    }

    #[test]
    fn test_rsi_equal_gains_losses() {
        let mut rsi = Rsi::new(4);

        // Alternating gains and losses
        rsi.next(100.0);
        rsi.next(102.0); // +2
        rsi.next(100.0); // -2
        rsi.next(102.0); // +2
        let val = rsi.next(100.0).unwrap(); // -2

        // Equal avg gain and loss -> RSI = 50
        assert_approx_eq!(val, 50.0);
    }

    #[test]
    fn test_rsi_reset() {
        let mut rsi = Rsi::new(3);

        rsi.next(100.0);
        rsi.next(101.0);
        rsi.next(102.0);
        rsi.next(103.0);

        rsi.reset();

        assert!(rsi.current().is_none());
        assert!(!rsi.is_initialized);
    }
}
