//! Exponential Moving Average (EMA) and variants

use crate::indicators::{Current, Next, Period, Reset};
use crate::math::average::{ema_multiplier, ema_next};
use crate::types::Ohlcv;

/// Exponential Moving Average (EMA)
///
/// EMA gives more weight to recent prices, making it more responsive
/// than SMA while still smoothing the data.
///
/// EMA = price * k + prev_EMA * (1 - k)
/// where k = 2 / (period + 1)
///
/// # Example
/// ```
/// use loom_indicators::indicators::{Ema, Next};
///
/// let mut ema = Ema::new(10);
///
/// // First value initializes the EMA
/// for price in [100.0, 101.0, 102.0, 101.5, 103.0] {
///     if let Some(value) = ema.next(price) {
///         println!("EMA: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Ema {
    period: usize,
    multiplier: f64,
    current: Option<f64>,
    count: usize,
    sum: f64, // For initial SMA calculation
}

impl Ema {
    /// Create a new EMA with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            multiplier: ema_multiplier(period),
            current: None,
            count: 0,
            sum: 0.0,
        }
    }

    /// Create an EMA with a custom multiplier.
    pub fn with_multiplier(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            current: None,
            count: 0,
            sum: 0.0,
        }
    }

    /// Get the EMA multiplier (smoothing factor).
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Next<f64> for Ema {
    type Output = f64;

    fn next(&mut self, price: f64) -> Option<f64> {
        self.count += 1;

        match self.current {
            Some(prev_ema) => {
                let new_ema = ema_next(price, prev_ema, self.multiplier);
                self.current = Some(new_ema);
                Some(new_ema)
            }
            None => {
                // Accumulate for initial SMA
                self.sum += price;

                if self.count >= self.period {
                    // Initialize with SMA
                    let sma = self.sum / self.period as f64;
                    self.current = Some(sma);
                    Some(sma)
                } else {
                    None
                }
            }
        }
    }
}

impl Next<&Ohlcv> for Ema {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Ema {
    fn reset(&mut self) {
        self.current = None;
        self.count = 0;
        self.sum = 0.0;
    }
}

impl Period for Ema {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Ema {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Ema {
    fn default() -> Self {
        Self::new(21)
    }
}

/// Double Exponential Moving Average (DEMA)
///
/// DEMA = 2 * EMA - EMA(EMA)
///
/// DEMA reduces the lag inherent in traditional moving averages.
#[derive(Clone)]
pub struct Dema {
    period: usize,
    ema1: Ema,
    ema2: Ema,
    ema1_value: Option<f64>,
}

impl Dema {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            ema1: Ema::new(period),
            ema2: Ema::new(period),
            ema1_value: None,
        }
    }
}

impl Next<f64> for Dema {
    type Output = f64;

    fn next(&mut self, price: f64) -> Option<f64> {
        // First EMA
        let ema1 = self.ema1.next(price)?;
        self.ema1_value = Some(ema1);

        // Second EMA (EMA of EMA)
        let ema2 = self.ema2.next(ema1)?;

        // DEMA = 2 * EMA - EMA(EMA)
        Some(2.0 * ema1 - ema2)
    }
}

impl Next<&Ohlcv> for Dema {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Dema {
    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema1_value = None;
    }
}

impl Period for Dema {
    fn period(&self) -> usize {
        self.period
    }
}

impl Default for Dema {
    fn default() -> Self {
        Self::new(21)
    }
}

/// Triple Exponential Moving Average (TEMA)
///
/// TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA))
///
/// TEMA provides even less lag than DEMA.
#[derive(Clone)]
pub struct Tema {
    period: usize,
    ema1: Ema,
    ema2: Ema,
    ema3: Ema,
}

impl Tema {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            ema1: Ema::new(period),
            ema2: Ema::new(period),
            ema3: Ema::new(period),
        }
    }
}

impl Next<f64> for Tema {
    type Output = f64;

    fn next(&mut self, price: f64) -> Option<f64> {
        let ema1 = self.ema1.next(price)?;
        let ema2 = self.ema2.next(ema1)?;
        let ema3 = self.ema3.next(ema2)?;

        Some(3.0 * ema1 - 3.0 * ema2 + ema3)
    }
}

impl Next<&Ohlcv> for Tema {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Tema {
    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema3.reset();
    }
}

impl Period for Tema {
    fn period(&self) -> usize {
        self.period
    }
}

impl Default for Tema {
    fn default() -> Self {
        Self::new(21)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::assert_approx_eq;

    #[test]
    fn test_ema_basic() {
        let mut ema = Ema::new(3);

        // First period values for SMA
        assert_eq!(ema.next(1.0), None);
        assert_eq!(ema.next(2.0), None);

        // Third value: SMA = (1+2+3)/3 = 2
        assert_approx_eq!(ema.next(3.0).unwrap(), 2.0);

        // Fourth value: EMA = 4 * k + 2 * (1-k) where k = 2/(3+1) = 0.5
        // EMA = 4 * 0.5 + 2 * 0.5 = 2 + 1 = 3
        assert_approx_eq!(ema.next(4.0).unwrap(), 3.0);
    }

    #[test]
    fn test_ema_multiplier() {
        let ema = Ema::new(9);
        assert_approx_eq!(ema.multiplier(), 0.2); // 2/(9+1) = 0.2

        let ema = Ema::new(21);
        assert_approx_eq!(ema.multiplier(), 0.090909, 0.0001);
    }

    #[test]
    fn test_ema_reset() {
        let mut ema = Ema::new(3);

        ema.next(1.0);
        ema.next(2.0);
        ema.next(3.0);

        ema.reset();

        assert_eq!(ema.current(), None);
        assert_eq!(ema.next(10.0), None);
    }

    #[test]
    fn test_dema() {
        let mut dema = Dema::new(3);

        // Need more values for DEMA (2 * period - 1 minimum)
        for _ in 0..4 {
            dema.next(100.0);
        }

        // Should have a value now
        let val = dema.next(100.0);
        assert!(val.is_some());
        assert_approx_eq!(val.unwrap(), 100.0); // Flat prices = EMA = price
    }

    #[test]
    fn test_tema() {
        let mut tema = Tema::new(3);

        // Need more values for TEMA (3 * period - 2 minimum)
        for _ in 0..6 {
            tema.next(100.0);
        }

        let val = tema.next(100.0);
        assert!(val.is_some());
        assert_approx_eq!(val.unwrap(), 100.0);
    }
}
