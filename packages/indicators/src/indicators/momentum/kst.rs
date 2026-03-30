//! Know Sure Thing (KST) Oscillator

use crate::indicators::{Next, Period, Reset, Current};
use crate::indicators::trend::Sma;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Know Sure Thing (KST) Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KstOutput {
    /// KST value
    pub kst: f64,
    /// Signal line (SMA of KST)
    pub signal: f64,
}

impl KstOutput {
    pub fn new(kst: f64, signal: f64) -> Self {
        Self { kst, signal }
    }

    /// Check if bullish (KST > signal)
    pub fn is_bullish(&self) -> bool {
        self.kst > self.signal
    }
}

/// Know Sure Thing (KST) Oscillator
///
/// Developed by Martin Pring, KST is a momentum oscillator based on the
/// smoothed rate-of-change for four different timeframes.
///
/// It combines four ROC periods with different weights:
/// - ROC(10) smoothed by SMA(10), weight 1
/// - ROC(15) smoothed by SMA(10), weight 2
/// - ROC(20) smoothed by SMA(10), weight 3
/// - ROC(30) smoothed by SMA(15), weight 4
///
/// Signal line is SMA(9) of KST
#[derive(Clone)]
pub struct Kst {
    roc_periods: [usize; 4],
    sma_periods: [usize; 4],
    weights: [f64; 4],
    signal_period: usize,
    // Circular buffers for each ROC calculation
    price_buffer: Vec<f64>,
    price_index: usize,
    price_count: usize,
    // SMAs for smoothing each ROC
    sma1: Sma,
    sma2: Sma,
    sma3: Sma,
    sma4: Sma,
    // Signal line SMA
    signal_sma: Sma,
    current: Option<KstOutput>,
}

impl Kst {
    pub fn new(
        roc_periods: [usize; 4],
        sma_periods: [usize; 4],
        weights: [f64; 4],
        signal_period: usize,
    ) -> Self {
        let max_roc = *roc_periods.iter().max().unwrap();

        Self {
            roc_periods,
            sma_periods,
            weights,
            signal_period,
            price_buffer: vec![0.0; max_roc + 1],
            price_index: 0,
            price_count: 0,
            sma1: Sma::new(sma_periods[0]),
            sma2: Sma::new(sma_periods[1]),
            sma3: Sma::new(sma_periods[2]),
            sma4: Sma::new(sma_periods[3]),
            signal_sma: Sma::new(signal_period),
            current: None,
        }
    }

    fn get_roc(&self, period: usize) -> Option<f64> {
        if self.price_count <= period {
            return None;
        }

        let current_price = self.price_buffer[(self.price_index + self.price_buffer.len() - 1) % self.price_buffer.len()];
        let past_idx = (self.price_index + self.price_buffer.len() - 1 - period) % self.price_buffer.len();
        let past_price = self.price_buffer[past_idx];

        if past_price != 0.0 {
            Some(100.0 * (current_price - past_price) / past_price)
        } else {
            None
        }
    }
}

impl Next<f64> for Kst {
    type Output = KstOutput;

    fn next(&mut self, value: f64) -> Option<KstOutput> {
        // Store price
        self.price_buffer[self.price_index] = value;
        self.price_index = (self.price_index + 1) % self.price_buffer.len();
        self.price_count += 1;

        // Calculate ROCs and smooth them
        let roc1 = self.get_roc(self.roc_periods[0]).and_then(|r| self.sma1.next(r));
        let roc2 = self.get_roc(self.roc_periods[1]).and_then(|r| self.sma2.next(r));
        let roc3 = self.get_roc(self.roc_periods[2]).and_then(|r| self.sma3.next(r));
        let roc4 = self.get_roc(self.roc_periods[3]).and_then(|r| self.sma4.next(r));

        match (roc1, roc2, roc3, roc4) {
            (Some(r1), Some(r2), Some(r3), Some(r4)) => {
                let kst = self.weights[0] * r1 + self.weights[1] * r2 +
                          self.weights[2] * r3 + self.weights[3] * r4;

                if let Some(signal) = self.signal_sma.next(kst) {
                    let output = KstOutput::new(kst, signal);
                    self.current = Some(output);
                    Some(output)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Reset for Kst {
    fn reset(&mut self) {
        self.price_buffer.fill(0.0);
        self.price_index = 0;
        self.price_count = 0;
        self.sma1.reset();
        self.sma2.reset();
        self.sma3.reset();
        self.sma4.reset();
        self.signal_sma.reset();
        self.current = None;
    }
}

impl Period for Kst {
    fn period(&self) -> usize {
        // Longest ROC + longest SMA + signal period
        self.roc_periods[3] + self.sma_periods[3] + self.signal_period
    }
}

impl Current for Kst {
    type Output = KstOutput;

    fn current(&self) -> Option<KstOutput> {
        self.current
    }
}

impl Default for Kst {
    fn default() -> Self {
        Self::new(
            [10, 15, 20, 30],
            [10, 10, 10, 15],
            [1.0, 2.0, 3.0, 4.0],
            9,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kst_basic() {
        let mut kst = Kst::new([3, 5, 7, 10], [3, 3, 3, 5], [1.0, 2.0, 3.0, 4.0], 3);

        for i in 0..50 {
            kst.next(100.0 + i as f64);
        }

        assert!(kst.current().is_some());
    }

    #[test]
    fn test_kst_reset() {
        let mut kst = Kst::default();

        for i in 0..100 {
            kst.next(100.0 + (i % 20) as f64);
        }

        kst.reset();
        assert!(kst.current().is_none());
    }
}
