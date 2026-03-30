//! Detrended Price Oscillator (DPO)

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Sma;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Detrended Price Oscillator (DPO)
///
/// DPO removes trend from prices to identify cycles. It compares a past
/// price to a moving average, removing the trend and leaving only the
/// cyclical component.
///
/// Formula: DPO = Price[period/2 + 1 bars ago] - SMA(period)
///
/// Usage:
/// - Identify cycles in price
/// - Overbought/oversold conditions
/// - Not a timing indicator (it's shifted back in time)
///
/// Default period: 20
#[derive(Clone)]
pub struct Dpo {
    period: usize,
    shift: usize,
    price_buffer: Vec<f64>,
    price_index: usize,
    price_count: usize,
    sma: Sma,
    current: Option<f64>,
}

impl Dpo {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        let shift = period / 2 + 1;

        Self {
            period,
            shift,
            price_buffer: vec![0.0; shift + 1],
            price_index: 0,
            price_count: 0,
            sma: Sma::new(period),
            current: None,
        }
    }

    fn get_shifted_price(&self) -> Option<f64> {
        if self.price_count <= self.shift {
            return None;
        }

        let idx = (self.price_index + self.price_buffer.len() - self.shift) % self.price_buffer.len();
        Some(self.price_buffer[idx])
    }
}

impl Next<f64> for Dpo {
    type Output = f64;

    fn next(&mut self, value: f64) -> Option<f64> {
        // Store price
        self.price_buffer[self.price_index] = value;
        self.price_index = (self.price_index + 1) % self.price_buffer.len();
        self.price_count += 1;

        // Calculate SMA
        let sma_val = self.sma.next(value)?;

        // Get shifted price
        let shifted = self.get_shifted_price()?;

        let dpo = shifted - sma_val;
        self.current = Some(dpo);
        Some(dpo)
    }
}

impl Reset for Dpo {
    fn reset(&mut self) {
        self.price_buffer.fill(0.0);
        self.price_index = 0;
        self.price_count = 0;
        self.sma.reset();
        self.current = None;
    }
}

impl Period for Dpo {
    fn period(&self) -> usize {
        self.period + self.shift
    }
}

impl Current for Dpo {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Dpo {
    fn default() -> Self {
        Self::new(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpo_basic() {
        let mut dpo = Dpo::new(10);

        for i in 0..25 {
            dpo.next(100.0 + i as f64);
        }

        assert!(dpo.current().is_some());
    }

    #[test]
    fn test_dpo_flat_prices() {
        let mut dpo = Dpo::new(5);

        // Flat prices should give DPO near zero
        for _ in 0..20 {
            dpo.next(100.0);
        }

        let value = dpo.current().unwrap();
        assert!(value.abs() < 0.01);
    }

    #[test]
    fn test_dpo_reset() {
        let mut dpo = Dpo::default();

        for i in 0..30 {
            dpo.next(100.0 + (i % 10) as f64);
        }

        dpo.reset();
        assert!(dpo.current().is_none());
    }
}
