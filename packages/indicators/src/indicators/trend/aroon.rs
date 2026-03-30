//! Aroon Indicator

use crate::indicators::{Next, Period, Reset, Current};
use crate::types::Ohlcv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Aroon Output
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AroonOutput {
    /// Aroon Up (0-100)
    pub up: f64,
    /// Aroon Down (0-100)
    pub down: f64,
    /// Aroon Oscillator (Up - Down, ranges from -100 to +100)
    pub oscillator: f64,
}

impl AroonOutput {
    pub fn new(up: f64, down: f64) -> Self {
        Self {
            up,
            down,
            oscillator: up - down,
        }
    }

    /// Check if bullish (Up > Down)
    pub fn is_bullish(&self) -> bool {
        self.up > self.down
    }

    /// Check if bearish (Down > Up)
    pub fn is_bearish(&self) -> bool {
        self.down > self.up
    }

    /// Check if strong uptrend (Up > 70, Down < 30)
    pub fn is_strong_uptrend(&self) -> bool {
        self.up > 70.0 && self.down < 30.0
    }

    /// Check if strong downtrend (Down > 70, Up < 30)
    pub fn is_strong_downtrend(&self) -> bool {
        self.down > 70.0 && self.up < 30.0
    }
}

/// Aroon Indicator
///
/// Developed by Tushar Chande, the Aroon indicator identifies when trends
/// are likely to change direction.
///
/// - Aroon Up = ((period - bars since highest high) / period) * 100
/// - Aroon Down = ((period - bars since lowest low) / period) * 100
///
/// Values:
/// - 100: New high/low occurred in current bar
/// - 0: High/low occurred 'period' bars ago
///
/// Trading signals:
/// - Aroon Up > Aroon Down: Bullish
/// - Aroon Down > Aroon Up: Bearish
/// - Crossovers signal trend changes
///
/// Default period: 25
#[derive(Clone)]
pub struct Aroon {
    period: usize,
    high_buffer: Vec<f64>,
    low_buffer: Vec<f64>,
    index: usize,
    count: usize,
    current: Option<AroonOutput>,
}

impl Aroon {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            high_buffer: vec![f64::NEG_INFINITY; period],
            low_buffer: vec![f64::INFINITY; period],
            index: 0,
            count: 0,
            current: None,
        }
    }

    fn find_highest_index(&self) -> usize {
        let mut max_val = f64::NEG_INFINITY;
        let mut max_idx = 0;

        for i in 0..self.count.min(self.period) {
            if self.high_buffer[i] >= max_val {
                max_val = self.high_buffer[i];
                max_idx = i;
            }
        }

        max_idx
    }

    fn find_lowest_index(&self) -> usize {
        let mut min_val = f64::INFINITY;
        let mut min_idx = 0;

        for i in 0..self.count.min(self.period) {
            if self.low_buffer[i] <= min_val {
                min_val = self.low_buffer[i];
                min_idx = i;
            }
        }

        min_idx
    }
}

impl Next<&Ohlcv> for Aroon {
    type Output = AroonOutput;

    fn next(&mut self, candle: &Ohlcv) -> Option<AroonOutput> {
        // Store high and low
        self.high_buffer[self.index] = candle.high;
        self.low_buffer[self.index] = candle.low;

        let current_index = self.index;

        // Advance index
        self.index = (self.index + 1) % self.period;
        if self.count < self.period {
            self.count += 1;
        }

        // Need full period
        if self.count < self.period {
            return None;
        }

        // Find indices of highest high and lowest low
        let high_idx = self.find_highest_index();
        let low_idx = self.find_lowest_index();

        // Calculate bars since highest/lowest
        let bars_since_high = if current_index >= high_idx {
            current_index - high_idx
        } else {
            current_index + self.period - high_idx
        };

        let bars_since_low = if current_index >= low_idx {
            current_index - low_idx
        } else {
            current_index + self.period - low_idx
        };

        // Calculate Aroon values
        let p = self.period as f64;
        let aroon_up = ((p - bars_since_high as f64) / p) * 100.0;
        let aroon_down = ((p - bars_since_low as f64) / p) * 100.0;

        let output = AroonOutput::new(aroon_up, aroon_down);
        self.current = Some(output);
        Some(output)
    }
}

impl Reset for Aroon {
    fn reset(&mut self) {
        self.high_buffer.fill(f64::NEG_INFINITY);
        self.low_buffer.fill(f64::INFINITY);
        self.index = 0;
        self.count = 0;
        self.current = None;
    }
}

impl Period for Aroon {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Aroon {
    type Output = AroonOutput;

    fn current(&self) -> Option<AroonOutput> {
        self.current
    }
}

impl Default for Aroon {
    fn default() -> Self {
        Self::new(25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(close, high, low, close, 1000.0)
    }

    #[test]
    fn test_aroon_new_high() {
        let mut aroon = Aroon::new(5);

        // Build up period with ascending highs
        for i in 0..5 {
            aroon.next(&make_candle(100.0 + i as f64, 90.0, 95.0));
        }

        let output = aroon.current().unwrap();
        // Most recent bar is highest -> Aroon Up = 100
        assert!((output.up - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_aroon_new_low() {
        let mut aroon = Aroon::new(5);

        // Build up period with descending lows
        for i in 0..5 {
            aroon.next(&make_candle(100.0, 95.0 - i as f64, 97.0));
        }

        let output = aroon.current().unwrap();
        // Most recent bar is lowest -> Aroon Down = 100
        assert!((output.down - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_aroon_reset() {
        let mut aroon = Aroon::new(5);

        for i in 0..10 {
            aroon.next(&make_candle(100.0 + i as f64, 90.0, 95.0));
        }

        aroon.reset();
        assert!(aroon.current().is_none());
    }
}
